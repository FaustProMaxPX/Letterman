use std::{
    collections::HashMap,
    fmt::{self, Formatter},
};

use crate::{
    routes::posts::PostResponseError,
    traits::Validate,
    utils::{self},
};
use chrono::NaiveDateTime;
use diesel::{deserialize::Queryable, prelude::Insertable, Selectable};
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    deserialize_from_string,
    github_record::{GithubRecord, GithubRecordVO},
    serialize_as_string, serialize_metadata, PageValidationError, Platform,
};

use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    #[serde(
        serialize_with = "serialize_as_string",
        deserialize_with = "deserialize_from_string"
    )]
    id: i64,
    #[serde(
        serialize_with = "serialize_as_string",
        deserialize_with = "deserialize_from_string"
    )]
    post_id: i64,
    title: String,
    #[serde(serialize_with = "serialize_metadata")]
    metadata: HashMap<String, String>,
    content: String,
    version: i32,
    pre_version: i32,
    create_time: NaiveDateTime,
    update_time: NaiveDateTime,
}

impl Post {
    pub fn new(
        id: i64,
        post_id: i64,
        title: String,
        metadata: HashMap<String, String>,
        content: String,
        version: i32,
        pre_version: i32,
        create_time: NaiveDateTime,
        update_time: NaiveDateTime,
    ) -> Self {
        Post {
            id,
            post_id,
            title,
            metadata,
            content,
            version,
            pre_version,
            create_time,
            update_time,
        }
    }

    pub fn package(base: BasePost, content: PostContent) -> Self {
        Self {
            id: base.id,
            post_id: base.post_id,
            title: base.title,
            metadata: serde_json::from_str(&base.metadata).unwrap(),
            content: content.content,
            version: base.version,
            pre_version: base.prev_version,
            create_time: base.create_time,
            update_time: base.update_time,
        }
    }

    pub fn post_id(&self) -> i64 {
        self.post_id
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn to_po(self) -> (InsertableBasePost, InsertablePostContent) {
        let base = InsertableBasePost {
            id: utils::snowflake::next_id(),
            post_id: self.post_id,
            title: self.title,
            metadata: serde_json::to_string(&self.metadata).unwrap(),
            version: self.version,
            prev_version: self.pre_version,
        };
        let content = InsertablePostContent {
            id: utils::snowflake::next_id(),
            post_id: self.post_id,
            version: self.version,
            content: self.content,
            prev_version: self.pre_version,
        };
        (base, content)
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

impl From<(InsertableBasePost, InsertablePostContent)> for Post {
    fn from(value: (InsertableBasePost, InsertablePostContent)) -> Self {
        let (base, content) = value;
        Self {
            id: base.id,
            post_id: base.post_id,
            title: base.title,
            metadata: serde_json::from_str(&base.metadata).unwrap(),
            content: content.content,
            version: base.version,
            pre_version: base.prev_version,
            create_time: utils::time_utils::now(),
            update_time: utils::time_utils::now(),
        }
    }
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::t_post)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct InsertableBasePost {
    pub id: i64,
    pub post_id: i64,
    pub title: String,
    pub metadata: String,
    pub version: i32,
    pub prev_version: i32,
}

impl InsertableBasePost {
    pub fn new(
        id: i64,
        post_id: i64,
        title: String,
        metadata: String,
        version: i32,
        prev_version: i32,
    ) -> Self {
        Self {
            id,
            post_id,
            title,
            metadata,
            version,
            prev_version,
        }
    }
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::t_post)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct BasePost {
    pub id: i64,
    pub post_id: i64,
    pub title: String,
    pub metadata: String,
    pub version: i32,
    pub prev_version: i32,
    pub create_time: NaiveDateTime,
    pub update_time: NaiveDateTime,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::t_post_content)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct InsertablePostContent {
    pub id: i64,
    pub post_id: i64,
    pub version: i32,
    pub content: String,
    pub prev_version: i32,
}

impl InsertablePostContent {
    pub fn new(id: i64, post_id: i64, version: i32, content: String, prev_version: i32) -> Self {
        Self {
            id,
            post_id,
            version,
            content,
            prev_version,
        }
    }
}

#[derive(Queryable, Debug, Clone)]
#[diesel(table_name = crate::schema::t_post_content)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct PostContent {
    pub id: i64,
    pub post_id: i64,
    pub version: i32,
    pub content: String,
    pub prev_version: i32,
    pub create_time: NaiveDateTime,
    pub update_time: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePostReq {
    title: String,
    metadata: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePostReq {
    #[serde(
        serialize_with = "serialize_as_string",
        deserialize_with = "deserialize_from_string"
    )]
    id: i64,
    title: String,
    metadata: String,
    content: String,
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub struct ValidateManipulatePostError {
    pub field: &'static str,
    pub msg: &'static str,
}

impl std::fmt::Display for ValidateManipulatePostError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.msg)
    }
}

