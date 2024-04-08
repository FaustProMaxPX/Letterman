use actix_web::web::{Data, Json};
use actix_web::{http::StatusCode, ResponseError};

use crate::traits::Validate;
use crate::{
    types::{
        posts::CreatePostReq,
        CommonResult,
    },
    State,
};

#[derive(Debug, Display)]
pub enum PostError {
    #[display(fmt = "Validation error on field: {}, msg: {}", field, msg)]
    ValidationError {
        field: String,
        msg: String,
    },
    Other(String),
}

impl ResponseError for PostError {
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
pub(crate) async fn create(
    state: Data<State>,
    req: Json<CreatePostReq>,
) -> Result<CommonResult<()>, PostError> {
    let req = req.into_inner();
    let validatedParam = req.validate()?;
    Ok(CommonResult::<()>::new())
}
