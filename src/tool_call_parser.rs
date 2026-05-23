use serde::{Serialize, de::DeserializeOwned};

use crate::{apperror::AppError, metadata::{ToolCall, WebParams, WebPayloadParallel}};

impl ToolCall {
  pub(crate) fn from_input(input: &str) -> Result<Self, AppError> {
    Ok(serde_json::from_str(input)?)
  }

  // pub(crate) fn parse_args<T: DeserializeOwned>(&self) -> Result<T, AppError> {
  //   Ok(serde_json::from_str(&self.arguments)?)
  // }

  pub(crate) fn build_payload<T: Serialize + DeserializeOwned>(self
  ) -> Result<WebPayloadParallel<T>, AppError> {
    let args: T = serde_json::from_str(&self.arguments)?;
    Ok(WebPayloadParallel { jsonrpc: "2.0".to_string(), id: 1, method: "tools/call".to_string(), 
      params: WebParams { name: self.name, arguments: args } })
  }
}