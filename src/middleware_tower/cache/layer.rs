use crate::middleware_tower::cache::service::CacheService;
use tower::Layer;

#[derive(Debug, Clone, Copy)]
pub struct CacheLayer;

impl CacheLayer {
    pub fn new() -> Self {
        Self
    }
}

#[allow(unused)]
impl<S> Layer<S> for CacheLayer {
    type Service = CacheService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheService { inner }
    }
}
