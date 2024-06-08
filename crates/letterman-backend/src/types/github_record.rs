use std::string::FromUtf8Error;

use base64::Engine;
use mongodb::bson::{doc, Bson};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{operations::remote::types::SyncError, traits::DocumentConvert, types::Platform};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GithubRecord {
    post_id: i64,
    version: i32,
    path: String,
    sha: String,
    repository: String,
    url: String,
}

impl GithubRecord {

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

}

pub struct InsertableGithubRecord {
    pub post_id: i64,
    pub version: i32,
    pub path: String,
    pub sha: String,
    pub repository: String,
    pub url: String,
}

impl DocumentConvert for InsertableGithubRecord {
    fn to_doc(self) -> mongodb::bson::Document {
        doc! {
            "post_id": self.post_id,
            "version": self.version,
            "path": self.path,
            "sha": self.sha,
            "repository": self.repository,
            "url": self.url,
            "platform": Bson::from(Platform::Github)
        }
    }
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
#[derive(Deserialize, Debug, Clone)]
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
    pub download_url: String,
}

#[derive(Debug, Clone, Error)]
pub enum DecodeError {
    #[error("Decode Error: algorithm: {0}, error: {1}")]
    Decode(String, String),
    #[error("Invalid content")]
    Convert,
    #[error("Unsupported encoding: {0}")]
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

#[derive(Debug, Clone, Error)]
pub enum QueryGithubRecordError {
    #[error("Database Error")]
    Database(#[source] mongodb::error::Error),
    #[error("Post not found")]
    NotFound,
}
