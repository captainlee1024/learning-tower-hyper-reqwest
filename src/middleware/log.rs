use http::{Request, Response};
use http_body::Body;
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{Level, event, instrument};

pub struct LogLayer;

impl<S> Layer<S> for LogLayer {
    type Service = LogMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LogMiddleware { inner }
    }
}

pin_project! {
    pub struct LogMiddleware<S> {
        #[pin]
        inner: S,
    }
}

impl<S, B> Service<Request<B>> for LogMiddleware<S>
where
    B: Body + Send + 'static,
    S: Service<Request<B>, Response = Response<B>, Error = hyper::Error> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[instrument(skip(self, req), fields(layer = "log"))]
    fn call(&mut self, req: Request<B>) -> Self::Future {
        let method = req.method().clone();
        let uri = req.uri().clone();
        // let span = Span::current();

        let fut = self.inner.call(req);
        Box::pin(async move {
            let res = fut.await;
            event!(Level::INFO, %method, %uri, "Request handled");
            res
        })
    }
}
