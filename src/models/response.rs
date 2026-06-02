use serde::Serialize;

/// Standard success envelope used by most endpoints.
#[derive(Debug, Serialize)]
pub struct SuccessResponse<T: Serialize> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T: Serialize> SuccessResponse<T> {
    #[allow(dead_code)]
    pub fn data(data: T) -> Self {
        Self {
            data: Some(data),
            message: None,
        }
    }

    #[allow(dead_code)]
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            data: None,
            message: Some(message.into()),
        }
    }
}

/// UUID response for `/uuid`.
#[derive(Debug, Serialize)]
pub struct UuidResponse {
    pub uuid: String,
}

/// Stream response item for `/stream/:n`.
#[derive(Debug, Serialize)]
pub struct StreamItem {
    pub id: usize,
    pub url: String,
}
