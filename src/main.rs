mod app;
mod middleware;

use crate::app::echo;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use std::net::SocketAddr;
// use hyper::server::Server;
// use hyper::service::make_service_fn;
use tokio::net::TcpListener;
use tower::{ServiceBuilder, service_fn};
// use tracing_opentelemetry::OpenTelemetryLayer;
// use tracing_subscriber::Registry;
// use tracing_subscriber::layer::SubscriberExt;

/// A simple echo server using hyper and tower
///
/// how to run:
/// ```bash
/// cargo run
/// ```
///
/// test with curl:
/// ```bash
/// curl -v -X POST -H "Authorization: Bearer token" -d "hello world" http://127.0.0.1:3000
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 tracing + OTEL
    // let tracer = opentelemetry_jaeger::new_pipeline()

    // let tracer = opentelemetry_jaeger::new_agent_pipeline()
    //     .with_service_name("tower-demo")
    //     .install_simple()
    //     .expect("Jaeger init failed");
    //
    // let telemetry = OpenTelemetryLayer::with_tracer(tracer);
    // let subscriber = Registry::default().with(telemetry);
    // tracing::subscriber::set_global_default(subscriber).expect("set tracing subscriber failed");
    //
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    // let t_service = create_service();

    //
    // let t_service = ServiceBuilder::new().service(service_fn(echo));

    // 初始化 Tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO) // 设置日志级别
        // .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .init(); // 初始化控制台输出

    let t_service = ServiceBuilder::new()
        .layer(middleware::tracing::TracingLayer)
        .layer(middleware::metrics::MetricsLayer)
        .layer(middleware::auth::AuthLayer)
        // FIXME: 这里的body limit 中间件会导致Service<Request<Limited<ReqBody>> Request类型不一致
        // .layer(middleware::ratelimit::ratelimit_layer())
        .layer(middleware::cache::CacheLayer)
        .layer(middleware::timeout::timeout_layer())
        .service(service_fn(echo));

    // TowerToHyperService<ServiceFn<fn(Request<Incoming>) ->impl Future<Output = Result<Response<BoxBody<Bytes, Error>>, Error>> + Sized>>>
    let h_service = TowerToHyperService::new(t_service);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        // TODO: optimize: clone service
        let cloned_service = h_service.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, cloned_service)
                .await
            {
                println!("Error serving connection: {}", err);
            }
        });
    }

    // let make_svc = make_service_fn(|_conn| async { Ok::<_, hyper::Error>(create_service()) });
    // Server::bind(&addr).serve(make_svc).await.unwrap();
}
