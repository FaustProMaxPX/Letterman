use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::Deserialize;
use thiserror::Error;

use crate::{
    traits::DbActionError,
    types::posts::{CreatePostError, QueryPostError},
};

#[derive(Debug, Deserialize)]
#[serde(tag = "platform", content = "details")]
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

#[derive(Debug, Default)]
pub struct Context {
    data: Arc<Mutex<HashMap<String, Box<dyn Any + Send>>>>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get<T: 'static + Any + Clone>(&self, key: &str) -> Option<T> {
        let data = self.data.lock().unwrap();
        data.get(key)?.as_ref().downcast_ref().cloned()
    }

    pub fn set<T: 'static + Any + Send>(&mut self, key: String, value: T) {
        let mut data = self.data.lock().unwrap();
        data.insert(key, Box::new(value));
    }
}

#[derive(Debug, Clone, Error)]
pub enum SyncError {
    #[error("Database Error")]
    Database,
    #[error("Post not found")]
    NotFound,
    #[error("System cannot decide push or pull")]
    Ambiguous,
    #[error("Failed to request remote server")]
    RemoteServer,
    #[error("User error: {0}")]
    UserError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Decode Error")]
    Decode,
    #[error("Unknown error: {0}")]
    Other(String),
}

impl From<DbActionError<QueryPostError>> for SyncError {
    fn from(value: DbActionError<QueryPostError>) -> Self {
        match value {
            DbActionError::Error(e) => e.into(),
            DbActionError::Pool(_) => SyncError::Database,
            DbActionError::Canceled => SyncError::Database,
        }
    }
}

impl From<QueryPostError> for SyncError {
    fn from(value: QueryPostError) -> Self {
        match value {
            QueryPostError::Database => SyncError::Database,
            QueryPostError::NotFound => SyncError::NotFound,
        }
    }
}

impl From<DbActionError<CreatePostError>> for SyncError {
    fn from(value: DbActionError<CreatePostError>) -> Self {
        match value {
            DbActionError::Error(e) => e.into(),
            DbActionError::Pool(_) => SyncError::Database,
            DbActionError::Canceled => SyncError::Database,
        }
    }
}

impl From<CreatePostError> for SyncError {
    fn from(item: CreatePostError) -> Self {
        match item {
            CreatePostError::Database => SyncError::Database,
        }
    }
}

impl From<reqwest::Error> for SyncError {
    fn from(value: reqwest::Error) -> Self {
        SyncError::Other(value.to_string())
    }
}

impl From<serde_yaml::Error> for SyncError {
    fn from(item: serde_yaml::Error) -> Self {
        SyncError::Other(item.to_string())
    }
}
