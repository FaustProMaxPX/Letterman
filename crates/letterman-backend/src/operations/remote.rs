use async_trait::async_trait;
use diesel::{r2d2::ConnectionManager, MysqlConnection};
use r2d2::Pool;

use crate::{traits::DbAction, types::posts::Post};

use self::types::SyncError;

use super::posts::{LatestPostQueryerByPostId, PostDirectCreator};

pub mod github;
pub mod types;

#[async_trait]
pub trait SyncAction {
    /// push post to create a new article in outer platform
    async fn push_create(
        &mut self,
        post: &Post,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<(), SyncError>;

    /// push post to update an article in outer platform
    async fn push_update(
        &mut self,
        post: &Post,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<(), SyncError>;

    /// pull latest post in outer platform by the param provided by syncer
    async fn pull(
        &mut self,
        post: &Post,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<Option<Post>, SyncError>;

    /// check if the article is changed
    /// return (is_latest, is_older_version, never_synced, local_is_older_version)
    async fn check_changed(
        &mut self,
        post: &Post,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> Result<(bool, bool, bool), SyncError>;
}

/// synchronize post to the outer platform
/// this function will just push post if there is any change in the article stored in the database.
/// It will not pull article from outer platform although there may be some changes.
/// if you need to pull article from outer platform, use `pull` to force that.
async fn synchronize(
    syncer: &mut impl SyncAction,
    post_id: i64,
    pool: Pool<ConnectionManager<MysqlConnection>>,
) -> Result<(), SyncError> {
    let post = LatestPostQueryerByPostId(post_id)
        .execute(pool.clone())
        .await?;
    let (is_latest, is_older_version, never_synced) =
        syncer.check_changed(&post, pool.clone()).await?;
    if never_synced {
        syncer.push_create(&post, pool.clone()).await
    } else if is_latest {
        Ok(())
    } else if is_older_version {
        syncer.push_update(&post, pool.clone()).await
    } else {
        Err(SyncError::Ambiguous)
    }
}

/// pull article from outer platform as the latest version
async fn pull(
    syncer: &mut impl SyncAction,
    post_id: i64,
    pool: Pool<ConnectionManager<MysqlConnection>>,
) -> Result<(), SyncError> {
    let post = LatestPostQueryerByPostId(post_id)
        .execute(pool.clone())
        .await?;
    let remote_post = syncer.pull(&post, pool.clone()).await?;
    if let Some(post) = remote_post {
        PostDirectCreator(post).execute(pool.clone()).await?;
        Ok(())
    } else {
        Err(SyncError::Other(
            "failed to pull article from remote".to_string(),
        ))
    }
}

/// push local newest article to outer platform
async fn force_push(
    syncer: &mut impl SyncAction,
    post_id: i64,
    pool: Pool<ConnectionManager<MysqlConnection>>,
) -> Result<(), SyncError> {
    let post = LatestPostQueryerByPostId(post_id)
        .execute(pool.clone())
        .await?;
    syncer.push_update(&post, pool.clone()).await
}