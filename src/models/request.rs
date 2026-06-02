use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A reflection of the incoming HTTP request.
///
/// Used by `/get`, `/post`, `/put`, `/patch`, `/delete`, and `/anything`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestInfo {
    /// Full request URL (scheme + host + path + query)
    pub url: String,

    /// Request headers as a map (original casing)
    pub headers: HashMap<String, String>,

    /// The origin IP address
    pub origin: String,

    /// Query string parameters
    #[serde(default)]
    pub args: HashMap<String, String>,

    /// JSON body, if submitted and parseable (always present, null when not JSON)
    #[serde(default)]
    pub json: Option<serde_json::Value>,

    /// Raw body as a UTF-8 string (empty when form or json parsed)
    #[serde(default)]
    pub data: String,

    /// Form parameters, if submitted as `application/x-www-form-urlencoded`
    #[serde(default)]
    pub form: HashMap<String, String>,

    /// Files submitted via multipart (filename → size in bytes)
    #[serde(default)]
    pub files: HashMap<String, u64>,
}
