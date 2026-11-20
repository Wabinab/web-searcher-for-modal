use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{apperror::AppError, metadata::{ToolCall, WebParams, WebPayloadParallel}};

impl ToolCall {
  pub(crate) fn from_input(input: &str) -> Result<Self, AppError> {
    serde_json::from_str(input).map_err(|e| {
      // Fetch the list of keys. 
      let keys = serde_json::from_str::<Value>(input).ok()
        .and_then(|v| v.as_object().map(
          |o| o.keys().cloned().collect::<Vec<_>>()));
      AppError::BadRequest(format!(
        "invalid tool-call envelope: {e}. 
        Expected {{\"name\":\"web_search\"|\"web_fetch\",\"arguments\":
        <JSON string or object>}} (\"id\" optional); got top-level keys: {keys:?}"))
    })
  }

  pub(crate) fn build_payload<T: Serialize + DeserializeOwned>(self
  ) -> Result<WebPayloadParallel<T>, AppError> {
    let args: T = serde_json::from_str(&self.arguments).map_err(|e| 
      AppError::BadRequest(format!("{} args invalid: {e} 
        (line/col refer to the arguments payload, not the envelope)", self.name)))?;
    Ok(WebPayloadParallel { jsonrpc: "2.0".to_string(), id: 1, method: "tools/call".to_string(), 
      params: WebParams { name: self.name, arguments: args } })
  }
}