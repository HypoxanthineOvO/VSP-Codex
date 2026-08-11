use serde_json::Value;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub(crate) fn release_matches_expected_version(release_dir: &Path, expected_version: &str) -> bool {
    let metadata_path = release_dir.join("codex-package.json");
    if metadata_path.is_file()
        && package_metadata_version(&metadata_path).as_deref() != Some(expected_version)
    {
        return false;
    }

    binary_version(&release_binary(release_dir)).as_deref() == Some(expected_version)
}

fn package_metadata_version(metadata_path: &Path) -> Option<String> {
    let metadata: Value =
        serde_json::from_str(&std::fs::read_to_string(metadata_path).ok()?).ok()?;
    metadata.get("version")?.as_str().map(str::to_string)
}

fn release_binary(release_dir: &Path) -> PathBuf {
    let binary_name = if cfg!(windows) {
        "vsp-codex.exe"
    } else {
        "vsp-codex"
    };
    let packaged = release_dir.join("bin").join(binary_name);
    if packaged.is_file() {
        packaged
    } else {
        release_dir.join(binary_name)
    }
}

fn binary_version(binary: &Path) -> Option<String> {
    let output = Command::new(binary).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_binary_version_output(&String::from_utf8(output.stdout).ok()?)
}

fn parse_binary_version_output(output: &str) -> Option<String> {
    output.split_whitespace().next_back().map(str::to_string)
}

#[cfg(test)]
#[path = "vsp_update_validation_tests.rs"]
mod tests;
