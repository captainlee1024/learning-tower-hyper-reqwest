use http::{Request, Response};
// use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};
use tracing::{Level, event, instrument};

pub struct MetricsLayer;

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsMiddleware { inner }
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

        Box::pin(async move {
            let res = fut.await;
            let elapsed = start.elapsed();
            // event!(Level::INFO, %method, elapsed_ms = elapsed.as_millis(), "Request metrics recorded");
            event!(target: "middleware::metrics", Level::INFO, %method, elapsed_us = elapsed.as_micros(), "Request metrics recorded");
            res
        })
    }
}
