// FIXME: CloneableService方案动态分发，会一直调用不到ServiceExt的call的接口方法
// use futures::future::BoxFuture;
// use http::{Method, Request, Response, StatusCode};
// use http_body::Body;
// use http_body_util::{BodyExt, Full, combinators::BoxBody};
// use hyper::body::{Bytes, Incoming};
// use std::collections::HashMap;
// use std::sync::Arc;
// use tower::{BoxError, Service, ServiceExt};
//
// // 定义新 trait，确保 dyn 兼容
// trait CloneableService<Req>: Service<Req> + Send + Sync + 'static {
//     fn clone_arc(
//         &self,
//     ) -> Arc<
//         dyn CloneableService<
//                 Req,
//                 Response = Self::Response,
//                 Error = Self::Error,
//                 Future = Self::Future,
//             >,
//     >;
// }
//
// impl<S, Req> CloneableService<Req> for S
// where
//     S: Service<Req> + Clone + Send + Sync + 'static,
//     S::Future: Send + 'static,
// {
//     fn clone_arc(
//         &self,
//     ) -> Arc<
//         dyn CloneableService<
//                 Req,
//                 Response = Self::Response,
//                 Error = Self::Error,
//                 Future = Self::Future,
//             >,
//     > {
//         Arc::new(self.clone())
//     }
// }
//
// pub trait IntoResponse {
//     fn into_response(self) -> Response<BoxBody<Bytes, BoxError>>;
// }
//
// impl IntoResponse for String {
//     fn into_response(self) -> Response<BoxBody<Bytes, BoxError>> {
//         let body = Full::new(Bytes::from(self)).map_err(|_| unreachable!() as BoxError);
//         Response::new(body.boxed())
//     }
// }
//
// impl<B> IntoResponse for Response<B>
// where
//     B: http_body::Body<Data = Bytes, Error = BoxError> + Send + Sync + 'static,
// {
//     fn into_response(self) -> Response<BoxBody<Bytes, BoxError>> {
//         self.map(|body| body.boxed())
//     }
// }
//
// // 提取器
// pub struct BodyString(pub String);
//
// impl BodyString {
//     pub async fn extract(req: Request<Incoming>) -> Result<Self, BoxError> {
//         let (_parts, body) = req.into_parts();
//         let collected = body.collect().await.map_err(BoxError::from)?;
//         Ok(BodyString(String::from_utf8(
//             collected.to_bytes().to_vec(),
//         )?))
//     }
// }
//
// pub trait Extractable: Clone + Send + Sync + 'static {
//     fn extract(req: Request<Incoming>) -> impl Future<Output = Result<Self, BoxError>> + Send
//     where
//         Self: Sized;
// }
//
// impl Extractable for BodyString {
//     fn extract(req: Request<Incoming>) -> impl Future<Output = Result<Self, BoxError>> + Send {
//         BodyString::extract(req)
//     }
// }
//
// impl Clone for BodyString {
//     fn clone(&self) -> Self {
//         BodyString(self.0.clone())
//     }
// }
//
// // 适配器
// #[derive(Clone)]
// struct ExtractService<S, E> {
//     inner: S,
//     _extractor: std::marker::PhantomData<E>,
// }
//
// impl<S, E, Resp> Service<Request<Incoming>> for ExtractService<S, E>
// where
//     S: Service<E, Response = Resp, Error = BoxError> + Send + Sync + Clone + 'static,
//     S::Future: Send + 'static,
//     E: Extractable,
//     Resp: IntoResponse + 'static,
// {
//     type Response = Response<BoxBody<Bytes, BoxError>>;
//     type Error = BoxError;
//     type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
//
//     fn poll_ready(
//         &mut self,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         self.inner.poll_ready(cx)
//     }
//
//     fn call(&mut self, req: Request<Incoming>) -> Self::Future {
//         let extract_fut = E::extract(req);
//         let mut inner = self.inner.clone();
//         Box::pin(async move {
//             let extracted = extract_fut.await?;
//             inner.call(extracted).await.map(|resp| resp.into_response())
//         })
//     }
// }
//
// // 路由器
// #[derive(Clone)]
// pub struct Router {
//     routes: HashMap<
//         (Method, String),
//         Arc<
//             dyn CloneableService<
//                     Request<Incoming>,
//                     Response = Response<BoxBody<Bytes, BoxError>>,
//                     Error = BoxError,
//                     Future = BoxFuture<
//                         'static,
//                         Result<Response<BoxBody<Bytes, BoxError>>, BoxError>,
//                     >,
//                 >,
//         >,
//     >,
// }
//
// impl Router {
//     pub fn new() -> Self {
//         Router {
//             routes: HashMap::new(),
//         }
//     }
//
//     pub fn at<S, E, Resp>(mut self, path: &str, method: Method, service: S) -> Self
//     where
//         S: Service<E, Response = Resp, Error = BoxError> + Send + Sync + Clone + 'static,
//         S::Future: Send + 'static,
//         E: Extractable,
//         Resp: IntoResponse + 'static,
//     {
//         let path = normalize_path(path);
//         let adapted = ExtractService {
//             inner: service,
//             _extractor: std::marker::PhantomData,
//         };
//         self.routes.insert((method, path), Arc::new(adapted));
//         self
//     }
//
//     pub fn get<S, E, Resp>(self, path: &str, service: S) -> Self
//     where
//         S: Service<E, Response = Resp, Error = BoxError> + Send + Sync + Clone + 'static,
//         S::Future: Send + 'static,
//         E: Extractable,
//         Resp: IntoResponse + 'static,
//     {
//         self.at(path, Method::GET, service)
//     }
//
//     pub fn post<S, E, Resp>(self, path: &str, service: S) -> Self
//     where
//         S: Service<E, Response = Resp, Error = BoxError> + Send + Sync + Clone + 'static,
//         S::Future: Send + 'static,
//         E: Extractable,
//         Resp: IntoResponse + 'static,
//     {
//         self.at(path, Method::POST, service)
//     }
//
//     pub fn nest(self, prefix: &str, sub_router: Router) -> Self {
//         let prefix = normalize_path(prefix);
//         let mut new_router = self;
//         for ((method, sub_path), service) in sub_router.routes {
//             let full_path = format!("{}{}", prefix, sub_path);
//             new_router.routes.insert((method, full_path), service);
//         }
//         new_router
//     }
// }
//
// fn normalize_path(path: &str) -> String {
//     let path = path.trim_end_matches('/');
//     if path.starts_with('/') {
//         path.to_string()
//     } else {
//         format!("/{}", path)
//     }
// }
//
// impl Service<Request<Incoming>> for Router {
//     type Response = Response<BoxBody<Bytes, BoxError>>;
//     type Error = BoxError;
//     type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
//
//     fn poll_ready(
//         &mut self,
//         _cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         std::task::Poll::Ready(Ok(()))
//     }
//
//     fn call(&mut self, req: Request<Incoming>) -> Self::Future {
//         let method = req.method().clone();
//         let path = normalize_path(req.uri().path());
//         if let Some(service) = self.routes.get(&(method, path)) {
//             let service = service.clone();
//             Box::pin(async move {
//                 // ServiceExt::call(&*service, req).await // 显式调用 ServiceExt::call
//                 // service.call(req).await
//                 (&*service).call(req).await
//             })
//         } else {
//             Box::pin(async {
//                 let body =
//                     Full::new(Bytes::from("Not Found")).map_err(|_| unreachable!() as BoxError);
//                 Ok(Response::builder()
//                     .status(StatusCode::NOT_FOUND)
//                     .body(body.boxed())?)
//             })
//         }
//     }
// }

