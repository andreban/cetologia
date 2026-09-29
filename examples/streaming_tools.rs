// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! Streaming tool calling example using `deepseek-chat` and `ToolCallAccumulator`.
//!
//! Demonstrates streaming tool call deltas and reconstructing complete `ToolCall`
//! objects using Cetologia's `ToolCallAccumulator`.
//!
//! Run with:
//! ```bash
//! cargo run --example streaming_tools
//! ```

use std::io::{self, Write};
use cetologia::prelude::*;
use serde_json::json;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| {
        eprintln!("Error: DEEPSEEK_API_KEY is not set.");
        eprintln!("Please create a .env file (see .env.example) or export DEEPSEEK_API_KEY.");
        std::process::exit(1);
    });

    let client = CetologiaClient::new(api_key);

    let calculator_tool = Tool::function(
        "calculate",
        Some("Perform an arithmetic computation".to_string()),
        Some(json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Mathematical expression to evaluate, e.g. 144 * 25"
                }
            },
            "required": ["expression"]
        })),
    );

    let user_prompt = "How much is 144 * 25? Use the calculator tool.";
    println!("Prompt: {}\n", user_prompt);

    let mut messages = vec![ChatMessage::user(user_prompt)];

    let request = ChatCompletionRequest::builder("deepseek-chat")
        .messages(messages.clone())
        .tools(vec![calculator_tool])
        .tool_choice(ToolChoice::Mode(ToolChoiceMode::Auto))
        .stream(true)
        .build();

    let mut stream = client.chat_stream(&request).await?;
    let mut accumulator = ToolCallAccumulator::new();

    println!("Streaming response chunks from DeepSeek...");

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res?;
        if let Some(choice) = chunk.choices.first() {
            if let Some(content) = &choice.delta.content {
                print!("{}", content);
                io::stdout().flush().ok();
            }
            if let Some(tool_calls) = &choice.delta.tool_calls {
                // Ingest chunk deltas into the accumulator
                accumulator.update_all(tool_calls);
                print!(".");
                io::stdout().flush().ok();
            }
        }
    }
    println!("\n");

    let tool_calls = accumulator.finish();
    if tool_calls.is_empty() {
        println!("No tool calls were generated.");
        return Ok(());
    }

    println!("=== Reconstructed Tool Calls from Stream ===");
    for tc in &tool_calls {
        println!("ID:        {}", tc.id);
        println!("Function:  {}", tc.function.name);
        println!("Arguments: {}", tc.function.arguments);
    }

    // Append the assistant tool-call message
    messages.push(ChatMessage::assistant_tool_calls(tool_calls.clone()));

    // Execute mock calculator
    for tc in &tool_calls {
        let result = if tc.function.name == "calculate" {
            // 144 * 25 = 3600
            json!({ "result": 3600 }).to_string()
        } else {
            json!({ "error": "Unknown tool" }).to_string()
        };

        messages.push(ChatMessage::tool(&tc.id, result));
    }

    // Get final answer
    let final_request = ChatCompletionRequest::builder("deepseek-chat")
        .messages(messages)
        .build();

    let final_response = client.chat(&final_request).await?;
    if let Some(text) = final_response.text() {
        println!("\n=== Final Answer ===");
        println!("{}", text);
    }

    Ok(())
}
