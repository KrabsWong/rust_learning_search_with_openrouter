use reqwest::Client;
use anyhow::{Context, Result, Ok};
use std::collections::HashMap;
use std::time::Duration;
use colored::Colorize; // Added for terminal styling
use crate::models::{
    ExaSearchRequest, ExaSearchResponse, ExaContentsRequest, ExaContentsResponse,
};
use crate::utils::create_spinner;
use crate::constants::exa::{SEARCH_API_URL, CONTENTS_API_URL, REQUEST_COUNT};

// Helper function to fetch search results from Exa API
pub async fn fetch_exa_search_results(
    http_client: &Client,
    exa_api_key: &str,
    search_keywords: &str,
) -> Result<String> {
    let exa_spinner = create_spinner(&format!("Searching with Exa: \"{}\"", search_keywords).yellow().to_string());
    let exa_request_payload = ExaSearchRequest {
        query: search_keywords,
        num_results: REQUEST_COUNT,
        use_autoprompt: false,
        text: true, // Request text content
    };

    let timeout = Duration::new(300, 0);

    let exa_search_response = http_client
        .post(SEARCH_API_URL)
        .timeout(timeout)
        .header("x-api-key", exa_api_key)
        .json(&exa_request_payload)
        .send()
        .await?
        .error_for_status()
        .context("Exa API request failed")?
        .json::<ExaSearchResponse>()
        .await
        .context("Failed to parse Exa API JSON response")?;

    // eprintln!("Raw Exa Search Response (debug): {:?}", exa_search_response);

    if exa_search_response.results.is_empty() {
        exa_spinner.finish_with_message("❌ Exa found no relevant results.".red().to_string());
        println!("{}", "Try using more general keywords or check the Exa API.".yellow());
        return Err(anyhow::anyhow!("Exa API returned no results.")); 
    }
    exa_spinner.finish_with_message("✅ Exa search completed.".green().to_string());

    let mut search_results_summary = String::new();
    // This title is now handled in main.rs, so we can remove it or keep it if it's meant to be part of the returned string.
    // For now, let's assume it's part of the string to be returned and styled there if needed, or styled here directly.
    search_results_summary.push_str(&format!("{}\n", "Summary of relevant web search results:".bold().underline()));

    let ids_to_fetch: Vec<String> = exa_search_response.results.iter()
        .filter_map(|r| r.id.clone())
        .collect();

    let mut contents_map: HashMap<String, String> = HashMap::new();

    if !ids_to_fetch.is_empty() {
        let exa_getting_data_notify_text = String::from("Fetching detailed content (via Exa /contents)...".yellow().to_string());
        let content_spinner = create_spinner(&exa_getting_data_notify_text);
        let contents_request_payload = ExaContentsRequest {
            ids: ids_to_fetch.iter().map(AsRef::as_ref).collect(),
        };

        match http_client
            .post(CONTENTS_API_URL)
            .header("x-api-key", exa_api_key)
            .header("Content-Type", "application/json")
            .json(&contents_request_payload)
            .send()
            .await
        {
            Result::Ok(response) => {
                match response.error_for_status() {
                    Result::Ok(resp) => {
                        match resp.json::<ExaContentsResponse>().await {
                            Result::Ok(contents_response) => {
                                for content_result in contents_response.results {
                                    contents_map.insert(content_result.id, content_result.text);
                                }
                                content_spinner.finish_with_message(format!("✅ Successfully fetched detailed content for {} results.", contents_map.len()).green().to_string());
                            }
                            Err(e) => {
                                content_spinner.finish_with_message("⚠️ Failed to parse Exa /contents response.".yellow().to_string());
                                eprintln!("{}", format!("Failed to parse Exa /contents JSON response: {:?}", e).red());
                            }
                        }
                    }
                    Err(e) => {
                        content_spinner.finish_with_message("❌ Exa /contents API request failed (status).".red().to_string());
                        eprintln!("{}", format!("Exa /contents API request failed (status): {:?}", e).red());
                    }
                }
            }
            Err(e) => {
                content_spinner.finish_with_message("❌ Failed to send Exa /contents API request.".red().to_string());
                eprintln!("{}", format!("Failed to send request to Exa /contents API: {:?}", e).red());
            }
        }
    }

    let summary_notify_text = String::from("Extracting main content from search results...".yellow().to_string());
    let summary_spinner = create_spinner(&summary_notify_text);
    for (i, result) in exa_search_response.results.iter().enumerate() {
        summary_spinner.set_message(format!("Processing result {}/{}...", i + 1, exa_search_response.results.len()).yellow().to_string());
        
        let title_str = format!("{}: {}", "Title".dimmed(), result.title.cyan());
        let url_str = format!("{}: {}", "URL".dimmed(), result.url.underline().blue());

        search_results_summary.push_str(&format!(
            "\n{}: {}\n{}
{}
",
            "🔍 Result".bold(), (i + 1).to_string().bold(),
            title_str,
            url_str
        ));

        let mut summary_to_display = format!("{}: {}\n", "Summary".dimmed(), "(No text content available)".italic());
        if let Some(id) = &result.id {
            if let Some(full_text) = contents_map.get(id) {
                let cleaned_text: String = full_text.lines().filter(|line| !line.trim().is_empty()).collect::<Vec<&str>>().join("\n");
                let snippet = cleaned_text.chars().take(500).collect::<String>(); // Keep snippet length reasonable
                if snippet.is_empty(){
                    summary_to_display = format!("{}: {}\n", "Summary".dimmed(), "(Content is empty or not fetched after cleaning)".italic());
                } else {
                    summary_to_display = format!("{}:\n{}...\n", "Summary".dimmed(), snippet);
                }
            } else if result.text.is_some() && !result.text.as_ref().unwrap().is_empty() {
                // Fallback to text field from initial search if /contents failed or wasn't used for this ID
                let text_content = result.text.as_ref().unwrap();
                let cleaned_text: String = text_content.lines().filter(|line| !line.trim().is_empty()).collect::<Vec<&str>>().join("\n");
                let snippet = cleaned_text.chars().take(500).collect::<String>();
                if snippet.is_empty(){
                    summary_to_display = format!("{}: {}\n", "Summary (from initial search)".dimmed(), "(Content is empty or not fetched after cleaning)".italic());
                } else {
                    summary_to_display = format!("{}:\n{}...\n", "Summary (from initial search)".dimmed(), snippet);
                }
            }
        }
        search_results_summary.push_str(&summary_to_display);
    }
    summary_spinner.finish_with_message("✅ Main content extracted from search results.".green().to_string());
    Ok(search_results_summary)
}
