use anyhow::{Context, Result, Ok};
use reqwest::Client;
use std::env;
use std::io::{self, Write};
use colored::Colorize; // Added for terminal styling

mod models;
mod utils;
mod openrouter_client;
mod exa_client;

use crate::openrouter_client::{generate_search_keywords, generate_final_answer};
use crate::exa_client::fetch_exa_search_results;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let openrouter_api_key = env::var("OPENROUTER_API_KEY")
        .context("OPENROUTER_API_KEY not found in .env file")?;
    let exa_api_key = env::var("EXA_API_KEY")
        .context("EXA_API_KEY not found in .env file")?;

    let http_client = Client::new();

    println!("{}", "Please input what you want in the next line...".yellow());
    io::stdout().flush()?;

    let mut user_query = String::new();
    let _ = io::stdin().read_line(&mut user_query);
    let user_query = user_query.trim();

    if user_query.is_empty() {
        println!("{}", "Input data is empty. Please provide a query.".red());
        return Ok(());
    }

    // 1. Generate search keywords
    println!("{}", "🔍 Phase 1: Generating Search Keywords".bright_blue().bold());
    let (search_keywords, keyword_usage) = match generate_search_keywords(&http_client, &openrouter_api_key, user_query).await {
        Result::Ok(result) => result,
        Err(e) => {
            eprintln!("{}", format!("Error generating search keywords: {:?}", e).red());
            return Err(e);
        }
    };
    if let Some(usage) = keyword_usage {
        println!("{}", format!("🔑 Keyword Generation Token Usage: Prompt: {}, Completion: {}, Total: {}", 
            usage.prompt_tokens, usage.completion_tokens.unwrap_or(0), usage.total_tokens).cyan());
    }

    // 2. Fetch Exa search results
    println!("\n{}", "🌐 Phase 2: Fetching Search Results (Exa)".bright_blue().bold());
    let search_results_summary = match fetch_exa_search_results(&http_client, &exa_api_key, &search_keywords).await {
        Result::Ok(summary) => summary,
        Err(e) => {
            eprintln!("{}", format!("Error fetching Exa search results: {:?}", e).red());
            return Err(e); 
        }
    };
    // Print the formatted Exa search results summary
    println!("{}", search_results_summary);
    // Exa search results will be processed and displayed with URL, title, and summary.
    // The function fetch_exa_search_results will be updated to return structured data.

    // 3. Generate final answer
    println!("\n{}", "💡 Phase 3: Generating Final Answer (OpenRouter)".bright_blue().bold());
    match generate_final_answer(&http_client, &openrouter_api_key, user_query, &search_results_summary).await {
        Result::Ok((final_answer, final_usage)) => {
            println!("\n{}", "Final Answer:".bright_green().bold());
            println!("{}", final_answer);
            // The final_answer is streamed directly by handle_openrouter_stream if stream_to_stdout is true.
            // No need to print it here again as it's displayed in real-time.
            if let Some(usage) = final_usage {
                println!("\n{}", format!("💬 Final Answer Token Usage: Prompt: {}, Completion: {}, Total: {}", 
                    usage.prompt_tokens, usage.completion_tokens.unwrap_or(0), usage.total_tokens).cyan());
            }
        }
        Err(e) => {
            eprintln!("{}", format!("Error generating final answer: {:?}", e).red());
            return Err(e);
        }
    }

    Ok(())
}
