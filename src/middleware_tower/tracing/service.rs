//! Understanding the TracingService
//!
//! 这里我们对要wrap的service做了Trait bound
//! 1. wrap的S 必须是Tower::Service
//!    Tower::Service是一个Trait, 需要指定Request, Response, Error, Future这几个泛型参数和关联类型
//!    - Request: 指定Request泛型为 Http::Request<ReqBody>, Request为泛型结构体，需要指定RequestBody类型
//!      - 使用ReqBody泛型参数来确定这个Http::Request<ReqBody>的ReqBody泛型类型
//!         NOTE: 这里这个ReqBody泛型参数由上游Service传入, 我们这里不关心
//!    - Response: 指定Response泛型为 Http::Response<ResBody>, Response为泛型结构体，需要指定ResponseBody类型
//!      - 使用ResBody泛型参数来确定这个Http::Response<ResBody>的ResBody泛型类型
//!         NOTE: 这里我们自定义RespoonseBody类型为TracingResponseBody<ResBody>
//!         因为如果当前middleware service需要在middleware log逻辑不通过时立即返回Response,
//!         那这个类型就必须在当前middleware service中指定
//!         并且如果middleware service logic逻辑通过，需要返回上游service的Response时
//!         这个类型又必须使用上游service的ResponseBody类型，所以这里定义了一个wrapped的Response
//!         它里面包含了一个enum, enum里有当前我们立即返回是指定的具体的ResponseBody类型和不需要立即返回时
//!         上游的ResponseBody泛型类型，这个类型由上游service传入
//!     - Error: middleware中不返回自己的Error, 直接返回上游service的Error
//!     - Future: Response不是直接返回的，而是通过Future返回的，需要返回自定义Response类型时就需要自定义Future
//!         来把多种ResponseBody类型wrap起来放到这个Future里返回

use crate::middleware_tower::tracing::body::TracingResponseBody;
use crate::middleware_tower::tracing::future::TracingResponseFuture;
use crate::middleware_tower::tracing::layer::TracingLayer;
use http::{Request, Response};
use http_body::Body;
use std::task::{Context, Poll};
use tower::Service;
use tracing::instrument;

#[derive(Clone, Copy, Debug)]
pub struct TracingService<S> {
    pub inner: S,
}

#[allow(unused)]
impl<S> TracingService<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }

    /// Gets a reference to the underlying service.
    pub fn get_ref(&self) -> &S {
        &self.inner
    }

    /// Gets a mutable reference to the underlying service.
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Consumes `self`, returning the underlying service.
    pub fn into_inner(self) -> S {
        self.inner
    }

    /// Returns a new [`Layer`] that wraps services with a `TracingService` middleware.
    ///
    /// [`Layer`]: tower_layer::Layer
    pub fn layer() -> TracingLayer {
        TracingLayer::new()
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for TracingService<S>
where
    ResBody: Body,
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = Response<TracingResponseBody<ResBody>>;
    type Error = S::Error;
    type Future = TracingResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[instrument(
        skip_all,
        name = "request",
        fields(method = %req.method(), uri = %req.uri()),
    )]
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let method = req.method().to_string();
        let uri = req.uri().to_string();

        // other middleware logic
        // if error, return the custom response with [`TracingResponseBody`].
        //
        // [`TracingResponseBody`]: super::TracingResponseBody
        //
        // return TracingResponseFuture::payload_example();
        // 这里通过middleware logic之后，返回上有的service的call 的response

        tracing::info!(target: "middleware::tracing", "handling request");

        let fut = self.inner.call(req);

        TracingResponseFuture::new(fut, method, uri)
    }
}
