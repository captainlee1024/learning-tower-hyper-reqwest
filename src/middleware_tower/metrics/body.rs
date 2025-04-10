use bytes::Bytes;
use http_body::{Body, Frame, SizeHint};
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    /// Response body for [`MetricsService`].
    ///
    /// [`MetricsService`]: super::MetricsService
    pub struct MetricsResponseBody<B> {
        #[pin]
        inner: B,
    }
}

impl<B> MetricsResponseBody<B> {
    pub(crate) fn new(body: B) -> Self {
        Self { inner: body }
    }
}

impl<B> Body for MetricsResponseBody<B>
where
    B: Body<Data = Bytes>,
{
    type Data = Bytes;
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let body = self.project().inner;
        // 这里是上游的ResponseBody
        body.poll_frame(cx)
    }

    fn is_end_stream(&self) -> bool {
        (&self.inner).is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        (&self.inner).size_hint()
    }
}
