use actix_web::web::{Data, Json, Path, Query};
use actix_web::HttpResponse;
use actix_web::{http::StatusCode, ResponseError};

use crate::operations::posts::{
    PostCreator, PostDeleter, PostPageQueryer, PostQueryer, PostUpdater,
};
use crate::operations::remote;
use crate::operations::remote::factory::SyncerFactory;
use crate::operations::remote::types::{SyncError, SyncReq};
use crate::traits::{DbAction, DbActionError, Validate};
use crate::types::posts::{
    CreatePostError, DeletePostError, PostPageReq, QueryPostError, UpdatePostError, UpdatePostReq,
};
use crate::types::PageValidationError;
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
    remote::synchronize(syncer, post_id, state.pool.clone()).await?;
    Ok(HttpResponse::Ok().json(CommonResult::<()>::success()))
}

pub(crate) async fn force_pull(
    state: Data<State>,
    post_id: Path<i64>,
    req: Json<SyncReq>,
) -> Result<HttpResponse, PostResponseError> {
    let post_id = post_id.into_inner();
    let syncer = SyncerFactory::create(req.into_inner())?;
    remote::force_pull(syncer, post_id, state.pool.clone()).await?;
    Ok(HttpResponse::Ok().json(CommonResult::<()>::success()))
}

pub(crate) async fn force_push(
    state: Data<State>,
    post_id: Path<i64>,
    req: Json<SyncReq>,
) -> Result<HttpResponse, PostResponseError> {
    let post_id = post_id.into_inner();
    let syncer = SyncerFactory::create(req.into_inner())?;
    remote::force_push(syncer, post_id, state.pool.clone()).await?;
    Ok(HttpResponse::Ok().json(CommonResult::<()>::success()))
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
            DbActionError::Error(e) => match e {
                QueryPostError::NotFound => PostResponseError::NotFound,
                _ => PostResponseError::Other(e.to_string()),
            },
            DbActionError::Pool(e) => PostResponseError::Pool(e),
            DbActionError::Canceled => PostResponseError::Canceled,
        }
    }
}

impl From<DbActionError<UpdatePostError>> for PostResponseError {
    fn from(item: DbActionError<UpdatePostError>) -> Self {
        match item {
            DbActionError::Error(e) => match e {
                UpdatePostError::Database => PostResponseError::Database,
                UpdatePostError::NotFound => PostResponseError::NotFound,
                UpdatePostError::NotLatestVersion => {
                    PostResponseError::UserError { msg: e.to_string() }
                }
            },
            DbActionError::Pool(e) => PostResponseError::Pool(e),
            DbActionError::Canceled => PostResponseError::Canceled,
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
            DbActionError::Error(e) => match e {
                DeletePostError::Database => PostResponseError::Database,
                DeletePostError::NotFound => PostResponseError::NotFound,
            },
            DbActionError::Pool(e) => PostResponseError::Pool(e),
            DbActionError::Canceled => PostResponseError::Canceled,
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
