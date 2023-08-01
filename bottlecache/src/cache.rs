mod artifact;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, SystemTimeError};
use std::{fs, io};

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
    Disk(#[from] io::Error),
}

// FIXME: We probably want to keep the last variation in a cache type or something
/// Cache for CI runs
pub struct Cache {
    /// Location of the cache on the disk
    location: Option<PathBuf>,
    fetcher: Fetcher,
    last_date: SystemTime,
    cached_data: HashSet<TestsuiteResult>,
    cached_runs: HashSet<RunId>,
    /// If we are in `mock` mode, then the cache never invalidates and never updates
    /// It will always return what it currently contains. This implies that you
    /// should give a valid location to this cache instance
    mock: bool,
}

impl Cache {
    fn fill_cache_from_dir(path: &Path) -> Result<HashSet<TestsuiteResult>, Error> {
        // The cache is as restrictive as possible, so if a file is malformed or not completely
        // valid JSON we just skip it
        let existing_cache = fs::read_dir(path)?
            .map(|entry| {
                let path = entry?.path();
                info!("reading from {}...", path.display());

                fs::read_to_string(path)
            })
            .filter_map(Result::ok)
            .map(|s| serde_json::from_str(&s))
            .filter_map(Result::ok)
            .collect::<HashSet<TestsuiteResult>>();

        Ok(existing_cache)
    }

    pub fn try_new(token: String, location: Option<PathBuf>, mock: bool) -> Result<Cache, Error> {
        let cached_data = if let Some(path) = &location {
            if path.exists() {
                Cache::fill_cache_from_dir(path)?
            } else {
                fs::create_dir_all(path)?;
                HashSet::new()
            }
        } else {
            HashSet::new()
        };

        Ok(Cache {
            location,
            fetcher: Fetcher::try_new(token)?,
            last_date: SystemTime::UNIX_EPOCH,
            cached_data,
            cached_runs: HashSet::new(),
            mock,
        })
    }

    fn is_invalidated(&self) -> Result<bool, Error> {
        if self.mock {
            Ok(false) // the cache is always valid
        } else {
            let now = SystemTime::now();
            let age = now.duration_since(self.last_date)?;

            // UNWRAP: If we have an issue here, this is a programmer error: We want
            // the program to crash as this should never happen
            Ok(age > Duration::hours(24).to_std().unwrap())
        }
    }

    fn try_write(&self, json: &TestsuiteResult) -> Result<(), io::Error> {
        match &self.location {
            Some(path) => {
                let path = path
                    .join(format!("{}-{}", &json.name, json.date.format("%Y-%m-%d")))
                    .with_extension("json");
                fs::write(path, serde_json::to_string_pretty(json)?)
            }
            None => Ok(()),
        }
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
                    self.try_write(&json)?;

                    self.cached_data.insert(json);
                    self.cached_runs.insert(run);
                }
                Err(e) => warn!(
                    "invalid json file downloaded from github... skipping it. Reason: `{}`",
                    e
                ),
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
