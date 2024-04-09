use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use actix_web::{http::StatusCode, ResponseError};

use crate::operations::posts::PostCreator;
use crate::traits::{DbAction, DbActionError, Validate};
use crate::types::posts::CreatePostError;
use crate::{
    types::{posts::CreatePostReq, CommonResult},
    State,
};

#[derive(Debug, Display)]
pub enum PostResponseError {
    #[display(fmt = "Validation error on field: {}, msg: {}", field, msg)]
    ValidationError {
        field: String,
        msg: String,
    },
    Pool(r2d2::Error),
    Canceled,
    Other(String),
}

impl ResponseError for PostResponseError {
    fn error_response(&self) -> actix_web::HttpResponse {
        let body = serde_json::to_string(&CommonResult::<()>::with_msg(&self.to_string())).unwrap();
        actix_web::HttpResponse::build(self.status_code())
            .insert_header(actix_web::http::header::ContentType::json())
            .body(body)
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            Self::ValidationError { .. } => StatusCode::BAD_REQUEST,
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

pub(crate) async fn create(
    state: Data<State>,
    req: Json<CreatePostReq>,
) -> Result<HttpResponse, PostResponseError> {
    let req = req.into_inner();
    let validated_param = req.validate()?;
    let _ = PostCreator(validated_param)
        .execute(state.pool.clone())
        .await?;
    Ok(HttpResponse::Ok().json(CommonResult::<()>::new()))
}
