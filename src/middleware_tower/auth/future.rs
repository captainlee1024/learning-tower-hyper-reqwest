use crate::middleware_tower::auth::AuthResponseBody;
use crate::middleware_tower::auth::create_unauthorized_response;
use http::Response;
use http_body::Body;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

pin_project! {
    /// Response future for [`AuthService`].
    ///
    /// [`AuthService`]: super::AuthService
    pub struct AuthResponseFuture<F> {
        #[pin]
        inner: ResponseFutureInner<F>,
    }
}

impl<F> AuthResponseFuture<F> {
    /// 返回我们的自定义的ResponseBody
    #[allow(unused)]
    pub fn unauthorized() -> Self {
        Self {
            inner: ResponseFutureInner::UnauthorizedFuture,
        }
    }

    /// 包装上游Service的Future
    pub fn new(future: F) -> Self {
        Self {
            inner: ResponseFutureInner::Future { future },
        }
    }
}

pin_project! {
    #[project = ResFutProj]
    enum ResponseFutureInner<F> {
        // 在Middleware中可能会有多种错误，每一种可能的错误都会构造一个Response
        // 但是这些Response的Body类型是相同的，我们指定的，所以在AuthResponseBody的Inner enum里就两个字段
        // 这里的Future Inner enum应当是有几种可能的内部错误就有几个字段, 这个字段需要一个对应的AuthResponseBody
        // 方法，这里对应的enum 字段会调用该方法构造Body payload信息
        // 所以这里的对应关系是 这里的一个字段对应AuthResponseBody的一个构造方法，Future同样对应一个构造方法
        UnauthorizedFuture,
        Future {
            #[pin]
            future: F,
        }
    }
}

impl<ResBody, F, E> Future for AuthResponseFuture<F>
where
    ResBody: Body,
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<AuthResponseBody<ResBody>>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().inner.project() {
            ResFutProj::UnauthorizedFuture => {
                let res = create_unauthorized_response();
                // event!(target: "middlewarer::auth", Level::WARN, "Unauthorized request");
                Poll::Ready(Ok(res))
            }
            ResFutProj::Future { future } => {
                // FIXME: 放在这里每次poll 都会打印
                // 还是应该放到Service call里打印
                // event!(target: "middleware::auth", Level::INFO, "Authorized request");
                let res = ready!(future.poll(cx))?.map(AuthResponseBody::new);
                Poll::Ready(Ok(res))
            }
        }
    }
}
