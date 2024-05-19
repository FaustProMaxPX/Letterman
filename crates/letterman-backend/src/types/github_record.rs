use std::string::FromUtf8Error;

use base64::Engine;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::t_github_post_record)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct GithubRecord {
    id: i32,
    post_id: i64,
    version: i32,
    path: String,
    sha: String,
    repository: String,
    url: String,
    create_time: NaiveDateTime,
    update_time: NaiveDateTime,
}

impl GithubRecord {
    pub fn update_time(&self) -> NaiveDateTime {
        self.update_time
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn post_id(&self) -> i64 {
        self.post_id
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn sha(&self) -> &str {
        &self.sha
    }

    pub fn repository(&self) -> &str {
        &self.repository
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn create_time(&self) -> NaiveDateTime {
        self.create_time
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::t_github_post_record)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct InsertableGithubRecord {
    pub post_id: i64,
    pub version: i32,
    pub path: String,
    pub sha: String,
    pub repository: String,
    pub url: String,
}

impl InsertableGithubRecord {
    pub fn new(
        post_id: i64,
        version: i32,
        path: String,
        sha: String,
        repository: String,
        url: String,
    ) -> Self {
        Self {
            post_id,
            version,
            path,
            sha,
            repository,
            url,
        }
    }
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

#[derive(Debug, Serialize, Clone)]
pub struct CreateContentParam {
    message: String,
    content: String,
}

impl CreateContentParam {
    pub fn new(message: &str, content: &str) -> CreateContentParam {
        CreateContentParam {
            message: message.to_string(),
            content: encode_content(content),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateContentParam {
    message: String,
    content: String,
    sha: String,
}

impl UpdateContentParam {
    pub fn new(message: &str, content: &str, sha: &str) -> UpdateContentParam {
        UpdateContentParam {
            message: message.to_string(),
            content: encode_content(content),
            sha: sha.to_string(),
        }
    }
}

fn encode_content(content: &str) -> String {
    base64::prelude::BASE64_STANDARD.encode(content)
}

#[derive(Debug, Deserialize, Clone)]
pub struct WriteContentResp {
    pub content: WriteContentRespInner,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WriteContentRespInner {
    pub sha: String,
    pub path: String,
    pub url: String,
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

impl std::error::Error for DecodeError {}

#[derive(Debug, Clone, Display, Error)]
pub enum QueryGithubRecordError {
    #[display(fmt = "database error")]
    Database,
    #[display(fmt = "not found")]
    NotFound,
}
