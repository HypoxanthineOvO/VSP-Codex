use super::*;
use crate::legacy_core::config::ConfigBuilder;
use pretty_assertions::assert_eq;
use tempfile::tempdir;

#[test]
fn legacy_version_cache_defaults_upstream_fields_to_none() {
    let codex_home = tempdir().expect("temp codex home");
    let version_file = codex_home.path().join(VERSION_FILENAME);
    std::fs::write(
        &version_file,
        r#"{
            "latest_version": "0.140.0-vsp.8.2",
            "last_checked_at": "2026-06-30T14:19:05Z",
            "dismissed_version": "0.140.0-vsp.8.2"
        }"#,
    )
    .expect("write legacy version cache");

    let info = read_version_info(&version_file).expect("read legacy version cache");
    assert_eq!(
        (
            info.latest_version,
            info.last_checked_at,
            info.dismissed_version,
            info.latest_upstream_tag,
            info.vsp_baseline_version,
            info.vsp_baseline_upstream_tag,
        ),
        (
            "0.140.0-vsp.8.2".to_string(),
            "2026-06-30T14:19:05Z"
                .parse::<DateTime<Utc>>()
                .expect("valid timestamp"),
            Some("0.140.0-vsp.8.2".to_string()),
            None,
            None,
            None,
        )
    );
}

#[tokio::test]
async fn dismiss_version_creates_cache_file_when_missing() {
    let codex_home = tempdir().expect("temp codex home");
    let config = ConfigBuilder::default()
        .codex_home(codex_home.path().to_path_buf())
        .build()
        .await
        .expect("load config");
    let version_file = version_filepath(&config);

    dismiss_version(&config, "999.0.0")
        .await
        .expect("dismiss version");

    let info = read_version_info(&version_file).expect("read version info");
    assert_eq!(info.last_checked_at, DateTime::<Utc>::UNIX_EPOCH);
    assert_eq!(
        (
            info.latest_version.as_str(),
            info.dismissed_version.as_deref()
        ),
        ("999.0.0", Some("999.0.0"))
    );
}
