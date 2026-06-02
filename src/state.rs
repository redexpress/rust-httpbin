/// Shared application state.
///
/// Currently a placeholder. Future fields may include:
/// - configuration values
/// - shared counters or rate-limiters
#[derive(Clone)]
pub struct AppState;

impl AppState {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
