use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};

const DEFAULT_BOOTSTRAP_VERSION: &str = "5.3.8";
const BOOTSTRAP_VERSION_FILE: &str = "bootstrap.version";
const DOWNLOAD_TIMEOUT_SECS: u64 = 30;

fn main() -> Result<()> {
    #[cfg(target_os = "windows")]
    embed_windows_resources()?;

    let build_dir = build_directory()?;
    download_bootstrap_files(&build_dir)
}

#[cfg(target_os = "windows")]
fn embed_windows_resources() -> Result<()> {
    embed_resource::compile("./resources/win/hello.rc", embed_resource::NONE)
        .manifest_required()
        .context("Failed to compile Windows resources")?;

    // Tell Cargo when to re-run this script
    println!("cargo:rerun-if-changed=./resources/win/hello.rc");
    println!("cargo:rerun-if-changed=./resources/win/hello.exe.manifest");
    println!("cargo:rerun-if-changed=Cargo.toml");

    Ok(())
}

fn build_directory() -> Result<PathBuf> {
    let out_dir = std::env::var("OUT_DIR").context("Failed to get OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    for ancestor in out_path.ancestors() {
        if ancestor.file_name() == Some(std::ffi::OsStr::new("build")) {
            return ancestor
                .parent()
                .map(Path::to_path_buf)
                .ok_or_else(|| anyhow!("Could not determine profile directory from OUT_DIR"));
        }
    }

    anyhow::bail!("Could not find 'build' component in OUT_DIR: {out_dir}")
}

fn download_bootstrap_files(target_dir: &Path) -> Result<()> {
    let version = bootstrap_version();
    let version_file = target_dir.join(BOOTSTRAP_VERSION_FILE);
    let is_up_to_date = version_is_up_to_date(&version_file, &version);

    for file_type in ["js", "css"] {
        let file = DownloadRequest::for_bootstrap_asset(&version, file_type, target_dir);

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

struct DownloadRequest {
    source_url: String,
    target_path: PathBuf,
}

impl DownloadRequest {
    fn for_bootstrap_asset(bootstrap_version: &str, file_type: &str, target_dir: &Path) -> Self {
        let file_name = format!("bootstrap.min.{file_type}");

        Self {
            source_url: format!(
                "https://cdn.jsdelivr.net/npm/bootstrap@{bootstrap_version}/dist/{file_type}/{file_name}"
            ),
            target_path: target_dir.join(&file_name),
        }
    }

    fn download(&self) -> Result<()> {
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
