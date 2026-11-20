//! All the structs and constants will go here. 
use serde::{Deserialize, Serialize};
use borsh::{BorshSerialize, BorshDeserialize};
use serde_json::Value;

// pub(crate) const PARALLEL_MCP_URL: &str = "https://search.parallel.ai/mcp";
pub(crate) const PARALLEL_MCP_URL: &str = "https://mcp.parallel.ai/mcp";
pub(crate) const OLLAMA_MCP_URL: &str = "https://ollama.com/api/";  // /web_search or web_fetch.

// ===============================================
// Parallel
#[cfg_attr(test, derive(Serialize, Clone))]
#[derive(Debug, Deserialize)]
pub(crate) struct ParallelResp<T> {
  pub jsonrpc: String,
  #[serde(default)] pub id: Value,
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
pub(crate) type ImageResp = ParallelResp<ImageStructuredContent>;

#[cfg_attr(test, derive(Serialize, Clone))]
#[derive(Debug, Deserialize)]
pub(crate) struct SearchStructuredContent {
  pub search_id: String, 
  pub results: Vec<SearchRes>,
  pub warnings: Option<Value>,  // Warnings won't be saved, but will be displayed to user. 
  #[serde(default)] pub usage: Vec<Usage>,  
  #[serde(default)] pub session_id: String,
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
  #[serde(default)] pub errors: Vec<Value>,
  pub warnings: Option<Value>,
  #[serde(default)] pub usage: Vec<Usage>,
  #[serde(default)] pub session_id: String,
}

#[cfg_attr(test, derive(Serialize))]
#[derive(Debug, Clone, Deserialize, BorshSerialize, BorshDeserialize)]
pub(crate) struct FetchRes {
  pub url: String,
  pub title: Option<String>,
  pub publish_date: Option<String>,
  pub excerpts: Vec<String>,
  pub full_content: Option<String>,  // Maybe Vec<String>??? Unsure, currently always return None. 
}

#[cfg_attr(test, derive(Serialize, Clone))]
#[derive(Debug, Deserialize)]
pub(crate) struct ImageStructuredContent {
  #[serde(default)] pub search_id: String,
  pub results: Vec<ImageRes>,
  pub warnings: Option<Value>,
  #[serde(default)] pub usage: Vec<Usage>,
  #[serde(default)] pub session_id: String,
}

// May need Serialize in non-test later.
#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub(crate) struct ImageRes {
  pub image_url: String,
  #[serde(default, skip_serializing_if = "Option::is_none")] pub source_page_url: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")] pub title: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")] pub width: Option<u32>,
  #[serde(default, skip_serializing_if = "Option::is_none")] pub height: Option<u32>
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
  pub name: String,  // "web_search" / "web_fetch" / "image_search"
  pub arguments: T
}

pub(crate) type WebSearchPayloadParallel = WebPayloadParallel<SearchArgsParallel>;
// Web-fetch is renamed "extract api" but the call is still "web_fetch"
pub(crate) type WebFetchPayloadParallel = WebPayloadParallel<FetchArgsParallel>;
pub(crate) type ImageSearchPayloadParallel = WebPayloadParallel<ImageArgsParallel>;

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

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FetchArgsParallel {
  /// A description of what to look for in the pages, up to 200 characters.
  pub objective: String,
  pub urls: Vec<String>,
  // New fields
  /// Keyword queries of 3–6 words each that focus the excerpts. When fetching results from web_search, pass the same queries.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub search_queries: Option<Vec<String>>,
  // pub full_content: bool,
  // pub allow_live_fetch: bool,
}

impl FetchArgsParallel {
  pub(crate) fn iter_urls(&self) -> impl Iterator<Item = &str> {
    self.urls.iter().map(|u| u.as_str())
  }
}

#[cfg_attr(test, derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ImageArgsParallel {
  pub objective: String,
  pub search_queries: Vec<String>,
  #[serde(default = "fast_mode", skip_deserializing)] pub mode: String,  // Always use "fast" as string. We supply, not AI.
}

fn fast_mode() -> String { "fast".to_string() }

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
  #[serde(default)] pub title: Option<String>,
  pub content: String, 
  #[serde(default)] pub links: Vec<String>
}

// ===============================================================
// Tool Calling
#[derive(Debug, Deserialize)]
pub(crate) struct ToolCall {
  #[serde(default)] pub id: String,  // Never read anywhere, don't reject input over it.
  pub name: String,  // web_search or web_fetch
  #[serde(deserialize_with = "args_as_string")] pub arguments: String
}

/// `arguments` may arrive as a JSON string (OpenAI style) or a JSON object 
/// (Ollama style). Normalize to string so `build_payload` has a single path.
fn args_as_string<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
  Ok(match Value::deserialize(d)? {
    Value::String(s) => s,
    other => other.to_string(),
  })
}

// NOTE: impl ToolCall in tool_call_parser. 

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

