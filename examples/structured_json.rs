// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

//! Structured JSON output example using `deepseek-chat` and `ResponseFormat::JsonObject`.
//!
//! DeepSeek supports JSON mode when `response_format` is set to `JsonObject`
//! and the prompt instructs the model to produce valid JSON.
//!
//! Run with:
//! ```bash
//! cargo run --example structured_json
//! ```

use cetologia::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct CetaceanProfile {
    common_name: String,
    scientific_name: String,
    suborder: String,
    average_length_meters: f32,
    max_weight_metric_tons: f32,
    diet: Vec<String>,
    conservation_status: String,
    fun_fact: String,
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

    let system_instructions = "\
You are a marine biology knowledge engine. \
When requested, return information strictly formatted as a valid JSON object matching this schema:
{
  \"common_name\": string,
  \"scientific_name\": string,
  \"suborder\": string,
  \"average_length_meters\": number,
  \"max_weight_metric_tons\": number,
  \"diet\": array of strings,
  \"conservation_status\": string,
  \"fun_fact\": string
}";

    let user_prompt = "Provide a detailed profile for the Humpback Whale (Megaptera novaeangliae).";

    println!("Requesting structured JSON for: Humpback Whale...\n");

    let request = ChatCompletionRequest::builder("deepseek-chat")
        .messages(vec![
            ChatMessage::system(system_instructions),
            ChatMessage::user(user_prompt),
        ])
        .response_format(ResponseFormat::JsonObject)
        .temperature(0.2)
        .build();

    let response = client.chat(&request).await?;
    let raw_json = response.text().expect("Expected response content");

    println!("=== Raw JSON Response ===");
    println!("{}\n", raw_json);

    // Deserialize into typed Rust struct
    let profile: CetaceanProfile = serde_json::from_str(raw_json)?;

    println!("=== Deserialized Rust Struct ===");
    println!("Common Name:         {}", profile.common_name);
    println!("Scientific Name:     {}", profile.scientific_name);
    println!("Suborder:            {}", profile.suborder);
    println!("Avg Length:          {} meters", profile.average_length_meters);
    println!("Max Weight:          {} metric tons", profile.max_weight_metric_tons);
    println!("Diet:                {}", profile.diet.join(", "));
    println!("Conservation Status: {}", profile.conservation_status);
    println!("Fun Fact:            {}", profile.fun_fact);

    Ok(())
}
