# API Reference

> Full endpoint documentation coming soon.
> See the Swagger UI at `/swagger-ui` for interactive docs generated from source code.

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
