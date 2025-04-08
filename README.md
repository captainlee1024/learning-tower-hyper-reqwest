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

2、launch the echo server:

```bash
cargo run
```

3、test with curl:

```bash
curl -v -X POST -H "Authorization: Bearer token" -d "hello world" http://127.0.0.1:3000
```

4、check the trace in Jaeger UI:

[open Jaeger UI in browser](http://localhost:16686/)

select the Service name `hyper-tower-service` and select the Operation name `request`, click `Find Traces` to see the
traces.