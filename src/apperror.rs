// Custom error return type
#[derive(Debug, thiserror::Error)]
pub(crate) enum AppError {
  #[error(transparent)] Db(#[from] redb::Error),
  #[error(transparent)] DbCommit(#[from] redb::CommitError),
  #[error(transparent)] DbTable(#[from] redb::TableError),
  #[error(transparent)] DbStorage(#[from] redb::StorageError),
  #[error(transparent)] DbTransaction(#[from] redb::TransactionError),
  #[error(transparent)] DbDatabase(#[from] redb::DatabaseError),
  // #[error(transparent)] SynError(#[from] syn::Error),
  #[error(transparent)] SerdeJson(#[from] serde_json::Error),
  #[error(transparent)] Reqwest(#[from] reqwest::Error),
  // #[error("NotFound: {0}")] NotFound(String),
  #[error("BadRequest: {0}")] BadRequest(String),
  // #[error("Internal: {0}")] Internal(String),
  // #[error("")] HasCache(),
  // #[error("Key not found for {0}")] KeyNotFound(String)
}

impl From<&str> for AppError {
  fn from(msg: &str) -> Self {
    AppError::BadRequest(msg.to_string())
  }
}

/// Attempt to imitate the require! macro in NEAR Protocol. 
/// This, however, will return BadRequest upon error. 
/// The first argument is the condition that should hold to NOT raise the error.
/// i.e. if c is true, it'll not raise the error. If false, it'll raise the error. 
/// If an error is raised, return BadRequest with the given error message.
pub(crate) fn require(c: bool, err_msg: impl Into<String>) -> Result<(), AppError> {
  if c { Ok(()) } else { Err(AppError::BadRequest(err_msg.into())) }
}