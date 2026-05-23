//! This will use Ollama search engine to do web search or web fetch. 

use reqwest::{blocking::Client, header};
use serde_json::json;

use crate::{apperror::AppError, metadata::{OLLAMA_MCP_URL, OllamaSearchResp}};

/// Calling Ollama's web_search api. 
/// Requires an API_KEY on OLLAMA_API environment variable. 
/// If top_k not set, default to 5. 
pub(crate) fn search_data_ollama(query: &str, top_k: Option<usize>
) -> Result<OllamaSearchResp, AppError> {
  let api_key = std::env::var("OLLAMA_API")
    .map_err(|_| AppError::BadRequest("Cannot find OLLAMA_API. Please set it.".to_string()))?;
  let top_k = top_k.unwrap_or(5);
  let url = format!("{}{}", OLLAMA_MCP_URL, "web_search");

  let resp = Client::new().post(url).bearer_auth(api_key)
    .header(header::CONTENT_TYPE, "application/json")
    .json(&json!({ "query": query, "max_results": top_k })).send()?;
  resp.error_for_status_ref()?;

  let body: OllamaSearchResp = resp.json()?;
  Ok(body)
}

// /// Calling Ollama's web_fetch api. 
// /// Requires an API_KEY on OLLAMA_API environment variable. 
// pub(crate) fn fetch_data_ollama(url: &str) -> Result<FetchResOl, AppError> {
//   let api_key = std::env::var("OLLAMA_API")
//     .map_err(|_| AppError::BadRequest("Cannot find OLLAMA_API. Please set it.".to_string()))?;
//   Ok(body)
// }
 

// ================================================
#[cfg(test)]
mod tests {
  use crate::test_utils::{save_output};

  use super::*;

  #[test]
  #[ignore]
  fn test_ollama_web_search_works() {
    let res = search_data_ollama("Top Cloud GPU Provider 2026", Some(3));
    assert!(res.is_ok());
    let ollama_resp = res.unwrap();

    // Save the result to test folder to see it with our very own eyes. 
    save_output(&ollama_resp, "ollama_search.json", "ollama_web_search");

    assert_eq!(ollama_resp.results.len(), 3);
  }
}