// FIXME: Arc<T>，保持类型安全和 Clone 支持, 未能实现，问题同 CloneableService 方案遇到的问题
// use futures::future::BoxFuture;
// use http::{Method, Request, Response, StatusCode};
// use http_body::Body;
// use http_body_util::{BodyExt, Full, combinators::BoxBody};
// use hyper::body::{Bytes, Incoming};
// use std::collections::HashMap;
// use std::sync::Arc;
// use tower::{BoxError, Service, ServiceExt};
//
// pub trait IntoResponse {
//     fn into_response(self) -> Response<BoxBody<Bytes, BoxError>>;
// }
//
// impl IntoResponse for String {
//     fn into_response(self) -> Response<BoxBody<Bytes, BoxError>> {
//         let body = Full::new(Bytes::from(self)).map_err(|_| unreachable!() as BoxError);
//         Response::new(body.boxed())
//     }
// }
//
// impl<B> IntoResponse for Response<B>
// where
//     B: http_body::Body<Data = Bytes, Error = BoxError> + Send + Sync + 'static,
// {
//     fn into_response(self) -> Response<BoxBody<Bytes, BoxError>> {
//         self.map(|body| body.boxed())
//     }
// }
//
// // 提取器
// pub struct BodyString(pub String);
//
// impl BodyString {
//     pub async fn extract(req: Request<Incoming>) -> Result<Self, BoxError> {
//         let (_parts, body) = req.into_parts();
//         let collected = body.collect().await.map_err(BoxError::from)?;
//         Ok(BodyString(String::from_utf8(
//             collected.to_bytes().to_vec(),
//         )?))
//     }
// }
//
// pub trait Extractable: Clone + Send + Sync + 'static {
//     fn extract(req: Request<Incoming>) -> impl Future<Output = Result<Self, BoxError>> + Send
//     where
//         Self: Sized;
// }
//
// impl Extractable for BodyString {
//     fn extract(req: Request<Incoming>) -> impl Future<Output = Result<Self, BoxError>> + Send {
//         BodyString::extract(req)
//     }
// }
//
// impl Clone for BodyString {
//     fn clone(&self) -> Self {
//         BodyString(self.0.clone())
//     }
// }
//
// // 适配器
// #[derive(Clone)]
// struct ExtractService<S, E> {
//     inner: S,
//     _extractor: std::marker::PhantomData<E>,
// }
//
// impl<S, E, Resp> Service<Request<Incoming>> for ExtractService<S, E>
// where
//     S: Service<E, Response = Resp, Error = BoxError> + Send + Sync + Clone + 'static,
//     S::Future: Send + 'static,
//     E: Extractable,
//     Resp: IntoResponse + 'static,
// {
//     type Response = Response<BoxBody<Bytes, BoxError>>;
//     type Error = BoxError;
//     type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
//
//     fn poll_ready(
//         &mut self,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         self.inner.poll_ready(cx)
//     }
//
//     fn call(&mut self, req: Request<Incoming>) -> Self::Future {
//         let extract_fut = E::extract(req);
//         let mut inner = self.inner.clone();
//         Box::pin(async move {
//             let extracted = extract_fut.await?;
//             inner.call(extracted).await.map(|resp| resp.into_response())
//         })
//     }
// }
//
// // 路由器
// #[derive(Clone)]
// pub struct Router {
//     routes: HashMap<
//         (Method, String),
//         Arc<
//             dyn Service<
//                     Request<Incoming>,
//                     Response = Response<BoxBody<Bytes, BoxError>>,
//                     Error = BoxError,
//                     Future = BoxFuture<
//                         'static,
//                         Result<Response<BoxBody<Bytes, BoxError>>, BoxError>,
//                     >,
//                 > + Send
//                 + Sync
//                 + 'static,
//         >,
//     >,
// }
//
// impl Router {
//     pub fn new() -> Self {
//         Router {
//             routes: HashMap::new(),
//         }
//     }
//
//     pub fn at<S, E, Resp>(mut self, path: &str, method: Method, service: S) -> Self
//     where
//         S: Service<E, Response = Resp, Error = BoxError> + Send + Sync + Clone + 'static,
//         S::Future: Send + 'static,
//         E: Extractable,
//         Resp: IntoResponse + 'static,
//     {
//         let path = normalize_path(path);
//         let adapted = ExtractService {
//             inner: service,
//             _extractor: std::marker::PhantomData,
//         };
//         self.routes.insert((method, path), Arc::new(adapted));
//         self
//     }
//
//     pub fn get<S, E, Resp>(self, path: &str, service: S) -> Self
//     where
//         S: Service<E, Response = Resp, Error = BoxError> + Send + Sync + Clone + 'static,
//         S::Future: Send + 'static,
//         E: Extractable,
//         Resp: IntoResponse + 'static,
//     {
//         self.at(path, Method::GET, service)
//     }
//
//     pub fn post<S, E, Resp>(self, path: &str, service: S) -> Self
//     where
//         S: Service<E, Response = Resp, Error = BoxError> + Send + Sync + Clone + 'static,
//         S::Future: Send + 'static,
//         E: Extractable,
//         Resp: IntoResponse + 'static,
//     {
//         self.at(path, Method::POST, service)
//     }
//
//     pub fn nest(self, prefix: &str, sub_router: Router) -> Self {
//         let prefix = normalize_path(prefix);
//         let mut new_router = self;
//         for ((method, sub_path), service) in sub_router.routes {
//             let full_path = format!("{}{}", prefix, sub_path);
//             new_router.routes.insert((method, full_path), service);
//         }
//         new_router
//     }
// }
//
// fn normalize_path(path: &str) -> String {
//     let path = path.trim_end_matches('/');
//     if path.starts_with('/') {
//         path.to_string()
//     } else {
//         format!("/{}", path)
//     }
// }
//
// impl Service<Request<Incoming>> for Router {
//     type Response = Response<BoxBody<Bytes, BoxError>>;
//     type Error = BoxError;
//     type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
//
//     fn poll_ready(
//         &mut self,
//         _cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         std::task::Poll::Ready(Ok(()))
//     }
//
//     fn call(&mut self, req: Request<Incoming>) -> Self::Future {
//         let method = req.method().clone();
//         let path = normalize_path(req.uri().path());
//         if let Some(service) = self.routes.get(&(method, path)) {
//             let service = service.clone();
//             Box::pin(async move {
//                 service.call(req).await // 使用 ServiceExt::call
//             })
//         } else {
//             Box::pin(async {
//                 let body =
//                     Full::new(Bytes::from("Not Found")).map_err(|_| unreachable!() as BoxError);
//                 Ok(Response::builder()
//                     .status(StatusCode::NOT_FOUND)
//                     .body(body.boxed())?)
//             })
//         }
//     }
// }

