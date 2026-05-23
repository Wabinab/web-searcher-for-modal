//! Design for Parallel Web Search and Web Fetch
//! We'll skip saving to redb first, so we can get up to speed. 

use reqwest::blocking::Client;

use crate::{apperror::AppError, metadata::{FetchResp, PARALLEL_MCP_URL, SearchResp, WebFetchPayloadParallel, WebSearchPayloadParallel}};

/// Search the web using Parallel API. 
pub(crate) fn search_data_parallel(mcp_payload: &WebSearchPayloadParallel) -> Result<SearchResp, AppError> {
  let mcp_resp = Client::new().post(PARALLEL_MCP_URL).json(mcp_payload).send()
    .map_err(|e| AppError::BadRequest(format!("HTTP Request failed to search web using Parallel: {}. Try using Ollama instead.", e)))?;
  if !mcp_resp.status().is_success() { return Err(AppError::BadRequest(
    format!("Server returned status: {}", mcp_resp.status())
  )); }
  let json_body: SearchResp = mcp_resp.json().map_err(|e| 
    AppError::BadRequest(format!("JSON decode error search_data_parallel: {}", e)))?;
  Ok(json_body)
}


/// Crawl a specific website using Parallel API. 
pub(crate) fn fetch_data_parallel(mcp_payload: &WebFetchPayloadParallel) -> Result<FetchResp, AppError> {
  let mcp_resp = Client::new().post(PARALLEL_MCP_URL).json(mcp_payload).send()
    .map_err(|e| AppError::BadRequest(format!("HTTP Request failed to fetch website using Parallel: {}. Try using Ollama instead.", e)))?;
  if !mcp_resp.status().is_success() { return Err(AppError::BadRequest(
    format!("Server returned status: {}", mcp_resp.status())
  )); }
  let json_body: FetchResp = mcp_resp.json().map_err(|e|
    AppError::BadRequest(format!("JSON decode error fetch_data_parallel: {}", e)))?;
  Ok(json_body)
}

// ========================================
#[cfg(test)]
mod tests {
  use crate::{metadata::ToolCall, test_utils::{save_output, web_fetch_json, web_search_json}};
  use super::*;

  #[test]
  #[ignore]
  fn test_web_search_parallel_works() {
    let call_opt = ToolCall::from_input(&web_search_json());
    assert!(call_opt.is_ok(), "Expected Ok, got {:#?}", call_opt.err());
    let call = call_opt.unwrap();
    assert_eq!(call.name.as_str(), "web_search", "You probably used the wrong json string. Check!");

    let payload_opt = call.build_payload();
    assert!(payload_opt.is_ok(), "Expected Ok, got {:#?}", payload_opt.err());
    let payload: WebSearchPayloadParallel = payload_opt.unwrap();

    let resp = search_data_parallel(&payload);
    assert!(resp.is_ok(), "Expected Ok, got {:#?}", resp.err());
    let parallel_resp = resp.unwrap();

    save_output(&parallel_resp, "parallel_search.json", "web_search_parallel");
    let first = parallel_resp.result.content.first();
    assert!(first.is_some());
    assert!(parallel_resp.result.structured_content.results.len() > 0);
  }

  #[test]
  #[ignore]
  fn test_web_fetch_parallel_works() {
    let call_opt = ToolCall::from_input(&web_fetch_json());
    assert!(call_opt.is_ok(), "Expected Ok, got {:#?}", call_opt.err());
    let call = call_opt.unwrap();
    assert_eq!(call.name.as_str(), "web_fetch", "You probably used the wrong json string. Check!");

    let payload_opt = call.build_payload();
    assert!(payload_opt.is_ok(), "Expected Ok, got {:#?}", payload_opt.err());
    let payload: WebFetchPayloadParallel = payload_opt.unwrap();

    let resp = fetch_data_parallel(&payload);
    assert!(resp.is_ok(), "Expected Ok, got {:#?}", resp.err());
    let parallel_resp = resp.unwrap();

    save_output(&parallel_resp, "parallel_fetch.json", "web_fetch_parallel");;
    let first = parallel_resp.result.content.first();
    assert!(first.is_some());
    assert_eq!(parallel_resp.result.structured_content.results.len(), 1);
  }
}