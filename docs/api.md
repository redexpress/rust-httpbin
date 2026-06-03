# API Reference

Start the server (`cargo run`) and:

- Visit `http://127.0.0.1:3000/` for the interactive Swagger UI.
- `GET http://127.0.0.1:3000/openapi.json` for the raw OpenAPI 3.x spec (importable into Postman / Insomnia / Bruno / Apifox).

The spec is generated from the source code via [`utoipa`](https://crates.io/crates/utoipa); the source remains the single source of truth.

## Endpoints

### Request Inspection

| Method   | Path       | Description            |
| -------- | ---------- | ---------------------- |
| `GET`    | `/get`     | Echo the GET request   |
| `POST`   | `/post`    | Echo the POST request  |
| `PUT`    | `/put`     | Echo the PUT request   |
| `PATCH`  | `/patch`   | Echo the PATCH request |
| `DELETE` | `/delete`  | Echo the DELETE request|

### Header / IP / UA Inspection

| Method | Path          | Description              |
| ------ | ------------- | ------------------------ |
| `GET`  | `/headers`    | Return request headers   |
| `GET`  | `/ip`         | Return client IP         |
| `GET`  | `/user-agent` | Return User-Agent string |

### Response Control

| Method | Path                 | Description                |
| ------ | -------------------- | -------------------------- |
| `GET`  | `/status/:code`      | Return given status code   |
| `GET`  | `/delay/:secs`       | Wait N seconds, then reply |
| `GET`  | `/redirect-to`       | 302 (or custom) redirect   |
| `GET`  | `/stream/:n`         | Stream N JSON objects (SSE)|

### Auth

| Method | Path                         | Description                |
| ------ | ---------------------------- | -------------------------- |
| `GET`  | `/basic-auth/:user/:passwd`   | HTTP Basic auth check      |
| `GET`  | `/bearer`                    | Bearer token extraction    |

### Utility

| Method | Path            | Description               |
| ------ | --------------- | ------------------------- |
| `GET`  | `/uuid`         | Return a random UUIDv4    |
| `ANY`  | `/anything`     | Echo the entire request   |
| `ANY`  | `/anything/*`   | Echo with captured path   |
| `GET`  | `/image`        | Return a PNG image (alias for `/image/png`) |
| `GET`  | `/image/png`    | Return a PNG image        |
| `GET`  | `/image/jpeg`   | Return a JPEG image       |
| `GET`  | `/image/webp`   | Return a WebP image       |