impl From<ValidateManipulatePostError> for PostResponseError {
    fn from(item: ValidateManipulatePostError) -> Self {
        PostResponseError::ValidationError {
            field: item.field,
            msg: item.msg,
        }
    }
}

impl Validate for CreatePostReq {
    type Item = ValidatedPostCreation;

    type Error = ValidateManipulatePostError;

    fn validate(self) -> Result<Self::Item, Self::Error> {
        validate_post_data(&self.title, &self.metadata, &self.content)?;
        Ok(ValidatedPostCreation {
            title: self.title,
            metadata: self.metadata,
            content: self.content,
        })
    }
}

impl Validate for UpdatePostReq {
    type Item = ValidatedPostUpdate;

    type Error = ValidateManipulatePostError;

    fn validate(self) -> Result<Self::Item, Self::Error> {
        validate_post_data(&self.title, &self.metadata, &self.content)?;
        Ok(ValidatedPostUpdate {
            id: self.id,
            title: self.title,
            metadata: self.metadata,
            content: self.content,
        })
    }
}

fn validate_post_data(
    title: &str,
    metadata: &str,
    _content: &str,
) -> Result<(), ValidateManipulatePostError> {
    if title.trim().is_empty() {
        return Err(ValidateManipulatePostError {
            field: "title",
            msg: "cannot be empty",
        });
    }

    if title.len() > 255 {
        return Err(ValidateManipulatePostError {
            field: "title",
            msg: "cannot be longer than 255 characters",
        });
    }

    if metadata.len() > 255 {
        return Err(ValidateManipulatePostError {
            field: "metadata",
            msg: "cannot be longer than 255 characters",
        });
    }

    if let Err(e) = serde_json::from_str::<Value>(metadata) {
        error!("failed to parse json:{}, error: {e}", metadata);
        return Err(ValidateManipulatePostError {
            field: "metadata",
            msg: "failed to parse metadata, please make sure it's a json",
        });
    }
    Ok(())
}

pub struct ValidatedPostUpdate {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) metadata: String,
    pub(crate) content: String,
}

pub struct ValidatedPostCreation {
    pub(crate) title: String,
    pub(crate) metadata: String,
    pub(crate) content: String,
}

impl ValidatedPostCreation {
    pub fn to_post_po(self) -> (InsertableBasePost, InsertablePostContent) {
        let post = InsertableBasePost {
            id: utils::snowflake::next_id(),
            post_id: utils::snowflake::next_id(),
            title: self.title,
            metadata: self.metadata,
            version: 1,
            prev_version: 0,
        };

        let content = InsertablePostContent {
            id: utils::snowflake::next_id(),
            post_id: post.post_id,
            version: 1,
            content: self.content,
            prev_version: 0,
        };
        (post, content)
    }
}

#[derive(Debug, Error, Clone, Serialize)]
pub enum CreatePostError {
    #[error("Database Error")]
    Database,
}

