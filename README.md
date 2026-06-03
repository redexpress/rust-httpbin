# Axum Httpbin

A lightweight HTTP testing service inspired by [httpbin.org](https://httpbin.org), built with [Axum](https://github.com/tokio-rs/axum).

## Quick Start

```bash
cargo run
```

Server starts at **http://127.0.0.1:3000**.

```bash
curl http://127.0.0.1:3000/get
curl http://127.0.0.1:3000/ip
curl http://127.0.0.1:3000/uuid
curl -X POST http://127.0.0.1:3000/post -d '{"hello":"world"}' -H "Content-Type: application/json"
```

Full API reference: [docs/api.md](docs/api.md)

## Configuration

```bash
RUST_LOG=debug cargo run   # per-request tracing
RUST_LOG=trace cargo run   # full internal details
```

Every response includes an `X-Request-Id` header.

## Development

```bash
cargo fmt --check
cargo clippy
cargo test
```

Or run the bundled CI mirror locally before pushing:

```bash
./build.sh        # fmt + clippy + test (full) + build --release
./build.sh ci     # exact GitHub Actions parity (test --lib)
```

## Design

Feature-oriented structure, explicit dependencies, no enterprise patterns. See [docs/design.md](docs/design.md).

## License

MIT — see [LICENSE](LICENSE).