/// For web_fetch, we'll define a custom struct to return. 
#[derive(Debug, Serialize)]
pub(crate) struct FetchRetVal {
  pub title: Option<String>,
  pub content: String,
  #[serde(default, skip_serializing_if="Option::is_none")]
  pub publish_date: Option<String>,
  #[serde(default, skip_serializing_if="Vec::is_empty")]
  pub links: Vec<String>,
  #[serde(default, skip_serializing_if="Option::is_none")]
  pub full_content: Option<String>
}

impl From<FetchResOl> for FetchRetVal {
  fn from(value: FetchResOl) -> Self {
    FetchRetVal { title: value.title, content: value.content, publish_date: None, 
      links: value.links, full_content: None }
  }
}

impl From<FetchStructuredContent> for Vec<FetchRetVal> {
  fn from(value: FetchStructuredContent) -> Self {
    value.results.into_iter().map(|r| r.into()).collect()
  }
}

impl From<FetchRes> for FetchRetVal {
  fn from(value: FetchRes) -> Self {
    FetchRetVal { title: value.title, content: value.excerpts.join("\n"), 
      publish_date: value.publish_date, links: vec![value.url], full_content: value.full_content }
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

  // ---------- ToolCall envelope ----------
  #[test]
  fn tool_call_id_is_optional() {
    let c: ToolCall = serde_json::from_str(r#"{"name":"web_fetch","arguments":{"urls":[],"objective":"o"}}"#).unwrap();
    assert_eq!(c.id, "");
  }

  #[test]
  fn tool_call_accepts_arguments_as_string_or_object() {
    let s: ToolCall = serde_json::from_str(r#"{"name":"n","arguments":"{\"a\":1}"}"#).unwrap();
    let o: ToolCall = serde_json::from_str(r#"{"name":"n","arguments":{"a":1}}"#).unwrap();
    assert_eq!(s.arguments, r#"{"a":1}"#);
    assert_eq!(o.arguments, r#"{"a":1}"#);
  }

  #[test]
  fn tool_call_rejects_bare_args_with_helpful_error() {
    let e = ToolCall::from_input(r#"{"urls":["https://x.com"],"objective":"o"}"#).unwrap_err().to_string();
    assert!(e.contains("missing field `name`") && e.contains("urls"), "got: {e}");
  }

  #[test]
  fn build_payload_names_the_tool_on_type_error() {
    let c = ToolCall::from_input(r#"{"name":"web_fetch","arguments":{"urls":["https://x.com"],"objective":-1}}"#).unwrap();
    let e = c.build_payload::<FetchArgsParallel>().unwrap_err().to_string();
    assert!(e.contains("web_fetch") && e.contains("invalid type"), "got: {e}");
  }

  // ---------- Ollama fetch ----------
  #[test]
  fn ollama_fetch_parses_full_shape() {
    let json = r#"{"title":"Example","content":"Body text","links":["https://a.com","https://b.com"]}"#;
    let r: FetchResOl = serde_json::from_str(json).unwrap();
    assert_eq!(r.title.as_deref(), Some("Example"));
    assert_eq!(r.content, "Body text");
    assert_eq!(r.links, vec!["https://a.com", "https://b.com"]);
  }

  #[test]
  fn ollama_fetch_tolerates_missing_title_and_links() {
    let r: FetchResOl = serde_json::from_str(r#"{"content":"x"}"#).unwrap();
    assert!(r.title.is_none());
    assert!(r.links.is_empty());
  }

  #[test]
  fn ollama_fetch_rejects_missing_content() {
    assert!(serde_json::from_str::<FetchResOl>(r#"{"title":"t","links":[]}"#).is_err());
  }

  #[test]
  fn ollama_fetch_to_ret_val_serialization_shape() {
    let r: FetchResOl = serde_json::from_str(r#"{"title":null,"content":"body","links":[]}"#).unwrap();
    let v: FetchRetVal = r.into();
    let j = serde_json::to_value(&v).unwrap();
    assert_eq!(j["content"], "body");
    assert!(j["title"].is_null());
    assert!(j.get("publish_date").is_none());
    assert!(j.get("full_content").is_none());
    assert!(j.get("links").is_none(), "empty links must be omitted");
  }

  #[test]
  fn parallel_fetch_res_to_ret_val_puts_url_in_links() {
    let r = FetchRes {
      url: "https://x.com".into(), title: None, publish_date: Some("2025-01-01".into()),
      excerpts: vec!["a".into(), "b".into()], full_content: None,
    };
    let v: FetchRetVal = r.into();
    assert_eq!(v.content, "a\nb");
    assert_eq!(v.links, vec!["https://x.com"]);
    assert_eq!(v.publish_date.as_deref(), Some("2025-01-01"));
  }

  // ---------- Image search ----------
  const IMAGE_RESP_SAMPLE: &str = r#"{
    "search_id": "search_fcb2b4f3c75e418687bccaa1a8381331",
    "session_id": "session_fcb2b4f3c75e418687bccaa1a8381331",
    "results": [
      {"title": "Golden Gate Bridge at sunset",
        "image_url": "https://www.example.com/images/golden-gate-sunset.jpg",
        "source_page_url": "https://www.example.com/golden-gate-bridge",
        "width": 1920, "height": 1080},
      {"title": "The Golden Gate Bridge seen from the Marin Headlands",
        "image_url": "https://www.example.org/photos/golden-gate-marin.jpg",
        "source_page_url": "https://www.example.org/travel/san-francisco",
        "width": 1600, "height": 1067}
    ]
  }"#;

  #[test]
  fn image_structured_content_parses_docs_sample() {
    let s: ImageStructuredContent = serde_json::from_str(IMAGE_RESP_SAMPLE).unwrap();
    assert_eq!(s.results.len(), 2);
    assert_eq!(s.results[0].image_url, "https://www.example.com/images/golden-gate-sunset.jpg");
    assert_eq!(s.results[0].source_page_url.as_deref(), Some("https://www.example.com/golden-gate-bridge"));
    assert_eq!((s.results[1].width, s.results[1].height), (Some(1600), Some(1067)));
    assert!(s.warnings.is_none() && s.usage.is_empty());
  }

  #[test]
  fn image_res_tolerates_missing_optionals() {
    let r: ImageRes = serde_json::from_str(r#"{"image_url":"https://i.com/a.jpg"}"#).unwrap();
    assert!(r.title.is_none() && r.source_page_url.is_none() && r.width.is_none() && r.height.is_none());
    let r: ImageRes = serde_json::from_str(
      r#"{"image_url":"https://i.com/a.jpg","title":null,"width":null,"height":null}"#).unwrap();
    assert!(r.title.is_none() && r.width.is_none());
  }

  #[test]
  fn image_res_requires_image_url() {
    assert!(serde_json::from_str::<ImageRes>(r#"{"title":"t"}"#).is_err());
  }

  #[test]
  fn image_res_serialization_skips_nones() {
    let r: ImageRes = serde_json::from_str(r#"{"image_url":"https://i.com/a.jpg"}"#).unwrap();
    let j = serde_json::to_value(&r).unwrap();
    assert_eq!(j.as_object().unwrap().len(), 1);
  }

  #[test]
  fn parallel_resp_tolerates_missing_id() {
    let sc: Value = serde_json::from_str(IMAGE_RESP_SAMPLE).unwrap();
    let env = serde_json::json!({
      "jsonrpc": "2.0",
      "result": {"content": [{"type": "text", "text": "x"}], "structuredContent": sc, "isError": false}
    }).to_string();
    let r: ParallelResp<ImageStructuredContent> = serde_json::from_str(&env).unwrap();
    assert_eq!(r.result.structured_content.results.len(), 2);
  }

  #[test]
  fn parallel_resp_tolerates_string_id() {
    let env = r#"{"jsonrpc":"2.0","id":"abc","result":{"content":[],"structuredContent":{"results":[]},"isError":false}}"#;
    assert!(serde_json::from_str::<ParallelResp<ImageStructuredContent>>(env).is_ok());
  }

  #[test]
  fn image_args_force_fast_even_if_caller_sends_advanced() {
    let a: ImageArgsParallel = serde_json::from_str(
      r#"{"objective":"o","search_queries":["q"],"mode":"advanced"}"#).unwrap();
    assert_eq!(a.mode, "fast");
    assert_eq!(serde_json::to_value(&a).unwrap()["mode"], "fast");
  }

  #[test]
  fn image_args_default_mode_is_fast() {
    let a: ImageArgsParallel = serde_json::from_str(r#"{"objective":"o","search_queries":["q"]}"#).unwrap();
    assert_eq!(a.mode, "fast");
  }

  #[test]
  fn image_args_require_objective_and_queries() {
    assert!(serde_json::from_str::<ImageArgsParallel>(r#"{"objective":"o"}"#).is_err());
    assert!(serde_json::from_str::<ImageArgsParallel>(r#"{"search_queries":["q"]}"#).is_err());
  }

  #[test]
  fn image_args_drop_caller_supplied_override_and_unknown_fields() {
    let a: ImageArgsParallel = serde_json::from_str(
      r#"{"objective":"o","search_queries":["q"],
          "override":{"advanced_settings":{"location":"US"}},
          "advanced_settings":{"max_results":20}}"#).unwrap();
    let j = serde_json::to_value(&a).unwrap();
    assert_eq!(j.as_object().unwrap().len(), 3);  // objective, search_queries, mode
    assert!(j.get("override").is_none());
    assert!(j.get("advanced_settings").is_none());
  }

  #[test]
  fn image_payload_builds_from_tool_call() {
    let env = serde_json::json!({
      "name": "image_search",
      "arguments": {"objective": "diagram of a transformer, not a photo", "search_queries": ["transformer architecture diagram"]}
    }).to_string();
    let call = ToolCall::from_input(&env).unwrap();
    let payload: ImageSearchPayloadParallel = call.build_payload().unwrap();
    let v = serde_json::to_value(&payload).unwrap();
    assert_eq!(v["method"], "tools/call");
    assert_eq!(v["params"]["name"], "image_search");
    assert_eq!(v["params"]["arguments"]["mode"], "fast");
  }
}