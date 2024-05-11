use actix_web::{error::BlockingError, web::block};
use derive_more::{Display, Error};
use diesel::{r2d2::ConnectionManager, MysqlConnection};
use futures::{future::BoxFuture, FutureExt, TryFutureExt};
use r2d2::Pool;

use crate::types::posts::Post;

pub trait Validate {
    type Item;
    type Error: std::error::Error;

    fn validate(self) -> Result<Self::Item, Self::Error>;
}

#[derive(Debug, Error, Display)]
pub enum DbActionError<E>
where
    E: std::error::Error,
{
    #[display(fmt = "database error in db action: {}", _0)]
    Error(E),

    #[display(fmt = "database error in pool: {}", _0)]
    Pool(r2d2::Error),

    #[display(fmt = "db action was canceled")]
    Canceled,
}

impl<E> From<BlockingError> for DbActionError<E>
where
    E: std::error::Error,
{
    fn from(_item: BlockingError) -> Self {
        DbActionError::Canceled
    }
}

impl<E> From<r2d2::Error> for DbActionError<E>
where
    E: std::error::Error,
{
    fn from(item: r2d2::Error) -> Self {
        DbActionError::Pool(item)
    }
}

pub trait DbAction {
    type Item: Send + 'static;
    type Error: std::error::Error + Send;

    fn db_action(self, conn: &mut MysqlConnection) -> Result<Self::Item, Self::Error>;

    fn execute(
        self,
        pool: Pool<ConnectionManager<MysqlConnection>>,
    ) -> BoxFuture<'static, Result<Self::Item, DbActionError<Self::Error>>>
    where
        Self: std::marker::Sized + Send + 'static,
    {
        let result = block(move || -> Result<Self::Item, DbActionError<Self::Error>> {
            let conn = &mut pool.get()?;
            self.db_action(conn).map_err(DbActionError::Error)
        })
        .map_err(DbActionError::from);

        let result = result.map(|r| r.and_then(|inner| inner));
        result.boxed()
    }
}

pub trait SyncAction {
    type Error: std::error::Error;

    /// push post to outer platform
    fn push(&self, post: Post) -> Result<(), SyncActionError<Self::Error>>;

    /// pull latest post in outer platform by the param provided by syncer
    fn pull(&self) -> Result<Post, SyncActionError<Self::Error>>;

    fn is_latest(&self) -> bool;

    fn is_old_version() -> bool;

    fn execute(&self) -> Result<(), SyncActionError<Self::Error>> {
        let post = self.pull()?;


        Ok(())
    }
}

pub enum SyncActionError<E>
where
    E: std::error::Error,
{
    Error(E),
}
