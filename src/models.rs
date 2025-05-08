use serde::{Deserialize, Serialize};

// OpenRouter related structs
#[derive(Serialize)]
pub struct OpenRouterRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<Message<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>, // Added for streaming
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<'a> {
    pub role: &'a str,
    pub content: &'a str,
}

#[derive(Deserialize, Debug, Clone, Default)] // Added Default
pub struct UsageInfo {
    pub prompt_tokens: u32,
    pub completion_tokens: Option<u32>, // Made optional as it might not always be present initially in stream
    pub total_tokens: u32,
}

#[derive(Deserialize, Debug)]
pub struct OpenRouterError {
    _message: String,
}

// Structs for handling streaming responses
#[derive(Deserialize, Debug)]
pub struct OpenRouterStreamResponse {
    pub _id: Option<String>,
    pub _model: Option<String>,
    pub choices: Vec<OpenRouterStreamChoice>,
    pub usage: Option<UsageInfo>, // To capture usage at the end of the stream
    pub error: Option<OpenRouterError>,
}

#[derive(Deserialize, Debug)]
pub struct OpenRouterStreamChoice {
    pub index: u32,
    pub delta: OpenRouterStreamDelta,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenRouterStreamDelta {
    pub content: Option<String>,
    // Potentially other fields like 'role' if the role can change mid-stream
}

// Exa related structs
#[derive(Serialize)]
pub struct ExaSearchRequest<'a> {
    pub query: &'a str,
    pub num_results: usize,
    pub use_autoprompt: bool,
    pub text: bool,
}

#[derive(Deserialize, Debug)]
pub struct ExaSearchResponse {
    pub results: Vec<ExaSearchResult>,
}

#[derive(Deserialize, Debug, Clone)] // Added Clone here as it might be useful later
pub struct ExaSearchResult {
    pub title: String,
    pub url: String,
    pub id: Option<String>,
    pub text: Option<String>,
    pub _score: Option<f64>,
    pub _published_date: Option<String>,
    pub _author: Option<String>,
}

#[derive(Serialize)]
pub struct ExaContentsRequest<'a> {
    pub ids: Vec<&'a str>,
}

#[derive(Deserialize, Debug)]
pub struct ExaContentResult {
    pub id: String,
    pub text: String,
    // pub url: Option<String>,
    // pub title: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ExaContentsResponse {
    pub results: Vec<ExaContentResult>,
}
