use std::time::Duration;
pub(crate) use tower_http::timeout::TimeoutLayer;

pub fn timeout_layer() -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(1))
}
