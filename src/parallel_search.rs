//! Design for Parallel Web Search, Web Fetch and Image Search
//! We'll skip saving to redb first, so we can get up to speed. 

use reqwest::{blocking::Client, header::{ACCEPT, CONTENT_TYPE}};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{apperror::AppError, metadata::{FetchResp, ImageSearchPayloadParallel, ImageResp, PARALLEL_MCP_URL, ParallelResp, SearchResp, WebFetchPayloadParallel, WebSearchPayloadParallel}};

/// Search the web using Parallel API.
pub(crate) fn search_data_parallel(mcp_payload: &WebSearchPayloadParallel) -> Result<SearchResp, AppError> {
  post_mcp(mcp_payload, "search_data_parallel", "search web")
}

/// Crawl a specific website using Parallel API.
pub(crate) fn fetch_data_parallel(mcp_payload: &WebFetchPayloadParallel) -> Result<FetchResp, AppError> {
  post_mcp(mcp_payload, "fetch_data_parallel", "fetch website")
}

/// Search images using Parallel API. No Ollama fallback exists for this.
pub(crate) fn image_search_parallel(mcp_payload: &ImageSearchPayloadParallel) -> Result<ImageResp, AppError> {
  post_mcp(mcp_payload, "image_search_parallel", "search images")
}

/// Shared transport: POST the JSON-RPC payload, accept JSON or SSE, then decode.
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
      "HTTP Request failed to {} using Parallel: {}.", action, e)))?;

  let status = resp.status();
  let content_type = resp.headers().get(CONTENT_TYPE)
    .and_then(|v| v.to_str().ok()).unwrap_or("<none>").to_string();
  let ctx = format!("status {}, content-type {}", status, content_type);
  let body = resp.text().map_err(|e| AppError::BadRequest(format!(
    "{}: failed reading response body ({}): {}", fn_name, ctx, e)))?;

  if !status.is_success() {
    return Err(AppError::BadRequest(format!(
      "{}: server returned {}, body head: {}", fn_name, ctx, head(&body))));
  }
  decode_mcp_body(&body, fn_name, &ctx)
}

/// Pure decode step (no I/O) so it's unit-testable. Tool-level errors (`isError: true`,
/// HTTP 200) and JSON-RPC `error` objects usually lack `structuredContent`, so they are
/// detected BEFORE the typed parse; otherwise they'd surface as a misleading
/// "missing field" decode error.
fn decode_mcp_body<T: DeserializeOwned>(body: &str, fn_name: &str, ctx: &str) -> Result<ParallelResp<T>, AppError> {
  let json_text = extract_json(body).ok_or_else(|| AppError::BadRequest(format!(
    "{}: no JSON found in response ({}), body head: {}", fn_name, ctx, head(body))))?;

  if let Ok(v) = serde_json::from_str::<Value>(&json_text) {
    if let Some(err) = v.get("error") {
      return Err(AppError::BadRequest(format!(
        "{}: JSON-RPC error: {}", fn_name, head(&err.to_string()))));
    }
    if v["result"]["isError"].as_bool() == Some(true) {
      let msg = v["result"]["content"][0]["text"].as_str().unwrap_or("<no content>");
      return Err(AppError::BadRequest(format!(
        "{}: Parallel tool returned isError: {}", fn_name, head(msg))));
    }
  }

  serde_json::from_str(&json_text).map_err(|e| AppError::BadRequest(format!(
    "{}: JSON decode error: {} (line {}, col {}); {}; body head: {}",
    fn_name, e, e.line(), e.column(), ctx, head(&json_text))))
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
  use crate::{metadata::{FetchRes, ImageStructuredContent, ToolCall}, test_utils::save_output};

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

  fn image_search_json() -> String {
    serde_json::json!({
      "name": "image_search",
      "arguments": {
        "objective": "official diagram of the Golden Gate Bridge, not a photo",
        "search_queries": ["golden gate bridge diagram"]
      }
    }).to_string()
  }

  fn image_ok_body() -> String {
    serde_json::json!({
      "jsonrpc": "2.0", "id": 1,
      "result": {
        "content": [{"type": "text", "text": "x"}],
        "structuredContent": {
          "search_id": "s", "session_id": "ss",
          "results": [{"image_url": "https://i.com/a.jpg", "width": 10, "height": 20}]
        },
        "isError": false
      }
    }).to_string()
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
  #[ignore]
  fn test_image_search_parallel_works() {
    let call = ToolCall::from_input(&image_search_json()).unwrap();
    assert_eq!(call.name, "image_search");
    let payload: ImageSearchPayloadParallel = call.build_payload().unwrap();
    let resp = image_search_parallel(&payload);
    assert!(resp.is_ok(), "Expected Ok, got {:#?}", resp.err());
    let resp = resp.unwrap();
    save_output(&resp, "parallel_image.json", "image_search_parallel");
    assert!(!resp.result.is_error, "tool error: {:#?}", resp.result.content);
    let results = &resp.result.structured_content.results;
    assert!(!results.is_empty());
    assert!(results.iter().all(|r| r.image_url.starts_with("http")));
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

  #[test]
  fn decode_plain_json_body() {
    let r: ParallelResp<ImageStructuredContent> = decode_mcp_body(&image_ok_body(), "t", "ctx").unwrap();
    assert_eq!(r.result.structured_content.results[0].width, Some(10));
  }

  #[test]
  fn decode_sse_body() {
    let body = format!("event: message\ndata: {}\n\n", image_ok_body());
    let r: ParallelResp<ImageStructuredContent> = decode_mcp_body(&body, "t", "ctx").unwrap();
    assert_eq!(r.result.structured_content.results.len(), 1);
  }

  #[test]
  fn decode_surfaces_tool_error_without_structured_content() {
    let body = r#"{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"Please sign in to use advanced mode"}],"isError":true}}"#;
    let e = decode_mcp_body::<ImageStructuredContent>(body, "t", "ctx").unwrap_err().to_string();
    assert!(e.contains("isError") && e.contains("sign in"), "got: {e}");
  }

  #[test]
  fn decode_surfaces_jsonrpc_error() {
    let body = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"Invalid params"}}"#;
    let e = decode_mcp_body::<ImageStructuredContent>(body, "t", "ctx").unwrap_err().to_string();
    assert!(e.contains("JSON-RPC") && e.contains("Invalid params"), "got: {e}");
  }

  #[test]
  fn decode_reports_garbage() {
    let e = decode_mcp_body::<ImageStructuredContent>("<html>502</html>", "t", "ctx").unwrap_err().to_string();
    assert!(e.contains("no JSON found"), "got: {e}");
  }

  #[test]
  fn extract_json_picks_last_sse_event_with_json() {
    let body = "event: ping\ndata: keepalive\n\nevent: message\ndata: {\"a\":1}\n\n";
    assert_eq!(extract_json(body).as_deref(), Some("{\"a\":1}"));
  }
}