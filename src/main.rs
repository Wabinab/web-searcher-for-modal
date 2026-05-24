use std::io::{self, Read};

use crate::{apperror::{AppError, require}, metadata::{FetchRetVal, OllamaSearchResp, SearchResOl, ToolCall, WebFetchPayloadParallel, WebSearchPayloadParallel}, ollama_search::{fetch_data_ollama, search_data_ollama}, parallel_search::{fetch_data_parallel, search_data_parallel}};

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
  match call.name.as_str() {
    "web_search" => {
      let payload: WebSearchPayloadParallel = call.build_payload()?;
      let res = search_data_parallel(&payload);
      if res.is_ok() { 
        let parallel_resp = res.unwrap();
        // We'll consume structured_content after this. If need to save, save BEFORE this point.
        let search_resp: OllamaSearchResp = parallel_resp.result.structured_content.into();
        return Ok(serde_json::to_string(&search_resp.results)?)
      }
      // If not ok, fall back on Ollama. 
      let queries: Vec<&str> = payload.params.arguments.iter_queries().collect();
      let top_k = ((12 + queries.len() / 2) / queries.len()).clamp(3, 10);
      let mut retval: Vec<SearchResOl> = Vec::new();
      for query in queries.iter().take(10) {
        let output = search_data_ollama(query, Some(top_k));
        if output.is_err() { continue; }
        let o = output.unwrap().results;
        retval.extend(o);
      }
      Ok(serde_json::to_string(&retval)?)
    },
    "web_fetch" => {
      let payload: WebFetchPayloadParallel = call.build_payload()?;
      let res = fetch_data_parallel(&payload);
      if res.is_ok() {
        let parallel_resp = res.unwrap();
        // Structured content will be consumed after this. If need to save, save NOW. 
        let fetch_resp: Vec<FetchRetVal> = parallel_resp.result.structured_content.into();
        return Ok(serde_json::to_string(&fetch_resp)?)
      }
      // If not ok, fall back on Ollama. 
      let urls: Vec<&str> = payload.params.arguments.iter_urls().collect();
      let mut retval: Vec<FetchRetVal> = Vec::new();
      for url in urls {  // No cutoff for individual links. 
        let output = fetch_data_ollama(url);
        if output.is_err() { continue; }
        let o = output.unwrap();
        retval.push(o.into());
      }
      Ok(serde_json::to_string(&retval)?)
    },
    _ => { Err(AppError::BadRequest("Unknown tool call. Only web_search and web_fetch allowed.".to_string())) }
  }
}