use std::fmt::{self, Formatter};

use crate::{
    routes::posts::PostResponseError,
    traits::Validate,
    utils::{Snowflake, TimeUtil},
};
use chrono::NaiveDateTime;
use diesel::{deserialize::Queryable, prelude::Insertable, Selectable};
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    id: i64,
    post_id: i64,
    title: String,
    metadata: Value,
    content: String,
    version: i32,
    pre_version: i32,
    create_time: NaiveDateTime,
    update_time: NaiveDateTime,
}

impl Post {
    pub fn new(base: BasePost, content: PostContent) -> Self {
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
}

#[derive(Insertable, Queryable, Selectable, Debug)]
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

#[derive(Insertable, Queryable, Debug, Clone)]
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

#[derive(Clone, Serialize, Deserialize)]
pub struct CreatePostReq {
    title: String,
    metadata: String,
    content: String,
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub struct ValidateCreatePostError {
    pub field: &'static str,
    pub msg: &'static str,
}

impl std::fmt::Display for ValidateCreatePostError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.msg)
    }
}

impl From<ValidateCreatePostError> for PostResponseError {
    fn from(item: ValidateCreatePostError) -> Self {
        PostResponseError::ValidationError {
            field: item.field,
            msg: item.msg,
        }
    }
}

impl Validate for CreatePostReq {
    type Item = ValidatedPostCreation;

    type Error = ValidateCreatePostError;

    fn validate(self) -> Result<Self::Item, Self::Error> {
        if self.title.trim().is_empty() {
            return Err(ValidateCreatePostError {
                field: "title",
                msg: "cannot be empty",
            });
        }

        if self.title.len() > 255 {
            return Err(ValidateCreatePostError {
                field: "title",
                msg: "cannot be longer than 255 characters",
            });
        }

        if self.metadata.len() > 255 {
            return Err(ValidateCreatePostError {
                field: "metadata",
                msg: "cannot be longer than 255 characters",
            });
        }

        if let Err(e) = serde_json::from_str::<Value>(&self.metadata) {
            error!("failed to parse json:{}, error: {e}", &self.metadata);
            return Err(ValidateCreatePostError {
                field: "metadata",
                msg: "failed to parse metadata, please make sure it's a json",
            });
        }

        Ok(ValidatedPostCreation {
            title: self.title,
            metadata: self.metadata,
            content: self.content,
        })
    }
}

pub struct ValidatedPostCreation {
    pub(crate) title: String,
    pub(crate) metadata: String,
    pub(crate) content: String,
}

impl ValidatedPostCreation {
    pub fn to_post_po(self) -> (BasePost, PostContent) {
        let post = BasePost {
            id: Snowflake::next_id(),
            post_id: Snowflake::next_id(),
            title: self.title,
            metadata: self.metadata,
            version: 1,
            prev_version: 0,
            create_time: TimeUtil::now(),
            update_time: TimeUtil::now(),
        };

        let content = PostContent {
            id: Snowflake::next_id(),
            post_id: post.post_id,
            version: 1,
            content: self.content,
            prev_version: 0,
            create_time: TimeUtil::now(),
            update_time: TimeUtil::now(),
        };
        (post, content)
    }
}

#[derive(Debug, Error, Clone, Serialize, Display)]
pub enum CreatePostError {
    #[display(fmt = "database error")]
    Database,
}

impl From<diesel::result::Error> for CreatePostError {
    fn from(e: diesel::result::Error) -> Self {
        error!("create post error: database error, e: {e}");
        CreatePostError::Database
    }
}

#[derive(Debug, Error, Clone, Serialize, Display)]
pub enum QueryPostError {
    #[display(fmt = "database error")]
    Database,
}

impl From<diesel::result::Error> for QueryPostError {
    fn from(item: diesel::result::Error) -> Self {
        error!("query post error: database error, e: {item}");
        QueryPostError::Database
    }
}
