use anyhow::{Context, Result};
use std::path::PathBuf;

const DOWNLOAD_TIMEOUT_SECS: u64 = 30;

pub(crate) struct DownloadRequest {
    pub(crate) source_url: String,
    pub(crate) target_path: PathBuf,
}

impl DownloadRequest {
    pub(crate) fn download(&self) -> Result<()> {
        eprintln!("Downloading {}", self.source_url);

        let config = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(DOWNLOAD_TIMEOUT_SECS)))
            .build();
        let agent: ureq::Agent = config.into();

        let response = agent
            .get(&self.source_url)
            .call()
            .with_context(|| format!("Failed to download '{}'", self.source_url))?;

        let mut reader = response.into_body().into_reader();

        let file = std::fs::File::create(&self.target_path)
            .with_context(|| format!("Failed to create file {:?}", self.target_path))?;
        let mut writer = std::io::BufWriter::new(file);

        std::io::copy(&mut reader, &mut writer)
            .with_context(|| format!("Failed to write response to {:?}", self.target_path))?;

        Ok(())
    }
}
