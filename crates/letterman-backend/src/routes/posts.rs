use std::collections::HashMap;

use actix_web::web::{Data, Json, Path, Query};
use actix_web::HttpResponse;
use actix_web::{http::StatusCode, ResponseError};
use diesel::r2d2::ConnectionManager;
use diesel::MysqlConnection;
use r2d2::Pool;

use crate::operations::posts::{
    BatchPostQueryerByPostIdAndVersion, LatestPostQueryerByPostIds, PagePostSyncRecordQueryer,
    PostCreator, PostDeleter, PostLatestSyncRecordQueryer, PostPageQueryer, PostQueryer,
    PostReverter, PostUpdater,
};
use crate::operations::remote;
use crate::operations::remote::factory::SyncerFactory;
use crate::operations::remote::types::SyncError;
use crate::traits::{DbAction, DbActionError, MongoAction, MongoActionError, Validate};
use crate::types::github_record::GithubRecordVO;
use crate::types::posts::{
    CreatePostError, DeletePostError, Post, PostPageReq, QueryPostError, QuerySyncRecordError,
    RevertPostError, RevertPostReq, SyncPageReq, SyncRecord, SyncRecordVO, SyncReq,
    UpdatePostError, UpdatePostReq,
};
use crate::types::{Page, PageValidationError};
use crate::{
    types::{posts::CreatePostReq, CommonResult},
    State,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PostResponseError {
    #[error("Validation Error: {field}: {msg}")]
    ValidationError {
        field: &'static str,
        msg: &'static str,
    },
    #[error("User Error: {msg}")]
    UserError { msg: String },
    #[error("Database Error")]
    Database,
    #[error("Pool Error")]
    Pool(#[source] r2d2::Error),
    #[error("Pool Error")]
    MongoPool(#[source] mongodb::error::Error),
    #[error("Request canceled")]
    Canceled,
    #[error("Post not found")]
    NotFound,
    #[error("Server Error: {0}")]
    Other(String),
}

pub(crate) async fn create(
    state: Data<State>,
    req: Json<CreatePostReq>,
) -> Result<HttpResponse, PostResponseError> {
    let req = req.into_inner();
    let validated_param = req.validate()?;
    PostCreator(validated_param)
        .execute(state.pool.clone())
        .await?;
    Ok(HttpResponse::Ok().json(CommonResult::<()>::success()))
}

pub(crate) async fn get_list(
    state: Data<State>,
    req: Query<PostPageReq>,
) -> Result<HttpResponse, PostResponseError> {
    let req = req.into_inner();
    let req = req.validate()?;
    let page = PostPageQueryer(req).execute(state.pool.clone()).await?;
    Ok(HttpResponse::Ok().json(CommonResult::success_with_data(page)))
}

pub(crate) async fn update(
    state: Data<State>,
    req: Json<UpdatePostReq>,
) -> Result<HttpResponse, PostResponseError> {
    let req = req.into_inner();
    let validated_param = req.validate()?;
    let post = PostUpdater(validated_param)
        .execute(state.pool.clone())
        .await?;
    Ok(HttpResponse::Ok().json(CommonResult::success_with_data(post)))
}

pub(crate) async fn get_post(
    state: Data<State>,
    id: Path<i64>,
) -> Result<HttpResponse, PostResponseError> {
    let id = id.into_inner();
    let post = PostQueryer(id).execute(state.pool.clone()).await?;
    Ok(HttpResponse::Ok().json(CommonResult::success_with_data(post)))
}

pub(crate) async fn delete_post(
    state: Data<State>,
    id: Path<i64>,
) -> Result<HttpResponse, PostResponseError> {
    let id = id.into_inner();
    PostDeleter(id).execute(state.pool.clone()).await?;
    Ok(HttpResponse::Ok().json(CommonResult::<()>::success()))
}

pub(crate) async fn synchronize(
    state: Data<State>,
    post_id: Path<i64>,
    req: Json<SyncReq>,
) -> Result<HttpResponse, PostResponseError> {
    let post_id = post_id.into_inner();
    let syncer = SyncerFactory::create(req.into_inner())?;
    remote::synchronize(
        syncer,
        post_id,
        state.pool.clone(),
        state.mongodb_database.clone(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(CommonResult::<()>::success()))
}

pub(crate) async fn force_pull(
    state: Data<State>,
    post_id: Path<i64>,
    req: Json<SyncReq>,
) -> Result<HttpResponse, PostResponseError> {
    let post_id = post_id.into_inner();
    let syncer = SyncerFactory::create(req.into_inner())?;
    remote::force_pull(
        syncer,
        post_id,
        state.pool.clone(),
        state.mongodb_database.clone(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(CommonResult::<()>::success()))
}

pub(crate) async fn force_push(
    state: Data<State>,
    post_id: Path<i64>,
    req: Json<SyncReq>,
) -> Result<HttpResponse, PostResponseError> {
    let post_id = post_id.into_inner();
    let syncer = SyncerFactory::create(req.into_inner())?;
    remote::force_push(
        syncer,
        post_id,
        state.pool.clone(),
        state.mongodb_database.clone(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(CommonResult::<()>::success()))
}

pub(crate) async fn get_sync_records(
    state: Data<State>,
    post_id: Path<i64>,
    req: Query<SyncPageReq>,
) -> Result<HttpResponse, PostResponseError> {
    let post_id = post_id.into_inner();
    let req = req.into_inner().validate()?;
    let page = PagePostSyncRecordQueryer(post_id, req.page, req.page_size, req.platform)
        .execute(state.mongodb_database.clone())
        .await?;
    let data = convert_sync_records(page.data, state.pool.clone()).await?;
    let len = data.len() as i32;
    Ok(
        HttpResponse::Ok().json(CommonResult::success_with_data(Page::new(
            page.total, req.page, data, len,
        ))),
    )
}

pub(crate) async fn get_latest_sync_records(
    state: Data<State>,
    post_id: Path<i64>,
) -> Result<HttpResponse, PostResponseError> {
    let post_id = post_id.into_inner();
    let list = PostLatestSyncRecordQueryer(post_id)
        .execute(state.mongodb_database.clone())
        .await?;
    let list = convert_sync_records(list, state.pool.clone()).await?;
    Ok(HttpResponse::Ok().json(CommonResult::success_with_data(list)))
}

pub(crate) async fn revert_post(
    state: Data<State>,
    req: Json<RevertPostReq>,
) -> Result<HttpResponse, PostResponseError> {
    let req = req.into_inner().validate()?;
    PostReverter(req.post_id, req.version).execute(state.pool.clone()).await?;
    Ok(HttpResponse::Ok().json(CommonResult::<()>::success()))
}

async fn convert_sync_records(
    data: Vec<SyncRecord>,
    pool: Pool<ConnectionManager<MysqlConnection>>,
) -> Result<Vec<SyncRecordVO>, PostResponseError> {
    let ids: Vec<(i64, String)> = data
        .iter()
        .map(|r| match r {
            SyncRecord::Github(r) => (r.post_id(), r.version().to_string()),
        })
        .collect();
    let map = BatchPostQueryerByPostIdAndVersion(ids)
        .execute(pool.clone())
        .await?;
    let post_ids: Vec<i64> = data
        .iter()
        .map(|r| match r {
            SyncRecord::Github(r) => r.post_id(),
        })
        .collect();
    let latest_posts: HashMap<i64, Post> = LatestPostQueryerByPostIds(post_ids)
        .execute(pool.clone())
        .await?
        .into_iter()
        .map(|p| (p.post_id(), p))
        .collect();
    let list: Vec<_> = data
        .into_iter()
        .map(|p| match p {
            SyncRecord::Github(p) => {
                let post = map
                    .get(&(p.post_id(), p.version().to_string()))
                    .cloned()
                    .unwrap_or_default();
                let latest = latest_posts.get(&p.post_id()).cloned().unwrap_or_default();
                SyncRecordVO::Github(GithubRecordVO::package(
                    p,
                    post.clone(),
                    latest.version().to_string(),
                ))
            }
        })
        .collect();
    Ok(list)
}

impl ResponseError for PostResponseError {
    fn error_response(&self) -> actix_web::HttpResponse {
        let body =
            serde_json::to_string(&CommonResult::<()>::fail_with_msg(&self.to_string())).unwrap();
        actix_web::HttpResponse::build(self.status_code())
            .insert_header(actix_web::http::header::ContentType::json())
            .body(body)
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            Self::ValidationError { .. } => StatusCode::BAD_REQUEST,
            Self::UserError { .. } => StatusCode::OK,
            Self::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<PageValidationError> for PostResponseError {
    fn from(item: PageValidationError) -> Self {
        PostResponseError::ValidationError {
            field: item.field,
            msg: item.msg,
        }
    }
}

impl From<DbActionError<CreatePostError>> for PostResponseError {
    fn from(item: DbActionError<CreatePostError>) -> Self {
        match item {
            DbActionError::Error(e) => PostResponseError::Other(e.to_string()),
            DbActionError::Pool(e) => PostResponseError::Pool(e),
            DbActionError::Canceled => PostResponseError::Canceled,
        }
    }
}

impl From<DbActionError<QueryPostError>> for PostResponseError {
    fn from(value: DbActionError<QueryPostError>) -> Self {
        match value {
            DbActionError::Error(e) => e.into(),
            DbActionError::Pool(e) => PostResponseError::Pool(e),
            DbActionError::Canceled => PostResponseError::Canceled,
        }
    }
}

impl From<QueryPostError> for PostResponseError {
    fn from(item: QueryPostError) -> Self {
        match item {
            QueryPostError::Database => PostResponseError::Database,
            QueryPostError::NotFound => PostResponseError::NotFound,
        }
    }
}

impl From<DbActionError<RevertPostError>> for PostResponseError {
    fn from(item: DbActionError<RevertPostError>) -> Self {
        match item {
            DbActionError::Error(e) => e.into(),
            DbActionError::Pool(e) => PostResponseError::Pool(e),
            DbActionError::Canceled => PostResponseError::Canceled,
        }
    }
}

impl From<RevertPostError> for PostResponseError {
    fn from(item: RevertPostError) -> Self {
        match item {
            RevertPostError::Database => PostResponseError::Database,
            RevertPostError::NotFound => PostResponseError::NotFound,
        }
    }
}

impl From<DbActionError<UpdatePostError>> for PostResponseError {
    fn from(item: DbActionError<UpdatePostError>) -> Self {
        match item {
            DbActionError::Error(e) => e.into(),
            DbActionError::Pool(e) => PostResponseError::Pool(e),
            DbActionError::Canceled => PostResponseError::Canceled,
        }
    }
}

impl From<UpdatePostError> for PostResponseError {
    fn from(item: UpdatePostError) -> Self {
        match item {
            UpdatePostError::Database => PostResponseError::Database,
            UpdatePostError::NotFound => PostResponseError::NotFound,
            UpdatePostError::NotLatestVersion => PostResponseError::UserError {
                msg: item.to_string(),
            },
        }
    }
}

impl From<actix_web::error::Error> for PostResponseError {
    fn from(err: actix_web::error::Error) -> Self {
        PostResponseError::Other(err.to_string())
    }
}

impl From<DbActionError<DeletePostError>> for PostResponseError {
    fn from(item: DbActionError<DeletePostError>) -> Self {
        match item {
            DbActionError::Error(e) => e.into(),
            DbActionError::Pool(e) => PostResponseError::Pool(e),
            DbActionError::Canceled => PostResponseError::Canceled,
        }
    }
}

impl From<DeletePostError> for PostResponseError {
    fn from(item: DeletePostError) -> Self {
        match item {
            DeletePostError::Database => PostResponseError::Database,
            DeletePostError::NotFound => PostResponseError::NotFound,
        }
    }
}

impl From<SyncError> for PostResponseError {
    fn from(item: SyncError) -> Self {
        match item {
            SyncError::Database => PostResponseError::Database,
            SyncError::NotFound => PostResponseError::NotFound,
            SyncError::Ambiguous => PostResponseError::UserError {
                msg: item.to_string(),
            },
            SyncError::RemoteServer => PostResponseError::Other(item.to_string()),
            SyncError::UserError(e) => PostResponseError::UserError { msg: e },
            SyncError::NetworkError(e) => PostResponseError::Other(e),
            SyncError::Decode => PostResponseError::Other(item.to_string()),
            SyncError::Other(e) => PostResponseError::Other(e),
        }
    }
}

impl From<MongoActionError<QuerySyncRecordError>> for PostResponseError {
    fn from(item: MongoActionError<QuerySyncRecordError>) -> Self {
        match item {
            MongoActionError::Pool(e) => PostResponseError::MongoPool(e),
            MongoActionError::Error(e) => e.into(),
        }
    }
}

impl From<QuerySyncRecordError> for PostResponseError {
    fn from(item: QuerySyncRecordError) -> Self {
        match item {
            QuerySyncRecordError::Database(_) => PostResponseError::Database,
            QuerySyncRecordError::Deserialize(_) => {
                PostResponseError::Other("System Error".to_string())
            }
        }
    }
}
