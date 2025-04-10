use http::{Request, Response};
use http_body::Body;
// use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{Instrument, info_span};

#[derive(Clone)]
pub struct TracingLayer;

impl<S> Layer<S> for TracingLayer {
    type Service = TracingMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TracingMiddleware { inner }
    }
}

// 这里不需要pin
// pin_project! {
//     pub struct TracingMiddleware<S> {
//         #[pin]
//         inner: S,
//     }
// }

#[derive(Clone)]
pub struct TracingMiddleware<S> {
    inner: S,
}

impl<S, ReqB, RespB> Service<Request<ReqB>> for TracingMiddleware<S>
where
    ReqB: Body + Send + 'static,
    S: Service<Request<ReqB>, Response = Response<RespB>, Error = hyper::Error> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqB>) -> Self::Future {
        let method = req.method().to_string();
        let uri = req.uri().to_string();

        let info_span = info_span!("request", %method, %uri);
        // let warn_span = warn_span!("request", %method, %uri);
        let fut = self.inner.call(req);
        // Box::pin(fut.instrument(info_span))
        Box::pin(
            async move {
                let res = fut.await;
                if res.is_err() {
                    tracing::warn!(target: "middleware_for_my_service::tracing", "Request failed");
                } else {
                    tracing::info!(target: "middleware_for_my_service::tracing", "Request succeeded");
                }
                // tracing::info!("Request handled");
                res
            }
            .instrument(info_span),
        )
    }
}
