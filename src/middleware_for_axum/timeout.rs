use std::time::Duration;
pub(crate) use tower_http::timeout::TimeoutLayer;

// TODO: 标准的中间件可以直接给axum使用
pub fn timeout_layer() -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(1))
}
