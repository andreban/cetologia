// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! Error types for the Cetologia client.

use thiserror::Error;

/// Errors returned by the Cetologia client.
#[derive(Debug, Error)]
pub enum CetologiaError {
    /// Network or HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization or deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error returned by the DeepSeek API endpoint.
    #[error("API error (status {status}): {message}")]
    Api {
        status: reqwest::StatusCode,
        message: String,
        error_type: Option<String>,
        code: Option<String>,
    },

    /// Streaming / SSE decoding error.
    #[error("Stream error: {0}")]
    Stream(String),

    /// Invalid request error before sending.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

/// Convenience type alias for [`Result<T, CetologiaError>`].
pub type Result<T> = std::result::Result<T, CetologiaError>;
