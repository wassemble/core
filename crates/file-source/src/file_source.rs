mod local;
mod oci;
mod remote;

use std::path::PathBuf;

pub use local::LocalFileSource;
pub use oci::OciFileSource;
pub use remote::RemoteFileSource;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Clone, Debug)]
pub enum FileSource {
    Local(LocalFileSource),
    Remote(RemoteFileSource),
    Oci(OciFileSource),
}

impl FileSource {
    pub fn local(path: PathBuf) -> Self {
        Self::Local(LocalFileSource::new(path))
    }

    pub fn remote(url: String) -> Self {
        Self::Remote(RemoteFileSource::new(url))
    }

    pub fn oci(url: String) -> Self {
        Self::Oci(OciFileSource::new(url))
    }

    pub fn parse(reference: &str) -> Result<Self, Error> {
        if let Ok(url) = Url::parse(reference) {
            match url.scheme() {
                "https" | "http" => return Ok(Self::remote(reference.to_string())),
                "oci" => return Ok(Self::oci(url.path().to_string())),
                _ => {}
            }
        }

        let path = PathBuf::from(reference);
        if path.starts_with("./") || path.starts_with("../") || path.starts_with("/") {
            return Ok(Self::local(path));
        }

        if Self::is_oci_image_ref(reference) {
            return Ok(Self::oci(reference.to_string()));
        }

        Err(Error::Parse(format!(
            "Unable to identify reference type: {}",
            reference
        )))
    }

    fn is_oci_image_ref(reference: &str) -> bool {
        let pattern =
            regex::Regex::new(r"^[\w\.-]+(?:/[\w\.-]+)+(?::[\w\.-]+)?(?:@sha256:[a-fA-F0-9]+)?$")
                .unwrap();
        pattern.is_match(reference)
    }

    pub async fn load(&self) -> Result<Vec<u8>, Error> {
        Ok(match self {
            FileSource::Local(source) => source.load().await?,
            FileSource::Remote(source) => source.load().await?,
            FileSource::Oci(source) => source.load().await?,
        })
    }

    fn as_str(&self) -> String {
        match self {
            FileSource::Local(source) => source.path().to_string_lossy().into_owned(),
            FileSource::Remote(source) => source.url().to_string(),
            FileSource::Oci(source) => source.url().to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for FileSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for FileSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.as_str())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read local file: {0}")]
    Local(#[from] local::Error),
    #[error("Failed to pull OCI artifact: {0}")]
    Oci(#[from] oci::Error),
    #[error("Failed to read remote file: {0}")]
    Remote(#[from] remote::Error),
    #[error("Failed to parse file source: {0}")]
    Parse(String),
}
