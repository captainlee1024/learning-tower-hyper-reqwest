use crate::middleware_tower::metrics::service::MetricsService;
use tower::Layer;

#[derive(Clone, Copy, Debug)]
pub struct MetricsLayer;

impl MetricsLayer {
    pub fn new() -> Self {
        Self
    }
}

#[allow(unused)]
impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsService::new(inner)
    }
}
