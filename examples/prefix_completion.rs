// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! Prefix completion example using `deepseek-chat` (DeepSeek-V3).
//!
//! DeepSeek supports "prefix completion" by passing an assistant message with
//! `prefix: Some(true)` as the final message in `messages`. DeepSeek will force
//! its output to continue directly from that prefix text.
//!
//! Useful for:
//! - Enforcing code formats / function signatures
//! - Starting JSON / markdown with exact opening tokens
//! - Steering responses toward specific styles
//!
//! Run with:
//! ```bash
//! cargo run --example prefix_completion
//! ```

use cetologia::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| {
        eprintln!("Error: DEEPSEEK_API_KEY is not set.");
        eprintln!("Please create a .env file (see .env.example) or export DEEPSEEK_API_KEY.");
        std::process::exit(1);
    });

    // DeepSeek requires the beta endpoint (https://api.deepseek.com/beta) for prefix completion
    let client = CetologiaClient::builder(api_key)
        .base_url("https://api.deepseek.com/beta")
        .build();

    let prompt = "Write a fast Rust function to calculate the nth Fibonacci number.";
    let prefix_text = "```rust\n/// Computes the nth Fibonacci number iteratively in O(n) time and O(1) space.\npub fn fibonacci(n: u32) -> u64 {";

    println!("Prompt: {}\n", prompt);
    println!("Enforced assistant prefix:\n{}\n", prefix_text);

    let prefix_message = ChatMessage {
        role: Role::Assistant,
        content: Some(prefix_text.to_string()),
        prefix: Some(true),
        ..Default::default()
    };

    let request = ChatCompletionRequest::builder("deepseek-chat")
        .messages(vec![
            ChatMessage::user(prompt),
            prefix_message,
        ])
        .temperature(0.3)
        .build();

    let response = client.chat(&request).await?;

    if let Some(completion) = response.text() {
        println!("=== DeepSeek Output (Continuation) ===");
        // DeepSeek returns the completion that follows the prefix
        println!("{}{}", prefix_text, completion);
    }

    Ok(())
}
