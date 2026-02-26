use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

// --- Models ---

/// Represents the data structure returned by DynaRust.
/// We use a generic type `T` so users can deserialize directly into their own structs.
/// It defaults to `serde_json::Value` if no specific type is provided.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionedValue<T = Value> {
    pub value: T,
    pub version: u64,
    pub timestamp: u64,
    pub owner: String,
}

// --- Configuration & Client ---

/// The main client used to interact with the DynaRust cluster.
#[derive(Debug, Clone)]
pub struct DynaClient {
    /// The base URL of the DynaRust node (e.g., "http://127.0.0.1:6660")
    pub base_url: String,
    /// The internal HTTP client used for connection pooling
    http_client: Client,
    /// Optional JWT token for operations that require auth (PUT, DELETE)
    pub jwt_token: Option<String>,
}

// --- Error Handling ---

/// Standardized errors for DynaRust interactions
#[derive(Debug)]
pub enum DynaError {
    /// Network or Serialization errors from reqwest
    RequestFailed(reqwest::Error),
    /// 404 Not Found (Key or Table does not exist)
    NotFound,
    /// 401 Unauthorized (Missing/invalid JWT, or not the owner)
    Unauthorized,
    /// Any other unhandled HTTP status codes
    UnexpectedStatus(u16, String),
}

impl std::error::Error for DynaError {}

impl fmt::Display for DynaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DynaError::RequestFailed(e) => write!(f, "Request failed: {}", e),
            DynaError::NotFound => write!(f, "Key or table not found (404)"),
            DynaError::Unauthorized => write!(f, "Unauthorized access (401)"),
            DynaError::UnexpectedStatus(code, msg) => {
                write!(f, "Unexpected status {}: {}", code, msg)
            }
        }
    }
}

// --- Implementations ---

impl DynaClient {
    /// Creates a new configured instance of the DynaClient.
    /// 
    /// # Example
    /// ```
    /// let client = DynaClient::new("http://localhost:6660");
    /// ```
    pub fn new(base_url: &str) -> Self {
        Self {
            // Trim trailing slashes to prevent double-slashes in URL formatting
            base_url: base_url.trim_end_matches('/').to_string(),
            http_client: Client::new(),
            jwt_token: None,
        }
    }

    /// Attaches a JWT token to the client for authenticated requests (PUT/DELETE).
    pub fn set_token(&mut self, token: String) {
        self.jwt_token = Some(token);
    }

    /// Fetches a value from the DynaRust database.
    /// According to the docs, GET requests do NOT require authentication.
    /// 
    /// # Arguments
    /// * `table` - The table name (e.g., "default")
    /// * `key` - The key to retrieve
    pub async fn get_value<T: for<'de> Deserialize<'de>>(
        &self,
        table: &str,
        key: &str,
    ) -> Result<VersionedValue<T>, DynaError> {
        let url = format!("{}/{}/key/{}", self.base_url, table, key);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(DynaError::RequestFailed)?;

        match response.status() {
            StatusCode::OK => {
                let data = response
                    .json::<VersionedValue<T>>()
                    .await
                    .map_err(DynaError::RequestFailed)?;
                Ok(data)
            }
            StatusCode::NOT_FOUND => Err(DynaError::NotFound),
            status => {
                let text = response.text().await.unwrap_or_default();
                Err(DynaError::UnexpectedStatus(status.as_u16(), text))
            }
        }
    }
}