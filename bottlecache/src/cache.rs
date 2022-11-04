mod artifact;

use std::collections::HashSet;
use std::time::{SystemTime, SystemTimeError};

use chrono::Duration;
use log::{debug, info, warn};
use octocrab::models::RunId;
use thiserror::Error;

use common::TestsuiteResult;

use self::artifact::Fetcher;

#[derive(Debug, Error)]
pub enum Error {
    #[error("couldn't fetch file creation date")]
    FileCreationDate(#[from] SystemTimeError),
    #[error("error when using github API: {0}")]
    GitHub(#[from] octocrab::Error),
    #[error("error when extracting archive: {0}")]
    Unzipping(#[from] zip::result::ZipError),
    #[error("writing to disk failed: {0}")]
    Disk(#[from] std::io::Error),
}

// FIXME: We probably want to keep the last variation in a cache type or something
pub struct Cache {
    fetcher: Fetcher,
    last_date: SystemTime,
    cached_data: HashSet<TestsuiteResult>,
    cached_runs: HashSet<RunId>,
}

impl Cache {
    pub fn try_new(token: String) -> Result<Cache, octocrab::Error> {
        Ok(Cache {
            fetcher: Fetcher::try_new(token)?,
            last_date: SystemTime::UNIX_EPOCH,
            cached_data: HashSet::new(),
            cached_runs: HashSet::new(),
        })
    }

    fn is_invalidated(&self) -> Result<bool, Error> {
        let now = SystemTime::now();
        let age = now.duration_since(self.last_date)?;

        // UNWRAP: If we have an issue here, this is a programmer error: We want
        // the program to crash as this should never happen
        Ok(age > Duration::hours(24).to_std().unwrap())
    }

    async fn update(&mut self) -> Result<(), Error> {
        let runs = self.fetcher.runs().await?;
        let runs: Vec<RunId> = runs
            .into_iter()
            .filter(|run| !self.cached_runs.contains(run))
            .collect();

        debug!("{:#?}", runs);

        let archives = self.fetcher.result_files(&runs).await?;

        for (run, archive) in archives {
            let bytes = artifact::extract_json(archive)?;

            debug!("{}", String::from_utf8_lossy(&bytes));

            let json = TestsuiteResult::from_bytes(bytes.as_slice());

            match json {
                Ok(json) => {
                    info!(
                        "valid json: {} ({})! Storing in cache",
                        json.name, json.date
                    );
                    self.cached_data.insert(json);
                    self.cached_runs.insert(run);
                }
                Err(e) => warn!("invalid json file... skipping it. Reason: `{}`", e),
            }
        }

        self.last_date = SystemTime::now();

        Ok(())
    }

    pub async fn data(&mut self) -> Result<HashSet<TestsuiteResult>, Error> {
        if self.is_invalidated()? {
            info!("updating cache");
            self.update().await?;
        } else {
            info!("return cached data!");
        }

        // UNWRAP: We can safely unwrap here as our data is updated. If there is
        // no data or none has been fetched/written to disk, then the `update`
        // function will have returned an error which would have been propagated
        // already.
        Ok(self.cached_data.clone())
    }
}
