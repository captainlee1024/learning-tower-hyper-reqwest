#![feature(duration_millis_float)]

mod app;
mod appv2;
mod middleware;
mod middleware_for_axum;

use crate::app::echo;
// use futures::SinkExt;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
// use opentelemetry_sdk::resource::ResourceBuilder;
use axum::Router;
use axum::routing::{get, post};
use opentelemetry::global;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
// use hyper::server::Server;
// use hyper::service::make_service_fn;
use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::Notify;
use tower::{ServiceBuilder, service_fn};
// use tracing::instrument::WithSubscriber;
// use tracing_opentelemetry::OpenTelemetryLayer;
// use tracing_subscriber::fmt::writer::MakeWriterExt;
use crate::appv2::{AppState, echo_handler, health_handler};
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
/// 2、launch the prometheus using docker:
///
/// ```bash
/// chmod -R 777 prometheus-data
/// ```
///
/// delete the prometheus data:
///
/// ```bash
/// sudo rm -rf prometheus-data/*
/// ```
///
/// launch prometheus:
///
/// ```bash
/// docker run -d \
///   --name prometheus \
///   --network host \
///   -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml \
///   -v $(pwd)/prometheus-data:/prometheus \
///   prom/prometheus:latest \
///   --web.listen-address=":19090" \
///   --config.file=/etc/prometheus/prometheus.yml \
///   --enable-feature=otlp-write-receiver
/// ```
///
/// 3、 launch the grafana using docker:
///
/// ```bash
/// chmod -R 777 grafana-data
/// ```
///
/// delete the grafana data:
///
/// ```bash
/// sudo rm -rf grafana-data/*
/// ```
///
/// launch grafana:
///
/// ```bash
/// docker run -d \
///   --name grafana \
///   --network host \
///   -e "GF_SERVER_HTTP_PORT=13000" \
///   -v $(pwd)/grafana-data:/var/lib/grafana \
///   grafana/grafana:latest
/// ```
///
/// 3、launch the echo server:
///
/// ```bash
/// cargo run
/// ```
///
/// 4、test with curl:
///
/// ```bash
/// curl -v -X POST -H "Authorization: Bearer token" -d "hello world" http://127.0.0.1:3000
/// ```
///
/// 5、check the trace in Jaeger UI:
///
/// [open Jaeger UI in browser](http://localhost:16686/)
///
/// select the Service name `hyper-tower-service` and select the Operation name `request`, click `Find Traces` to see the
/// traces.
///
/// 6、 check the metrics in prometheus:
///
/// [open prometheus UI in browser](http://localhost:19090/)
///
/// open metrics explorer, select `http_request_total`, `http_request_duration_seconds_bucket` ... to see the metrics.
///
/// 7、 check the metrics in grafana:
///
/// [open grafana UI in browser](http://localhost:13000/)
///
/// add the prometheus data source: http://localhost:19090
///
/// create a new dashboard, add a new panel, select the prometheus data source, and select the metrics you want to see.
///
/// Grafana Chart Examples
///
/// 1. **Request Rate Line Chart (Time Series)**
///     - **Purpose**: Show request rate per second over time.
///     - **Query**: `rate(http_requests_total[5m])`
///         - `rate`: Calculates per-second increase over a 5-minute window.
///     - **Visualization**:
///         - Type: Time Series
///         - Config: X-axis: time, Y-axis: req/s, split by `method`
///     - **Value**: Monitor request trends and detect peaks.
///
/// 2. **Duration Distribution Histogram (Histogram)**
///     - **Purpose**: Display request duration distribution (your bar chart need).
///     - **Query**: `http_request_duration_seconds_bucket{method="POST"}`
///         - Uses Histogram bucket data directly.
///     - **Visualization**:
///         - Type: Histogram
///         - Config: X-axis: `le` (bucket boundaries), Y-axis: count, unit: milliseconds
///     - **Value**: Understand duration spread, e.g., most requests in low latency.
///
/// 8、test the graceful shutdown using ab:
///
/// install and check the ab:
///
/// ```bash
/// ab -V
/// This is ApacheBench, Version 2.3 <$Revision: 1923142 $>
/// Copyright 1996 Adam Twiss, Zeus Technology Ltd, http://www.zeustech.net/
/// Licensed to The Apache Software Foundation, http://www.apache.org/
/// ```
///
/// using ab to send 100 requests by 100 connections, ctrl+c to stop the server immediately:
///
/// ```bash
/// ab -n 100 -c 100 -H "Authorization: Bearer token" -p ab_post_data_for_test.txt -T "application/json" http://127.0.0.1:3000/
/// ```
///
/// check the trace in terminal:
///
/// ```text
/// ...
/// 2025-04-09T11:02:41.505884Z  INFO server::shutdown: Received SIGINT, shutting down...
/// 2025-04-09T11:02:41.505920Z  INFO server::shutdown: Shutting down: stopping new connections
/// 2025-04-09T11:02:41.505932Z  INFO server::shutdown: Waiting for active tasks to complete
/// 2025-04-09T11:02:41.505938Z  INFO server::shutdown: Waiting for 100 active tasks to complete
/// ...
/// 2025-04-09T11:02:41.827603Z  INFO server::shutdown: Waiting for 75 active tasks to complete
/// ...
/// 2025-04-09T11:02:42.214166Z  INFO server::shutdown: All active tasks completed
/// 2025-04-09T11:02:42.214235Z  INFO server::shutdown: Shutting down OpenTelemetry
/// 2025-04-09T11:02:42.267836Z  INFO server::shutdown: Server shutdown complete
///
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // // 初始化 Tracing
    // tracing_subscriber::fmt()
    //     .with_max_level(tracing::Level::INFO) // 设置日志级别
    //     // .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
    //     .init(); // 初始化控制台输出
    //
    // 初始化Tracing和OpenTelemetry
    let (otlp_tracer_provider, otlp_meter_provider) = init_tracing().await?;

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

    // 用于通知关闭服务，释放资源
    let shutdown = Arc::new(Notify::new());
    let shutdown_clone = shutdown.clone();

    // 跟踪活跃任务
    // let active_tasks = Arc::new(tokio::sync::Mutex::new(0));
    // 使用 AtomicUsize 替代 Mutex
    let active_tasks = Arc::new(AtomicUsize::new(0));
    let active_tasks_clone = active_tasks.clone();

    // 处理信号
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::spawn(async move {
        tokio::select! {
            _ = sigint.recv() => tracing::info!(target: "server::shutdown", "Received SIGINT, shutting down..."),
            _ = sigterm.recv() => tracing::info!(target: "server::shutdown", "Received SIGTERM, shutting down..."),
        }
        shutdown_clone.notify_one();
    });

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
    let hyper_service = TowerToHyperService::new(t_service);

    // 初始化状态
    let state = Arc::new(AppState {
        message: "Server is running".to_string(),
    });

    // 构建 Router
    let app: Router = Router::new()
        .route("/health", get(health_handler))
        .route("/echo", post(echo_handler))
        .with_state(state) // 注入状态
        .layer(
            ServiceBuilder::new()
                .layer(middleware_for_axum::tracing::TracingLayer)
                .layer(middleware_for_axum::metrics::MetricsLayer)
                .layer(middleware_for_axum::auth::AuthLayer)
                // FIXME: 这里的body limit 中间件会导致Service<Request<Limited<ReqBody>> Request类型不一致
                // .layer(middleware::ratelimit::ratelimit_layer())
                .layer(middleware_for_axum::cache::CacheLayer)
                .layer(middleware_for_axum::timeout::timeout_layer()),
        ); // 添加 Tower 中间件

    let axum_service = TowerToHyperService::new(app.into_service());

    loop {
        tokio::select! {
            // TODO:这里是否只支持单线程处理请求？
            result = listener.accept() => {
                match result {
                    Ok((stream, _addr)) => {
                        let io = TokioIo::new(stream);
                        let _cloned_hyper_service = hyper_service.clone();
                        let cloned_axum_service = axum_service.clone();
                        let active_tasks = active_tasks.clone();

                        // 增加活跃任务计数
                        // let mut locked_active_tasks = active_tasks.lock().await;
                        // *locked_active_tasks += 1;
                        // 增加活跃任务计数
                        let active_tasks_clone_for_current_connection = active_tasks.clone();
                        active_tasks_clone.fetch_add(1, Ordering::SeqCst);

                        tokio::spawn(async move {
                            if let Err(e) = http1::Builder::new()
                            .serve_connection(io, cloned_axum_service)
                            .await {
                                tracing::error!(target: "server::connection", "Error serving connection: {}", e);
                            }

                            // 任务完成后减少活跃任务计数
                            // let mut locked_active_tasks = active_tasks.lock().await;
                            // *locked_active_tasks -= 1;
                            // 任务完成，减少计数
                            active_tasks_clone_for_current_connection.fetch_sub(1, Ordering::SeqCst);
                        });
                    }
                    Err(e) => {
                        tracing::error!(target: "server::accept", "Failed to accept connection: {}", e);
                        break;
                    }
                }
            }

            // 收到退出信号，推出循环
            _ = shutdown.notified() => {
                tracing::info!(target: "server::shutdown", "Shutting down: stopping new connections");
                break;
            }
        }

        // let (stream, _) = listener.accept().await?;
        // let io = TokioIo::new(stream);
        //
        // // TODO: optimize: clone service
        // let cloned_service = h_service.clone();
        // tokio::task::spawn(async move {
        //     if let Err(err) = http1::Builder::new()
        //         .serve_connection(io, cloned_service)
        //         .await
        //     {
        //         tracing::error!("Error serving connection: {}", err);
        //     }
        // });
    }

    // 等待所有活跃任务完成
    tracing::info!(target: "server::shutdown", "Waiting for active tasks to complete");
    // FIXME: 这里的active_tasks.lock().await == 0, sleep 会长时间占有锁，虽然不会阻塞
    // 让出线程后锁还是被占有，其他active task执行完无法获取锁更新activeTaskCount
    // NOTE: 已经修复，这里使用原子计数器即可
    // loop {
    //     if *active_tasks.lock().await == 0 {
    //         break;
    //     }
    //     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    // }
    // TODO: 是否需要使用watch的方式优化这里的轮训
    // watch主动通知，类似
    // let (shutdown_tx, shutdown_rx) = watch::channel(false);
    // // 任务线程
    // let remaining = active_tasks_clone.fetch_sub(1, Ordering::SeqCst);
    // if remaining == 1 && *shutdown_rx_task.borrow() {
    //     shutdown_complete_clone.notify_one();
    // }
    while active_tasks.load(Ordering::SeqCst) > 0 {
        tracing::info!(target: "server::shutdown",
            "Waiting for {} active tasks to complete",
            active_tasks.load(Ordering::SeqCst)
        );
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    tracing::info!(target: "server::shutdown", "All active tasks completed");

    // 关闭 OpenTelemetry
    tracing::info!(target: "server::shutdown", "Shutting down OpenTelemetry");
    otlp_tracer_provider.force_flush()?;
    otlp_tracer_provider.shutdown()?;

    otlp_meter_provider.force_flush()?;
    otlp_meter_provider.shutdown()?;

    tracing::info!(target: "server::shutdown", "Server shutdown complete");
    Ok(())
    // let make_svc = make_service_fn(|_conn| async { Ok::<_, hyper::Error>(create_service()) });
    // Server::bind(&addr).serve(make_svc).await.unwrap();
}

// 初始化 Tracing和OpenTelemetry
async fn init_tracing() -> Result<(SdkTracerProvider, SdkMeterProvider), Box<dyn std::error::Error>>
{
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
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

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
    global::set_meter_provider(meter_provider.clone());

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

    Ok((tracer_provider, meter_provider))
}
