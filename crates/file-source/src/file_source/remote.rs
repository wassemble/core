use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RemoteFileSource(String);

impl RemoteFileSource {
    pub fn new(url: String) -> Self {
        Self(url)
    }

    pub fn url(&self) -> &str {
        &self.0
    }

    pub async fn load(&self) -> Result<Vec<u8>, Error> {
        Ok(reqwest::get(&self.0)
            .await?
            .bytes()
            .await
            .map(|b| b.to_vec())?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to read remote file: {0}")]
    Reqwest(#[from] reqwest::Error),
}
