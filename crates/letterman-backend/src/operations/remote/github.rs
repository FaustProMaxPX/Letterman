use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;

use log::error;
use markdown::{mdast::Node, Constructs};
use reqwest::header::{self, HeaderValue};
use thiserror::Error;

use crate::{
    operations::github_record::{GithubRecordCreator, GithubRecordQueryerByPostId},
    traits::{MongoAction, MongoActionError},
    types::{
        github_record::{
            CreateContentParam, GithubArticleRecord, GithubRecord, InsertableGithubRecord,
            QueryGithubRecordError, UpdateContentParam, WriteContentResp,
        },
        posts::Post,
    },
    utils::{self, Snowflake},
};

use super::{
    types::{Context, SyncError},
    SyncAction,
};

static GITHUB_SYNC_RECORDS_KEY: &str = "records";
static GITHUB_SYNC_POST_KEY: &str = "post";
pub struct GithubSyncer {
    /// path is required for the first time sync
    path: Option<String>,
    repository: Option<String>,
    client: reqwest::Client,
    ctx: Context,
}

#[async_trait]
impl SyncAction for GithubSyncer {
    async fn push_create(
        &mut self,
        post: &Post,
        mongo_db: mongodb::Database,
    ) -> Result<(), super::SyncError> {
        if self.path.is_none() || self.repository.is_none() {
            return Err(GithubSyncError::UserError(
                "path and repository is required for the first time sync".to_string(),
            )
            .into());
        }
        let repo = self.repository.clone().unwrap();
        let path = self.path.clone().unwrap();
        let url = format!("https://api.github.com/repos/{repo}/contents/{path}");
        let param = CreateContentParam::new(&format!("create {}", path), &package(post)?);
        let resp = self.client.put(url).json(&param).send().await?;
        if !resp.status().is_success() {
            error!("push create error: {}", resp.text().await?);
            return Err(SyncError::RemoteServer);
        }
        let resp = resp.json::<WriteContentResp>().await?;
        GithubRecordCreator(InsertableGithubRecord::new(
            post.post_id(),
            post.version(),
            resp.content.path,
            resp.content.sha,
            repo.clone(),
            resp.content.download_url,
        ))
        .execute(mongo_db.clone())
        .await?;
        Ok(())
    }

    async fn push_update(
        &mut self,
        post: &Post,
        mongo_db: mongodb::Database,
    ) -> Result<(), super::SyncError> {
        let records = self
            .get_github_sync_records(post.post_id(), mongo_db.clone())
            .await?;
        let record = records.unwrap().first().unwrap().clone();
        let url = format!(
            "https://api.github.com/repos/{repo}/contents/{path}",
            repo = record.repository(),
            path = record.path()
        );
        let req = UpdateContentParam::new(
            &format!("update {}", record.path()),
            &package(post)?,
            record.sha(),
        );
        let resp = self.client.put(url).json(&req).send().await?;
        if !resp.status().is_success() {
            error!("push update error: {}", resp.text().await?);
            return Err(SyncError::RemoteServer);
        }
        let resp: WriteContentResp = resp.json().await?;
        GithubRecordCreator(InsertableGithubRecord::new(
            post.post_id(),
            post.version() + 1,
            resp.content.path,
            resp.content.sha,
            record.repository().to_string(),
            resp.content.download_url,
        ))
        .execute(mongo_db.clone())
        .await?;
        Ok(())
    }

    async fn pull(
        &mut self,
        post: &Post,
        mongo_db: mongodb::Database,
    ) -> Result<Option<Post>, super::SyncError> {
        let content = self
            .get_github_post(post.post_id(), mongo_db.clone())
            .await?;
        if content.is_none() {
            return Ok(None);
        }

        let res = extract(&content.unwrap().content)?;
        let post = Post::new(
            Snowflake::next_id(),
            post.post_id(),
            if let Some(title) = res.title {
                title
            } else {
                post.title().to_string()
            },
            res.metadata,
            res.content,
            post.version() + 1,
            post.version(),
            utils::time_utils::now(),
            utils::time_utils::now(),
        );

        Ok(Some(post))
    }

