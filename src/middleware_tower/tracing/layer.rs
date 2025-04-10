use crate::middleware_tower::tracing::service::TracingService;
use tower::Layer;

#[derive(Clone, Copy, Debug)]
pub struct TracingLayer;

impl TracingLayer {
    pub fn new() -> Self {
        Self
    }
}

#[allow(unused)]
impl<S> Layer<S> for TracingLayer {
    type Service = TracingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TracingService { inner }
    }
}
