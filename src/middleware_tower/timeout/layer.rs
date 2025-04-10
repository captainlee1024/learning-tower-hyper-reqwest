use crate::middleware_tower::timeout::service::TimeoutService;
use std::time::Duration;
use tower::Layer;

#[derive(Debug, Clone, Copy)]
pub struct TimeoutLayer {
    timeout: Duration,
}

impl TimeoutLayer {
    pub fn new(timeout: Duration) -> Self {
        TimeoutLayer { timeout }
    }
}

#[allow(unused)]
impl<S> Layer<S> for TimeoutLayer {
    type Service = TimeoutService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TimeoutService {
            inner,
            timeout: self.timeout,
        }
    }
}
