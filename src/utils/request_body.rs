use axum::body::Bytes;
use axum::extract::FromRequest;
use std::collections::HashMap;

/// Request body, transparently handling `multipart/form-data`.
///
/// For `multipart/*` content types, the body is parsed into:
/// - `files`: filename → size in bytes
/// - `text_fields`: form field name → value (no filename in the part)
///
/// For all other content types, the body is read as raw `Bytes` and both
/// `files` and `text_fields` are empty.
///
/// `bytes` is the raw body. For multipart it is empty (consumed by the
/// multipart parser); use `files` and `text_fields` instead.
#[derive(Debug, Default)]
pub struct RequestBody {
    pub content_type: String,
    pub bytes: Bytes,
    pub files: HashMap<String, u64>,
    pub text_fields: HashMap<String, String>,
}

impl RequestBody {
    /// True when the request used `multipart/form-data` (or any other multipart subtype).
    pub fn is_multipart(&self) -> bool {
        self.content_type.starts_with("multipart/")
    }
}

impl<S> FromRequest<S> for RequestBody
where
    S: Send + Sync,
{
    type Rejection = crate::error::AppError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let content_type = parts
            .headers
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let bytes = Bytes::from_request(axum::extract::Request::from_parts(parts, body), state)
            .await
            .map_err(|e| crate::error::AppError::BadRequest(format!("body: {e}")))?;

        let is_multipart = content_type.starts_with("multipart/");

        if is_multipart {
            let Some(boundary) = extract_boundary(&content_type) else {
                return Err(crate::error::AppError::BadRequest(
                    "multipart content-type missing boundary".into(),
                ));
            };
            let (files, text_fields) = parse_multipart(&bytes, &boundary);
            Ok(RequestBody {
                content_type,
                bytes: Bytes::new(),
                files,
                text_fields,
            })
        } else {
            Ok(RequestBody {
                content_type,
                bytes,
                files: HashMap::new(),
                text_fields: HashMap::new(),
            })
        }
    }
}

fn extract_boundary(content_type: &str) -> Option<String> {
    for part in content_type.split(';').skip(1) {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("boundary=") {
            let value = rest.trim_matches('"');
            return Some(value.to_string());
        }
    }
    None
}

/// Minimal RFC 7578 multipart parser.
///
/// Returns `(files, text_fields)`. Each part is identified by the value of
/// its `Content-Disposition: form-data; name="..."; filename="..."` header.
/// Parts with a `filename` are treated as files; the rest are text fields.
fn parse_multipart(body: &[u8], boundary: &str) -> (HashMap<String, u64>, HashMap<String, String>) {
    let mut files = HashMap::new();
    let mut text_fields = HashMap::new();
    let delimiter = format!("--{boundary}");
    let crlf = b"\r\n";

    // Split body by delimiter. The first chunk is preamble (discarded) and
    // the last chunk should be the closing delimiter followed by `--`.
    let chunks: Vec<&[u8]> = split_on(body, delimiter.as_bytes());
    for chunk in chunks.iter().skip(1) {
        // Skip closing delimiter "--\r\n" or "--" at end.
        let chunk = if chunk.starts_with(b"--") {
            continue;
        } else if let Some(stripped) = strip_prefix(chunk, crlf) {
            stripped
        } else {
            *chunk
        };
        // Strip trailing CRLF before next delimiter.
        let chunk = strip_suffix(chunk, b"\r\n").unwrap_or(chunk);
        if chunk.is_empty() {
            continue;
        }

        // Headers are separated from body by \r\n\r\n.
        let header_end = find_subsequence(chunk, b"\r\n\r\n");
        let Some(header_end) = header_end else {
            continue;
        };
        let (raw_headers, part_body) = chunk.split_at(header_end);
        let part_body = &part_body[4..]; // skip \r\n\r\n

        let headers_str = match std::str::from_utf8(raw_headers) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let disposition = match parse_content_disposition(headers_str) {
            Some(d) => d,
            None => continue,
        };

        match disposition.filename {
            Some(fname) => {
                files.insert(fname, part_body.len() as u64);
            }
            None => {
                let value = String::from_utf8_lossy(part_body).into_owned();
                text_fields.insert(disposition.name, value);
            }
        }
    }

    (files, text_fields)
}

