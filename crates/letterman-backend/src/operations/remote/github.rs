use std::{cell::RefCell, string::FromUtf8Error, time::Duration};

use base64::Engine;
use diesel::{r2d2::ConnectionManager, MysqlConnection};
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
    utils::TimeUtil,
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
    ctx: RefCell<Context>,
}

impl SyncAction for GithubSyncer {
    async fn push_create(
        &self,
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
        let url = format!("https://api.github.com/repos/{repo}/contents/{path}",);
        let param = CreateContentParam::new(&format!("create {}", path), post.get_content());
        let resp = self.client.post(url).json(&param).send().await?;
        if !resp.status().is_success() {
            return Err(GithubSyncError::Request.into());
        }
        let resp = resp.json::<WriteContentResp>().await?;
        GithubRecordCreator(InsertableGithubRecord::new(
            post.get_post_id(),
            post.get_version(),
            resp.path,
            resp.sha,
            repo.clone(),
            resp.url,
        ))
        .execute(pool.clone())
        .await?;
        Ok(())
    }

    async fn push_update(
        &self,
        post: &Post,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<(), super::SyncError> {
        let records = self
            .get_github_sync_records(post.get_post_id(), pool.clone())
            .await?;
        let record = records.first().unwrap();
        let url = format!(
            "https://api.github.com/repos/{repo}/contents/{path}",
            repo = record.repository(),
            path = record.path()
        );
        let req = UpdateContentParam::new(
            &format!("update {}", record.path()),
            post.get_content(),
            record.sha(),
        );
        let resp = self.client.post(url).json(&req).send().await?;
        if !resp.status().is_success() {
            return Err(super::SyncError::RemoteServer);
        }
        let resp: WriteContentResp = resp.json().await?;
        GithubRecordCreator(InsertableGithubRecord::new(
            post.get_post_id(),
            post.get_version() + 1,
            resp.path,
            resp.sha,
            record.repository().to_string(),
            resp.url,
        ))
        .execute(pool.clone())
        .await?;
        Ok(())
    }

    async fn pull(
        &self,
        post_id: i64,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<Option<Post>, super::SyncError> {
        let content = self.get_github_post(post_id, pool.clone()).await?;
        // TODO: parse content
        if let Some(content) = content {
            Ok(Some(Post::new(
                -1,
                post_id,
                "".to_string(),
                serde_json::Value::Null,
                content.content.to_string(),
                -1,
                -1,
                TimeUtil::now(),
                TimeUtil::now(),
            )))
        } else {
            Ok(None)
        }
    }

    async fn check_changed(
        &self,
        post_id: i64,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<(bool, bool, bool), super::SyncError> {
        let records = self.get_github_sync_records(post_id, pool.clone()).await?;
        let post = self.get_github_post(post_id, pool.clone()).await?;
        if records.is_empty() {
            return Ok((false, false, true));
        }
        let post = post.unwrap();
        if records.first().unwrap().sha() == post.sha {
            Ok((true, false, false))
        } else if records.iter().map(|r| r.sha()).any(|sha| sha == post.sha) {
            Ok((false, true, false))
        } else {
            Ok((false, false, false))
        }
    }
}

impl GithubSyncer {
    pub fn new(
        path: Option<String>,
        repository: Option<String>,
        token: &str,
    ) -> Result<GithubSyncer, GithubSyncError> {
        let mut header_map = header::HeaderMap::new();
        header_map.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("letterman"),
        );
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
            ctx: RefCell::new(Context::new()),
        })
    }

    async fn get_github_sync_records(
        &self,
        post_id: i64,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<Vec<GithubRecord>, DbActionError<QueryGithubRecordError>> {
        // 定义一个局部作用域，以限制可变借用的范围
        let records: Option<Vec<GithubRecord>> = {
            // 仅当需要插入记录时，才进行可变借用
            let ctx = self.ctx.borrow();
            let records: Option<&Vec<GithubRecord>> = ctx.get(GITHUB_SYNC_RECORDS_KEY);
            if records.is_none() {
                None
            } else {
                Some(records.unwrap_or(&vec![]).clone())
            }
        };
        {
            if records.is_none() {
                let new_records = GithubRecordQueryerByPostId(post_id)
                    .execute(pool.clone())
                    .await?;
                let mut ctx = self.ctx.borrow_mut();
                ctx.set(GITHUB_SYNC_RECORDS_KEY.to_string(), new_records);
            }
        }
        let ctx = self.ctx.borrow();
        let records: &Vec<GithubRecord> = ctx.get(GITHUB_SYNC_RECORDS_KEY).unwrap();
        Ok(records.clone())
    }

    async fn get_github_post(
        &self,
        post_id: i64,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<Option<GithubArticleRecord>, SyncError> {
        {
            let ctx = self.ctx.borrow();
            let post: Option<&GithubArticleRecord> = ctx.get(GITHUB_SYNC_POST_KEY);
            if let Some(post) = post {
                return Ok(Some(post.clone()));
            }
        }

        let records = self.get_github_sync_records(post_id, pool.clone()).await?;
        if records.is_empty() {
            return Ok(None);
        }
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
            let mut ctx = self.ctx.borrow_mut();
            ctx.set(GITHUB_SYNC_POST_KEY.to_string(), content);
        }

        let ctx = self.ctx.borrow();
        Ok(Some(
            ctx.get::<GithubArticleRecord>(GITHUB_SYNC_POST_KEY)
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
                let content = base64::prelude::BASE64_STANDARD_NO_PAD.decode(content)?;
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
    fn from(_: DecodeError) -> Self {
        todo!()
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
mod github_sync_test {
    use std::env;

    use crate::types::github_record::GithubArticleRecord;

    use super::*;
    use base64::prelude::*;

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
}
