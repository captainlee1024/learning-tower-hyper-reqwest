pub(crate) use tower_http::limit::RequestBodyLimitLayer;

// TODO: remove me
#[allow(dead_code)]
pub fn ratelimit_layer() -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(1000)
}
