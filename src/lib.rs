// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! # cetologia
//!
//! An ergonomic, async Rust client for the DeepSeek API.
//!
//! Named after *cetology* (the scientific study of whales), inspired by
//! DeepSeek's whale mascot and following the naming tradition of [`geologia`](https://github.com/andreban/geologia).
//!
//! ## Features
//!
//! - **Full DeepSeek Model Support**: First-class handling of `deepseek-chat` and `deepseek-reasoner` (R1).
//! - **Streaming & Reasoning**: Preserves and streams `reasoning_content` (thinking tokens) alongside message text.
//! - **Multi-Turn Reasoning**: Preserves reasoning traces on assistant messages to avoid 400 Bad Request errors.
//! - **Tool / Function Calling**: Fully typed tool definitions, choices, and chunked streaming aggregation.
//! - **Prompt Caching Metrics**: Exposes `prompt_cache_hit_tokens` and `prompt_cache_miss_tokens` in `usage`.
//!
//! ## Example
//!
//! ```no_run
//! use cetologia::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
//!     let client = CetologiaClient::new("DEEPSEEK_API_KEY");
//!
//!     let request = ChatCompletionRequest::builder("deepseek-chat")
//!         .messages(vec![
//!             ChatMessage::system("You are a helpful assistant."),
//!             ChatMessage::user("Hello!"),
//!         ])
//!         .build();
//!
//!     let response = client.chat(&request).await?;
//!     println!("Response: {:?}", response.text());
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod network;
pub mod types;

pub use client::{CetologiaClient, CetologiaClientBuilder, DeepSeekClient, DEFAULT_BASE_URL};
pub use error::{CetologiaError, Result};
pub use types::*;

/// Convenient re-exports for common usage.
pub mod prelude {
    pub use crate::client::{CetologiaClient, CetologiaClientBuilder, DeepSeekClient};
    pub use crate::error::{CetologiaError, Result};
    pub use crate::types::chat::{
        ChatMessage, ChatCompletionRequest, ChatCompletionRequestBuilder, ResponseFormat, Role,
        StreamOptions,
    };
    pub use crate::types::response::{
        ChatCompletionChunk, ChatCompletionResponse, Choice, ChunkChoice, ChunkDelta,
        ResponseMessage, Usage,
    };
    pub use crate::types::tools::{
        ChunkFunctionCall, ChunkToolCall, FunctionCall, FunctionDefinition, NamedToolChoice, Tool,
        ToolChoice, ToolChoiceMode, ToolType,
    };
}
