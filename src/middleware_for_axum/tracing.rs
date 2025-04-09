// src/middleware/tracing.rs
use http::{Request, Response};
use http_body::Body;
use std::convert::Infallible;
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

#[derive(Clone)]
pub struct TracingMiddleware<S> {
    inner: S,
}

impl<S, ReqB, RespB> Service<Request<ReqB>> for TracingMiddleware<S>
where
    ReqB: Body + Send + 'static,
    RespB: Body + Send + 'static,
    S: Service<Request<ReqB>, Response = Response<RespB>, Error = Infallible> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<RespB>;
    type Error = Infallible; // 与 Axum 的 Route 一致
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx) // S::Error 是 Infallible，无需转换
    }

    fn call(&mut self, req: Request<ReqB>) -> Self::Future {
        let method = req.method().to_string();
        let uri = req.uri().to_string();
        let info_span = info_span!("request", %method, %uri);
        let fut = self.inner.call(req);

        Box::pin(
            async move {
                let res = fut.await?; // S::Error 是 Infallible，不会失败
                if res.status().is_server_error() || res.status().is_client_error() {
                    tracing::warn!(target: "middleware_for_axum::tracing", "Request failed");
                } else {
                    tracing::info!(target: "middleware_for_axum::tracing", "Request succeeded");
                }
                Ok(res)
            }
            .instrument(info_span),
        )
    }
}
