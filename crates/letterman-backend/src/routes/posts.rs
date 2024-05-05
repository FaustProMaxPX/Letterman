use actix_web::web::{Data, Json, Path, Query};
use actix_web::HttpResponse;
use actix_web::{http::StatusCode, ResponseError};

use crate::operations::posts::{PostCreator, PostPageQueryer, PostQueryer, PostUpdater};
use crate::traits::{DbAction, DbActionError, Validate};
use crate::types::posts::{CreatePostError, PostPageReq, QueryPostError, UpdatePostError, UpdatePostReq};
use crate::types::PageValidationError;
use crate::{
    types::{posts::CreatePostReq, CommonResult},
    State,
};

#[derive(Debug, Display)]
pub enum PostResponseError {
    #[display(fmt = "Validation error on field: {}, msg: {}", field, msg)]
    ValidationError {
        field: &'static str,
        msg: &'static str,
    },
    #[display(fmt = "{msg}")]
    UserError {
        msg: String,
    },
    Pool(r2d2::Error),
    Canceled,
    Other(String),
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
            _ => StatusCode::INTERNAL_SERVER_ERROR,
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
                QueryPostError::NotFound => PostResponseError::UserError { msg: e.to_string() },
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
                UpdatePostError::Database => PostResponseError::Other(e.to_string()),
                UpdatePostError::NotFound => PostResponseError::UserError { msg: e.to_string() },
                UpdatePostError::NotLatestVersion => {
                    PostResponseError::UserError { msg: e.to_string() }
                }
            },
            DbActionError::Pool(e) => PostResponseError::Pool(e),
            DbActionError::Canceled => PostResponseError::Canceled,
        }
    }
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

impl From<PageValidationError> for PostResponseError {
    fn from(item: PageValidationError) -> Self {
        PostResponseError::ValidationError {
            field: item.field,
            msg: item.msg,
        }
    }
}

pub(crate) async fn get_post(
    state: Data<State>,
    id: Path<i64>,
) -> Result<HttpResponse, PostResponseError> {
    let id = id.into_inner();
    let post = PostQueryer(id).execute(state.pool.clone()).await?;
    Ok(HttpResponse::Ok().json(CommonResult::success_with_data(post)))
}