// 1. 放弃动态分发，使用具体类型:
// 将 Router 的 routes 从 Arc<dyn Service<...>> 改为存储具体类型（如 Arc<ExtractService<S, E>>），通过泛型或类型擦除解决。
//
// 2. 使用 Box 代替 Arc:
// axum 使用 Box<dyn Service<...>>，在调用时手动管理服务实例。
//
// 但这可能需要重新设计 Router 的克隆逻辑。
//
// 3. 手动克隆服务:
// 在 call 中，通过某种方式（如 Arc::clone 或自定义克隆方法）获取服务实例，避免直接调用 call(&mut self)。
//
// 采用方案1
use futures::future::BoxFuture;
use http::{Method, Request, Response, StatusCode};
use http_body::Body;
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::body::{Bytes, Incoming};
use std::collections::HashMap;
use std::sync::Arc;
use tower::{BoxError, Service, ServiceExt};

pub trait IntoResponse {
    fn into_response(self) -> Response<BoxBody<Bytes, BoxError>>;
}

impl IntoResponse for String {
    fn into_response(self) -> Response<BoxBody<Bytes, BoxError>> {
        let body = Full::new(Bytes::from(self)).map_err(|_| unreachable!() as BoxError);
        Response::new(body.boxed())
    }
}

impl<B> IntoResponse for Response<B>
where
    B: http_body::Body<Data = Bytes, Error = BoxError> + Send + Sync + 'static,
{
    fn into_response(self) -> Response<BoxBody<Bytes, BoxError>> {
        self.map(|body| body.boxed())
    }
}