impl From<diesel::result::Error> for CreatePostError {
    fn from(_: diesel::result::Error) -> Self {
        CreatePostError::Database
    }
}

#[derive(Debug, Error, Clone, Serialize)]
pub enum QueryPostError {
    #[error("Database Error")]
    Database,
    #[error("Post not found")]
    NotFound,
}

impl From<diesel::result::Error> for QueryPostError {
    fn from(item: diesel::result::Error) -> Self {
        match item {
            diesel::result::Error::NotFound => QueryPostError::NotFound,
            _ => QueryPostError::Database,
        }
    }
}

#[derive(Debug, Clone, Serialize, Error)]
pub enum UpdatePostError {
    #[error("Database Error")]
    Database,
    #[error("Post not found")]
    NotFound,
    #[error("Please use the latest version of the post")]
    NotLatestVersion,
}

impl From<diesel::result::Error> for UpdatePostError {
    fn from(item: diesel::result::Error) -> Self {
        error!("update post error: database error, e: {item}");
        match item {
            diesel::result::Error::NotFound => UpdatePostError::NotFound,
            _ => UpdatePostError::Database,
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum DeletePostError {
    #[error("Database Error")]
    Database,
    #[error("Post not found")]
    NotFound,
}

impl From<diesel::result::Error> for DeletePostError {
    fn from(item: diesel::result::Error) -> Self {
        match item {
            diesel::result::Error::NotFound => DeletePostError::NotFound,
            _ => DeletePostError::Database,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostPageReq {
    pub page: i32,
    pub page_size: i32,
    pub all: Option<bool>,
}

impl Validate for PostPageReq {
    type Item = PostPageReq;

    type Error = PageValidationError;

    fn validate(self) -> Result<Self::Item, Self::Error> {
        if self.page <= 0 {
            return Err(PageValidationError {
                field: "page",
                msg: "page must be greater than 0",
            });
        };
        if self.page_size <= 0 {
            return Err(PageValidationError {
                field: "page_size",
                msg: "page_size must be greater than 0",
            });
        }
        Ok(self)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "platform")]
pub enum SyncReq {
    Github(GithubSyncReq),
}

#[derive(Debug, Deserialize)]

pub struct GithubSyncReq {
    path: Option<String>,
    repository: Option<String>,
}

impl GithubSyncReq {
    pub fn path(&self) -> Option<String> {
        self.path.clone()
    }

    pub fn repository(&self) -> Option<String> {
        self.repository.clone()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPageReq {
    pub page: i32,
    pub page_size: i32,
    pub platform: Platform,
}

impl Validate for SyncPageReq {
    type Item = SyncPageReq;

    type Error = PageValidationError;

    fn validate(self) -> Result<Self::Item, Self::Error> {
        if self.page <= 0 {
            return Err(PageValidationError {
                field: "page",
                msg: "page must be greater than 0",
            });
        };
        if self.page_size <= 0 {
            return Err(PageValidationError {
                field: "page_size",
                msg: "page_size must be greater than 0",
            });
        }
        Ok(self)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "platform")]
pub enum SyncRecord {
    Github(GithubRecord),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SyncRecordVO {
    Github(GithubRecordVO),
}

#[derive(Debug, Error)]
pub enum QuerySyncRecordError {
    #[error("Database Error: {0}")]
    Database(#[source] mongodb::error::Error),
    #[error("Deserialize Error: {0}")]
    Deserialize(#[source] bson::de::Error),
}

impl From<mongodb::error::Error> for QuerySyncRecordError {
    fn from(item: mongodb::error::Error) -> Self {
        QuerySyncRecordError::Database(item)
    }
}

impl From<bson::de::Error> for QuerySyncRecordError {
    fn from(item: bson::de::Error) -> Self {
        QuerySyncRecordError::Deserialize(item)
    }
}
