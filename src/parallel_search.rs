//! Design for Parallel Web Search and Web Fetch
//! We'll skip saving to redb first, so we can get up to speed. 

use reqwest::{blocking::Client, header::{ACCEPT, CONTENT_TYPE}};
use serde::{Serialize, de::DeserializeOwned};

use crate::{apperror::AppError, metadata::{FetchResp, PARALLEL_MCP_URL, ParallelResp, SearchResp, WebFetchPayloadParallel, WebSearchPayloadParallel}};

// /// Search the web using Parallel API. 
// pub(crate) fn search_data_parallel(mcp_payload: &WebSearchPayloadParallel) -> Result<SearchResp, AppError> {
//   let mcp_resp = Client::new().post(PARALLEL_MCP_URL).json(mcp_payload).send()
//     .map_err(|e| AppError::BadRequest(format!("HTTP Request failed to search web using Parallel: {}. Try using Ollama instead.", e)))?;
//   if !mcp_resp.status().is_success() { return Err(AppError::BadRequest(
//     format!("Server returned status: {}", mcp_resp.status())
//   )); }
//   let json_body: SearchResp = mcp_resp.json().map_err(|e| 
//     AppError::BadRequest(format!("JSON decode error search_data_parallel: {}", e)))?;
//   Ok(json_body)
// }


// /// Crawl a specific website using Parallel API. 
// pub(crate) fn fetch_data_parallel(mcp_payload: &WebFetchPayloadParallel) -> Result<FetchResp, AppError> {
//   let mcp_resp = Client::new().post(PARALLEL_MCP_URL).json(mcp_payload).send()
//     .map_err(|e| AppError::BadRequest(format!("HTTP Request failed to fetch website using Parallel: {}. Try using Ollama instead.", e)))?;
//   if !mcp_resp.status().is_success() { return Err(AppError::BadRequest(
//     format!("Server returned status: {}", mcp_resp.status())
//   )); }
//   let json_body: FetchResp = mcp_resp.json().map_err(|e|
//     AppError::BadRequest(format!("JSON decode error fetch_data_parallel: {}", e)))?;
//   Ok(json_body)
// }

/// Search the web using Parallel API.
pub(crate) fn search_data_parallel(mcp_payload: &WebSearchPayloadParallel) -> Result<SearchResp, AppError> {
  post_mcp(mcp_payload, "search_data_parallel", "search web")
}

/// Crawl a specific website using Parallel API.
pub(crate) fn fetch_data_parallel(mcp_payload: &WebFetchPayloadParallel) -> Result<FetchResp, AppError> {
  post_mcp(mcp_payload, "fetch_data_parallel", "fetch website")
}

/// Shared transport: POST the JSON-RPC payload, accept JSON or SSE, decode the envelope,
/// and surface tool-level errors (`isError: true` arrives with HTTP 200).
fn post_mcp<P, T>(payload: &P, fn_name: &str, action: &str) -> Result<ParallelResp<T>, AppError>
where
  P: Serialize,
  T: DeserializeOwned,
{
  let resp = Client::new()
    .post(PARALLEL_MCP_URL)
    .header(ACCEPT, "application/json, text/event-stream")
    .json(payload)
    .send()
    .map_err(|e| AppError::BadRequest(format!(
      "HTTP Request failed to {} using Parallel: {}. Try using Ollama instead.", action, e)))?;

  let status = resp.status();
  let content_type = resp.headers().get(CONTENT_TYPE)
    .and_then(|v| v.to_str().ok()).unwrap_or("<none>").to_string();
  let body = resp.text().map_err(|e| AppError::BadRequest(format!(
    "{}: failed reading response body (status {}, content-type {}): {}", fn_name, status, content_type, e)))?;

  if !status.is_success() {
    return Err(AppError::BadRequest(format!(
      "{}: server returned status {} (content-type {}), body head: {}",
      fn_name, status, content_type, head(&body))));
  }

  let json_text = extract_json(&body).ok_or_else(|| AppError::BadRequest(format!(
    "{}: no JSON found in response (status {}, content-type {}), body head: {}",
    fn_name, status, content_type, head(&body))))?;

  let parsed: ParallelResp<T> = serde_json::from_str(&json_text).map_err(|e| AppError::BadRequest(format!(
    "{}: JSON decode error: {} (line {}, col {}); content-type {}; body head: {}",
    fn_name, e, e.line(), e.column(), content_type, head(&json_text))))?;

  if parsed.result.is_error {
    let msg = parsed.result.content.first().map(|c| c.text.as_str()).unwrap_or("<no content>");
    return Err(AppError::BadRequest(format!("{}: Parallel tool returned isError: {}", fn_name, head(msg))));
  }
  Ok(parsed)
}

