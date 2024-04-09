use std::fmt::{self, Formatter};

use diesel::{deserialize::Queryable, prelude::Insertable, Selectable};
use serde::{Deserialize, Serialize};

use crate::{routes::posts::PostResponseError, traits::Validate, utils::Snowflake};

pub struct Post {
    id: i64,
    post_id: i64,
    title: String,
    metadata: String,
    content: String,
    version: i64,
    pre_version: i64,
}

#[derive(Insertable, Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::t_post)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct BasePost {
    id: i64,
    post_id: i64,
    title: String,
    metadata: String,
    version: i32,
    prev_version: i32,
}

#[derive(Insertable, Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::t_post_content)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct PostContent {
    id: i64,
    post_id: i64,
    version: i32,
    content: String,
    prev_version: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreatePostReq {
    title: String,
    metadata: String,
    content: String,
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub struct ValidateCreatePostError {
    pub field: String,
    pub msg: String,
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
                field: "title".to_string(),
                msg: "cannot be empty".to_string(),
            });
        }

        if self.title.len() > 255 {
            return Err(ValidateCreatePostError {
                field: "title".to_string(),
                msg: "cannot be longer than 255 characters".to_string(),
            });
        }

        if self.metadata.len() > 255 {
            return Err(ValidateCreatePostError {
                field: "metadata".to_string(),
                msg: "cannot be longer than 255 characters".to_string(),
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
        };

        let content = PostContent {
            id: Snowflake::next_id(),
            post_id: post.post_id,
            version: 1,
            content: self.content,
            prev_version: 0,
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
    fn from(_: diesel::result::Error) -> Self {
        CreatePostError::Database
    }
}
