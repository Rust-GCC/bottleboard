use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Debug)]
pub struct RunResults {
    pub tests: u64,
    pub passes: u64,
    pub failures: u64,
}

#[derive(Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Debug)]
pub struct TestsuiteResult {
    pub name: String,
    pub commit: String,
    pub date: NaiveDate,
    pub results: RunResults,
}

impl TestsuiteResult {
    // FIXME: Add doc
    /// This is needed to validate the contents of the testsuite results we got
    pub fn from_bytes(bytes: &[u8]) -> Result<TestsuiteResult, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}