// 提取器
pub struct BodyString(pub String);

impl BodyString {
    pub async fn extract(req: Request<Incoming>) -> Result<Self, BoxError> {
        let (_parts, body) = req.into_parts();
        let collected = body.collect().await.map_err(BoxError::from)?;
        Ok(BodyString(String::from_utf8(
            collected.to_bytes().to_vec(),
        )?))
    }
}

pub trait Extractable: Clone + Send + Sync + 'static {
    fn extract(req: Request<Incoming>) -> impl Future<Output = Result<Self, BoxError>> + Send
    where
        Self: Sized;
}

impl Extractable for BodyString {
    fn extract(req: Request<Incoming>) -> impl Future<Output = Result<Self, BoxError>> + Send {
        BodyString::extract(req)
    }
}

impl Clone for BodyString {
    fn clone(&self) -> Self {
        BodyString(self.0.clone())
    }
}

// 适配器
#[derive(Clone)]
pub struct ExtractService<S, E> {
    inner: S,
    _extractor: std::marker::PhantomData<E>,
}

impl<S, E, Resp> Service<Request<Incoming>> for ExtractService<S, E>
where
    S: Service<E, Response = Resp, Error = BoxError> + Send + Sync + Clone + 'static,
    S::Future: Send + 'static,
    E: Extractable,
    Resp: IntoResponse + 'static,
{
    type Response = Response<BoxBody<Bytes, BoxError>>;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        let extract_fut = E::extract(req);
        let mut inner = self.inner.clone();
        Box::pin(async move {
            let extracted = extract_fut.await?;
            inner.call(extracted).await.map(|resp| resp.into_response())
        })
    }
}

