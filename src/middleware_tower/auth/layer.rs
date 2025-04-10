use crate::middleware_tower::auth::service::AuthService;
use tower::Layer;

#[derive(Clone, Copy, Debug)]
pub struct AuthLayer;

impl AuthLayer {
    pub fn new() -> Self {
        Self
    }
}

#[allow(unused)]
impl<S> Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService { inner }
    }
}
