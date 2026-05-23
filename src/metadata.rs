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
  pub content: String,
  // This doesn't exist in Ollama, just used when converting from Parallel to this format. 
  #[serde(default, skip_serializing_if="Option::is_none")]
  pub publish_date: Option<String>
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

// ================================================================
// These are the output format we'll return to the python side for compacting. 
// We need to standardized them. 

/// For web_search result, we'll use Ollama's returned result. We'll need an implementation
/// to do the conversion. Note that ownership will be CONSUMED after this, so if we need to 
/// clone to save stuffs, it had to be done BEFORE this stage. 
impl From<SearchStructuredContent> for OllamaSearchResp {
  fn from(value: SearchStructuredContent) -> Self {
    let results: Vec<SearchResOl> = value.results.into_iter().map(|r| SearchResOl { 
      title: r.title, url: r.url, content: r.excerpts.join("\n"), publish_date: r.publish_date 
    }).collect();
    Self { results }
  }
}


// ==========================================================
#[cfg(test)]
mod tests {
  use crate::{parallel_search::search_data_parallel, test_utils::web_search_json};

use super::*;

  #[test]
  #[ignore]
  fn test_conversion_from_structured_content_to_ollama_search_resp_as_expected() {
    let call = ToolCall::from_input(&web_search_json()).expect("Tool Call Error.");
    assert_eq!(call.name.as_str(), "web_search", "You probably used the wrong json string. Check!");
    let payload: WebSearchPayloadParallel = call.build_payload().expect("Build payload problem.");
    let parallel_resp = search_data_parallel(&payload)
      .expect("Search data problem.");

    // Clone it since we need to do comparison and testing. 
    let search_resp: OllamaSearchResp = parallel_resp.result.structured_content.clone().into();
    let orig_content = parallel_resp.result.structured_content.results;
    let mutated_content = search_resp.results;
    assert_eq!(orig_content.len(), mutated_content.len());
    assert_eq!(orig_content[0].title, mutated_content[0].title);
    assert_eq!(orig_content[1].url, mutated_content[1].url);
    assert_eq!(orig_content[2].excerpts.join("\n"), mutated_content[2].content);
  }

  #[test]
  fn test_publish_date_serde_behavior() {
    let item = SearchResOl {
      title: "Test Title".into(),
      url: "https://example.com".into(),
      content: "Some content".into(),
      publish_date: None,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&item).unwrap();
    assert!(!json.contains("publish_date"));  // `publish_date` should NOT appear in the output

    // Deserialize back
    let deserialized: SearchResOl = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.publish_date, None);
  }

  #[test]
  fn test_missing_publish_date_on_deser() {
    // JSON without a publish_date field (Ollama-style)
    let json = r#"{
        "title": "Another Title",
        "url": "https://other.com",
        "content": "More content"
    }"#;
  
    let item: SearchResOl = serde_json::from_str(json).unwrap();
    assert_eq!(item.title, "Another Title");
    assert_eq!(item.publish_date, None);
  }

  #[test]
  fn test_from_search_structured_to_ollama_response() {
    // Simulate a SearchStructuredContent from the parallel API
    let structured = SearchStructuredContent {
      search_id: "s123".into(),
      results: vec![
        SearchRes {
          url: "https://a.com".into(),
          title: "A".into(),
          publish_date: Some("2025-01-01".into()),
          excerpts: vec!["excerpt1".into(), "excerpt2".into()],
        },
        SearchRes {
          url: "https://b.com".into(),
          title: "B".into(),
          publish_date: None,
          excerpts: vec!["excerpt3".into()],
        },
      ],
      warnings: None,
      usage: vec![],
      session_id: "sess1".into(),
    };

    // Convert to OllamaSearchResp
    let ollama: OllamaSearchResp = structured.into();

    assert_eq!(ollama.results.len(), 2);
    assert_eq!(ollama.results[0].title, "A");
    assert_eq!(ollama.results[0].url, "https://a.com");
    assert_eq!(ollama.results[0].content, "excerpt1\nexcerpt2");
    assert_eq!(ollama.results[0].publish_date, Some("2025-01-01".into()));

    assert_eq!(ollama.results[1].title, "B");
    assert_eq!(ollama.results[1].publish_date, None);
    assert_eq!(ollama.results[1].content, "excerpt3");
  }
}