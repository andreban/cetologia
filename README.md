# Cetologia

An ergonomic, async Rust client for the [DeepSeek](https://www.deepseek.com/) API.

Named after *cetology* (the scientific study of whales), inspired by DeepSeek's whale mascot and following the naming tradition of [`geologia`](https://github.com/andreban/geologia) (Google AI).

## Features

- **DeepSeek-Chat (V3) & DeepSeek-Reasoner (R1)**: First-class typed support for all models.
- **Thinking / Reasoning Tokens**: Full preservation and SSE streaming of `reasoning_content`.
- **Reasoning Effort**: Set thinking effort per request with `reasoning_effort` (`none`, `low`, `high`, `max`).
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

## Reasoning Effort

Thinking models accept a `reasoning_effort` of `none` (thinking off), `low`, `high` (the default), or `max`:

```rust
let request = ChatCompletionRequest::builder("deepseek-flash")
    .messages(vec![ChatMessage::user("Explain what cetology is.")])
    .reasoning_effort(ReasoningEffort::Max)
    .build();
```

When thinking mode is active, DeepSeek ignores `temperature`, `presence_penalty`, and `frequency_penalty`.

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

## Examples

All examples automatically load your DeepSeek API key from a `.env` file using [`dotenvy`](https://crates.io/crates/dotenvy) (or from the `DEEPSEEK_API_KEY` environment variable).

1. Copy [`.env.example`](.env.example) to `.env` and add your API key:
   ```bash
   cp .env.example .env
   ```

2. Run any example using `cargo run --example <name>`:

| Example | Description | Command |
|---|---|---|
| **Basic Chat** | Standard chat completion with DeepSeek-Chat (V3) and cache metrics | `cargo run --example basic_chat` |
| **Streaming Reasoner** | Real-time SSE streaming with DeepSeek-Reasoner (R1), displaying thinking tokens vs answer tokens | `cargo run --example streaming_reasoner` |
| **Multi-Turn Reasoning** | Multi-turn chat with DeepSeek-R1, automatically preserving `reasoning_content` | `cargo run --example multi_turn_reasoning` |
| **Function Calling** | OpenAI-compatible tool calling, handling invocations and returning results | `cargo run --example function_calling` |
| **Streaming Tools** | Streaming tool call deltas aggregated using `ToolCallAccumulator` | `cargo run --example streaming_tools` |
| **Structured JSON** | JSON mode (`ResponseFormat::JsonObject`) deserialized into typed Rust structs | `cargo run --example structured_json` |
| **Prefix Completion** | Forcing assistant prefix completion with `prefix: true` | `cargo run --example prefix_completion` |

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE)).

