# hyper-tower-echo-demo
A simple echo server using hyper and tower

how to run:
```bash
cargo run
```

test with curl:
```bash
curl -v -X POST -H "Authorization: Bearer token" -d "hello world" http://127.0.0.1:3000
```