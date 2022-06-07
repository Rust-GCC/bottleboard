use std::fs;
use std::io;
use std::path::Path;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TestsuiteResult {
    pub name: String,
    pub commit: String,
    pub date: NaiveDate,
}

impl TestsuiteResult {
    // FIXME: Add doc
    /// This is needed to validate the contents of the testsuite results we got
    pub fn from_bytes(bytes: &[u8]) -> Result<TestsuiteResult, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    pub fn write_to(&self, path: &Path) -> Result<(), io::Error> {
        fs::write(path, serde_json::to_string(self)?.as_bytes())
    }
}
