// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! DeepSeek API data transfer objects.

pub mod chat;
pub mod response;
pub mod tools;

pub use chat::{
    ChatMessage, ChatCompletionRequest, ChatCompletionRequestBuilder, ResponseFormat, Role,
    StreamOptions,
};
pub use response::{
    ChatCompletionChunk, ChatCompletionResponse, Choice, ChunkChoice, ChunkDelta, PromptTokensDetails,
    ResponseMessage, Usage,
};
pub use tools::{
    ChunkFunctionCall, ChunkToolCall, FunctionCall, FunctionDefinition, NamedToolChoice,
    NamedToolChoiceFunction, Tool, ToolCall, ToolCallAccumulator, ToolChoice, ToolChoiceMode,
    ToolType,
};
