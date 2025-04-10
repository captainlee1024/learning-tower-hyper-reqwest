use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

// 初始化 Tracing和OpenTelemetry
pub async fn init_tracing()
-> Result<(SdkTracerProvider, SdkMeterProvider), Box<dyn std::error::Error>> {
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
