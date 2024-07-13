use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error};
use actix_web::body::EitherBody;
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use log::error;

pub struct ErrorLogger;

impl<S, B> Transform<S, ServiceRequest> for ErrorLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ErrorLoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ErrorLoggerMiddleware { service })
    }
}

pub struct ErrorLoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ErrorLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            match fut.await {
                Ok(res) => Ok(res.map_into_left_body()),
                Err(err) => {
                    error!("Error: {:?}", err);
                    Err(err)
                }
            }
        })
    }
}