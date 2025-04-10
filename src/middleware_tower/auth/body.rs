use bytes::Bytes;
use http::{HeaderValue, Response, StatusCode};
use http_body::{Body, Frame, SizeHint};
use http_body_util::Full;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    /// Response body for [`AuthService`].
    ///
    /// [`AuthService`]: super::AuthService
    pub struct AuthResponseBody<B> {
        #[pin]
        inner: ResponseBodyInner<B>,
    }
}
const BODY: &[u8] = b"auth failed, please check your token";

impl<B> AuthResponseBody<B> {
    pub fn payload_unauthorized() -> Self {
        Self {
            inner: ResponseBodyInner::UnAuthorized {
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

pin_project! {
    #[project = ResponseBodyProj]
    enum ResponseBodyInner<B> {
        // auth 认证失败，构建Response时指定的Body类型
        UnAuthorized {
            #[pin]
            body: Full<Bytes>,
        },
        // 上游返回的Response，我们不关心具体类型
        Body {
            #[pin]
            body: B,
        },
    }
}

impl<B> Body for AuthResponseBody<B>
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
            ResponseBodyProj::UnAuthorized { body } => {
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
            ResponseBodyInner::UnAuthorized { body } => body.is_end_stream(),
            ResponseBodyInner::Body { body } => body.is_end_stream(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match &self.inner {
            ResponseBodyInner::UnAuthorized { body } => body.size_hint(),
            ResponseBodyInner::Body { body } => body.size_hint(),
        }
    }
}

pub fn create_unauthorized_response<B>() -> Response<AuthResponseBody<B>> {
    let mut res = Response::new(AuthResponseBody::payload_unauthorized());
    *res.status_mut() = StatusCode::UNAUTHORIZED;

    const TEXT_PLAIN: HeaderValue = HeaderValue::from_static("text/plain; charset=utf-8");
    res.headers_mut()
        .insert(http::header::CONTENT_TYPE, TEXT_PLAIN);

    res
}
