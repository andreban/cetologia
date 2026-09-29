// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! Chat completion request types for the DeepSeek API.

use serde::{Deserialize, Serialize};

use super::tools::{Tool, ToolChoice};

/// Role of a message in a conversation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System prompt or instructions.
    System,
    /// Human user prompt.
    User,
    /// Assistant response.
    Assistant,
    /// Tool result response.
    Tool,
}

/// Message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChatMessage {
    /// The role of the author.
    pub role: Role,

    /// Text content of the message. Optional when `tool_calls` are present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// An optional name for the participant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The reasoning content for DeepSeek-R1 (`deepseek-reasoner`).
    ///
    /// When continuing a multi-turn conversation with `deepseek-reasoner`,
    /// DeepSeek expects previous assistant turns to preserve this field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,

    /// Tool calls made by the assistant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<super::tools::ToolCall>>,

    /// The ID of the tool call this message is responding to (when `role` is `Tool`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,

    /// DeepSeek prefix completion toggle for assistant messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<bool>,
}

impl Default for Role {
    fn default() -> Self {
        Role::User
    }
}

impl ChatMessage {
    /// Creates a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: Some(content.into()),
            ..Default::default()
        }
    }

    /// Creates a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: Some(content.into()),
            ..Default::default()
        }
    }

    /// Creates an assistant text message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: Some(content.into()),
            ..Default::default()
        }
    }

    /// Creates an assistant message with both reasoning and final text content.
    pub fn assistant_with_reasoning(
        content: Option<String>,
        reasoning_content: Option<String>,
    ) -> Self {
        Self {
            role: Role::Assistant,
            content,
            reasoning_content,
            ..Default::default()
        }
    }

    /// Creates an assistant message containing tool calls.
    pub fn assistant_tool_calls(calls: Vec<super::tools::ToolCall>) -> Self {
        Self {
            role: Role::Assistant,
            tool_calls: Some(calls),
            ..Default::default()
        }
    }

    /// Creates a tool response message.
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            tool_call_id: Some(tool_call_id.into()),
            content: Some(content.into()),
            ..Default::default()
        }
    }
}

/// Output format for the response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    /// Plain text format.
    Text,
    /// JSON object format (requires system instructions describing the schema).
    JsonObject,
}

/// Streaming options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StreamOptions {
    /// Whether to include token usage statistics in the streaming response.
    pub include_usage: bool,
}

/// Request body for the DeepSeek `/chat/completions` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChatCompletionRequest {
    /// Model name (e.g. `"deepseek-chat"` or `"deepseek-reasoner"`).
    pub model: String,

    /// Conversation messages.
    pub messages: Vec<ChatMessage>,

    /// Frequency penalty between -2.0 and 2.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Maximum tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Presence penalty between -2.0 and 2.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// Structured output specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// Stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Whether to stream partial message deltas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Streaming configuration options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

    /// Sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Nucleus sampling threshold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Tools available for the model to call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Tool calling control mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Whether to return log probabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// Top log probabilities count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
}

impl ChatCompletionRequest {
    /// Creates a builder for a chat completion request.
    pub fn builder(model: impl Into<String>) -> ChatCompletionRequestBuilder {
        ChatCompletionRequestBuilder::new(model)
    }
}

/// Builder for [`ChatCompletionRequest`].
pub struct ChatCompletionRequestBuilder {
    request: ChatCompletionRequest,
}

impl ChatCompletionRequestBuilder {
    /// Creates a new builder with the specified model identifier.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            request: ChatCompletionRequest {
                model: model.into(),
                ..Default::default()
            },
        }
    }

    /// Sets the conversation messages.
    pub fn messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.request.messages = messages;
        self
    }

    /// Appends a single message to the conversation.
    pub fn add_message(mut self, message: ChatMessage) -> Self {
        self.request.messages.push(message);
        self
    }

    /// Sets sampling temperature.
    pub fn temperature(mut self, temp: f32) -> Self {
        self.request.temperature = Some(temp);
        self
    }

    /// Sets top-p nucleus sampling.
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.request.top_p = Some(top_p);
        self
    }

    /// Sets max tokens.
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.request.max_tokens = Some(max_tokens);
        self
    }

    /// Sets frequency penalty.
    pub fn frequency_penalty(mut self, penalty: f32) -> Self {
        self.request.frequency_penalty = Some(penalty);
        self
    }

    /// Sets presence penalty.
    pub fn presence_penalty(mut self, penalty: f32) -> Self {
        self.request.presence_penalty = Some(penalty);
        self
    }

    /// Sets response format.
    pub fn response_format(mut self, format: ResponseFormat) -> Self {
        self.request.response_format = Some(format);
        self
    }

    /// Sets stop sequences.
    pub fn stop(mut self, stop: Vec<String>) -> Self {
        self.request.stop = Some(stop);
        self
    }

    /// Sets streaming mode.
    pub fn stream(mut self, stream: bool) -> Self {
        self.request.stream = Some(stream);
        self
    }

    /// Sets stream options (e.g. including token usage).
    pub fn stream_options(mut self, options: StreamOptions) -> Self {
        self.request.stream_options = Some(options);
        self
    }

    /// Sets available tools.
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.request.tools = Some(tools);
        self
    }

    /// Sets tool choice.
    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.request.tool_choice = Some(choice);
        self
    }

    /// Builds the [`ChatCompletionRequest`].
    pub fn build(self) -> ChatCompletionRequest {
        self.request
    }
}
