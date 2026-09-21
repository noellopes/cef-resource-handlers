use super::cargo_rerun_if_changed;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

fn manifest_file(resource_file: &Path) -> Result<PathBuf> {
    let resource_contents = std::fs::read_to_string(resource_file)
        .with_context(|| format!("Failed to read Windows resource file {resource_file:?}"))?;

    let resource_dir = resource_file.parent().unwrap_or_else(|| Path::new("."));

    let Some(manifest_path) = resource_contents.lines().find_map(|line| {
        let (_, rest) = line.split_once("RT_MANIFEST")?;
        rest.split('"').nth(1)
    }) else {
        anyhow::bail!("Manifest missing from Windows resource file {resource_file:?}");
    };

    let manifest_file = resource_dir.join(manifest_path);

    std::fs::canonicalize(&manifest_file).with_context(|| {
        format!("Failed to resolve manifest referenced by Windows resource file {resource_file:?}")
    })
}

pub fn embed_windows_resources(resource_file: &Path) -> Result<()> {
    let manifest_file = manifest_file(resource_file)?;
    cargo_rerun_if_changed(&manifest_file);

    embed_resource::compile(resource_file, embed_resource::NONE)
        .manifest_required()
        .context("Failed to compile Windows resources")?;
    cargo_rerun_if_changed(resource_file);

    Ok(())
}
