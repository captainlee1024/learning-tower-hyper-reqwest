use bytes::Bytes;
use http::{HeaderValue, Response, StatusCode};
use http_body::{Body, Frame, SizeHint};
use http_body_util::Full;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    /// Response body for [`TracingService`].
    ///
    /// [`TracingService`]: super::TracingService
    pub struct TracingResponseBody<B> {
        #[pin]
        inner: ResponseBodyInner<B>,
    }
}

impl<B> TracingResponseBody<B> {
    pub fn payload_example() -> Self {
        Self {
            inner: ResponseBodyInner::TracingCustomBody {
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

const BODY: &[u8] = b"TracingServiceCustomBody never used, just for example";

pin_project! {
/// 为什么会有这个enum
/// - 在不需要拦截request, 在某些情况下直接在当前中间件返回Response时，我们总是返回上游返回过来的Response
/// - 在需要拦截并有可能立即返回Response时，我们是Response的源头，需要在当前中间件自己构建，所以需要具体的类型
/// 该enum的作用是为了在需要拦截时，使用Full<Bytes>来构建ResponseBody, 务必实现Body
/// 这里只是一个示例代码，TracingService中并不会返回自定义的Response
/// Tracing没有自己的逻辑需要处理直接返回上游的Response
    #[project = ResponseBodyProj]
    enum ResponseBodyInner<B> {
        // 这是一个自定义的ResponseBody
        TracingCustomBody {
            #[pin]
            body: Full<Bytes>,
        },
        Body {
            #[pin]
            body: B,
        },
    }
}

impl<B> Body for TracingResponseBody<B>
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
            ResponseBodyProj::TracingCustomBody { body } => {
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
            ResponseBodyInner::TracingCustomBody { body } => body.is_end_stream(),
            ResponseBodyInner::Body { body } => body.is_end_stream(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match &self.inner {
            ResponseBodyInner::TracingCustomBody { body } => body.size_hint(),
            ResponseBodyInner::Body { body } => body.size_hint(),
        }
    }
}

// 构建当前middleware内想要拦截req并立即返回的Response
pub fn create_error_response<B>() -> Response<TracingResponseBody<B>>
where
    B: Body,
{
    let mut res = Response::new(TracingResponseBody::payload_example());
    *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

    const TEXT_PLAIN: HeaderValue = HeaderValue::from_static("text/plain; charset=utf-8");
    res.headers_mut()
        .insert(http::header::CONTENT_TYPE, TEXT_PLAIN);

    res
}
