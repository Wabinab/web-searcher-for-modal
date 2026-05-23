//! All the structs and constants will go here. 
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use borsh::{BorshSerialize, BorshDeserialize};
use serde_json::Value;

use crate::apperror::AppError;

pub(crate) const PARALLEL_MCP_URL: &str = "https://search.parallel.ai/mcp";
pub(crate) const OLLAMA_MCP_URL: &str = "https://ollama.com/api/";  // /web_search or web_fetch.

// ===============================================
// Parallel
#[cfg_attr(test, derive(Serialize, Clone))]
#[derive(Debug, Deserialize)]
pub(crate) struct ParallelResp<T> {
  pub jsonrpc: String,
  pub id: i32,
  pub result: ParallelRespRes<T>,
}

#[cfg_attr(test, derive(Serialize, Clone))]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParallelRespRes<T> {
  pub content: Vec<ContentItem>,  // Vector of length 1. 
  pub structured_content: T,
  pub is_error: bool
}

#[cfg_attr(test, derive(Serialize, Clone))]
#[derive(Debug, Deserialize)]
pub(crate) struct ContentItem {
  #[serde(rename = "type")]
  pub content_type: String,  // Usually returns "type": "text", we parse into content_type.
  pub text: String,  // Stringified JSON of the structuredContent. 
}

pub(crate) type SearchResp = ParallelResp<SearchStructuredContent>;
pub(crate) type FetchResp = ParallelResp<FetchStructuredContent>;

#[cfg_attr(test, derive(Serialize, Clone))]
#[derive(Debug, Deserialize)]
pub(crate) struct SearchStructuredContent {
  pub search_id: String, 
  pub results: Vec<SearchRes>,
  pub warnings: Option<Value>,  // Warnings won't be saved, but will be displayed to user. 
  pub usage: Vec<Usage>,  
  pub session_id: String,
}

#[cfg_attr(test, derive(Serialize, Clone))]
#[derive(Debug, Deserialize, BorshSerialize, BorshDeserialize)]
pub(crate) struct SearchRes {
  pub url: String,
  pub title: String, 
  pub publish_date: Option<String>,
  pub excerpts: Vec<String>,
}

#[cfg_attr(test, derive(Serialize, Clone))]
#[derive(Debug, Deserialize)]
pub(crate) struct FetchStructuredContent {
  pub extract_id: String, 
  pub results: Vec<FetchRes>,
  pub errors: Vec<Value>,
  pub warnings: Option<Value>,
  pub usage: Vec<Usage>,
  pub session_id: String,
}

#[cfg_attr(test, derive(Serialize, Clone))]
#[derive(Debug, Deserialize, BorshSerialize, BorshDeserialize)]
pub(crate) struct FetchRes {
  pub url: String,
  pub title: String,
  pub publish_date: Option<String>,
  pub excerpts: Vec<String>,
  pub full_content: Option<String>,  // Maybe Vec<String>??? Unsure, currently always return None. 
}

#[cfg_attr(test, derive(Serialize, Clone))]
#[derive(Debug, Deserialize, BorshSerialize, BorshDeserialize)]
pub(crate) struct Usage {
  pub name: String,
  pub count: i32
}

// ============================
#[cfg_attr(test, derive(Deserialize, Clone))]
#[derive(Debug, Serialize)]
pub(crate) struct WebPayloadParallel<T> {
  pub jsonrpc: String,  // "2.0"
  pub id: i32,
  pub method: String,  // "tools/call"
  pub params: WebParams<T>
}

#[cfg_attr(test, derive(Deserialize, Clone))]
#[derive(Debug, Serialize)]
pub(crate) struct WebParams<T> {
  pub name: String,  // "web_search" / "web_fetch"
  pub arguments: T
}

pub(crate) type WebSearchPayloadParallel = WebPayloadParallel<SearchArgsParallel>;
pub(crate) type WebFetchPayloadParallel = WebPayloadParallel<FetchArgsParallel>;

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SearchArgsParallel {
  pub objective: String,  // The main "query" string. 
  pub search_queries: Vec<String>  // Some extra search queries. 
}

impl SearchArgsParallel {
  pub(crate) fn iter_queries(&self) -> impl Iterator<Item = &str> {
    std::iter::once(&self.objective as &str).chain(self.search_queries.iter().map(|s| s as &str))
  }
}

impl From<WebSearchArgs> for SearchArgsParallel {
  fn from(s: WebSearchArgs) -> Self {
    SearchArgsParallel { objective: s.objective, search_queries: s.search_queries }
  }
}

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FetchArgsParallel {
  pub objective: String,
  pub urls: Vec<String>
}

impl FetchArgsParallel {
  pub(crate) fn iter_urls(&self) -> impl Iterator<Item = &str> {
    self.urls.iter().map(|u| u.as_str())
  }
}

impl From<WebFetchArgs> for FetchArgsParallel {
  fn from(value: WebFetchArgs) -> Self {
    FetchArgsParallel { objective: value.objective, urls: value.urls }
  }
}

// ========================================================
// Ollama
#[cfg_attr(test, derive(Serialize))]
#[derive(Debug, Deserialize)]
pub(crate) struct OllamaSearchResp {
  pub results: Vec<SearchResOl>
}

#[derive(Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub(crate) struct SearchResOl {
  pub title: String,
  pub url: String,
  pub content: String
}

/// Ollama web_fetch result don't have the results vec, only something 
/// that looks like SearchResOl but single. 
#[derive(Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub(crate) struct FetchResOl {
  pub title: String,
  pub content: String, 
  pub links: Vec<String>
}

// ===============================================================
// Tool Calling
#[derive(Debug, Deserialize)]
pub(crate) struct ToolCall {
  pub id: String,
  pub name: String,  // web_search or web_fetch
  pub arguments: String
}

// NOTE: impl ToolCall in tool_call_parser. 

#[derive(Debug, Deserialize)]
pub(crate) struct WebFetchArgs {
  pub objective: String,
  pub urls: Vec<String>
}

#[derive(Debug, Deserialize)]
pub(crate) struct WebSearchArgs {
  pub objective: String,
  pub search_queries: Vec<String>
}