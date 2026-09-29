// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! Basic chat completion example using `deepseek-chat` (DeepSeek-V3).
//!
//! Run with:
//! ```bash
//! cargo run --example basic_chat
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

    let client = CetologiaClient::new(api_key);

    println!("Sending chat request to DeepSeek-Chat (V3)...");

    let request = ChatCompletionRequest::builder("deepseek-chat")
        .messages(vec![
            ChatMessage::system("You are an enthusiastic marine biologist specializing in cetaceans."),
            ChatMessage::user("What is cetology, and why are sperm whales so fascinating?"),
        ])
        .temperature(0.7)
        .max_tokens(1024)
        .build();

    let response = client.chat(&request).await?;

    if let Some(text) = response.text() {
        println!("\n=== DeepSeek-Chat Response ===\n");
        println!("{}", text);
    }

    if let Some(usage) = &response.usage {
        println!("\n=== Token Usage & Cache Metrics ===");
        println!("Prompt tokens:     {}", usage.prompt_tokens);
        println!("Completion tokens: {}", usage.completion_tokens);
        println!("Total tokens:      {}", usage.total_tokens);
        if let Some(hits) = usage.prompt_cache_hit_tokens {
            println!("Cache hit tokens:  {}", hits);
        }
        if let Some(misses) = usage.prompt_cache_miss_tokens {
            println!("Cache miss tokens: {}", misses);
        }
    }

    Ok(())
}
