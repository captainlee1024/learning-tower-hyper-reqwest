use http::{Request, Response};
// use http_body::Body;
use http_body_util::{BodyExt, combinators::BoxBody};
use hyper::body::Bytes;
// use tower::ServiceExt;
use tracing::{Span, info, instrument};
// pub fn create_service() -> impl tower::Service<
//     Request<hyper::body::Incoming>,
//     Response = Response<BoxBody<Bytes, hyper::Error>>,
//     Error = hyper::Error,
//     Future = impl Future<Output = Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>> + Send,
// > + Clone {
//     let base = tower::service_fn(echo);
//
//     ServiceBuilder::new()
//         // .layer(TracingLayer)
//         // .layer(LogLayer)
//         // .layer(MetricsLayer)
//         // .layer(AuthLayer)
//         // .layer(CacheLayer)
//         // .layer(timeout_layer)
//         // // .layer(retry_layer)
//         // .layer(ratelimit_layer)
//         .service(base)
// }
// #[instrument(skip(req), fields(layer = "auth"))]
// #[instrument(name = "echo", skip(req))]
#[instrument(skip(req), fields(layer = "echo"), target = "service::echo")]
pub async fn echo(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // 收集请求 body 并转换为 Bytes
    // let collected = req.collect().await?;
    // let body = collected.to_bytes();

    // let frame = req.into_body().frame();
    let span = Span::current();

    // let frame = req.into_body().map_frame(|frame| {

    // 配合 let _guard = span.enter(); 这里使用move
    let frame = req.into_body().map_frame(move |frame| {
        let _guard = span.enter();
        let frame = if let Ok(data) = frame.into_data() {
            let uppercased = data
                .iter()
                .map(|byte| byte.to_ascii_uppercase())
                .collect::<Bytes>();
            // FIXME: Tracing span只会追加到
            info!("Transformed data: {:?}", uppercased);
            uppercased
        } else {
            Bytes::new()
        };

        hyper::body::Frame::data(frame)
    });

    // 用于测试timeout middleware
    // tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    Ok(Response::new(frame.boxed()))
}
