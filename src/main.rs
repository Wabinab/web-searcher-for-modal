use std::io::{self, Read};

use crate::{
  apperror::AppError,
  metadata::{FetchRetVal, ImageSearchPayloadParallel, OllamaSearchResp, SearchResOl, ToolCall, WebFetchPayloadParallel, WebSearchPayloadParallel},
  ollama_search::{fetch_data_ollama, search_data_ollama},
  parallel_search::{fetch_data_parallel, image_search_parallel, search_data_parallel},
};

mod parallel_search;
mod ollama_search;
mod metadata;
mod apperror;
mod tool_call_parser;
#[cfg(test)] mod test_utils;

#[cfg(not(test))] pub(crate) const DB_FILENAME: &str = "rust_chat.redb";
#[cfg(test)] pub(crate) const DB_FILENAME: &str = "test_files/rust_chat.redb";

fn main() {
  let mut input = String::new();
  io::stdin().read_to_string(&mut input).expect("Failed to read stdin");

  match handle_tool_call(&input) {
    Ok(output) => println!("{}", output),
    Err(e) => {
      eprintln!("Error: {}", e);
      std::process::exit(1);
    }
  }
}

fn handle_tool_call(input: &str) -> Result<String, AppError> {
  let call = ToolCall::from_input(input)?;
  let name = call.name.clone();
  match name.as_str() {
    "web_search" => handle_web_search(call),
    "web_fetch" => handle_web_fetch(call),
    "image_search" => handle_image_search(call),
    other => Err(AppError::BadRequest(format!(
      "Unknown tool call '{}'. Only web_search, web_fetch and image_search allowed.", other))),
  }
}

fn handle_web_search(call: ToolCall) -> Result<String, AppError> {
  let payload: WebSearchPayloadParallel = call.build_payload()?;
  let parallel_err = match search_data_parallel(&payload) {
    Ok(r) => {
      // structured_content is consumed here. If we need to save, save BEFORE this point.
      let search_resp: OllamaSearchResp = r.result.structured_content.into();
      return Ok(serde_json::to_string(&search_resp.results)?);
    }
    Err(e) => { eprintln!("parallel web_search failed, falling back to ollama: {}", e); e }
  };

  let queries: Vec<&str> = payload.params.arguments.iter_queries().collect();
  let top_k = ((12 + queries.len() / 2) / queries.len()).clamp(3, 10);
  let mut retval: Vec<SearchResOl> = Vec::new();
  let mut errors: Vec<String> = Vec::new();
  for query in queries.iter().take(10) {
    match search_data_ollama(query, Some(top_k)) {
      Ok(o) => retval.extend(o.results),
      Err(e) => errors.push(format!("'{}': {}", query, e)),
    }
  }
  if retval.is_empty() && !errors.is_empty() {
    return Err(AppError::BadRequest(format!(
      "web_search failed. parallel: {}; ollama: {}", parallel_err, errors.join("; "))));
  }
  Ok(serde_json::to_string(&retval)?)
}

fn handle_web_fetch(call: ToolCall) -> Result<String, AppError> {
  let payload: WebFetchPayloadParallel = call.build_payload()?;
  let parallel_err = match fetch_data_parallel(&payload) {
    Ok(r) => {
      let fetch_resp: Vec<FetchRetVal> = r.result.structured_content.into();
      return Ok(serde_json::to_string(&fetch_resp)?);
    }
    Err(e) => { eprintln!("parallel web_fetch failed, falling back to ollama: {}", e); e }
  };

  let mut retval: Vec<FetchRetVal> = Vec::new();
  let mut errors: Vec<String> = Vec::new();
  for url in payload.params.arguments.iter_urls() {  // No cutoff for individual links.
    match fetch_data_ollama(url) {
      Ok(o) => retval.push(o.into()),
      Err(e) => errors.push(format!("'{}': {}", url, e)),
    }
  }
  if retval.is_empty() && !errors.is_empty() {
    return Err(AppError::BadRequest(format!(
      "web_fetch failed. parallel: {}; ollama: {}", parallel_err, errors.join("; "))));
  }
  Ok(serde_json::to_string(&retval)?)
}

/// No Ollama equivalent exists, so a Parallel failure is a hard error.
fn handle_image_search(call: ToolCall) -> Result<String, AppError> {
  let payload: ImageSearchPayloadParallel = call.build_payload()?;
  let resp = image_search_parallel(&payload)?;
  Ok(serde_json::to_string(&resp.result.structured_content.results)?)
}