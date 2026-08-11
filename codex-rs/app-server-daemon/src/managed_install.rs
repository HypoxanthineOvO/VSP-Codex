use std::path::Path;
use std::path::PathBuf;

#[cfg(unix)]
use anyhow::Context;
#[cfg(unix)]
use anyhow::Result;
#[cfg(unix)]
use anyhow::anyhow;
#[cfg(unix)]
use sha2::Digest;
#[cfg(unix)]
use sha2::Sha256;
#[cfg(unix)]
use tokio::fs;
#[cfg(unix)]
use tokio::process::Command;

pub(crate) fn managed_codex_bin(codex_home: &Path) -> PathBuf {
    let current = managed_standalone_current_dir(codex_home);
    for relative_path in expected_managed_codex_relative_paths() {
        let candidate = current.join(relative_path);
        if candidate.is_file() {
            return candidate;
        }
    }
    current.join(default_managed_codex_relative_path())
}

fn managed_standalone_current_dir(codex_home: &Path) -> PathBuf {
    codex_home
        .join("packages")
        .join("standalone")
        .join("current")
}

#[cfg(unix)]
pub(crate) async fn resolved_managed_codex_bin(codex_bin: &Path) -> Result<PathBuf> {
    fs::canonicalize(codex_bin).await.with_context(|| {
        format!(
            "failed to resolve managed Codex binary {}",
            codex_bin.display()
        )
    })
}

#[cfg(unix)]
pub(crate) async fn managed_codex_version(codex_bin: &Path) -> Result<String> {
    let output = Command::new(codex_bin)
        .arg("--version")
        .output()
        .await
        .with_context(|| {
            format!(
                "failed to invoke managed Codex binary {}",
                codex_bin.display()
            )
        })?;
    if !output.status.success() {
        return Err(anyhow!(
            "managed Codex binary {} exited with status {}",
            codex_bin.display(),
            output.status
        ));
    }

    let stdout = String::from_utf8(output.stdout).with_context(|| {
        format!(
            "managed Codex version was not utf-8: {}",
            codex_bin.display()
        )
    })?;
    parse_codex_version(&stdout)
}

#[cfg(unix)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutableIdentity {
    digest: [u8; 32],
}

#[cfg(unix)]
pub(crate) async fn executable_identity(executable: &Path) -> Result<ExecutableIdentity> {
    let bytes = fs::read(executable)
        .await
        .with_context(|| format!("failed to read executable {}", executable.display()))?;
    Ok(executable_identity_from_bytes(&bytes))
}

#[cfg(unix)]
pub(crate) fn executable_identity_from_bytes(bytes: &[u8]) -> ExecutableIdentity {
    ExecutableIdentity {
        digest: Sha256::digest(bytes).into(),
    }
}

pub(crate) fn expected_managed_codex_relative_paths() -> [&'static str; 4] {
    [
        bin_vsp_codex_relative_path(),
        bin_official_codex_relative_path(),
        vsp_codex_file_name(),
        official_codex_file_name(),
    ]
}

fn default_managed_codex_relative_path() -> &'static str {
    bin_vsp_codex_relative_path()
}

fn bin_vsp_codex_relative_path() -> &'static str {
    if cfg!(windows) {
        "bin/vsp-codex.exe"
    } else {
        "bin/vsp-codex"
    }
}

fn bin_official_codex_relative_path() -> &'static str {
    if cfg!(windows) {
        "bin/codex.exe"
    } else {
        "bin/codex"
    }
}

fn vsp_codex_file_name() -> &'static str {
    if cfg!(windows) {
        "vsp-codex.exe"
    } else {
        "vsp-codex"
    }
}

fn official_codex_file_name() -> &'static str {
    if cfg!(windows) { "codex.exe" } else { "codex" }
}

#[cfg(unix)]
fn parse_codex_version(output: &str) -> Result<String> {
    let version = output
        .split_whitespace()
        .nth(1)
        .filter(|version| !version.is_empty())
        .ok_or_else(|| anyhow!("managed Codex version output was malformed"))?;
    Ok(version.to_string())
}

#[cfg(all(test, unix))]
#[path = "managed_install_tests.rs"]
mod tests;