    async fn check_changed(
        &mut self,
        post: &Post,
        mongo_db: mongodb::Database,
    ) -> Result<(bool, bool, bool), super::SyncError> {
        let records = self
            .get_github_sync_records(post.post_id(), mongo_db.clone())
            .await?;
        if records.is_none() {
            return Ok((false, false, true));
        }
        let records = records.unwrap();
        let first = records.first().unwrap();
        match post.version().cmp(&first.version()) {
            std::cmp::Ordering::Greater => Ok((false, true, false)),
            std::cmp::Ordering::Equal => Ok((true, false, false)),
            std::cmp::Ordering::Less => Ok((false, false, false)),
        }
    }
}

impl GithubSyncer {
    pub fn new(
        path: Option<String>,
        repository: Option<String>,
    ) -> Result<GithubSyncer, GithubSyncError> {
        let mut header_map = header::HeaderMap::new();
        header_map.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("letterman"),
        );
        let token = std::env::var("GITHUB_TOKEN").unwrap();
        header_map.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );
        header_map.insert(
            header::ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .default_headers(header_map)
            .build()?;
        Ok(GithubSyncer {
            path,
            repository,
            client,
            ctx: Context::new(),
        })
    }

    async fn get_github_sync_records(
        &mut self,
        post_id: i64,
        mongo_db: mongodb::Database,
    ) -> Result<Option<Vec<GithubRecord>>, MongoActionError<QueryGithubRecordError>> {
        // 定义一个局部作用域，以限制可变借用的范围
        let records: Option<Vec<GithubRecord>> = {
            // 仅当需要插入记录时，才进行可变借用
            let records: Option<Vec<GithubRecord>> = self.ctx.get(GITHUB_SYNC_RECORDS_KEY);
            records
        };
        {
            if records.is_none() {
                let new_records = GithubRecordQueryerByPostId(post_id)
                    .execute(mongo_db.clone())
                    .await?;
                if !new_records.is_empty() {
                    self.ctx
                        .set(GITHUB_SYNC_RECORDS_KEY.to_string(), new_records);
                }
            }
        }
        let records: Option<Vec<GithubRecord>> = self.ctx.get(GITHUB_SYNC_RECORDS_KEY);
        Ok(records)
    }

    async fn get_github_post(
        &mut self,
        post_id: i64,
        mongo_db: mongodb::Database,
    ) -> Result<Option<GithubArticleRecord>, SyncError> {
        {
            let post: Option<&GithubArticleRecord> = self.ctx.get(GITHUB_SYNC_POST_KEY);
            if let Some(post) = post {
                return Ok(Some(post.clone()));
            }
        }

        let records = self
            .get_github_sync_records(post_id, mongo_db.clone())
            .await?;
        if records.is_none() {
            return Ok(None);
        }
        let records = records.unwrap();
        let record = records.first().unwrap();
        let url = format!(
            "https://api.github.com/repos/{repo}/contents/{path}",
            repo = record.repository(),
            path = record.path()
        );
        let resp = self.client.get(url).send().await?;
        if !resp.status().is_success() {
            error!("get github article error: {}", resp.text().await?);
            return Err(SyncError::RemoteServer);
        }
        let content = resp.json::<GithubArticleRecord>().await?.decode_content()?;

        {
            self.ctx.set(GITHUB_SYNC_POST_KEY.to_string(), content);
        }

        Ok(Some(
            self.ctx
                .get::<GithubArticleRecord>(GITHUB_SYNC_POST_KEY)
                .unwrap()
                .clone(),
        ))
    }
}

struct ExtractResult {
    title: Option<String>,
    content: String,
    metadata: HashMap<String, String>,
}

