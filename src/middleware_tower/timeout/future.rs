use crate::middleware_tower::timeout::TimeoutResponseBody;
use crate::middleware_tower::timeout::create_request_timeout_response;
use http::Response;
use http_body::Body;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll, ready};
use tokio::time::Sleep;
use tracing::{Level, event, instrument};

pin_project! {
    /// Response future for [`TimeoutService`].
    ///
    /// [`TimeoutService`]: super::TimeoutService
    pub struct TimeoutResponseFuture<F> {
        #[pin]
        inner: F,
        #[pin]
        sleep: Sleep,
        #[pin]
        time_duration: u128,
    }
}

impl<F> TimeoutResponseFuture<F> {
    /// 包装上游Service的Future
    pub fn new(future: F, sleep: Sleep, time_duration: u128) -> Self {
        Self {
            inner: future,
            sleep,
            time_duration,
        }
    }
}

impl<ResBody, F, E> Future for TimeoutResponseFuture<F>
where
    ResBody: Body,
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<TimeoutResponseBody<ResBody>>, E>;

    #[instrument(skip_all, name = "timeout", fields(timeout_duration_ms=%self.time_duration), target = "middleware::timeout")]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if this.sleep.poll(cx).is_ready() {
            let res = create_request_timeout_response();
            event!(target: "middleware::timeout", Level::WARN, "Request timeout");
            return Poll::Ready(Ok(res));
        }

        // this.inner.poll(cx)
        let res = ready!(this.inner.poll(cx))?.map(TimeoutResponseBody::new);
        event!(target: "middleware::timeout", Level::INFO, "Request completed in time");
        Poll::Ready(Ok(res))
    }
}
