use crate::middleware_tower::timeout::body::TimeoutResponseBody;
use crate::middleware_tower::timeout::future::TimeoutResponseFuture;
use crate::middleware_tower::timeout::layer::TimeoutLayer;
use http::{Request, Response};
use http_body::Body;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time;
use tower::Service;

#[derive(Clone, Copy, Debug)]
pub struct TimeoutService<S> {
    pub inner: S,
    pub timeout: Duration,
}

#[allow(unused)]
impl<S> TimeoutService<S> {
    pub fn new(inner: S, timeout: Duration) -> Self {
        Self { inner, timeout }
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

    /// Returns a new [`Layer`] that wraps services with a `TimeoutLayer` middleware.
    ///
    /// [`Layer`]: tower_layer::Layer
    pub fn layer(timeout: Duration) -> TimeoutLayer {
        TimeoutLayer::new(timeout)
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for TimeoutService<S>
where
    ResBody: Body,
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = Response<TimeoutResponseBody<ResBody>>;
    type Error = S::Error;
    type Future = TimeoutResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let time_duratiom = self.timeout.as_micros();
        let sleep = time::sleep(self.timeout);

        let fut = self.inner.call(req);

        TimeoutResponseFuture::new(fut, sleep, time_duratiom)
    }
}
