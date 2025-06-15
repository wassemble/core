use oci_client::{Client, Reference, secrets::RegistryAuth};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OciFileSource(String);

impl OciFileSource {
    pub fn new(url: String) -> Self {
        Self(url)
    }

    pub async fn load(&self) -> Result<Vec<u8>, Error> {
        let client = Client::default();
        let reference: Reference = self.0.parse()?;
        let auth = RegistryAuth::Anonymous;
        let image_data = client
            .pull(&reference, &auth, vec!["application/wasm"])
            .await?;
        Ok(image_data.layers[0].data.clone())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to pull OCI artifact: {0}")]
    OciDistribution(#[from] oci_client::errors::OciDistributionError),
    #[error("Failed to parse OCI reference: {0}")]
    Parse(#[from] oci_client::ParseError),
}
