mod artifact;

use std::path::PathBuf;
use std::time::{SystemTime, SystemTimeError};

use chrono::Duration;
use thiserror::Error;

use crate::json::TestsuiteResult;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("couldn't fetch file creation date")]
    FileCreationDate(#[from] SystemTimeError),
    #[error("error when using github API: {0}")]
    GitHub(#[from] octocrab::Error),
    #[error("error when extracting archive: {0}")]
    Unzipping(#[from] zip::result::ZipError),
    #[error("writing to disk failed: {0}")]
    Disk(#[from] std::io::Error),
}

#[derive(Clone)]
pub struct Data(pub () /* FIXME: Store a proper type here */);

// FIXME: We probably want to keep the last variation in a cache type or something
pub struct Cache {
    token: Option<String>,
    location: PathBuf,
    last_date: SystemTime,
    data: Option<Data>,
}

impl Cache {
    pub fn new(token: Option<String>, location: PathBuf) -> Cache {
        Cache {
            token,
            location,
            last_date: SystemTime::UNIX_EPOCH,
            data: None,
        }
    }

    fn is_invalidated(&self) -> Result<bool, CacheError> {
        let now = SystemTime::now();
        let age = now.duration_since(self.last_date)?;

        // UNWRAP: If we have an issue here, this is a programmer error: We want
        // the program to crash as this should never happen
        Ok(age > Duration::hours(24).to_std().unwrap() || self.data.is_none())
    }

    async fn update(&mut self) -> Result<(), CacheError> {
        let token = self.token.clone();
        let archives = artifact::fetch_result_files(token).await?;
        for archive in archives {
            let bytes = artifact::extract_json(archive).await?;
            let json = TestsuiteResult::from_bytes(bytes.as_slice());

            match json {
                Ok(json) => {
                    let path = self
                        .location
                        .join(PathBuf::from(format!("{}-{}.json", json.name, json.date)));
                    eprintln!("valid json! Writing to `{}`", path.display());
                    json.write_to(&path)?;
                }
                Err(e) => eprintln!("invalid json file... skipping it. Reason: `{}`", e),
            }
        }

        self.last_date = SystemTime::now();

        // FIXME: Do something here for real
        self.data = Some(Data(()));

        Ok(())
    }

    pub async fn data(&mut self) -> Result<Data, CacheError> {
        if self.is_invalidated()? {
            eprintln!("updating cache");
            self.update().await?;
        } else {
            eprintln!("return cached data!");
        }

        // UNWRAP: We can safely unwrap here as our data is updated. If there is
        // no data or none has been fetched/written to disk, then the `update`
        // function will have returned an error which would have been propagated
        // already.
        Ok(self.data.clone().unwrap())
    }
}
