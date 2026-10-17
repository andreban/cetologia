// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! Chat completion request types for the DeepSeek API.

use serde::{Deserialize, Serialize};

use super::response::ResponseMessage;
use super::tools::{Tool, ToolChoice};

/// Role of a message in a conversation.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System prompt or instructions.
    System,
    /// Human user prompt.
    #[default]
    User,
    /// Assistant response.
    Assistant,
    /// Tool result response.
    Tool,
}

/// An image URL or `data:` URI inside an [`ContentPart::ImageUrl`] block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageUrl {
    /// Image URL or base64 data URI (e.g. `"data:image/png;base64,..."`).
    pub url: String,
    /// Optional resolution detail (`"low"`, `"high"`, `"auto"`, `"original"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ImageUrl {
    /// Creates an [`ImageUrl`] from a URL or `data:` URI.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            detail: None,
        }
    }

    /// Sets the optional detail level.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// A single content part in a multimodal message (`text` or `image_url`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// A plain text block.
    Text {
        /// The text content.
        text: String,
    },
    /// An image block (`image_url`).
    ImageUrl {
        /// The image URL or base64 data URI.
        image_url: ImageUrl,
    },
}

impl ContentPart {
    /// Creates a text content part.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Creates an `image_url` content part from a URL or `data:` URI.
    pub fn image_url(url: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl::new(url),
        }
    }

    /// Creates an `image_url` content part from inline base64 data and MIME type.
    pub fn image_base64(mime_type: impl AsRef<str>, data: impl AsRef<str>) -> Self {
        Self::image_url(format!(
            "data:{};base64,{}",
            mime_type.as_ref(),
            data.as_ref()
        ))
    }
}

/// Content of a [`ChatMessage`]: either a single string or a list of multimodal [`ContentPart`]s.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum MessageContent {
    /// Plain string content.
    Text(String),
    /// Array of multimodal content parts (text and/or images).
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    /// Returns the content if it is a single plain string (`MessageContent::Text`).
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            Self::Parts(_) => None,
        }
    }

    /// Returns the content parts if this is `MessageContent::Parts`.
    pub fn as_parts(&self) -> Option<&[ContentPart]> {
        match self {
            Self::Text(_) => None,
            Self::Parts(parts) => Some(parts),
        }
    }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl From<Vec<ContentPart>> for MessageContent {
    fn from(parts: Vec<ContentPart>) -> Self {
        Self::Parts(parts)
    }
}

impl PartialEq<str> for MessageContent {
    fn eq(&self, other: &str) -> bool {
        matches!(self, Self::Text(s) if s == other)
    }
}

impl PartialEq<&str> for MessageContent {
    fn eq(&self, other: &&str) -> bool {
        matches!(self, Self::Text(s) if s == *other)
    }
}

/// Message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChatMessage {
    /// The role of the author.
    pub role: Role,

    /// Content of the message (plain string or multimodal content parts).
    /// Optional when `tool_calls` are present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,

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

impl ChatMessage {
    /// Creates a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: Some(MessageContent::Text(content.into())),
            ..Default::default()
        }
    }

    /// Creates a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: Some(MessageContent::Text(content.into())),
            ..Default::default()
        }
    }

    /// Creates a user message with multimodal content parts.
    pub fn user_parts(parts: Vec<ContentPart>) -> Self {
        Self {
            role: Role::User,
            content: Some(MessageContent::Parts(parts)),
            ..Default::default()
        }
    }

    /// Creates an assistant text message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: Some(MessageContent::Text(content.into())),
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
            content: content.map(MessageContent::Text),
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

    /// Creates a tool response message with text content.
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            tool_call_id: Some(tool_call_id.into()),
            content: Some(MessageContent::Text(content.into())),
            ..Default::default()
        }
    }

    /// Creates a tool response message with multimodal content parts.
    pub fn tool_parts(tool_call_id: impl Into<String>, parts: Vec<ContentPart>) -> Self {
        Self {
            role: Role::Tool,
            tool_call_id: Some(tool_call_id.into()),
            content: Some(MessageContent::Parts(parts)),
            ..Default::default()
        }
    }

    /// Returns the message content if it is a single plain string (`MessageContent::Text`).
    pub fn text(&self) -> Option<&str> {
        self.content.as_ref()?.as_text()
    }
}

impl From<ResponseMessage> for ChatMessage {
    fn from(msg: ResponseMessage) -> Self {
        Self {
            role: msg.role,
            content: msg.content.map(MessageContent::Text),
            name: None,
            reasoning_content: msg.reasoning_content,
            tool_calls: msg.tool_calls,
            tool_call_id: None,
            prefix: None,
        }
    }
}

impl From<&ResponseMessage> for ChatMessage {
    fn from(msg: &ResponseMessage) -> Self {
        Self {
            role: msg.role,
            content: msg.content.clone().map(MessageContent::Text),
            name: None,
            reasoning_content: msg.reasoning_content.clone(),
            tool_calls: msg.tool_calls.clone(),
            tool_call_id: None,
            prefix: None,
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

/// Level of reasoning effort for DeepSeek models in thinking mode.
///
/// Supported values are `none` (disables thinking), `low`, `high` (default),
/// and `max`.
///
/// Note: When thinking mode is active, DeepSeek ignores `temperature`,
/// `presence_penalty`, and `frequency_penalty`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    /// Disables thinking mode.
    None,
    /// Low reasoning effort.
    Low,
    /// High reasoning effort (default for reasoning models).
    High,
    /// Maximum reasoning effort.
    Max,
}

/// Request body for the DeepSeek `/chat/completions` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChatCompletionRequest {
    /// Model name (e.g. `"deepseek-chat"` or `"deepseek-reasoner"`).
    pub model: String,

    /// Conversation messages.
    pub messages: Vec<ChatMessage>,

    /// Frequency penalty between -2.0 and 2.0.
    ///
    /// Note: Ignored by DeepSeek when thinking mode is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Maximum tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Presence penalty between -2.0 and 2.0.
    ///
    /// Note: Ignored by DeepSeek when thinking mode is active.
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
    ///
    /// Note: Ignored by DeepSeek when thinking mode is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Nucleus sampling threshold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Level of reasoning effort for DeepSeek models in thinking mode.
    ///
    /// Note: DeepSeek's thinking mode ignores `temperature`, `presence_penalty`,
    /// and `frequency_penalty`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,

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
    ///
    /// Note: Ignored by DeepSeek when thinking mode is active.
    pub fn temperature(mut self, temp: f32) -> Self {
        self.request.temperature = Some(temp);
        self
    }

    /// Sets top-p nucleus sampling.
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.request.top_p = Some(top_p);
        self
    }

    /// Sets reasoning effort for thinking mode (`none`, `low`, `high`, `max`).
    ///
    /// Note: When thinking mode is active, DeepSeek ignores `temperature`,
    /// `presence_penalty`, and `frequency_penalty`.
    pub fn reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.request.reasoning_effort = Some(effort);
        self
    }

    /// Sets max tokens.
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.request.max_tokens = Some(max_tokens);
        self
    }

    /// Sets frequency penalty.
    ///
    /// Note: Ignored by DeepSeek when thinking mode is active.
    pub fn frequency_penalty(mut self, penalty: f32) -> Self {
        self.request.frequency_penalty = Some(penalty);
        self
    }

    /// Sets presence penalty.
    ///
    /// Note: Ignored by DeepSeek when thinking mode is active.
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
