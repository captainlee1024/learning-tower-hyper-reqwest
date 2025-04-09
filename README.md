# hyper-tower-echo-demo

A simple echo server using hyper and tower

## how to use:

1、launch the Jaeger agent using docker:

```bash
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 6831:6831/udp \
  -p 16686:16686 \
  -p 4317:4317 \
  jaegertracing/all-in-one:latest
```

stop and restart the Jaeger agent:

```bash
docker stop jaeger
docker start jaeger
```

2、launch the prometheus using docker:

```bash
chmod -R 777 prometheus-data
```

delete the prometheus data:

```bash
sudo rm -rf prometheus-data/*
```

launch prometheus:

```bash
docker run -d \
  --name prometheus \
  --network host \
  -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml \
  -v $(pwd)/prometheus-data:/prometheus \
  prom/prometheus:latest \
  --web.listen-address=":19090" \
  --config.file=/etc/prometheus/prometheus.yml \
  --enable-feature=otlp-write-receiver
```

3、 launch the grafana using docker:

```bash
chmod -R 777 grafana-data
```

delete the grafana data:

```bash
sudo rm -rf grafana-data/*
```

launch grafana:

```bash
docker run -d \
  --name grafana \
  --network host \
  -e "GF_SERVER_HTTP_PORT=13000" \
  -v $(pwd)/grafana-data:/var/lib/grafana \
  grafana/grafana:latest
```

3、launch the echo server:

```bash
cargo run
```

4、test with curl:

```bash
curl -v -X POST -H "Authorization: Bearer token" -d "hello world" http://127.0.0.1:3000
```

test the axum router feature:

```bash
url -X GET \
     -H "Authorization: Bearer token" \
     -H "Content-Type: application/json" \
     -d '{"text":"hello world!"}' \
     http://127.0.0.1:3000/health
```

```bash
url -X POST \
     -H "Authorization: Bearer token" \
     -H "Content-Type: application/json" \
     -d '{"text":"hello world!"}' \
     http://127.0.0.1:3000/echo
```

5、check the trace in Jaeger UI:

[open Jaeger UI in browser](http://localhost:16686/)

select the Service name `hyper-tower-service` and select the Operation name `request`, click `Find Traces` to see the
traces.

6、 check the metrics in prometheus:

[open prometheus UI in browser](http://localhost:19090/)

open metrics explorer, select `http_request_total`, `http_request_duration_seconds_bucket` ... to see the metrics.

7、 check the metrics in grafana:

[open grafana UI in browser](http://localhost:13000/)

add the prometheus data source: http://localhost:19090

create a new dashboard, add a new panel, select the prometheus data source, and select the metrics you want to see.

Grafana Chart Examples

1. **Request Rate Line Chart (Time Series)**
    - **Purpose**: Show request rate per second over time.
    - **Query**: `rate(http_requests_total[5m])`
        - `rate`: Calculates per-second increase over a 5-minute window.
    - **Visualization**:
        - Type: Time Series
        - Config: X-axis: time, Y-axis: req/s, split by `method`
    - **Value**: Monitor request trends and detect peaks.

2. **Duration Distribution Histogram (Histogram)**
    - **Purpose**: Display request duration distribution (your bar chart need).
    - **Query**: `http_request_duration_seconds_bucket{method="POST"}`
        - Uses Histogram bucket data directly.
    - **Visualization**:
        - Type: Histogram
        - Config: X-axis: `le` (bucket boundaries), Y-axis: count, unit: milliseconds
    - **Value**: Understand duration spread, e.g., most requests in low latency.

8、test the graceful shutdown using ab:

install and check the ab:

```bash
ab -V
This is ApacheBench, Version 2.3 <$Revision: 1923142 $>
Copyright 1996 Adam Twiss, Zeus Technology Ltd, http://www.zeustech.net/
Licensed to The Apache Software Foundation, http://www.apache.org/
```

using ab to send 100 requests by 100 connections, ctrl+c to stop the server immediately:

```bash
ab -n 100 -c 100 -H "Authorization: Bearer token" -p ab_post_data_for_test.txt -T "application/json" http://127.0.0.1:3000/
```

check the trace in terminal:

```text
...
2025-04-09T11:02:41.505884Z  INFO server::shutdown: Received SIGINT, shutting down...
2025-04-09T11:02:41.505920Z  INFO server::shutdown: Shutting down: stopping new connections
2025-04-09T11:02:41.505932Z  INFO server::shutdown: Waiting for active tasks to complete
2025-04-09T11:02:41.505938Z  INFO server::shutdown: Waiting for 100 active tasks to complete
...
2025-04-09T11:02:41.827603Z  INFO server::shutdown: Waiting for 75 active tasks to complete
...
2025-04-09T11:02:42.214166Z  INFO server::shutdown: All active tasks completed
2025-04-09T11:02:42.214235Z  INFO server::shutdown: Shutting down OpenTelemetry
2025-04-09T11:02:42.267836Z  INFO server::shutdown: Server shutdown complete

```