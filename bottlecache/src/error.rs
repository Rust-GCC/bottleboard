use thiserror::Error;

// FIXME: Move in its own module
#[derive(Debug, Error)]
pub enum Error {
    #[error("error when using github API: {0}")]
    GitHub(#[from] octocrab::Error),
    #[error("error when extracting archive: {0}")]
    Unzipping(#[from] zip::result::ZipError),
    #[error("writing to disk failed: {0}")]
    Disk(#[from] std::io::Error),
}
