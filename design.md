# Axum Httpbin

# Architecture & Detailed Design

Version: 1.0

---

# 1. Project Overview

## 1.1 Purpose

Axum Httpbin is a lightweight HTTP testing service inspired by httpbin.org.

The project serves three goals:

1. Self-hosted HTTP testing service
2. Axum reference implementation
3. Rust web development learning project

The codebase prioritizes:

* simplicity
* readability
* maintainability
* extensibility

over architectural sophistication.

---

# 2. Design Principles

## 2.1 Keep It Small

The project intentionally avoids:

* DDD
* CQRS
* Event Sourcing
* Repository Pattern
* Service Layer

Reason:

The application contains no domain model and no persistence layer.

Introducing enterprise patterns would increase complexity without improving maintainability.

---

## 2.2 Feature-Oriented Structure

Code is organized around HTTP features.

Preferred:

```text
endpoints/status.rs
endpoints/delay.rs
endpoints/headers.rs
```

Avoid:

```text
controllers/
services/
repositories/
```

Reason:

Each endpoint is largely independent.

Feature-oriented organization minimizes cognitive overhead.

---

## 2.3 Explicit Dependencies

No hidden global state.

Preferred:

```rust
State<AppState>
```

Avoid:

```rust
static mut CONFIG
lazy_static GLOBAL_CONTEXT
```

Reason:

Explicit dependencies improve testability and readability.

---

# 3. System Architecture

## 3.1 High-Level Architecture

```text
                ┌──────────┐
                │  Client  │
                └────┬─────┘
                     │
                     ▼
             ┌──────────────┐
             │ Middleware   │
             └──────┬───────┘
                    │
                    ▼
             ┌──────────────┐
             │ Router       │
             └──────┬───────┘
                    │
                    ▼
             ┌──────────────┐
             │ Endpoint     │
             │ Handler      │
             └──────┬───────┘
                    │
                    ▼
             ┌──────────────┐
             │ Utilities    │
             └──────┬───────┘
                    │
                    ▼
             ┌──────────────┐
             │ Response     │
             └──────────────┘
```

No database layer exists.

No business service layer exists.

No repository layer exists.

---

# 4. Source Tree Layout

```text
axum-httpbin/

├── Cargo.toml
├── README.md
├── LICENSE
│
├── docs/
│   ├── design.md
│   ├── api.md
│   └── roadmap.md
│
├── src/
│   │
│   ├── main.rs
│   ├── app.rs
│   ├── state.rs
│   │
│   ├── api/
│   │   └── openapi.rs
│   │
│   ├── endpoints/
│   │   │
│   │   ├── request/
│   │   │   ├── get.rs
│   │   │   ├── post.rs
│   │   │   ├── put.rs
│   │   │   ├── patch.rs
│   │   │   └── delete.rs
│   │   │
│   │   ├── inspect/
│   │   │   ├── headers.rs
│   │   │   ├── ip.rs
│   │   │   └── user_agent.rs
│   │   │
│   │   ├── response/
│   │   │   ├── status.rs
│   │   │   ├── delay.rs
│   │   │   ├── redirect.rs
│   │   │   └── stream.rs
│   │   │
│   │   ├── auth/
│   │   │   ├── basic.rs
│   │   │   └── bearer.rs
│   │   │
│   │   └── utility/
│   │       ├── uuid.rs
│   │       └── anything.rs
│   │
│   ├── middleware/
│   │   ├── request_id.rs
│   │   ├── access_log.rs
│   │   └── trace.rs
│   │
│   ├── models/
│   │   ├── request.rs
│   │   ├── response.rs
│   │   └── error.rs
│   │
│   ├── utils/
│   │   ├── client_ip.rs
│   │   ├── header_utils.rs
│   │   ├── json_utils.rs
│   │   └── response_utils.rs
│   │
│   └── error.rs
│
├── tests/
│   ├── request_tests.rs
│   ├── response_tests.rs
│   ├── auth_tests.rs
│   └── integration_tests.rs
│
└── examples/
```

---

# 5. Dependency Selection

## Runtime

Tokio

Reason:

* ecosystem standard
* Axum native runtime
* async timers for /delay

---

## HTTP Framework

Axum

Reason:

* modern API design
* excellent extractor model
* official Tokio ecosystem

---

## Serialization

Serde

Reason:

* de-facto Rust serialization framework

---

## Logging

Tracing

Reason:

* structured logging
* production-ready ecosystem

---

## OpenAPI

Utoipa

Reason:

* compile-time schema generation
* Axum integration
* Swagger UI support

---

# 6. Module Responsibilities

## main.rs

Responsibilities:

* runtime initialization
* configuration loading
* startup logging
* listener creation

Must not contain endpoint logic.

---

## app.rs

Responsibilities:

* router construction
* middleware registration
* endpoint registration

Acts as the application composition root.

---

## endpoints/

Responsibilities:

* route definitions
* extractors
* response generation

Rules:

Endpoints may depend on:

```text
models
utils
state
```

Endpoints may not depend on:

```text
other endpoints
```

---

## middleware/

Responsibilities:

* request tracing
* request id generation
* access logging

Forbidden:

* business logic
* endpoint-specific behavior

---

## models/

Responsibilities:

Shared DTOs.

Must remain framework-agnostic whenever possible.

---

## utils/

Responsibilities:

Reusable helpers.

Must be:

* stateless
* deterministic
* independently testable

---

# 7. Dependency Rules

Allowed:

```text
endpoint
    ↓
models

endpoint
    ↓
utils

middleware
    ↓
utils
```

Forbidden:

```text
endpoint
    ↓
endpoint

utils
    ↓
endpoint

models
    ↓
endpoint
```

Reason:

Prevent cyclic dependencies.

---

# 8. OpenAPI Strategy

All public endpoints should be documented through Utoipa.

Documentation generation must be automatic.

Manual API documentation is discouraged.

Source code remains the single source of truth.

---

# 9. Testing Strategy

## Unit Tests

Location:

```text
src/**/mod tests
```

Coverage:

* utilities
* parsers
* helper functions

---

## Integration Tests

Location:

```text
tests/
```

Coverage:

* routing
* middleware
* HTTP behavior
* response validation

---

## CI Requirements

Every pull request must pass:

```bash
cargo fmt --check
cargo clippy
cargo test
```

---

# 10. Extension Strategy

When adding a new endpoint:

Step 1

Create:

```text
endpoints/<feature>.rs
```

Step 2

Register route in app.rs.

Step 3

Add OpenAPI annotation.

Step 4

Add integration test.

No additional architectural changes should be required.

---

# 11. Release Strategy

Versioning follows Semantic Versioning.

Examples:

```text
v1.0.0
v1.1.0
v2.0.0
```

Breaking changes require a major version bump.

---

# 12. Future Roadmap

Planned extensions:

* gzip compression
* HTTP/2 examples
* Prometheus metrics
* rate limiting
* websocket testing
* SSE examples

The architecture should remain lightweight even as features grow.
