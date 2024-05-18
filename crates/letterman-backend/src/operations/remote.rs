use diesel::{r2d2::ConnectionManager, MysqlConnection};
use r2d2::Pool;

use crate::{traits::DbAction, types::posts::{InsertableBasePost, InsertablePostContent, Post}};

use self::types::SyncError;

use super::posts::LatestPostQueryerByPostId;

pub mod github;
pub mod types;

pub trait SyncAction {
    /// push post to create a new article in outer platform
    async fn push_create(
        &self,
        post: &Post,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<(), SyncError>;

    /// push post to update an article in outer platform
    async fn push_update(
        &self,
        post: &Post,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<(), SyncError>;

    /// pull latest post in outer platform by the param provided by syncer
    async fn pull(
        &self,
        post: &Post,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<Option<(InsertableBasePost, InsertablePostContent)>, SyncError>;

    /// check if the article is changed
    /// return (is_latest, is_older_version, never_synced)
    async fn check_changed(
        &self,
        post_id: i64,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<(bool, bool, bool), SyncError>;
}

async fn synchronize(
    syncer: &impl SyncAction,
    post_id: i64,
    pool: Pool<ConnectionManager<MysqlConnection>>,
) -> Result<(), SyncError> {
    let post = LatestPostQueryerByPostId(post_id)
        .execute(pool.clone())
        .await?;
    let (is_latest, is_older_version, never_synced) =
        syncer.check_changed(post_id, pool.clone()).await?;
    if never_synced {
        syncer.push_create(&post, pool.clone()).await?;
    }
    if is_latest {
        return Ok(());
    }
    if is_older_version {
        syncer.push_update(&post, pool.clone()).await?;
    }
    Err(SyncError::Ambiguous)
}