/// package markdown content with metadata
fn package(post: &Post) -> Result<String, serde_yaml::Error> {
    let mut metadata = post.metadata().clone();
    metadata.insert("title".to_string(), post.title().to_string());
    let frontmatter = serde_yaml::to_string(&metadata)?;
    let content = format!("---\n{}\n---\n{}", frontmatter, post.content());
    Ok(content)
}

/// extract metadata from content
/// return (title, content, metadata)
fn extract(content: &str) -> Result<ExtractResult, markdown::message::Message> {
    let constructs = Constructs {
        frontmatter: true,
        ..Constructs::default()
    };
    let ast = markdown::to_mdast(
        content,
        &markdown::ParseOptions {
            constructs,
            ..markdown::ParseOptions::default()
        },
    )?;
    let content = {
        if let Some(idx) = content.find("---") {
            if let Some(idx2) = content[idx + 3..].find("---") {
                &content[idx + idx2 + 6..]
            } else {
                content
            }
        } else {
            content
        }
    };

    if let Some(children) = ast.children() {
        let mut frontmatters = vec![];
        for ele in children {
            frontmatters.extend(dfs(ele));
        }
        let mut metadata = HashMap::new();
        let mut title = None;
        for ele in frontmatters {
            if ele.0 == "title" {
                title = Some(ele.1);
                continue;
            }
            metadata.insert(ele.0, ele.1);
        }
        Ok(ExtractResult {
            title,
            content: content.to_string(),
            metadata,
        })
    } else {
        Ok(ExtractResult {
            title: None,
            content: content.to_string(),
            metadata: HashMap::new(),
        })
    }
}

fn dfs(node: &markdown::mdast::Node) -> Vec<(String, String)> {
    let mut ret = vec![];
    if let Node::Yaml(markdown::mdast::Yaml { value, .. }) = node {
        let mut split = value.split(':');
        let key = split.next().unwrap();
        let value = split.next().unwrap();
        ret.push((key.to_string(), value.to_string()));
    }
    let children = node.children();
    if let Some(children) = children {
        for child in children {
            ret.extend(dfs(child))
        }
    }

    ret
}

#[derive(Debug, Clone, Error)]
pub enum GithubSyncError {
    #[error("Network Error")]
    NetworkError(String),
    #[error("Unknown Error: {0}")]
    Other(String),
    #[error("User Error: {0}")]
    UserError(String),
    #[error("Please set GITHUB_TOKEN env if you want to use github synchronize")]
    NoToken,
    #[error("Post not found")]
    NotFound,
}

impl From<reqwest::Error> for GithubSyncError {
    fn from(item: reqwest::Error) -> Self {
        GithubSyncError::NetworkError(item.to_string())
    }
}

impl From<MongoActionError<QueryGithubRecordError>> for SyncError {
    fn from(value: MongoActionError<QueryGithubRecordError>) -> Self {
        match value {
            MongoActionError::Pool(_) => SyncError::Database,
            MongoActionError::Error(e) => e.into(),
        }
    }
}

impl From<QueryGithubRecordError> for SyncError {
    fn from(item: QueryGithubRecordError) -> Self {
        match item {
            QueryGithubRecordError::Database(_) => SyncError::Database,
            QueryGithubRecordError::NotFound => {
                SyncError::UserError("not found the query record".to_string())
            }
        }
    }
}

impl From<MongoActionError<CreateGithubRecordError>> for SyncError {
    fn from(value: MongoActionError<CreateGithubRecordError>) -> Self {
        match value {
            MongoActionError::Error(e) => e.into(),
            MongoActionError::Pool(_) => SyncError::Database,
        }
    }
}

