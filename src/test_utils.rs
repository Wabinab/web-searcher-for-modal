use std::sync::OnceLock;

use serde::Serialize;

use crate::DB_FILENAME;

const TEST_PATH_MAINDIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test_files/");
static TEST_DIR: OnceLock<String> = OnceLock::new();

pub(crate) fn home_folder() -> &'static String {
  TEST_DIR.get_or_init(|| {
    // let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/test_files/"));
    let dir = TEST_PATH_MAINDIR.to_string();
    std::fs::create_dir_all(&dir).unwrap();

    // Reusing this function to init database so we don't have to specifically
    // call it. 
    init_test_db();  // dir is already in DB_FILENAME when we define it. 
    
    dir
  })
}

fn init_test_db() {
  use redb::Database;
  let _db = match Database::create(DB_FILENAME) {
    Ok(db) => db,
    Err(redb::DatabaseError::DatabaseAlreadyOpen) => return,  // already init, skip.
    Err(e) => panic!("{:#?}", e),
  };
}

// ============================================================================
/// Write output to file to see the results with our very own eyes. 
/// Default to home folder. 
pub(crate) fn save_output<T: Serialize>(data: T, filename: &str, identifier: &str) {
  use std::{fs::File, path::Path};
  let path = Path::new(&home_folder()).join(filename);
  let file = File::create(&path)
    .expect(format!("[{}] Failed to create output file.", identifier).as_str());
  serde_json::to_writer_pretty(file, &data)
    .expect(format!("[{}] Failed to write json.", identifier).as_str());
}

// =============================================================================
// The json required to test out parallel search. 
pub(crate) fn web_search_json() -> String {
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

pub(crate) fn web_fetch_json() -> String {
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

/// We'll try to return 2 urls and see how it looks like. 
/// LLM most likely won't do this, but we need to prepare for it. 
pub(crate) fn multiple_fetch_json() -> String {
  use uuid::Uuid;
  let payload = serde_json::json!({
    "id": Uuid::new_v4(),
    "name": "web_fetch",
    "arguments": serde_json::json!({
      "urls": ["https://www.morphllm.com/pricing", "https://www.koyeb.com/pricing#compute"],
      "objective": "Get detailed pricing information for comparison between both providers."
    }).to_string()
  });
  payload.to_string()
}