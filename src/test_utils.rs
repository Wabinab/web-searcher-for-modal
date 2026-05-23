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