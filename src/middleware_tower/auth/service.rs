use crate::middleware_tower::auth::body::AuthResponseBody;
use crate::middleware_tower::auth::future::AuthResponseFuture;
use crate::middleware_tower::auth::layer::AuthLayer;
use http::{Request, Response};
use http_body::Body;
use std::task::{Context, Poll};
use tower::Service;
use tracing::{Level, event, field};
use tracing::{Span, instrument};

#[derive(Clone, Copy, Debug)]
pub struct AuthService<S> {
    pub inner: S,
}

#[allow(unused)]
impl<S> AuthService<S> {
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

    /// Returns a new [`Layer`] that wraps services with a `AuthService` middleware.
    ///
    /// [`Layer`]: tower_layer::Layer
    pub fn layer() -> AuthLayer {
        AuthLayer::new()
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for AuthService<S>
where
    ResBody: Body,
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    // auth需要拦截并立即返回Response, 需要定义一个具体ResponseBody类型
    // 所以需要AuthResponseBody来wrap 自己的ResponseBody和上游的ResponseBody
    type Response = Response<AuthResponseBody<ResBody>>;
    type Error = S::Error;
    // Future，在Future内调用wrap Response的逻辑
    type Future = AuthResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[instrument(skip_all, name="auth", fields(authorized = field::Empty), target = "middleware::auth")]
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let span = Span::current();
        // let authorized = req.headers().get("Authorization").is_some();
        // 适配swagger, 暂时使用自定义的Auth-Key 通过auth认证，Authorization是security内置key不让用
        let authorized = req.headers().get("Auth-Key").is_some();
        span.record("authorized", &authorized);

        if !authorized {
            event!(target: "middleware::auth", Level::WARN, "Unauthorized request");

            // return the custom response with [`AuthResponseBody`].
            //
            // [`AuthResponseBody`]: super::AuthResponseBody
            //
            return AuthResponseFuture::unauthorized();
        }

        let fut = self.inner.call(req);
        event!(target: "middleware::auth", Level::INFO, "Authorized request");

        AuthResponseFuture::new(fut)
    }
}
