use bytes::Bytes;
use http::{HeaderValue, Response, StatusCode};
use http_body::{Body, Frame, SizeHint};
use http_body_util::Full;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    /// Response body for [`TimeoutService`].
    ///
    /// [`TimeoutService`]: super::TimeoutService
     pub struct TimeoutResponseBody<B> {
        #[pin]
        inner: ResponseBodyInner<B>,
    }
}

impl<B> TimeoutResponseBody<B> {
    pub fn payload_request_timeout() -> Self {
        Self {
            inner: ResponseBodyInner::TimeoutRespBody {
                body: Full::from(BODY),
            },
        }
    }

    pub(crate) fn new(body: B) -> Self {
        Self {
            inner: ResponseBodyInner::Body { body },
        }
    }
}

const BODY: &[u8] = b"Request timeout, please retry later";
pin_project! {
    #[project = ResponseBodyProj]
    enum ResponseBodyInner<B> {
        // 这是一个自定义的ResponseBody
        TimeoutRespBody {
            #[pin]
            body: Full<Bytes>,
        },
        Body {
            #[pin]
            body: B,
        },
    }
}

impl<B> Body for TimeoutResponseBody<B>
where
    B: Body<Data = Bytes>,
{
    type Data = Bytes;
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.project().inner.project() {
            ResponseBodyProj::TimeoutRespBody { body } => {
                // 这里是自定义的ResponseBody
                body.poll_frame(cx).map_err(|err| match err {})
            }
            ResponseBodyProj::Body { body } => {
                // 这里是上游的ResponseBody
                body.poll_frame(cx)
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        match &self.inner {
            ResponseBodyInner::TimeoutRespBody { body } => body.is_end_stream(),
            ResponseBodyInner::Body { body } => body.is_end_stream(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match &self.inner {
            ResponseBodyInner::TimeoutRespBody { body } => body.size_hint(),
            ResponseBodyInner::Body { body } => body.size_hint(),
        }
    }
}

pub fn create_request_timeout_response<B>() -> Response<TimeoutResponseBody<B>>
where
    B: Body,
{
    let mut res = Response::new(TimeoutResponseBody::payload_request_timeout());
    *res.status_mut() = StatusCode::REQUEST_TIMEOUT;

    const TEXT_PLAIN: HeaderValue = HeaderValue::from_static("text/plain; charset=utf-8");
    res.headers_mut()
        .insert(http::header::CONTENT_TYPE, TEXT_PLAIN);

    res
}
