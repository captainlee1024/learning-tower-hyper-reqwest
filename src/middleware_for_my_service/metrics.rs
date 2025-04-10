use http::{Request, Response};
// use pin_project_lite::pin_project;
use opentelemetry::KeyValue;
use opentelemetry::metrics::{Counter, Histogram};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};
use tracing::{Level, event, instrument};

#[derive(Clone)]
pub struct MetricsLayer;

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsMiddleware::new(inner)
    }
}

// 不需要pin
// pin_project! {
//     pub struct MetricsMiddleware<S> {
//         #[pin]
//         inner: S,
//     }
// }

#[derive(Clone)]
pub struct MetricsMiddleware<S> {
    inner: S,
    request_counter: Counter<u64>,
    request_duration: Histogram<f64>,
}

impl<S> MetricsMiddleware<S> {
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
}

impl<S, ReqB, RespB> Service<Request<ReqB>> for MetricsMiddleware<S>
where
    S: Service<Request<ReqB>, Response = Response<RespB>, Error = hyper::Error> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[instrument(skip(self, req), fields(layer = "metrics"))]
    fn call(&mut self, req: Request<ReqB>) -> Self::Future {
        let start = Instant::now();
        let method = req.method().to_string();
        let fut = self.inner.call(req);
        self.request_counter
            .add(1, &[KeyValue::new("method", method.clone())]);
        let request_duration = self.request_duration.clone();

        Box::pin(async move {
            let res = fut.await;
            let elapsed = start.elapsed();
            request_duration.record(
                elapsed.as_millis_f64(),
                &[KeyValue::new("method", method.clone())],
            );
            // event!(Level::INFO, %method, elapsed_ms = elapsed.as_millis(), "Request metrics recorded");
            event!(target: "middleware_for_my_service::metrics", Level::INFO, %method, elapsed_us = elapsed.as_micros(), "Request metrics recorded");
            res
        })
    }
}