// 路由器
#[derive(Clone)]
pub struct Router<S> {
    routes: HashMap<(Method, String), Arc<S>>,
}

impl<S> Router<S>
where
    S: Service<Request<Incoming>, Response = Response<BoxBody<Bytes, BoxError>>, Error = BoxError>
        + Send
        + Sync
        + Clone
        + 'static,
    S::Future: Send + 'static,
{
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    pub fn at<T, E, Resp>(mut self, path: &str, method: Method, service: T) -> Self
    where
        T: Service<E, Response = Resp, Error = BoxError> + Send + Sync + Clone + 'static,
        T::Future: Send + 'static,
        E: Extractable,
        Resp: IntoResponse + 'static,
        S: std::convert::From<ExtractService<T, E>>,
    {
        let path = normalize_path(path);
        let adapted = ExtractService {
            inner: service,
            _extractor: std::marker::PhantomData,
        };
        self.routes.insert((method, path), Arc::new(adapted.into()));
        self
    }

    pub fn get<T, E, Resp>(self, path: &str, service: T) -> Self
    where
        T: Service<E, Response = Resp, Error = BoxError> + Send + Sync + Clone + 'static,
        T::Future: Send + 'static,
        E: Extractable,
        Resp: IntoResponse + 'static,
        S: std::convert::From<ExtractService<T, E>>,
    {
        self.at(path, Method::GET, service)
    }

    pub fn post<T, E, Resp>(self, path: &str, service: T) -> Self
    where
        T: Service<E, Response = Resp, Error = BoxError> + Send + Sync + Clone + 'static,
        T::Future: Send + 'static,
        E: Extractable,
        Resp: IntoResponse + 'static,
        S: std::convert::From<ExtractService<T, E>>,
    {
        self.at(path, Method::POST, service)
    }

    pub fn nest(self, prefix: &str, sub_router: Router<S>) -> Self {
        let prefix = normalize_path(prefix);
        let mut new_router = self;
        for ((method, sub_path), service) in sub_router.routes {
            let full_path = format!("{}{}", prefix, sub_path);
            new_router.routes.insert((method, full_path), service);
        }
        new_router
    }
}

fn normalize_path(path: &str) -> String {
    let path = path.trim_end_matches('/');
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    }
}

impl<S> Service<Request<Incoming>> for Router<S>
where
    S: Service<Request<Incoming>, Response = Response<BoxBody<Bytes, BoxError>>, Error = BoxError>
        + Send
        + Sync
        + Clone
        + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<BoxBody<Bytes, BoxError>>;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        let method = req.method().clone();
        let path = normalize_path(req.uri().path());
        if let Some(service) = self.routes.get(&(method, path)) {
            let service = service.clone();
            Box::pin(async move {
                tower::ServiceExt::call(&*service, req).await // 显式调用 ServiceExt::call
            })
        } else {
            Box::pin(async {
                let body =
                    Full::new(Bytes::from("Not Found")).map_err(|_| unreachable!() as BoxError);
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(body.boxed())?)
            })
        }
    }
}