impl From<CreateGithubRecordError> for SyncError {
    fn from(value: CreateGithubRecordError) -> Self {
        match value {
            CreateGithubRecordError::Database(_) => SyncError::Database,
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum CreateGithubRecordError {
    #[error("Database error")]
    Database(#[source] mongodb::error::Error),
}

impl From<GithubSyncError> for SyncError {
    fn from(val: GithubSyncError) -> Self {
        match val {
            GithubSyncError::NetworkError(e) => SyncError::NetworkError(e),
            GithubSyncError::Other(e) => SyncError::Other(e),
            GithubSyncError::UserError(e) => SyncError::UserError(e),
            GithubSyncError::NotFound => SyncError::NotFound,
            GithubSyncError::NoToken => SyncError::UserError(val.to_string()),
        }
    }
}

impl From<markdown::message::Message> for SyncError {
    fn from(value: markdown::message::Message) -> Self {
        SyncError::Other(format!("failed to parse markdown: {}", value))
    }
}

impl From<serde_json::Error> for SyncError {
    fn from(value: serde_json::Error) -> Self {
        SyncError::Other(format!("failed to parse json: {}", value))
    }
}

#[cfg(test)]
mod github_sync_test {
    use std::env;

    use crate::traits::DbAction;
    use crate::{
        database_pool, mongodb_database,
        operations::{posts::LatestPostQueryerByPostId, remote::synchronize},
        types::github_record::GithubArticleRecord,
    };

    use super::*;

    #[actix_web::test]
    async fn github_syncer_test() -> Result<(), Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        let pool = database_pool()?;
        let db = mongodb_database().await?;
        let syncer = GithubSyncer::new(
            Some("README.md".to_string()),
            Some("ZephyrZenn/test-repo".to_string()),
        )?;
        synchronize(
            Box::new(syncer),
            7183026894152011778,
            pool.clone(),
            db.clone(),
        )
        .await?;
        Ok(())
    }

    #[actix_web::test]
    async fn get_content_test() {
        dotenv::dotenv().ok();
        let github_token = env::var("GITHUB_TOKEN").unwrap_or_default();
        let client = reqwest::Client::new();
        let url = "https://api.github.com/repos/ZephyrZenn/letterman/contents/README.md";
        let resp = client
            .get(url)
            .header("User-Agent", "letterman")
            .header("accept", "application/vnd.github+json")
            .header("Authorization", github_token)
            .send()
            .await
            .unwrap();
        // let content = resp.text().await.unwrap();
        // println!("{:#?}", content);
        let content = resp.json::<GithubArticleRecord>().await.unwrap();
        let content = content.decode_content().unwrap();
        println!("{:?}", content);
    }

    #[actix_web::test]
    async fn pull_test() -> Result<(), Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        let pool = database_pool()?;
        let db = mongodb_database().await?;
        let mut syncer = GithubSyncer::new(None, None)?;
        let post = LatestPostQueryerByPostId(7183657854551855106)
            .execute(pool.clone())
            .await?;
        let remote_post = syncer.pull(&post, db.clone()).await?;
        if let Some(remote_post) = remote_post {
            assert!(!remote_post.metadata().is_empty())
        }
        Ok(())
    }

    #[test]
    fn markdown_extract() {
        let content = "---\na:b\n---\ncontent";
        let constructs = Constructs {
            frontmatter: true,
            ..Constructs::default()
        };
        let ast = markdown::to_mdast(
            content,
            &markdown::ParseOptions {
                constructs,
                ..markdown::ParseOptions::default()
            },
        )
        .unwrap();

        println!("{:#?}", ast);
    }

    #[actix_web::test]
    async fn context_test() -> Result<(), Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        let db = mongodb_database().await?;
        {
            let mut ctx = Context::new();
            let new_records: Vec<GithubRecord> = GithubRecordQueryerByPostId(7191634464299159554)
                .execute(db.clone())
                .await?;
            if !new_records.is_empty() {
                ctx.set(GITHUB_SYNC_RECORDS_KEY.to_string(), new_records);
            }
            let records: Option<Vec<GithubRecord>> = ctx.get(GITHUB_SYNC_RECORDS_KEY);
            assert!(records.is_some())
        }

        Ok(())
    }
}
