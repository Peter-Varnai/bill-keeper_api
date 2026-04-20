use std::future::{ready, Ready};
use std::time::Instant;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use log::{debug, info};

pub struct RequestLogger;

impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestLoggerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggerMiddleware { service }))
    }
}

pub struct RequestLoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let method = req.method().clone();
        let path = req.path().to_string();
        let headers = req.headers().clone();

        let debug_enabled = log::max_level() >= log::Level::Debug;

        if debug_enabled {
            debug!("Request headers: {:?}", headers);
        }

        let future = self.service.call(req);

        Box::pin(async move {
            let res = future.await?;
            let elapsed = start.elapsed();
            let status = res.status().as_u16();

            // Log basic request/response at INFO level
            info!(
                "{} {} -> {} ({}ms)",
                method,
                path,
                status,
                elapsed.as_millis()
            );

            if debug_enabled {
                let res_headers = res.headers().clone();
                debug!("Response headers: {:?}", res_headers);
            }

            Ok(res)
        })
    }
}
