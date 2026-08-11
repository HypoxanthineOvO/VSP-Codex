//! Reports VSP Codex update state without probing official OpenAI release channels.

use std::path::Path;

use codex_core::config::Config;
use serde::Deserialize;

use super::CheckStatus;
use super::DoctorCheck;

const VERSION_FILE_NAME: &str = "version.json";

/// Builds the update-health row for the current installation.
pub(super) fn updates_check(config: &Config) -> DoctorCheck {
    let mut details = vec![
        format!(
            "check for update on startup: {}",
            config.check_for_update_on_startup
        ),
        "update action: disabled in VSP Codex".to_string(),
    ];
    let version_file = config.codex_home.join(VERSION_FILE_NAME);
    push_cached_version_details(&mut details, &version_file);

    DoctorCheck::new(
        "updates.status",
        "updates",
        CheckStatus::Ok,
        "official update checks are disabled",
    )
    .details(details)
}

fn push_cached_version_details(details: &mut Vec<String>, version_file: &Path) {
    details.push(format!("version cache: {}", version_file.display()));
    match std::fs::read_to_string(version_file) {
        Ok(contents) => match serde_json::from_str::<VersionInfo>(&contents) {
            Ok(info) => {
                details.push(format!("cached latest version: {}", info.latest_version));
                if let Some(last_checked_at) = info.last_checked_at {
                    details.push(format!("last checked at: {last_checked_at}"));
                }
                if let Some(dismissed_version) = info.dismissed_version {
                    details.push(format!("dismissed version: {dismissed_version}"));
                }
            }
            Err(err) => details.push(format!("version cache parse: {err}")),
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            details.push("version cache: missing".to_string());
        }
        Err(err) => details.push(format!("version cache read: {err}")),
    }
}

#[derive(Deserialize)]
struct VersionInfo {
    latest_version: String,
    #[serde(default)]
    last_checked_at: Option<String>,
    #[serde(default)]
    dismissed_version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn update_check_does_not_probe_official_release_channels() {
        let codex_home = TempDir::new().expect("temp home");
        let config = codex_core::config::ConfigBuilder::default()
            .codex_home(codex_home.path().to_path_buf())
            .build()
            .await
            .expect("load config");
        let check = updates_check(&config);

        assert_eq!(check.summary, "official update checks are disabled");
        assert!(
            check
                .details
                .iter()
                .any(|detail| detail == "update action: disabled in VSP Codex")
        );
    }
}
