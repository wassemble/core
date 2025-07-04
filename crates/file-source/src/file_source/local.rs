use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocalFileSource(PathBuf);

impl LocalFileSource {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn path(&self) -> &PathBuf {
        &self.0
    }

    pub async fn load(&self) -> Result<Vec<u8>, Error> {
        Ok(fs::read(&self.0).await?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to read local file: {0}")]
    Io(#[from] std::io::Error),
}
