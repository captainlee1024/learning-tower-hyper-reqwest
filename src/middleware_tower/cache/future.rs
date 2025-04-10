use crate::middleware_tower::cache::CacheResponseBody;
use http::Response;
use http_body::Body;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

pin_project! {
    /// Response future for [`CacheService`].
    ///
    /// [`CacheService`]: super::CacheService
    pub struct CacheResponseFuture<F> {
        #[pin]
        inner: F,
    }
}

impl<F> CacheResponseFuture<F> {
    /// 包装上游Service的Future
    pub fn new(future: F) -> Self {
        Self { inner: future }
    }
}

impl<ResBody, F, E> Future for CacheResponseFuture<F>
where
    ResBody: Body,
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<CacheResponseBody<ResBody>>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let f = this.inner;

        // 同样的问题，需要先记录再执行后续，会导致每次poll都记录
        // event!(target: "middleware::cache", Level::INFO, "Cache checked (no real cache in demo)");

        let res = ready!(f.poll(cx))?.map(CacheResponseBody::new);

        Poll::Ready(Ok(res))
    }
}
