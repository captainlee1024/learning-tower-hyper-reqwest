use crate::middleware_tower::metrics::MetricsResponseBody;
use http::Response;
use http_body::Body;
use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Histogram};
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll, ready};
use std::time::Instant;
use tracing::{Level, event, instrument};

pin_project! {
    /// Response future for [`MetricsService`].
    ///
    /// [`MetricsService`]: super::MetricsService
    pub struct MetricsResponseFuture<F> {
        #[pin]
        inner: F,
        #[pin]
        method: String,
        #[pin]
        request_counter: Counter<u64>,
        #[pin]
        request_duration: Histogram<f64>,
        #[pin]
        start_instant: Instant,
    }
}

impl<F> MetricsResponseFuture<F> {
    /// 包装上游Service的Future
    pub fn new(
        future: F,
        method: String,
        request_counter: Counter<u64>,
        request_duration: Histogram<f64>,
        start_instant: Instant,
    ) -> Self {
        Self {
            inner: future,
            method,
            request_counter,
            request_duration,
            start_instant,
        }
    }
}

impl<ResBody, F, E> Future for MetricsResponseFuture<F>
where
    ResBody: Body,
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<MetricsResponseBody<ResBody>>, E>;

    #[instrument(skip_all, name = "metrics", target = "middleware::metrics")]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let f = this.inner;

        let res = ready!(f.poll(cx))?.map(MetricsResponseBody::new);
        let method = this.method.clone();
        let elapsed = this.start_instant.elapsed();
        this.request_counter
            .add(1, &[KeyValue::new("method", method.clone())]);

        this.request_duration.record(
            elapsed.as_millis_f64(),
            &[KeyValue::new("method", method.clone())],
        );

        event!(target: "middleware::metrics", Level::INFO, %method, elapsed_ms = elapsed.as_millis_f64(), "Request metrics recorded");

        Poll::Ready(Ok(res))
    }
}
