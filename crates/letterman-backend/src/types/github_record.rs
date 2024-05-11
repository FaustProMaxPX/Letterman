use std::string::FromUtf8Error;

use base64::Engine;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Deserialize;

use crate::traits::DbActionError;

#[derive(Insertable, Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::t_github_post_record)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct GithubRecord {
    pub id: i32,
    pub post_id: i64,
    pub version: i32,
    pub path: String,
    pub sha: String,
    pub repository: String,
    pub url: String,
    pub create_time: NaiveDateTime,
    pub update_time: NaiveDateTime,
}

/// schema of response from github
#[derive(Deserialize, Debug)]
pub struct GithubArticleRecord {
    pub name: String,
    pub path: String,
    pub content: String,
    pub sha: String,
    pub url: String,
    pub encoding: String,
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
    fn from(value: FromUtf8Error) -> Self {
        DecodeError::Convert
    }
}

impl std::error::Error for DecodeError {}

#[derive(Debug, Clone, Display, Error)]
pub enum QueryGithubRecordError {
    #[display(fmt = "database error")]
    Database,
    #[display(fmt = "not found")]
    NotFound,
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
}

impl std::error::Error for GithubSyncError {}

impl From<DbActionError<QueryGithubRecordError>> for GithubSyncError {
    fn from(value: DbActionError<QueryGithubRecordError>) -> Self {
        match value {
            DbActionError::Pool(_) | DbActionError::Canceled => GithubSyncError::Database,
            DbActionError::Error(e) => e.into(),
        }
    }
}

impl From<QueryGithubRecordError> for GithubSyncError {
    fn from(item: QueryGithubRecordError) -> Self {
        match item {
            QueryGithubRecordError::Database => GithubSyncError::Database,
            QueryGithubRecordError::NotFound => {
                GithubSyncError::UserError("not found the query record".to_string())
            }
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
