use std::{collections::HashMap, string::FromUtf8Error, time::Duration};

use async_trait::async_trait;
use base64::Engine;
use diesel::{r2d2::ConnectionManager, MysqlConnection};

use markdown::{mdast::Node, Constructs};
use r2d2::Pool;
use reqwest::header::{self, HeaderValue};
use serde::Deserialize;

use crate::{
    operations::github_record::{GithubRecordCreator, GithubRecordQueryerByPostId},
    traits::{DbAction, DbActionError},
    types::{
        github_record::{
            CreateContentParam, GithubRecord, InsertableGithubRecord, QueryGithubRecordError,
            UpdateContentParam, WriteContentResp,
        },
        posts::Post,
    },
    utils::{Snowflake, TimeUtil},
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
        pool: Pool<ConnectionManager<MysqlConnection>>,
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
        let param = CreateContentParam::new(
            &format!("create {}", path),
            &package(post.content(), post.metadata())?,
        );
        let resp = self.client.put(url).json(&param).send().await?;
        if !resp.status().is_success() {
            println!("{:#?}", resp);
            println!("{:#?}", resp.text().await?);
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
        .execute(pool.clone())
        .await?;
        Ok(())
    }

    async fn push_update(
        &mut self,
        post: &Post,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<(), super::SyncError> {
        let records = self
            .get_github_sync_records(post.post_id(), pool.clone())
            .await?;
        let record = records.unwrap().first().unwrap().clone();
        let url = format!(
            "https://api.github.com/repos/{repo}/contents/{path}",
            repo = record.repository(),
            path = record.path()
        );
        let req = UpdateContentParam::new(
            &format!("update {}", record.path()),
            &package(post.content(), post.metadata())?,
            record.sha(),
        );
        let resp = self.client.put(url).json(&req).send().await?;
        if !resp.status().is_success() {
            return Err(super::SyncError::RemoteServer);
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
        .execute(pool.clone())
        .await?;
        Ok(())
    }

    async fn pull(
        &mut self,
        post: &Post,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<Option<Post>, super::SyncError> {
        let content = self.get_github_post(post.post_id(), pool.clone()).await?;
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
            TimeUtil::now(),
            TimeUtil::now(),
        );

        Ok(Some(post))
    }

    async fn check_changed(
        &mut self,
        post: &Post,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<(bool, bool, bool), super::SyncError> {
        let records = self
            .get_github_sync_records(post.post_id(), pool.clone())
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
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<Option<Vec<GithubRecord>>, DbActionError<QueryGithubRecordError>> {
        // 定义一个局部作用域，以限制可变借用的范围
        let records: Option<Vec<GithubRecord>> = {
            // 仅当需要插入记录时，才进行可变借用
            let records: Option<Vec<GithubRecord>> = self.ctx.get(GITHUB_SYNC_RECORDS_KEY);
            records
        };
        {
            if records.is_none() {
                let new_records = GithubRecordQueryerByPostId(post_id)
                    .execute(pool.clone())
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
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<Option<GithubArticleRecord>, SyncError> {
        {
            let post: Option<&GithubArticleRecord> = self.ctx.get(GITHUB_SYNC_POST_KEY);
            if let Some(post) = post {
                return Ok(Some(post.clone()));
            }
        }

        let records = self.get_github_sync_records(post_id, pool.clone()).await?;
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
            return Err(super::SyncError::RemoteServer);
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

#[derive(Debug, Clone, Deserialize)]
pub struct GithubArticleRecord {
    name: String,
    path: String,
    content: String,
    sha: String,
    url: String,
    encoding: String,
}

impl GithubArticleRecord {
    pub fn decode_content(self) -> Result<GithubArticleRecord, DecodeError> {
        match &*self.encoding {
            "base64" => {
                let content = self.content.replace('\n', "");
                let content = base64::prelude::BASE64_STANDARD.decode(content)?;
                let content = String::from_utf8(content)?;
                Ok(GithubArticleRecord {
                    name: self.name,
                    path: self.path,
                    content,
                    sha: self.sha,
                    url: self.url,
                    encoding: self.encoding,
                })
            }
            _ => Err(DecodeError::UnsupportedEncoding(self.encoding.clone())),
        }
    }
}

struct ExtractResult {
    title: Option<String>,
    content: String,
    metadata: HashMap<String, String>,
}

/// package markdown content with metadata
fn package(content: &str, metadata: &HashMap<String, String>) -> Result<String, serde_yaml::Error> {
    let frontmatter = serde_yaml::to_string(metadata)?;
    let content = format!("---\n{}\n---\n{}", frontmatter, content);
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

#[derive(Debug, Clone, Display)]
pub enum DecodeError {
    #[display(fmt = "decode failed, {}, {}", _0, _1)]
    Decode(String, String),
    #[display(fmt = "convert failed")]
    Convert,
    #[display(fmt = "unsupported encoding: {}", _0)]
    UnsupportedEncoding(String),
}

impl From<base64::DecodeError> for DecodeError {
    fn from(value: base64::DecodeError) -> Self {
        DecodeError::Decode("base64".to_string(), value.to_string())
    }
}

impl From<FromUtf8Error> for DecodeError {
    fn from(_: FromUtf8Error) -> Self {
        DecodeError::Convert
    }
}

impl From<DecodeError> for SyncError {
    fn from(_value: DecodeError) -> Self {
        SyncError::Decode
    }
}

#[derive(Debug, Clone, Display)]
pub enum GithubSyncError {
    #[display(fmt = "network error")]
    NetworkError(String),
    #[display(fmt = "unknown error: {}", _0)]
    Other(String),
    #[display(fmt = "user error: {}", _0)]
    UserError(String),
    #[display(fmt = "database error")]
    Database,
    #[display(fmt = "failed to build client")]
    Builder,
    #[display(fmt = "request failed")]
    Request,
    #[display(fmt = "Please set GITHUB_TOKEN env if you want to use github synchronize")]
    NoToken,
    #[display(fmt = "not found")]
    NotFound,
}

impl std::error::Error for GithubSyncError {}

impl From<DbActionError<QueryGithubRecordError>> for SyncError {
    fn from(value: DbActionError<QueryGithubRecordError>) -> Self {
        match value {
            DbActionError::Pool(_) | DbActionError::Canceled => SyncError::Database,
            DbActionError::Error(e) => e.into(),
        }
    }
}

impl From<QueryGithubRecordError> for SyncError {
    fn from(item: QueryGithubRecordError) -> Self {
        match item {
            QueryGithubRecordError::Database => SyncError::Database,
            QueryGithubRecordError::NotFound => {
                SyncError::UserError("not found the query record".to_string())
            }
        }
    }
}

impl From<DbActionError<CreateGithubRecordError>> for SyncError {
    fn from(value: DbActionError<CreateGithubRecordError>) -> Self {
        match value {
            DbActionError::Error(e) => e.into(),
            DbActionError::Pool(_) | DbActionError::Canceled => SyncError::Database,
        }
    }
}

impl From<CreateGithubRecordError> for SyncError {
    fn from(value: CreateGithubRecordError) -> Self {
        match value {
            CreateGithubRecordError::Database => SyncError::Database,
        }
    }
}

impl From<reqwest::Error> for GithubSyncError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_builder() {
            return GithubSyncError::Builder;
        }
        GithubSyncError::NetworkError(value.to_string())
    }
}

#[derive(Debug, Clone, Display, Error)]
pub enum CreateGithubRecordError {
    #[display(fmt = "database error")]
    Database,
}

impl From<diesel::result::Error> for CreateGithubRecordError {
    fn from(_item: diesel::result::Error) -> Self {
        CreateGithubRecordError::Database
    }
}

impl From<GithubSyncError> for SyncError {
    fn from(val: GithubSyncError) -> Self {
        match val {
            GithubSyncError::NetworkError(e) => SyncError::NetworkError(e),
            GithubSyncError::Other(e) => SyncError::Other(e),
            GithubSyncError::Builder => SyncError::Client,
            GithubSyncError::Database => SyncError::Database,
            GithubSyncError::Request => SyncError::RemoteServer,
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

    use crate::{
        database_pool,
        operations::{posts::LatestPostQueryerByPostId, remote::synchronize},
        types::github_record::GithubArticleRecord,
    };

    use super::*;

    #[actix_web::test]
    async fn github_syncer_test() -> Result<(), Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        let pool = database_pool()?;
        let syncer = GithubSyncer::new(
            Some("README.md".to_string()),
            Some("ZephyrZenn/test-repo".to_string()),
        )?;
        synchronize(Box::new(syncer), 7183026894152011778, pool.clone()).await?;
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
        let mut syncer = GithubSyncer::new(None, None)?;
        let post = LatestPostQueryerByPostId(7183657854551855106)
            .execute(pool.clone())
            .await?;
        let remote_post = syncer.pull(&post, pool.clone()).await?;
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
        let pool = database_pool().unwrap();
        {
            let mut ctx = Context::new();
            let new_records: Vec<GithubRecord> = GithubRecordQueryerByPostId(7191634464299159554)
                .execute(pool.clone())
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
