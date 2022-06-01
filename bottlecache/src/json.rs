use std::fs;
use std::io;
use std::path::PathBuf;

use chrono::prelude::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TestsuiteResult {
    pub name: String,
    pub commit: String,
    pub date: DateTime<Utc>,
}

impl TestsuiteResult {
    // FIXME: Add doc
    /// This is needed to validate the contents of the testsuite results we got
    pub fn from_bytes(bytes: &[u8]) -> Result<TestsuiteResult, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}
