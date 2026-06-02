use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::http;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf};
use tower::Service;

// ── task-local: per-request raw headers, set by InjectRawHeaders ──────────

tokio::task_local! {
    pub(crate) static RAW_HEADERS: Option<HashMap<String, String>>;
}

// ── HeaderPeeker: peeks at HTTP/1.1 headers before hyper sees them ────────

/// Wraps an async stream, peeks at the HTTP/1.1 request headers, stores them
/// for replay so hyper can read the same bytes, and parses them with original
/// casing into a `HashMap`.
pub struct HeaderPeeker<S> {
    inner: S,
    buf: Vec<u8>,   // bytes to replay before delegating to inner
    buf_pos: usize, // read cursor within buf
}

impl<S> HeaderPeeker<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            buf: Vec::new(),
            buf_pos: 0,
        }
    }

    /// Read raw header bytes from `inner` until `\r\n\r\n`, parse them,
    /// and set up `buf` to replay them to the next reader (hyper).
    pub async fn peek_and_parse(mut self) -> io::Result<(Self, HashMap<String, String>)>
    where
        S: AsyncRead + Unpin,
    {
        let mut raw = Vec::with_capacity(4096);
        let mut tmp = [0u8; 512];

        // Read until we find \r\n\r\n (end of HTTP headers)
        loop {
            let n = self.inner.read(&mut tmp).await?;
            if n == 0 {
                // EOF before headers complete — replay what we have
                break;
            }
            raw.extend_from_slice(&tmp[..n]);
            if let Some(body_start) = find_header_end(&raw) {
                // split: headers (including \r\n\r\n) | body (rest)
                let body = raw.split_off(body_start);
                self.buf = body;
                break;
            }
        }

        let headers = parse_raw_headers(&raw);

        // Prepend the raw header bytes so hyper can read them
        let mut replay = raw;
        replay.extend_from_slice(&self.buf);
        self.buf = replay;
        self.buf_pos = 0;

        Ok((self, headers))
    }

}

// ── AsyncRead / AsyncWrite delegation ──────────────────────────────────────

impl<S: AsyncRead + Unpin> AsyncRead for HeaderPeeker<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = &mut *self;

        // Drain replay buffer first
        if this.buf_pos < this.buf.len() {
            let remaining = &this.buf[this.buf_pos..];
            let n = remaining.len().min(buf.remaining());
            buf.put_slice(&remaining[..n]);
            this.buf_pos += n;
            return Poll::Ready(Ok(()));
        }

        // Delegate to inner stream
        Pin::new(&mut this.inner).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for HeaderPeeker<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

// ── InjectRawHeaders: Tower service that sets task-local per request ───────

/// Wraps an inner service, injecting `RawHeaders` into a task-local
/// before every call so downstream `collect_headers` can pick it up.
#[derive(Clone)]
pub struct InjectRawHeaders<S> {
    inner: S,
    raw_headers: HashMap<String, String>,
}

impl<S> InjectRawHeaders<S> {
    pub fn new(inner: S, raw_headers: HashMap<String, String>) -> Self {
        Self {
            inner,
            raw_headers,
        }
    }
}

impl<S, B> Service<http::Request<B>> for InjectRawHeaders<S>
where
    S: Service<http::Request<B>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<S::Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        let raw = self.raw_headers.clone();
        let mut inner = self.inner.clone();
        let fut = inner.call(req);
        Box::pin(async move {
            RAW_HEADERS
                .scope(Some(raw), fut)
                .await
        })
    }
}

// ── HTTP/1.1 header parser ─────────────────────────────────────────────────

/// Find the position just past `\r\n\r\n` (start of HTTP body).
fn find_header_end(data: &[u8]) -> Option<usize> {
    data.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4)
}

/// Parse HTTP/1.1 headers from raw bytes, preserving original casing.
///
/// Expected format:
/// ```text
/// GET /path HTTP/1.1\r\n
/// Host: example.com\r\n
/// Accept: text/html\r\n
/// X-Custom-Header: value\r\n
/// \r\n
/// ```
fn parse_raw_headers(raw: &[u8]) -> HashMap<String, String> {
    let text = String::from_utf8_lossy(raw);
    let mut headers = HashMap::new();

    // Skip the request line (first line)
    let lines: Vec<&str> = text.split("\r\n").collect();
    for line in lines.iter().skip(1) {
        if line.is_empty() {
            break; // empty line = end of headers
        }
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim().to_string();
            let value = value.trim().to_string();
            if !name.is_empty() {
                headers.insert(name, value);
            }
        }
    }

    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_headers_with_original_casing() {
        let raw = b"GET /get HTTP/1.1\r\nHost: example.com\r\nContent-Type: application/json\r\nX-Custom-Header: foo\r\n\r\nbody";
        let headers = parse_raw_headers(raw);
        assert_eq!(headers.get("Host").unwrap(), "example.com");
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(headers.get("X-Custom-Header").unwrap(), "foo");
    }

    #[test]
    fn empty_headers() {
        let raw = b"GET / HTTP/1.1\r\n\r\n";
        let headers = parse_raw_headers(raw);
        assert!(headers.is_empty());
    }
}
