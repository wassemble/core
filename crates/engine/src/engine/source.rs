use std::{
    fmt::{self, Display},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Source {
    Local(PathBuf),
    Remote(Url),
}

impl Source {
    pub fn list_local_sources() -> Vec<Source> {
        let allowed_root = Self::allowed_root();
        let mut sources = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&allowed_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    sources.push(Source::Local(path));
                }
            }
        }
        sources
    }

    pub async fn load(self) -> Result<Vec<u8>, Error> {
        let allowed_root = Self::allowed_root();
        match self {
            Self::Local(path) => {
                let path = path.canonicalize()?;
                if !path.starts_with(&allowed_root) {
                    return Err(Error::ForbiddenPath(path));
                }
                Ok(std::fs::read(path)?)
            }
            Self::Remote(url) => {
                if url.scheme() != "https" {
                    return Err(Error::ForbiddenScheme(url.to_string()));
                }
                tracing::info!("Downloading component from URL: {}", url);
                let response = reqwest::get(url).await?;
                tracing::info!("Response status: {}", response.status());
                let bytes = response.bytes().await?;
                tracing::info!("Downloaded {} bytes", bytes.len());
                Ok(bytes.to_vec())
            }
        }
    }

    fn allowed_root() -> PathBuf {
        if let Ok(wasm_path) = std::env::var("WASM_PATH") {
            if let Ok(path) = PathBuf::from(wasm_path).canonicalize() {
                return path;
            }
        }

        if let Ok(cwd) = std::env::current_dir() {
            let wasm_dir = cwd.join("wasm");
            if let Ok(path) = wasm_dir.canonicalize() {
                return path;
            }
        }

        PathBuf::from("./wasm")
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local(path) => write!(f, "{}", path.display()),
            Self::Remote(url) => write!(f, "{url}"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid path: {0}")]
    ForbiddenPath(PathBuf),
    #[error("Forbidden scheme (only https is allowed): {0}")]
    ForbiddenScheme(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
