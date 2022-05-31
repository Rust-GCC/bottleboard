use std::fs;
use std::io;
use std::path::PathBuf;

use chrono::prelude::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TestsuiteResult {
    name: String,
    commit: String,
    date: DateTime<Utc>,
}

impl TestsuiteResult {
    // FIXME: Add doc
    /// This is needed to validate the contents of the testsuite results we got
    fn from_bytes(bytes: &[u8]) -> Result<TestsuiteResult, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    fn write_to_file(&self, path: PathBuf) -> Result<(), io::Error> {
        let json = serde_json::to_string(self)?;
        fs::write(path, json)
    }
}
