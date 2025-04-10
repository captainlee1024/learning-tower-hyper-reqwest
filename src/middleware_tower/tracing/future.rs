//! Response future for [`TracingService`]
//!
//! [`TracingService`]: super::TracingService
//!
//! 为什么需要这个Future?
//!
//! 因为我们定义了一个自定义的ResponseBody，用于在当前middleware出错时立即返回Response,
//! 而ResponseBody是一个Trait, 使用时需要指定泛型。
//!
//! 所以从我们这里实际上会有两种不同的ResponseBody:
//! 1. 上游的ResponseBody, 这个是上游的ResponseBody
//! 2. 我们自定义的ResponseBody, 这个是我们自己实现的ResponseBody
//!
//! 定义ResponseFuture来处理这两种情况，统一ResponseBody的类型, 做两件事:
//! - 将上游的ResponseBody转换成我们自定义的ResponseBody并返回Response
//! - 构建回我们自定义的ResponseBody并返回Response

use crate::middleware_tower::tracing::TracingResponseBody;
use crate::middleware_tower::tracing::body::create_error_response;
use http::Response;
use http_body::Body;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll, ready};
use tracing::instrument;

pin_project! {
    /// Response future for [`TracingService`].
    ///
    /// [`TracingService`]: super::TracingService
    pub struct TracingResponseFuture<F> {
        #[pin]
        inner: ResponseFutureInner<F>,
        #[pin]
        method: String,
        #[pin]
        uri: String,
    }
}

impl<F> TracingResponseFuture<F> {
    /// 返回我们的自定义的ResponseBody
    #[allow(unused)]
    pub fn payload_example(method: String, uri: String) -> Self {
        Self {
            inner: ResponseFutureInner::PyaloadExample,
            method,
            uri,
        }
    }

    /// 包装上游Service的Future
    pub fn new(future: F, method: String, uri: String) -> Self {
        Self {
            inner: ResponseFutureInner::Future { future },
            method,
            uri,
        }
    }
}

pin_project! {
    #[project = ResFutProj]
    enum ResponseFutureInner<F> {
        PyaloadExample,
        Future {
            #[pin]
            future: F,
        }
    }
}

impl<ResBody, F, E> Future for TracingResponseFuture<F>
where
    ResBody: Body,
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<TracingResponseBody<ResBody>>, E>;

    #[instrument(
        skip_all,
        name = "request",
        target = "middleware::tracing",
        fields(method=%self.method, uri=%self.uri),
    )]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res = match self.project().inner.project() {
            ResFutProj::PyaloadExample => {
                // 这里是我们自定义的Response
                let res = create_error_response();
                tracing::warn!(
                    target: "middleware::tracing",
                    "Request failed, returning response with error payload(example payload)"
                );
                res
            }

            ResFutProj::Future { future } => {
                // 需要把上游的Response转换成我们的Response
                let res = ready!(future.poll(cx))?.map(TracingResponseBody::new);
                tracing::info!(target: "middleware::tracing", "handle successfully");
                res
            }
        };
        Poll::Ready(Ok(res))
    }
}
