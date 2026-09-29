// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! Multi-turn conversation example with `deepseek-reasoner` (DeepSeek-R1).
//!
//! DeepSeek-R1 requires that reasoning tokens (`reasoning_content`) from prior
//! assistant turns be preserved in subsequent turns of the conversation.
//! Omitting `reasoning_content` causes DeepSeek to respond with HTTP 400.
//!
//! In Cetologia, converting a `ResponseMessage` to `ChatMessage` via
//! `ChatMessage::from(&choice.message)` preserves `reasoning_content`
//! automatically.
//!
//! Run with:
//! ```bash
//! cargo run --example multi_turn_reasoning
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

    let mut messages = Vec::new();

    // --- Turn 1 ---
    let prompt1 = "A farmer needs to cross a river with a wolf, a goat, and a cabbage. \
                   His boat can only carry him and one other item. \
                   If left alone together, the wolf will eat the goat, or the goat will eat the cabbage. \
                   How does he get everything across safely?";

    println!("=== Turn 1: User ===");
    println!("{}\n", prompt1);

    messages.push(ChatMessage::user(prompt1));

    let request1 = ChatCompletionRequest::builder("deepseek-reasoner")
        .messages(messages.clone())
        .build();

    let response1 = client.chat(&request1).await?;
    let choice1 = response1
        .choices
        .first()
        .expect("Expected at least one choice from DeepSeek");

    if let Some(reasoning) = &choice1.message.reasoning_content {
        println!("=== Turn 1: DeepSeek-R1 Reasoning (Truncated preview) ===");
        let preview: String = reasoning.chars().take(200).collect();
        println!("{}...\n", preview);
    }

    if let Some(content) = &choice1.message.content {
        println!("=== Turn 1: DeepSeek-R1 Answer ===");
        println!("{}\n", content);
    }

    // Preserve the assistant's turn (including reasoning_content) for Turn 2.
    // ChatMessage::from automatically copies role, content, and reasoning_content.
    messages.push(ChatMessage::from(&choice1.message));

    // --- Turn 2 ---
    let prompt2 = "Can the farmer solve it in fewer steps? Explain why or why not.";
    println!("=== Turn 2: User ===");
    println!("{}\n", prompt2);

    messages.push(ChatMessage::user(prompt2));

    let request2 = ChatCompletionRequest::builder("deepseek-reasoner")
        .messages(messages.clone())
        .build();

    let response2 = client.chat(&request2).await?;
    let choice2 = response2
        .choices
        .first()
        .expect("Expected at least one choice from DeepSeek");

    if let Some(content) = &choice2.message.content {
        println!("=== Turn 2: DeepSeek-R1 Answer ===");
        println!("{}", content);
    }

    if let Some(usage) = &response2.usage {
        println!("\n=== Turn 2 Token Usage ===");
        println!("Prompt tokens:     {}", usage.prompt_tokens);
        println!("Completion tokens: {}", usage.completion_tokens);
        if let Some(hits) = usage.prompt_cache_hit_tokens {
            println!("Cache hit tokens:  {} (Turn 1 history re-used!)", hits);
        }
    }

    Ok(())
}
