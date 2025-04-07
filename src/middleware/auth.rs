use http::{Request, Response, StatusCode};
// use http_body::Body;
use http_body::Body;
// use hyper::body::Bytes;
// use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
// use tower_http::body::Full;
// use http_body_util::Full;
use tracing::{Level, Span, event, instrument};

#[derive(Clone)]
pub struct AuthLayer;

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware { inner }
    }
}

// 这里不需要pin
// pin_project! {
//     pub struct AuthMiddleware<S> {
//         #[pin]
//         inner: S,
//     }
// }

#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
}

// const BODY: &[u8] = b"length limit exceeded";

impl<S, ReqB, RespB> Service<Request<ReqB>> for AuthMiddleware<S>
where
    ReqB: Body + Send + 'static,
    RespB: Body + Default + Send + 'static,
    S: Service<Request<ReqB>, Response = Response<RespB>, Error = hyper::Error> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[instrument(skip(self, req), fields(layer = "auth"))]
    fn call(&mut self, req: Request<ReqB>) -> Self::Future {
        let span = Span::current();
        let authorized = req.headers().get("Authorization").is_some();

        if !authorized {
            span.record("authorized", &false);
            // let response = Response::builder()
            //     .status(StatusCode::UNAUTHORIZED);

            // FIXME: 直接使用req.into_body()有问题，如果echo定义的request和response的body类型不一致会导致
            // response类型不匹配
            let mut res = Response::new(RespB::default());
            *res.status_mut() = StatusCode::UNAUTHORIZED;

            return Box::pin(async move {
                // FIXME: 为什么这里的trace输出没有layer="auth"
                event!(Level::WARN, "Unauthorized request");
                // 效果同上
                // warn!("Unauthorized request");
                Ok(res)
            });
        }

        let fut = self.inner.call(req);
        Box::pin(async move {
            span.record("authorized", &true);
            event!(Level::INFO, "Authorized request");
            let res = fut.await;
            res
        })
    }
}
