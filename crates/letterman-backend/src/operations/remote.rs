use async_trait::async_trait;
use diesel::{r2d2::ConnectionManager, MysqlConnection};
use r2d2::Pool;

use crate::{traits::DbAction, types::posts::Post};

use self::types::SyncError;

use super::posts::{LatestPostQueryerByPostId, PostDirectCreator};

pub mod factory;
pub mod github;
pub mod types;

#[async_trait]
pub trait SyncAction {
    /// push post to create a new article in outer platform
    async fn push_create(
        &mut self,
        post: &Post,
        mongo_db: mongodb::Database,
    ) -> Result<(), SyncError>;

    /// push post to update an article in outer platform
    async fn push_update(
        &mut self,
        post: &Post,
        mongo_db: mongodb::Database,
    ) -> Result<(), SyncError>;

    /// pull latest post in outer platform by the param provided by syncer
    async fn pull(
        &mut self,
        post: &Post,
        mongo_db: mongodb::Database,
    ) -> Result<Option<Post>, SyncError>;

    /// check if the article is changed
    /// return (is_latest, is_older_version, never_synced, local_is_older_version)
    async fn check_changed(
        &mut self,
        post: &Post,
        mongo_db: mongodb::Database,
    ) -> Result<(bool, bool, bool), SyncError>;
}

/// synchronize post to the outer platform
/// this function will just push post if there is any change in the article stored in the database.
/// It will not pull article from outer platform although there may be some changes.
/// if you need to pull article from outer platform, use `pull` to force that.
pub(crate) async fn synchronize(
    mut syncer: Box<dyn SyncAction>,
    post_id: i64,
    pool: Pool<ConnectionManager<MysqlConnection>>,
    mongo_db: mongodb::Database,
) -> Result<(), SyncError> {
    let post = LatestPostQueryerByPostId(post_id)
        .execute(pool.clone())
        .await?;
    let (is_latest, is_older_version, never_synced) =
        syncer.check_changed(&post, mongo_db.clone()).await?;
    if never_synced {
        syncer.push_create(&post, mongo_db.clone()).await
    } else if is_latest {
        Ok(())
    } else if is_older_version {
        syncer.push_update(&post, mongo_db.clone()).await
    } else {
        Err(SyncError::Ambiguous)
    }
}

/// pull article from outer platform as the latest version
pub(crate) async fn force_pull(
    mut syncer: Box<dyn SyncAction>,
    post_id: i64,
    pool: Pool<ConnectionManager<MysqlConnection>>,
    mongo_db: mongodb::Database,
) -> Result<(), SyncError> {
    let post = LatestPostQueryerByPostId(post_id)
        .execute(pool.clone())
        .await?;
    let remote_post = syncer.pull(&post, mongo_db.clone()).await?;
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
pub(crate) async fn force_push(
    mut syncer: Box<dyn SyncAction>,
    post_id: i64,
    pool: Pool<ConnectionManager<MysqlConnection>>,
    mongo_db: mongodb::Database,
) -> Result<(), SyncError> {
    let post = LatestPostQueryerByPostId(post_id)
        .execute(pool.clone())
        .await?;
    // TODO: 如果没有历史同步记录，使用push_create
    syncer.push_update(&post, mongo_db.clone()).await
}
