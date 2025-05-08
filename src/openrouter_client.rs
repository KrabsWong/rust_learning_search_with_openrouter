use reqwest::Client;
use anyhow::{Context, Result, Ok};
use futures_util::StreamExt;
use std::io::{stdout, Write};
use crate::models::{OpenRouterRequest, Message, OpenRouterStreamResponse, UsageInfo};
use crate::utils::create_spinner;
use colored::Colorize;
use crate::constants::open_router::{API_URL, SUMMARY_MODEL, SEARCH_MODEL, REFERER, APPNAME};

// Helper function to generate search keywords using OpenRouter
pub async fn generate_search_keywords(
    http_client: &Client,
    openrouter_api_key: &str,
    user_query: &str,
) -> Result<(String, Option<UsageInfo>)> {
    let keyword_spinner = create_spinner("Building search query data (via OpenRouter)...");
    let keyword_prompt = format!(
        "Based on the following user query, generate 3-5 concise search keywords suitable for a web search engine. Return only the keywords, comma-separated. User query: \"{}\"",
        user_query,
    );

    let keyword_request_payload = OpenRouterRequest {
        model: SEARCH_MODEL, // Or your preferred model for keyword generation
        stream: Some(true),
        messages: vec![Message {
            role: "user",
            content: &keyword_prompt,
        }],
    };

    let keyword_response_raw = http_client
        .post(API_URL)
        .bearer_auth(openrouter_api_key)
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", REFERER) 
        .header("X-Title", APPNAME)
        .json(&keyword_request_payload)
        .send()
        .await
        .context("Failed to send request to OpenRouter for keyword generation")?;

    // Streaming for keywords might be overkill for display, but useful for consistent API usage and getting token counts.
    // For keywords, we'll collect the full response then return, not printing chunk by chunk.
    let (keywords_content, usage_info) = 
        handle_openrouter_stream(keyword_response_raw, "OpenRouter Keyword Generation", false, &keyword_spinner).await?; // Retained original context_msg as it's for internal logging/error handling
    
    keyword_spinner.finish_with_message(format!("✅ Search keywords generated successfully: {}", keywords_content));
    Ok((keywords_content, usage_info))
}

// Helper function to generate the final answer using OpenRouter
pub async fn generate_final_answer(
    http_client: &Client,
    openrouter_api_key: &str,
    user_query: &str,
    search_results_summary: &str,
) -> Result<(String, Option<UsageInfo>)> {
    let final_answer_spinner = create_spinner("Generating final answer using combined information (via OpenRouter)...");
    let final_prompt = format!(
        "Based on your existing knowledge and the following web search results, please provide a comprehensive answer to the user's original query. \n\nUser Query: \"{}\"\n\nWeb Search Results:\n{}\n\nYour Answer:",
        user_query,
        search_results_summary
    );

    let final_request_payload = OpenRouterRequest {
        stream: Some(true),
        model: SUMMARY_MODEL, 
        messages: vec![Message {
            role: "user",
            content: &final_prompt,
        }],
    };
    
    let final_response_raw = http_client
        .post(API_URL)
        .bearer_auth(openrouter_api_key)
        .header("HTTP-Referer", REFERER)
        .header("X-Title", APPNAME)
        .json(&final_request_payload)
        .send()
        .await
        .context("Failed to send final request to OpenRouter")?;

    // final_answer_spinner.set_message("Receiving final answer from OpenRouter...");
    // Print the message directly before starting the stream handling if stream_to_stdout is true.
    // The handle_openrouter_stream function is called with stream_to_stdout = true for final answer.
    println!("{}", "Receiving final answer from OpenRouter...".yellow()); 

    let (final_answer_content, usage_info) = 
        handle_openrouter_stream(final_response_raw, "Final OpenRouter Answer Generation", false, &final_answer_spinner).await?; // Retained original context_msg as it's for internal logging/error handling

    final_answer_spinner.finish_with_message("✅ Final answer received successfully:");
    Ok((final_answer_content, usage_info))
}

// Helper function to handle OpenRouter streaming responses
async fn handle_openrouter_stream(
    response: reqwest::Response,
    context_msg: &str,
    stream_to_stdout: bool, // If true, prints content chunks to stdout
    spinner: &indicatif::ProgressBar, // Pass spinner to update its message
) -> Result<(String, Option<UsageInfo>)> {
    if !response.status().is_success() {
        let error_body = response.text().await.unwrap_or_else(|_| "Unknown error reading response body".to_string());
        spinner.finish_with_message(format!("❌ {} failed", context_msg)); // context_msg is already in English or a placeholder, no change needed for this specific line's user-facing part
        return Err(anyhow::anyhow!(
            "{}. Response: {}",
            context_msg,
            // response.status(),
            error_body
        ));
    }

    let mut byte_stream = response.bytes_stream();
    let mut accumulated_content = String::new();
    let mut final_usage_info: Option<UsageInfo> = None;

    if stream_to_stdout {
        println!(); // New line before streaming starts
    }

    while let Some(item) = byte_stream.next().await {
        let chunk = item.context(format!("Error reading chunk from {} stream", context_msg))?;
        let chunk_str = std::str::from_utf8(&chunk)
            .context(format!("Failed to decode UTF-8 chunk from {}", context_msg))?;

        for line in chunk_str.lines() {
            if line.starts_with("data: ") {
                let json_data = &line[6..];
                if json_data.trim() == "[DONE]" {
                    break; // Stream finished
                }
                match serde_json::from_str::<OpenRouterStreamResponse>(json_data) {
                    Result::Ok(stream_resp) => {
                        if let Some(usage) = stream_resp.usage {
                            final_usage_info = Some(usage.clone());
                        }
                        for choice in stream_resp.choices {
                            if let Some(content_delta) = choice.delta.content {
                                accumulated_content.push_str(&content_delta);
                                if stream_to_stdout {
                                    print!("{}", content_delta);
                                    stdout().flush().context("Failed to flush stdout")?;
                                }
                            }
                            if choice.finish_reason.is_some() {
                                // Potentially handle finish reason if needed
                            }
                        }
                    }
                    Err(e) => {
                        // It's possible to get non-JSON metadata or empty lines in the stream
                        // eprintln!(
                        //     "Warning: Failed to parse stream data chunk from {}: {}. Chunk: '{}'",
                        //     context_msg,
                        //     e,
                        //     json_data
                        // );
                        // For now, we'll be a bit lenient with parsing errors in individual chunks if they are not [DONE]
                        // but if it's persistent, it indicates a problem.
                        let trimmed_json_data = json_data.trim();
                        if !trimmed_json_data.is_empty() { // Log if it's not empty after trimming
                             eprintln!(
                                "Warning: Failed to parse stream data chunk from {}: {}. Chunk: '{}'",
                                context_msg,
                                e,
                                trimmed_json_data // Log the trimmed version for clarity
                            );
                        }
                    }
                }
            } else if !line.trim().is_empty() {
                // Potentially log other non-data lines if necessary for debugging
                // eprintln!("Non-data line from stream: {}", line);
            }
        }
    }
    if stream_to_stdout {
        println!(); // New line after streaming finishes
    }
    Ok((accumulated_content, final_usage_info))
}
