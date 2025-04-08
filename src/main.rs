#![feature(duration_millis_float)]

mod app;
mod middleware;

use crate::app::echo;
// use futures::SinkExt;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
// use opentelemetry_sdk::resource::ResourceBuilder;
use opentelemetry::global;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use std::net::SocketAddr;
// use hyper::server::Server;
// use hyper::service::make_service_fn;
use tokio::net::TcpListener;
use tower::{ServiceBuilder, service_fn};
// use tracing::instrument::WithSubscriber;
// use tracing_opentelemetry::OpenTelemetryLayer;
// use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

// use tracing_opentelemetry::OpenTelemetryLayer;
// use tracing_subscriber::Registry;
// use tracing_subscriber::layer::SubscriberExt;

/// # hyper-tower-echo-demo
///
/// A simple echo server using hyper and tower
///
/// ## how to use:
///
/// 1、launch the Jaeger agent using docker:
///
/// ```bash
/// docker run -d --name jaeger \
/// -e COLLECTOR_OTLP_ENABLED=true \
/// -p 6831:6831/udp \
/// -p 16686:16686 \
/// -p 4317:4317 \
/// jaegertracing/all-in-one:latest
/// ```
///
/// stop and restart the Jaeger agent:
///
/// ```bash
/// docker stop jaeger
/// docker start jaeger
/// ```
///
/// 2、launch the echo server:
///
/// ```bash
/// cargo run
/// ```
///
/// 3、test with curl:
///
/// ```bash
/// curl -v -X POST -H "Authorization: Bearer token" -d "hello world" http://127.0.0.1:3000
/// ```
///
/// 4、check the trace in Jaeger UI:
///
/// [open Jaeger UI in browser](http://localhost:16686/)
///
/// select the Service name `hyper-tower-service` and select the Operation name `request`, click `Find Traces` to see the
/// traces.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // // 初始化 Tracing
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::INFO) // 设置日志级别
    //     // .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
    //     .init(); // 初始化控制台输出
    //
    // 初始化Tracing和OpenTelemetry
    init_tracing().await?;

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

    tracing::info!(
        target: "server::startup",
        service_name = "echo-server",
        service_protocol = "http",
        service_address = %addr,
        "HTTP service is now listening on {} (Powered by hyper and tower), press Ctrl+C to stop",
        addr
    );

    // let t_service = create_service();

    //
    // let t_service = ServiceBuilder::new().service(service_fn(echo));

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
                tracing::error!("Error serving connection: {}", err);
            }
        });
    }

    // let make_svc = make_service_fn(|_conn| async { Ok::<_, hyper::Error>(create_service()) });
    // Server::bind(&addr).serve(make_svc).await.unwrap();
}

// 初始化 Tracing和OpenTelemetry
async fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    // 配置 tracer的 OTLP 导出器
    // Initialize OTLP exporter using gRPC (Tonic)
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        // .with_tonic()
        // .build()?;
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    // Create a tracer provider with the exporter
    // let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
    //     .with_simple_exporter(otlp_exporter)
    //     .build();

    // 配置 TracerProvider
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_resource(
            Resource::builder()
                .with_service_name("hyper-tower-service")
                .build(),
        )
        .build();
    // info!("TracerProvider created");

    // let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
    //     .with_batch_exporter(otlp_exporter, opentelemetry_sdk::runtime::Tokio)
    //     .with_config(opentelemetry_sdk::trace::Config::default().with_resource(
    //         opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
    //             "service.name",
    //             "hyper-tower-service",
    //         )]),
    //     ))
    //     .build();

    let provider_tracer = tracer_provider.tracer("hyper-tower-service");

    // 设置全局 TracerProvider（追踪）
    // opentelemetry::global::set_meter_provider(tracer_provider);
    opentelemetry::global::set_tracer_provider(tracer_provider);

    // 配置Metrics
    // 配置 Metrics 的 OTLP 导出器
    // Initialize OTLP exporter using HTTP binary protocol
    // TODO:
    //     NOTE! 这里使用OTLP协议，是主动向Prometheus推送数据
    //     1. 准备一个空的Prometheus配置文件 prometheus.yml
    //     2. 使用docker启动Prometheus, 开启OTLP支持--enable-feature=otlp-write-receiver
    //     3. 启动我们的服务，向prometheus ip:port/api/v1/otlp/v1/metrics推送服务
    let otlp_metrics_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint("http://localhost:19090/api/v1/otlp/v1/metrics")
        .build()?;

    // 使用OTLP Metrics 导出器创建 Meter Provider
    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(otlp_metrics_exporter)
        .with_resource(
            Resource::builder()
                .with_service_name("hyper-tower-service")
                .build(),
        )
        .build();
    // 设置全局 MeterProvider 用于程序内其他地方记录指标
    global::set_meter_provider(meter_provider);

    // 创建 Tracing-OpenTelemetry层
    // let telemetry_tracer = opentelemetry::global::tracer("hyper-tower-service");
    // let provider_tracer = tracer_provider.tracer("hyper-tower-service");
    // 配置 tracing 订阅者
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(provider_tracer);
    // 配置终端输出
    let fmt_layer =
        tracing_subscriber::fmt::layer().with_filter(tracing_subscriber::filter::LevelFilter::INFO);
    // let fmt_layer = tracing_subscriber::fmt().with_max_level(tracing::Level::INFO);
    // .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
    // .finish();
    // 设置全局 subscriber
    let subscriber = Registry::default()
        // .with_subscriber(fmt_layer)
        .with(fmt_layer) // 终端输出
        .with(telemetry_layer); // 导出到OTEL
    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!(
        // target: "telemetry::tracing",
        target: "telemetry::init",
        telemetry_backend = "tracing",
        "Tracing initialized"
    );

    tracing::info!(
        // target: "telemetry::tracing",
        target: "telemetry::init",
        exporter_backend = "opentelemetry metrics export",
        exporter_protocol = "otlp",
        exporter_destination = "prometheus",
        exporter_endpoint = "http://localhost:19090/api/v1/otlp/v1/metrics",
        "OTLP metrics exporter initialized and connected to Prometheus http endpoint"
    );

    tracing::info!(
        // target: "telemetry::exporter",
        target: "telemetry::init",
        exporter_backend = "opentelemetry tracing export",
        exporter_protocol = "otlp",
        exporter_destination = "jaeger",
        exporter_endpoint = "http://localhost:4317",
        "OTLP exporter initialized and connected to Jaeger grpc endpoint"
    );

    // tracing::info!("Tracing initialized using OTLP exporter to Jaeger");
    // tracing::info!(
    //     "Initializing OTLP exporter and connecting to Jaeger endpoint at http://localhost:4317"
    // );

    Ok(())
}
