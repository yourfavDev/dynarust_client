use futures_util::{Stream, StreamExt};
use reqwest::{Client, StatusCode};
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

// --- Models ---

/// Represents the data structure returned by DynaRust.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionedValue<T = Value> {
    pub value: T,
    pub version: u64,
    pub timestamp: u64,
    pub owner: String,
}

#[derive(Serialize)]
struct AuthRequest<'a> {
    secret: &'a str,
}

#[derive(Deserialize)]
struct AuthResponse {
    token: Option<String>,
    #[allow(dead_code)] // status is returned on registration
    status: Option<String>,
}

// --- Configuration & Client ---

/// The main client used to interact with the DynaRust cluster.
#[derive(Debug, Clone)]
pub struct DynaClient {
    pub base_url: String,
    http_client: Client,
    pub jwt_token: Option<String>,
}

// --- Error Handling ---

#[derive(Debug)]
pub enum DynaError {
    RequestFailed(reqwest::Error),
    NotFound,
    Unauthorized,
    UnexpectedStatus(u16, String),
    ParseError(String),
    StreamError(String),
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
            DynaError::ParseError(msg) => write!(f, "Failed to parse data: {}", msg),
            DynaError::StreamError(msg) => write!(f, "SSE Stream error: {}", msg),
        }
    }
}

// --- Implementations ---

impl DynaClient {
    /// Creates a new configured instance of the DynaClient.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http_client: Client::new(),
            jwt_token: None,
        }
    }

    /// Manually attaches a JWT token to the client.
    pub fn set_token(&mut self, token: String) {
        self.jwt_token = Some(token);
    }

    /// Helper to get the token or return an Unauthorized error
    fn get_bearer(&self) -> Result<String, DynaError> {
        self.jwt_token
            .as_ref()
            .map(|t| format!("Bearer {}", t))
            .ok_or(DynaError::Unauthorized)
    }

    // --- Core API Methods ---

    /// Registers a new user or logs in an existing one.
    /// Automatically saves the JWT token to the client instance if successful.
    pub async fn auth(&mut self, user: &str, secret: &str) -> Result<(), DynaError> {
        let url = format!("{}/auth/{}", self.base_url, user);
        let payload = AuthRequest { secret };

        let response = self
            .http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(DynaError::RequestFailed)?;

        if response.status().is_success() {
            let auth_data = response
                .json::<AuthResponse>()
                .await
                .map_err(DynaError::RequestFailed)?;

            // If a token is returned (login), store it. 
            // If it's just a status message (registration), we might need to log in again or wait for next call.
            if let Some(token) = auth_data.token {
                self.jwt_token = Some(token);
            }
            Ok(())
        } else if response.status() == StatusCode::UNAUTHORIZED {
            Err(DynaError::Unauthorized)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(DynaError::UnexpectedStatus(status.as_u16(), text))
        }
    }

    /// Fetches a value from the DynaRust database (No auth required).
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

    /// Creates or updates a value. Requires the client to be authenticated as the owner.
    pub async fn put_value<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        table: &str,
        key: &str,
        value: &T,
    ) -> Result<VersionedValue<T>, DynaError> {
        let bearer = self.get_bearer()?;
        let url = format!("{}/{}/key/{}", self.base_url, table, key);

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", bearer)
            .json(value)
            .send()
            .await
            .map_err(DynaError::RequestFailed)?;

        match response.status() {
            StatusCode::CREATED | StatusCode::OK => {
                let data = response
                    .json::<VersionedValue<T>>()
                    .await
                    .map_err(DynaError::RequestFailed)?;
                Ok(data)
            }
            StatusCode::UNAUTHORIZED => Err(DynaError::Unauthorized),
            status => {
                let text = response.text().await.unwrap_or_default();
                Err(DynaError::UnexpectedStatus(status.as_u16(), text))
            }
        }
    }

    /// Deletes a value. Requires the client to be authenticated as the owner.
    pub async fn delete_value(&self, table: &str, key: &str) -> Result<(), DynaError> {
        let bearer = self.get_bearer()?;
        let url = format!("{}/{}/key/{}", self.base_url, table, key);

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", bearer)
            .send()
            .await
            .map_err(DynaError::RequestFailed)?;

        match response.status() {
            StatusCode::OK => Ok(()),
            StatusCode::NOT_FOUND => Err(DynaError::NotFound),
            StatusCode::UNAUTHORIZED => Err(DynaError::Unauthorized),
            status => {
                let text = response.text().await.unwrap_or_default();
                Err(DynaError::UnexpectedStatus(status.as_u16(), text))
            }
        }
    }

    /// Subscribes to a key using Server-Sent Events (SSE).
    /// Yields a stream of real-time updates for the generic type T.
    pub async fn subscribe<T: for<'de> Deserialize<'de> + 'static>(
        &self,
        table: &str,
        key: &str,
    ) -> Result<impl Stream<Item = Result<VersionedValue<T>, DynaError>>, DynaError> {
        let url = format!("{}/{}/subscribe/{}", self.base_url, table, key);

        // We use reqwest_eventsource to handle the SSE connection
        let mut event_source = EventSource::get(&url);

        // Convert the raw EventSource into a clean Rust stream of typed structs
        let stream = async_stream::stream! {
            while let Some(event) = event_source.next().await {
                match event {
                    Ok(Event::Open) => continue, // Connection established
                    Ok(Event::Message(message)) => {
                        // DynaRust pushes JSON. We parse it: {"event": "Updated", "value": {...}}
                        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&message.data);
                        if let Ok(json_obj) = parsed {
                            if let Some(val) = json_obj.get("value") {
                                match serde_json::from_value::<VersionedValue<T>>(val.clone()) {
                                    Ok(versioned_val) => yield Ok(versioned_val),
                                    Err(e) => yield Err(DynaError::ParseError(e.to_string())),
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(DynaError::StreamError(e.to_string()));
                        break; // Stop streaming on connection error
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}