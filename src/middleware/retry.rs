// TODO: Implement a retry middleware for the HTTP client
// use http::{Request, Response};
// use http_body::Body;
// use http_body_util::BodyExt;
// use std::future::Future;
// use std::pin::Pin;
// use std::task::{Context, Poll};
// use std::time::Duration;
// pub(crate) use tower::retry::RetryLayer;
// use tower::{Layer, ServiceBuilder, retry::Policy};
// use tracing::warn;
//
// #[derive(Clone)]
// pub struct RetryPolicy;
//
// impl<E, B> Policy<Request<B>, Response<B>, E> for RetryPolicy
// where
//     B: Body + Send + 'static,
//     Request<B>: Clone,
//     E: std::fmt::Debug + Send + Sync + 'static,
// {
//     type Future = futures::future::Ready<Self>;
//
//     fn retry(&self, _req: &Request<B>, result: Result<&Response<B>, &E>) -> Option<Self::Future> {
//         match result {
//             Ok(_) => None,
//             Err(err) => {
//                 warn!("Retrying due to error: {:?}", err);
//                 Some(futures::future::ready(Self))
//             }
//         }
//     }
//
//     fn clone_request(&self, req: &Request<B>) -> Option<Request<B>> {
//         Some(req.clone())
//     }
// }
//
// pub fn retry_layer() -> RetryLayer<RetryPolicy> {
//     RetryLayer::new(RetryPolicy)
// }
