use std::{any::Any, collections::HashMap};

use crate::{traits::DbActionError, types::posts::QueryPostError};

#[derive(Debug)]
pub struct Context {
    data: HashMap<String, Box<dyn Any>>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            data: HashMap::new(),
        }
    }

    pub fn get<T: 'static + Any>(&self, key: &str) -> Option<&T> {
        self.data.get(key)?.as_ref().downcast_ref()
    }

    pub fn set<T: 'static + Any>(&mut self, key: String, value: T) {
        self.data.insert(key, Box::new(value));
    }
}

#[derive(Debug, Clone, Display)]
pub enum SyncError {
    #[display(fmt = "database error")]
    Database,
    #[display(fmt = "not found")]
    NotFound,
    #[display(fmt = "System cannot decide push or pull")]
    Ambiguous,
    #[display(fmt = "Something error in client")]
    Client,
    #[display(fmt = "Get error from remote server")]
    RemoteServer,
    #[display(fmt = "user error: {}", _0)]
    UserError(String),
    #[display(fmt = "network error: {}", _0)]
    NetworkError(String),
    #[display(fmt = "decode error")]
    Decode,
    #[display(fmt = "unknown error: {}", _0)]
    Other(String),
}

impl std::error::Error for SyncError {}

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

impl From<reqwest::Error> for SyncError {
    fn from(value: reqwest::Error) -> Self {
        SyncError::Other(value.to_string())
    }
}
