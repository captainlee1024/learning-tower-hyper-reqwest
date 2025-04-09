use http::{Request, Response};
use http_body::Body;
// use pin_project_lite::pin_project;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{Level, event, instrument};

#[derive(Clone)]
pub struct CacheLayer;

impl<S> Layer<S> for CacheLayer {
    type Service = CacheMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheMiddleware { inner }
    }
}

//  不确定这里是否需要pin
// pin_project! {
//     pub struct CacheMiddleware<S> {
//         #[pin]
//         inner: S,
//     }
// }

#[derive(Clone)]
pub struct CacheMiddleware<S> {
    inner: S,
}

impl<S, ReqB, RespB> Service<Request<ReqB>> for CacheMiddleware<S>
where
    ReqB: Body + Send + 'static,
    S: Service<Request<ReqB>, Response = Response<RespB>, Error = Infallible> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = Infallible; // 与 Axum 的 Route 一致
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx) // S::Error 是 Infallible，无需转换
    }

    #[instrument(skip(self, req), fields(layer = "cache"))]
    fn call(&mut self, req: Request<ReqB>) -> Self::Future {
        let _span = tracing::Span::current();
        let fut = self.inner.call(req);

        Box::pin(async move {
            // 在这里可以添加缓存逻辑
            // 然后记录日志
            event!(target: "middleware_for_axum::cache", Level::INFO, "Cache checked (no real cache in demo)");
            // 然后再接着处理
            fut.await
        })
    }
}
