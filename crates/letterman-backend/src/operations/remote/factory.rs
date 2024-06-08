use crate::types::posts::SyncReq;

use super::{github::GithubSyncer, types::SyncError, SyncAction};

#[derive(Debug, Default)]
pub struct SyncerFactory;

impl SyncerFactory {
    pub fn create(req: SyncReq) -> Result<Box<dyn SyncAction>, SyncError> {
        match req {
            SyncReq::Github(req) => Ok(Box::new(GithubSyncer::new(req.path(), req.repository())?)),
        }
    }
}
