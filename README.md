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

4、launch the postgres using docker:

data directory for postgres:

```bash
chmod -R 777 postgresql-data
```

run postgres:

```bash
docker run -d \
  --name postgres \
  -e POSTGRES_PASSWORD=password \
  --network host \
  -v $(pwd)/conf/postgresql.conf:/etc/postgresql/postgresql.conf \
  -v $(pwd)/postgresql-data:/var/lib/postgresql/data \
  postgres \
  -c config_file=/etc/postgresql/postgresql.conf
```

check the startup log:

```bash
docker logs postgres

PostgreSQL Database directory appears to contain a database; Skipping initialization

2025-04-14 10:05:03.269 GMT [1] LOG:  starting PostgreSQL 17.4 (Debian 17.4-1.pgdg120+2) on x86_64-pc-linux-gnu, compiled by gcc (Debian 12.2.0-14) 12.2.0, 64-bit
2025-04-14 10:05:03.269 GMT [1] LOG:  listening on IPv4 address "0.0.0.0", port 5432
2025-04-14 10:05:03.269 GMT [1] LOG:  listening on IPv6 address "::", port 5432
2025-04-14 10:05:03.272 GMT [1] LOG:  listening on Unix socket "/var/run/postgresql/.s.PGSQL.5432"
2025-04-14 10:05:03.281 GMT [29] LOG:  database system was shut down at 2025-04-14 10:01:21 GMT
2025-04-14 10:05:03.293 GMT [1] LOG:  database system is ready to accept connections

```

create the kv_store database:

```bash
psql -h localhost -U postgres -c "CREATE DATABASE kv_store;"
```

check the database:

```bash
psql -h localhost -U postgres -c "\l"

# output
                                                        数据库列表
   名称    |  拥有者  | 字元编码 | Locale Provider |  校对规则  |   Ctype    | Locale | ICU Rules |       存取权限        
-----------+----------+----------+-----------------+------------+------------+--------+-----------+-----------------------
 kv_store  | postgres | UTF8     | libc            | en_US.utf8 | en_US.utf8 |        |           | 
 postgres  | postgres | UTF8     | libc            | en_US.utf8 | en_US.utf8 |        |           | 
 template0 | postgres | UTF8     | libc            | en_US.utf8 | en_US.utf8 |        |           | =c/postgres          +
           |          |          |                 |            |            |        |           | postgres=CTc/postgres
 template1 | postgres | UTF8     | libc            | en_US.utf8 | en_US.utf8 |        |           | =c/postgres          +
           |          |          |                 |            |            |        |           | postgres=CTc/postgres
(4 行记录)
```

5、launch Redis using docker:

the data directory for Redis:

```bash
chmod -R 777 redis-data
```

run redis:

```bash
docker run -d \
  --name redis \
  -v $(pwd)/conf/redis.conf:/usr/local/etc/redis/redis.conf \
  -v $(pwd)/redis-data:/data \
  --network host \
  redis \
  redis-server /usr/local/etc/redis/redis.conf
```

check the startup log:

```bash
docker logs redis

# output
1:C 14 Apr 2025 11:09:10.831 # WARNING Memory overcommit must be enabled! Without it, a background save or replication may fail under low memory condition. Being disabled, it can also cause failures without low memory condition, see https://github.com/jemalloc/jemalloc/issues/1328. To fix this issue add 'vm.overcommit_memory = 1' to /etc/sysctl.conf and then reboot or run the command 'sysctl vm.overcommit_memory=1' for this to take effect.
1:C 14 Apr 2025 11:09:10.831 * oO0OoO0OoO0Oo Redis is starting oO0OoO0OoO0Oo
1:C 14 Apr 2025 11:09:10.831 * Redis version=7.4.2, bits=64, commit=00000000, modified=0, pid=1, just started
1:C 14 Apr 2025 11:09:10.831 * Configuration loaded
1:M 14 Apr 2025 11:09:10.831 * Increased maximum number of open files to 1032 (it was originally set to 1024).
1:M 14 Apr 2025 11:09:10.831 * monotonic clock: POSIX clock_gettime
1:M 14 Apr 2025 11:09:10.832 # Failed to write PID file: Permission denied
1:M 14 Apr 2025 11:09:10.832 * Running mode=standalone, port=6379.
1:M 14 Apr 2025 11:09:10.832 * Server initialized
1:M 14 Apr 2025 11:09:10.832 * Ready to accept connections tcp
```

check the redis server with redis-cli:

```bash
redis-cli -h 127.0.0.1 -p 6379 -a redis123456 PING
Warning: Using a password with '-a' or '-u' option on the command line interface may not be safe.
PONG
```

or

```bash
redis-cli -h 127.0.0.1 -p 6379     
127.0.0.1:6379> AUTH redis123456
OK
127.0.0.1:6379> PING
PONG
127.0.0.1:6379> 
```

3、launch the echo server:

```bash
cargo run --no-default-features --features "service-axum middleware-tower"
cargo run --no-default-features --features "service-axum middleware-axum"
cargo run --no-default-features --features "service-my middleware-my"
cargo run --no-default-features --features "service-my middleware-tower"
```

4、test with curl:

```bash
curl -v -X POST -H "Auth-Key: Bearer token" -d "hello world" http://127.0.0.1:3000
```

test the axum router feature:

```bash
curl -v -X GET \
     -H "Auth-Key: Bearer token" \
     -H "Content-Type: application/json" \
     -d '{"text":"hello world!"}' \
     http://127.0.0.1:3000/health
```

```bash
curl -v -X POST \
     -H "Auth-Key: Bearer token" \
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