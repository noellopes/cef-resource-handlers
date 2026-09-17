use super::download::DownloadRequest;
use anyhow::Result;
use std::path::Path;

const DEFAULT_BOOTSTRAP_VERSION: &str = "5.3.8";
const BOOTSTRAP_VERSION_FILE: &str = "bootstrap.version";

pub fn download_bootstrap_files(target_dir: &Path) -> Result<()> {
    let version = bootstrap_version();
    let version_file = target_dir.join(BOOTSTRAP_VERSION_FILE);
    let is_up_to_date = version_is_up_to_date(&version_file, &version);

    for file_type in ["js", "css"] {
        let file = bootstrap_file_request(&version, file_type, target_dir);

        if !is_up_to_date || !file.target_path.exists() {
            file.download()?;
        }
    }

    if let Err(err) = std::fs::write(&version_file, &version) {
        println!("cargo:warning=Failed to write {version_file:?}: {err}");
    }

    Ok(())
}

fn bootstrap_version() -> String {
    std::env::var("CARGO_PKG_METADATA_BOOTSTRAP_VERSION")
        .unwrap_or_else(|_| DEFAULT_BOOTSTRAP_VERSION.to_string())
}

fn version_is_up_to_date(version_file: &Path, version: &str) -> bool {
    std::fs::read_to_string(version_file)
        .is_ok_and(|current_version| current_version.trim() == version)
}

fn bootstrap_file_request(
    bootstrap_version: &str,
    file_type: &str,
    target_dir: &Path,
) -> DownloadRequest {
    let file_name = format!("bootstrap.min.{file_type}");

    DownloadRequest {
        source_url: format!(
            "https://cdn.jsdelivr.net/npm/bootstrap@{bootstrap_version}/dist/{file_type}/{file_name}"
        ),
        target_path: target_dir.join(&file_name),
    }
}
