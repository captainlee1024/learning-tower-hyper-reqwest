use crate::middleware_tower::cache::body::CacheResponseBody;
use crate::middleware_tower::cache::future::CacheResponseFuture;
use crate::middleware_tower::cache::layer::CacheLayer;
use http::{Request, Response};
use http_body::Body;
use std::task::{Context, Poll};
use tower::Service;
use tracing::instrument;
use tracing::{Level, event};

#[derive(Clone, Copy, Debug)]
pub struct CacheService<S> {
    pub inner: S,
}

#[allow(unused)]
impl<S> CacheService<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }

    /// Gets a reference to the underlying service.
    pub fn get_ref(&self) -> &S {
        &self.inner
    }

    /// Gets a mutable reference to the underlying service.
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Consumes `self`, returning the underlying service.
    pub fn into_inner(self) -> S {
        self.inner
    }

    /// Returns a new [`Layer`] that wraps services with a `CacheService` middleware.
    ///
    /// [`Layer`]: tower_layer::Layer
    pub fn layer() -> CacheLayer {
        CacheLayer::new()
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for CacheService<S>
where
    ResBody: Body,
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = Response<CacheResponseBody<ResBody>>;
    type Error = S::Error;
    type Future = CacheResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[instrument(skip_all, name = "cache", target = "middleware::cache")]
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // 1. 缓存逻辑
        // TODO: 添加缓存组件

        let fut = self.inner.call(req);

        // 2. 记录日志
        event!(target: "middleware::cache", Level::INFO, "Cache checked (no real cache in demo)");

        CacheResponseFuture::new(fut)
    }
}