/// Plain JSON body -> returned as is.
/// SSE body -> the last event whose joined `data:` lines start with '{'.
fn extract_json(body: &str) -> Option<String> {
  let trimmed = body.trim_start();
  if trimmed.starts_with('{') { return Some(trimmed.to_string()); }

  let normalized = body.replace("\r\n", "\n");
  normalized
    .rsplit("\n\n")
    .find_map(|event| {
      let data = event.lines()
        .filter_map(|l| l.strip_prefix("data:"))
        .map(|l| l.strip_prefix(' ').unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n");
      let data = data.trim();
      if data.starts_with('{') { Some(data.to_string()) } else { None }
    })
}

/// First 500 chars, char-boundary safe. Temporary debugging aid; shrink once stable.
fn head(s: &str) -> String { s.chars().take(500).collect() }

// ========================================
#[cfg(test)]
mod tests {
  use crate::{metadata::{FetchRes, ToolCall}, test_utils::save_output};

use super::*;

  fn web_search_json() -> String {
    let payload = serde_json::json!({
      "id": "call_7945d98c45dd4e0583c04db8",
      "name": "web_search",
      "arguments": serde_json::json!({
        "objective": "Find github repositories for text-splitter and semchunk-rs",
        "search_queries": ["github text-splitter rust", "github semchunk-rs"]
      }).to_string()
    });

    payload.to_string()
  }

  fn web_fetch_json() -> String {
    let payload = serde_json::json!({
      "id": "8124f905-3d1b-4c67-b6e6-c70de2a26684",
      "name": "web_fetch",
      "arguments": serde_json::json!({
        "urls": ["https://crates.io/crates/text-splitter/0.30.1"],
        "objective": "Get detailed information about the text-splitter crate version 0.30.1 including features, documentation, and usage",
        "search_queries": ["text-splitter crate features", "text-splitter usage documentation"]
      }).to_string()
    });
    payload.to_string()
  }

  fn web_fetch_json_no_queries() -> String {
    let payload = serde_json::json!({
      "id": "8124f905-3d1b-4c67-b6e6-c70de2a26684",
      "name": "web_fetch",
      "arguments": serde_json::json!({
        "urls": ["https://crates.io/crates/text-splitter/0.30.1"],
        "objective": "Get detailed information about the text-splitter crate version 0.30.1 including features, documentation, and usage"
      }).to_string()
    });

    payload.to_string()
  }

  #[test]
  #[ignore]
  fn test_web_search_parallel_works() {
    let call_opt = ToolCall::from_input(&web_search_json());
    assert!(call_opt.is_ok(), "Expected Ok, got {:#?}", call_opt.err());
    let call = call_opt.unwrap();
    match call.name.as_str() {
      "web_search" => {
        let payload_opt = call.build_payload();
        assert!(payload_opt.is_ok(), "Expected Ok, got {:#?}", payload_opt.err());
        let payload: WebSearchPayloadParallel = payload_opt.unwrap();

        let resp = search_data_parallel(&payload);
        assert!(resp.is_ok(), "Expected Ok, got {:#?}", resp.err());
        let parallel_resp = resp.unwrap();

        save_output(&parallel_resp, "parallel_search.json", "web_search_parallel");
        assert!(!parallel_resp.result.is_error, "tool error: {:#?}", parallel_resp.result.content);
        let first = parallel_resp.result.content.first();
        assert!(first.is_some());
        assert!(parallel_resp.result.structured_content.results.len() > 0);
      }
      _ =>  panic!("You probably used the wrong json string. Check!"),
    }
  }

  #[test]
  #[ignore]
  fn test_web_fetch_parallel_works_without_queries() {
    let call_opt = ToolCall::from_input(&web_fetch_json_no_queries());
    assert!(call_opt.is_ok(), "Expected Ok, got {:#?}", call_opt.err());
    let call = call_opt.unwrap();
    match call.name.as_str() {
      "web_fetch" => {
        let payload_opt = call.build_payload();
        assert!(payload_opt.is_ok(), "Expected Ok, got {:#?}", payload_opt.err());
        let payload: WebFetchPayloadParallel = payload_opt.unwrap();

        let resp = fetch_data_parallel(&payload);
        assert!(resp.is_ok(), "Expected Ok, got {:#?}", resp.err());
        let parallel_resp = resp.unwrap();

        save_output(&parallel_resp, "parallel_fetch.json", "web_fetch_parallel");
        assert!(!parallel_resp.result.is_error, "tool error: {:#?}", parallel_resp.result.content);
        let first = parallel_resp.result.content.first();
        assert!(first.is_some());
        assert_eq!(parallel_resp.result.structured_content.results.len(), 1);
      }, 
      _ => panic!("You probably used the wrong json string. Check!"),
    }
  }

  #[test]
  #[ignore]
  fn test_web_fetch_parallel_works_with_queries() {
    let call_opt = ToolCall::from_input(&web_fetch_json());
    assert!(call_opt.is_ok(), "Expected Ok, got {:#?}", call_opt.err());
    let call = call_opt.unwrap();
    match call.name.as_str() {
      "web_fetch" => {
        let payload_opt = call.build_payload();
        assert!(payload_opt.is_ok(), "Expected Ok, got {:#?}", payload_opt.err());
        let payload: WebFetchPayloadParallel = payload_opt.unwrap();

        let resp = fetch_data_parallel(&payload);
        assert!(resp.is_ok(), "Expected Ok, got {:#?}", resp.err());
        let parallel_resp = resp.unwrap();

        save_output(&parallel_resp, "parallel_fetch.json", "web_fetch_parallel");
        assert!(!parallel_resp.result.is_error, "tool error: {:#?}", parallel_resp.result.content);
        let first = parallel_resp.result.content.first();
        assert!(first.is_some());
        assert_eq!(parallel_resp.result.structured_content.results.len(), 1);
      }, 
      _ => panic!("You probably used the wrong json string. Check!"),
    }
  }

  #[test]
  fn fetch_payload_omits_search_queries_when_none() {
    let call = ToolCall::from_input(&web_fetch_json_no_queries()).unwrap();
    let payload: WebFetchPayloadParallel = call.build_payload().unwrap();
    let v = serde_json::to_value(&payload).unwrap();
    assert_eq!(v["params"]["name"], "web_fetch");
    assert!(v["params"]["arguments"].get("search_queries").is_none());
  }
  
  #[test]
  fn fetch_payload_includes_search_queries_when_some() {
    let call = ToolCall::from_input(&web_fetch_json()).unwrap();
    let payload: WebFetchPayloadParallel = call.build_payload().unwrap();
    let v = serde_json::to_value(&payload).unwrap();
    assert_eq!(v["params"]["arguments"]["search_queries"].as_array().unwrap().len(), 2);
  }
  
  #[test]
  fn fetch_res_tolerates_null_title_and_missing_full_content() {
    let json = r#"{"url":"https://x.com","title":null,"publish_date":null,"excerpts":["a"]}"#;
    let r: FetchRes = serde_json::from_str(json).unwrap();
    assert!(r.title.is_none() && r.full_content.is_none());
  }
}