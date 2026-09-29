// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use cetologia::prelude::*;
use serde_json::json;

#[test]
fn test_serialize_chat_request_with_reasoning() {
    let req = ChatCompletionRequest::builder("deepseek-reasoner")
        .messages(vec![
            ChatMessage::user("Why is the sky blue?"),
            ChatMessage::assistant_with_reasoning(
                Some("Rayleigh scattering.".into()),
                Some("The user is asking about optics and Rayleigh scattering...".into()),
            ),
            ChatMessage::user("Tell me more."),
        ])
        .temperature(0.6)
        .build();

    let val = serde_json::to_value(&req).unwrap();
    assert_eq!(val["model"], "deepseek-reasoner");
    assert_eq!(
        val["messages"][1]["reasoning_content"],
        "The user is asking about optics and Rayleigh scattering..."
    );
    assert_eq!(val["messages"][1]["content"], "Rayleigh scattering.");
}

#[test]
fn test_deserialize_response_with_reasoning_and_usage() {
    let raw = json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1700000000,
        "model": "deepseek-reasoner",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "42",
                    "reasoning_content": "Deep thought computed for 7.5 million years."
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 15,
            "completion_tokens": 5,
            "total_tokens": 20,
            "prompt_cache_hit_tokens": 10,
            "prompt_cache_miss_tokens": 5
        }
    });

    let resp: ChatCompletionResponse = serde_json::from_value(raw).unwrap();
    assert_eq!(resp.text(), Some("42"));
    assert_eq!(
        resp.reasoning_content(),
        Some("Deep thought computed for 7.5 million years.")
    );

    let usage = resp.usage.unwrap();
    assert_eq!(usage.prompt_cache_hit_tokens, Some(10));
    assert_eq!(usage.prompt_cache_miss_tokens, Some(5));
}

#[test]
fn test_deserialize_stream_chunk_with_tool_calls() {
    let raw = json!({
        "id": "chunk-1",
        "object": "chat.completion.chunk",
        "created": 1700000001,
        "model": "deepseek-chat",
        "choices": [
            {
                "index": 0,
                "delta": {
                    "role": "assistant",
                    "tool_calls": [
                        {
                            "index": 0,
                            "id": "call_abc",
                            "type": "function",
                            "function": {
                                "name": "get_weather",
                                "arguments": "{\"city\":"
                            }
                        }
                    ]
                },
                "finish_reason": null
            }
        ]
    });

    let chunk: ChatCompletionChunk = serde_json::from_value(raw).unwrap();
    let choice = &chunk.choices[0];
    let tc = &choice.delta.tool_calls.as_ref().unwrap()[0];
    assert_eq!(tc.id.as_deref(), Some("call_abc"));
    assert_eq!(tc.function.as_ref().unwrap().name.as_deref(), Some("get_weather"));
    assert_eq!(tc.function.as_ref().unwrap().arguments.as_deref(), Some("{\"city\":"));
}

#[test]
fn test_deserialize_stream_chunk_with_reasoning_delta() {
    let raw = json!({
        "id": "chunk-2",
        "object": "chat.completion.chunk",
        "created": 1700000002,
        "model": "deepseek-reasoner",
        "choices": [
            {
                "index": 0,
                "delta": {
                    "reasoning_content": "Thinking step 1..."
                },
                "finish_reason": null
            }
        ]
    });

    let chunk: ChatCompletionChunk = serde_json::from_value(raw).unwrap();
    assert_eq!(
        chunk.choices[0].delta.reasoning_content.as_deref(),
        Some("Thinking step 1...")
    );
    assert_eq!(chunk.choices[0].delta.content, None);
}
