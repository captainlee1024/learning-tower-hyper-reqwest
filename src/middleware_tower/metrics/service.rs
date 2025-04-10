use crate::middleware_tower::metrics::body::MetricsResponseBody;
use crate::middleware_tower::metrics::future::MetricsResponseFuture;
use crate::middleware_tower::metrics::layer::MetricsLayer;
use http::{Request, Response};
use http_body::Body;
use opentelemetry::metrics::{Counter, Histogram};
use std::task::{Context, Poll};
use std::time::Instant;
use tower::Service;

#[derive(Clone, Debug)]
pub struct MetricsService<S> {
    pub inner: S,
    request_counter: Counter<u64>,
    request_duration: Histogram<f64>,
}

#[allow(unused)]
impl<S> MetricsService<S> {
    pub fn new(inner: S) -> Self {
        let meter = opentelemetry::global::meter("hyper-tower-service");
        let request_counter = meter
            .u64_counter("http_requests_total")
            .with_description("Total number of HTTP requests")
            .build();
        let request_duration = meter
            .f64_histogram("http_request_duration_seconds")
            .with_description("HTTP request duration in seconds")
            .build();
        Self {
            inner,
            request_counter,
            request_duration,
        }
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

    /// Returns a new [`Layer`] that wraps services with a `MetricsLayer` middleware.
    ///
    /// [`Layer`]: tower_layer::Layer
    pub fn layer() -> MetricsLayer {
        MetricsLayer::new()
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for MetricsService<S>
where
    ResBody: Body,
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = Response<MetricsResponseBody<ResBody>>;
    type Error = S::Error;
    type Future = MetricsResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let start = Instant::now();
        let method = req.method().to_string();

        let fut = self.inner.call(req);

        MetricsResponseFuture::new(
            fut,
            method,
            self.request_counter.clone(),
            self.request_duration.clone(),
            start,
        )
    }
}
