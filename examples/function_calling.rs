// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! Function / Tool calling example using `deepseek-chat` (DeepSeek-V3).
//!
//! Demonstrates defining function tools with JSON Schema parameters,
//! handling model tool invocations, executing the tool locally, and
//! returning tool results back to the model for a final synthesized response.
//!
//! Run with:
//! ```bash
//! cargo run --example function_calling
//! ```

use cetologia::prelude::*;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct WeatherArgs {
    location: String,
    unit: Option<String>,
}

fn simulate_get_weather(args_json: &str) -> String {
    let args: WeatherArgs = serde_json::from_str(args_json).unwrap_or(WeatherArgs {
        location: "Unknown".into(),
        unit: Some("celsius".into()),
    });

    let unit = args.unit.unwrap_or_else(|| "celsius".into());
    let temp = if unit.eq_ignore_ascii_case("fahrenheit") {
        "68°F"
    } else {
        "20°C"
    };

    json!({
        "location": args.location,
        "temperature": temp,
        "condition": "Partly Cloudy",
        "humidity": "65%",
        "wind_speed": "12 km/h"
    })
    .to_string()
}

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

    // 1. Define the tool specification
    let weather_tool = Tool::function(
        "get_current_weather",
        Some("Get the current weather conditions for a given city".to_string()),
        Some(json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city and state/country, e.g. Tokyo, JP or San Francisco, CA"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "The temperature unit to use"
                }
            },
            "required": ["location"]
        })),
    );

    let user_prompt = "What's the weather like in Tokyo right now in Celsius?";
    println!("=== User Question ===");
    println!("{}\n", user_prompt);

    let mut messages = vec![
        ChatMessage::system("You are a helpful assistant with access to real-time weather information."),
        ChatMessage::user(user_prompt),
    ];

    // 2. Request completion with available tools
    let request = ChatCompletionRequest::builder("deepseek-chat")
        .messages(messages.clone())
        .tools(vec![weather_tool])
        .tool_choice(ToolChoice::Mode(ToolChoiceMode::Auto))
        .build();

    let response = client.chat(&request).await?;
    let choice = response
        .choices
        .first()
        .expect("Expected at least one choice");

    // 3. Check if the model invoked any tools
    let tool_calls = choice.message.tool_calls.as_deref().unwrap_or_default();
    if tool_calls.is_empty() {
        println!("No tool called. Response: {:?}", choice.message.content);
        return Ok(());
    }

    // Append the assistant's tool-call message to conversation history
    messages.push(ChatMessage::from(&choice.message));

    // 4. Execute each requested tool call
    for tool_call in tool_calls {
        println!("=== Tool Call Invoked ===");
        println!("ID:        {}", tool_call.id);
        println!("Function:  {}", tool_call.function.name);
        println!("Arguments: {}", tool_call.function.arguments);

        let result = if tool_call.function.name == "get_current_weather" {
            simulate_get_weather(&tool_call.function.arguments)
        } else {
            json!({ "error": "Unknown function" }).to_string()
        };

        println!("Result:    {}\n", result);

        // Append the tool result message
        messages.push(ChatMessage::tool(&tool_call.id, result));
    }

    // 5. Send updated conversation back to DeepSeek to get the final answer
    println!("Sending tool results back to DeepSeek...");
    let final_request = ChatCompletionRequest::builder("deepseek-chat")
        .messages(messages)
        .build();

    let final_response = client.chat(&final_request).await?;

    if let Some(text) = final_response.text() {
        println!("\n=== Final Assistant Response ===");
        println!("{}", text);
    }

    Ok(())
}
