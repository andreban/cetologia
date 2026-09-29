# Cetologia

An ergonomic, async Rust client for the [DeepSeek](https://www.deepseek.com/) API.

Named after *cetology* (the scientific study of whales), inspired by DeepSeek's whale mascot and following the naming tradition of [`geologia`](https://github.com/andreban/geologia) (Google AI).

## Features

- **DeepSeek-Chat (V3) & DeepSeek-Reasoner (R1)**: First-class typed support for all models.
- **Thinking / Reasoning Tokens**: Full preservation and SSE streaming of `reasoning_content`.
- **Multi-Turn Reasoning Support**: Automatically preserves reasoning traces on assistant turns to avoid DeepSeek 400 Bad Request errors.
- **OpenAI-Compatible Tool Calling**: Function declarations, tool choice, and streaming tool call chunk aggregation.
- **Prompt Caching Metrics**: Exposes `prompt_cache_hit_tokens` and `prompt_cache_miss_tokens` in `usage`.
- **Zero Heavy Dependencies**: Built cleanly on `reqwest`, `serde`, and `tokio`.

## Quick Start

```rust
use cetologia::prelude::*;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let client = CetologiaClient::new(std::env::var("DEEPSEEK_API_KEY")?);

    let request = ChatCompletionRequest::builder("deepseek-chat")
        .messages(vec![
            ChatMessage::system("You are a helpful assistant."),
            ChatMessage::user("Explain what cetology is."),
        ])
        .temperature(0.7)
        .build();

    let response = client.chat(&request).await?;
    println!("Response: {}", response.text().unwrap_or_default());
    Ok(())
}
```

## Streaming Example with DeepSeek-R1 (Reasoning)

```rust
use cetologia::prelude::*;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let client = CetologiaClient::new(std::env::var("DEEPSEEK_API_KEY")?);

    let request = ChatCompletionRequest::builder("deepseek-reasoner")
        .messages(vec![ChatMessage::user("Which is larger: 9.9 or 9.11?")])
        .build();

    let mut stream = client.chat_stream(&request).await?;

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res?;
        if let Some(choice) = chunk.choices.first() {
            if let Some(reasoning) = &choice.delta.reasoning_content {
                print!("[THINK] {}", reasoning);
            }
            if let Some(text) = &choice.delta.content {
                print!("{}", text);
            }
        }
    }
    Ok(())
}
```

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE)).
