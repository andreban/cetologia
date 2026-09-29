// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! HTTP client for the DeepSeek API.

use std::time::Duration;

use reqwest::{Client, Response};
use serde::Deserialize;
use tokio_stream::{Stream, StreamExt};
use tokio_util::codec::LinesCodecError;

use crate::{
    error::{CetologiaError, Result},
    network::event_source::{EventSource, ServerSentEvent},
    types::{
        chat::{ChatCompletionRequest, StreamOptions},
        response::{ChatCompletionChunk, ChatCompletionResponse},
    },
};

/// Default DeepSeek API base URL.
pub const DEFAULT_BASE_URL: &str = "https://api.deepseek.com";

/// DeepSeek API client.
#[derive(Clone, Debug)]
pub struct CetologiaClient {
    api_key: String,
    base_url: String,
    client: Client,
}

/// Convenience alias for [`CetologiaClient`].
pub type DeepSeekClient = CetologiaClient;

impl CetologiaClient {
    /// Creates a new client with the given API key and default base URL (`https://api.deepseek.com`).
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::builder(api_key).build()
    }

    /// Creates a builder to configure the client.
    pub fn builder(api_key: impl Into<String>) -> CetologiaClientBuilder {
        CetologiaClientBuilder::new(api_key)
    }

    /// Returns the configured base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Sends a non-streaming chat completion request.
    pub async fn chat(&self, request: &ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        let mut req = request.clone();
        req.stream = Some(false);

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await?;

        let response = Self::handle_error_status(response).await?;
        let result = response.json::<ChatCompletionResponse>().await?;
        Ok(result)
    }

    /// Sends a streaming chat completion request and returns an SSE stream of [`ChatCompletionChunk`]s.
    pub async fn chat_stream(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<impl Stream<Item = Result<ChatCompletionChunk>>> {
        let mut req = request.clone();
        req.stream = Some(true);
        if req.stream_options.is_none() {
            req.stream_options = Some(StreamOptions { include_usage: true });
        }

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await?;

        let response = Self::handle_error_status(response).await?;

        Ok(response.event_stream().filter_map(Self::parse_sse_event))
    }

    /// Parses an SSE event into a [`ChatCompletionChunk`].
    fn parse_sse_event(
        event_res: std::result::Result<ServerSentEvent, LinesCodecError>,
    ) -> Option<Result<ChatCompletionChunk>> {
        let sse = match event_res {
            Ok(ev) => ev,
            Err(e) => return Some(Err(CetologiaError::Stream(e.to_string()))),
        };

        let data = sse.data?;
        let trimmed = data.trim();
        if trimmed.is_empty() {
            return None;
        }

        // DeepSeek signals completion of the SSE stream with [DONE]
        if trimmed == "[DONE]" {
            return None;
        }

        match serde_json::from_str::<ChatCompletionChunk>(trimmed) {
            Ok(chunk) => Some(Ok(chunk)),
            Err(e) => {
                tracing::warn!(data = trimmed, error = %e, "Failed to parse ChatCompletionChunk");
                Some(Err(CetologiaError::Json(e)))
            }
        }
    }

    /// Inspects HTTP response status and decodes error payloads if not 2xx.
    async fn handle_error_status(response: Response) -> Result<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let body = response.text().await.unwrap_or_default();
        if let Ok(err_envelope) = serde_json::from_str::<ErrorEnvelope>(&body) {
            return Err(CetologiaError::Api {
                status,
                message: err_envelope.error.message,
                error_type: err_envelope.error.error_type,
                code: err_envelope.error.code,
            });
        }

        Err(CetologiaError::Api {
            status,
            message: if body.is_empty() {
                format!("HTTP error {}", status)
            } else {
                body
            },
            error_type: None,
            code: None,
        })
    }
}

/// Internal struct for deserializing DeepSeek error responses.
#[derive(Deserialize)]
struct ErrorEnvelope {
    error: ApiErrorDetail,
}

#[derive(Deserialize)]
struct ApiErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<String>,
}

/// Builder for [`CetologiaClient`].
pub struct CetologiaClientBuilder {
    api_key: String,
    base_url: String,
    timeout: Option<Duration>,
    custom_client: Option<Client>,
}

impl CetologiaClientBuilder {
    /// Creates a new builder with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: None,
            custom_client: None,
        }
    }

    /// Sets a custom base URL (e.g. for proxy or self-hosted endpoint).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Sets request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets a custom [`reqwest::Client`].
    pub fn client(mut self, client: Client) -> Self {
        self.custom_client = Some(client);
        self
    }

    /// Builds the [`CetologiaClient`].
    pub fn build(self) -> CetologiaClient {
        let client = self.custom_client.unwrap_or_else(|| {
            let mut builder = Client::builder();
            if let Some(t) = self.timeout {
                builder = builder.timeout(t);
            }
            builder.build().unwrap_or_default()
        });

        CetologiaClient {
            api_key: self.api_key,
            base_url: self.base_url,
            client,
        }
    }
}
