// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! Streaming reasoning completion example using `deepseek-reasoner` (DeepSeek-R1).
//!
//! Demonstrates SSE streaming of both the model's internal thinking process
//! (`reasoning_content`) and the final response content (`content`) in real-time.
//!
//! Run with:
//! ```bash
//! cargo run --example streaming_reasoner
//! ```

use std::io::{self, Write};
use cetologia::prelude::*;
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

    let prompt = "Which is larger: 9.9 or 9.11? Explain your reasoning step by step.";
    println!("Prompt: {}\n", prompt);

    let request = ChatCompletionRequest::builder("deepseek-reasoner")
        .messages(vec![ChatMessage::user(prompt)])
        .stream(true)
        .stream_options(StreamOptions { include_usage: true })
        .build();

    let mut stream = client.chat_stream(&request).await?;

    let mut in_reasoning_phase = false;
    let mut in_content_phase = false;
    let mut final_usage = None;

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res?;

        if let Some(usage) = chunk.usage {
            final_usage = Some(usage);
        }

        if let Some(choice) = chunk.choices.first() {
            // Print reasoning/thinking tokens as they stream in
            if let Some(reasoning) = &choice.delta.reasoning_content {
                if !in_reasoning_phase {
                    println!("=== DeepSeek-R1 Thinking Process ===");
                    in_reasoning_phase = true;
                }
                print!("{}", reasoning);
                io::stdout().flush().ok();
            }

            // Print final answer tokens as they stream in
            if let Some(content) = &choice.delta.content {
                if !in_content_phase {
                    if in_reasoning_phase {
                        println!("\n\n=== Final Answer ===");
                    }
                    in_content_phase = true;
                }
                print!("{}", content);
                io::stdout().flush().ok();
            }
        }
    }

    println!();

    if let Some(usage) = final_usage {
        println!("\n=== Token Usage ===");
        println!("Prompt tokens:     {}", usage.prompt_tokens);
        println!("Completion tokens: {}", usage.completion_tokens);
        println!("Total tokens:      {}", usage.total_tokens);
        if let Some(hits) = usage.prompt_cache_hit_tokens {
            println!("Cache hit tokens:  {}", hits);
        }
    }

    Ok(())
}