struct ContentDisposition {
    name: String,
    filename: Option<String>,
}

fn parse_content_disposition(headers: &str) -> Option<ContentDisposition> {
    for line in headers.split("\r\n") {
        let (k, v) = line.split_once(':')?;
        if k.trim().eq_ignore_ascii_case("content-disposition") {
            return parse_disposition_value(v.trim());
        }
    }
    None
}

fn parse_disposition_value(value: &str) -> Option<ContentDisposition> {
    let mut name = None;
    let mut filename = None;
    for part in value.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("name=") {
            name = Some(unquote(rest.trim()));
        } else if let Some(rest) = part.strip_prefix("filename=") {
            filename = Some(unquote(rest.trim()));
        }
    }
    Some(ContentDisposition {
        name: name?,
        filename,
    })
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn split_on<'a>(haystack: &'a [u8], needle: &[u8]) -> Vec<&'a [u8]> {
    if needle.is_empty() {
        return vec![haystack];
    }
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i + needle.len() <= haystack.len() {
        if &haystack[i..i + needle.len()] == needle {
            out.push(&haystack[start..i]);
            start = i + needle.len();
            i = start;
        } else {
            i += 1;
        }
    }
    out.push(&haystack[start..]);
    out
}

fn strip_prefix<'a>(haystack: &'a [u8], prefix: &[u8]) -> Option<&'a [u8]> {
    if haystack.len() >= prefix.len() && &haystack[..prefix.len()] == prefix {
        Some(&haystack[prefix.len()..])
    } else {
        None
    }
}

fn strip_suffix<'a>(haystack: &'a [u8], suffix: &[u8]) -> Option<&'a [u8]> {
    if haystack.len() >= suffix.len() && &haystack[haystack.len() - suffix.len()..] == suffix {
        Some(&haystack[..haystack.len() - suffix.len()])
    } else {
        None
    }
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    fn test_app() -> axum::Router {
        crate::app::build_app(crate::state::AppState::new())
    }

    /// Build a `multipart/form-data` body with the given parts.
    fn build_multipart_body(parts: &[(&str, Option<&str>, &[u8])]) -> (String, Vec<u8>) {
        let boundary = "----testboundary12345";
        let mut body = Vec::new();
        for (name, filename, data) in parts {
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            match filename {
                Some(fname) => {
                    body.extend_from_slice(
                        format!(
                            "Content-Disposition: form-data; name=\"{name}\"; filename=\"{fname}\"\r\n\
                             Content-Type: application/octet-stream\r\n\
                             \r\n"
                        )
                        .as_bytes(),
                    );
                }
                None => {
                    body.extend_from_slice(
                        format!(
                            "Content-Disposition: form-data; name=\"{name}\"\r\n\
                             \r\n"
                        )
                        .as_bytes(),
                    );
                }
            }
            body.extend_from_slice(data);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        (format!("multipart/form-data; boundary={boundary}"), body)
    }

    #[tokio::test]
    async fn parses_multipart_text_and_file_fields() {
        let (ct, body) = build_multipart_body(&[
            ("text_field", None, b"hello"),
            ("upload", Some("hello.txt"), b"file contents here"),
        ]);
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/post")
            .header("content-type", ct)
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["form"]["text_field"], "hello");
        // "file contents here" is 18 bytes
        assert_eq!(json["files"]["hello.txt"], 18u64);
    }

    #[tokio::test]
    async fn non_multipart_keeps_files_empty() {
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/post")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"k":"v"}"#))
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["files"], serde_json::json!({}));
        assert_eq!(json["json"]["k"], "v");
    }

    #[tokio::test]
    async fn empty_multipart_yields_no_fields_or_files() {
        let (ct, body) = build_multipart_body(&[]);
        let app = test_app();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/post")
            .header("content-type", ct)
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.expect("request failed");
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["files"], serde_json::json!({}));
        assert_eq!(json["form"], serde_json::json!({}));
    }
}
