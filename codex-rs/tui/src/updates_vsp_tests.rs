use super::*;
use chrono::TimeZone;
use pretty_assertions::assert_eq;

const CURRENT_VERSION: &str = "0.145.0-vsp.9";
const LATEST_VERSION: &str = "0.145.0-vsp.10";
const TARGET: &str = "x86_64-unknown-linux-gnu";
const MANIFEST_URL: &str = "https://gitlab.vsplab.cn/vsp/manifest.json";

fn version_info(latest_version: &str) -> VersionInfo {
    VersionInfo {
        latest_version: latest_version.to_string(),
        last_checked_at: Utc
            .with_ymd_and_hms(2026, 7, 1, 0, 0, 0)
            .single()
            .expect("valid timestamp"),
        dismissed_version: None,
        latest_upstream_tag: Some("rust-v0.145.0-alpha.3".to_string()),
        vsp_baseline_version: Some(CURRENT_VERSION.to_string()),
        vsp_baseline_upstream_tag: Some("rust-v0.145.0-alpha.2".to_string()),
    }
}

#[test]
fn upstream_sync_notice_waits_until_the_installed_vsp_is_current() {
    assert_eq!(
        upstream_sync_info(version_info("0.145.0-vsp.10"), "0.145.0-vsp.9"),
        None
    );

    let sync = upstream_sync_info(version_info("0.145.0-vsp.9"), "0.145.0-vsp.9")
        .expect("current VSP should surface upstream drift");
    assert_eq!(
        sync,
        UpstreamSyncInfo {
            latest_tag: "rust-v0.145.0-alpha.3".to_string(),
            baseline_tag: "rust-v0.145.0-alpha.2".to_string(),
            release_url: "https://github.com/openai/codex/releases/tag/rust-v0.145.0-alpha.3"
                .to_string(),
        }
    );
}

#[test]
fn partial_vsp_refresh_keeps_the_previous_retry_timestamp() {
    let previous = version_info("0.145.0-vsp.9");
    let now = Utc
        .with_ymd_and_hms(2026, 7, 10, 0, 0, 0)
        .single()
        .expect("valid timestamp");

    assert_eq!(
        next_vsp_checked_at(Some(&previous), VspRefreshOutcome::Partial, now),
        previous.last_checked_at
    );
    assert_eq!(
        next_vsp_checked_at(None, VspRefreshOutcome::Partial, now),
        DateTime::<Utc>::UNIX_EPOCH
    );
    assert_eq!(
        next_vsp_checked_at(Some(&previous), VspRefreshOutcome::Complete, now),
        now
    );
}

#[test]
fn upstream_sync_notice_requires_a_baseline_proven_for_the_installed_version() {
    let mut info = version_info(CURRENT_VERSION);
    info.vsp_baseline_version = Some("0.140.0-vsp.8.2".to_string());

    assert_eq!(upstream_sync_info(info, CURRENT_VERSION), None);
}

#[test]
fn manifest_info_uses_only_the_exact_installed_release_as_upstream_baseline() {
    let manifest = manifest_with_two_releases();
    let expected_latest = manifest
        .select_latest_compatible(&VspReleaseChannel::Lab, TARGET)
        .expect("latest release");

    assert_eq!(
        select_vsp_manifest_info(
            &manifest,
            CURRENT_VERSION,
            VspReleaseChannel::Lab,
            TARGET.to_string(),
        )
        .expect("manifest info"),
        VspManifestInfo {
            latest_release: expected_latest.clone(),
            installed_upstream_tag: Some("rust-v0.145.0-alpha.2".to_string()),
        }
    );
    assert_eq!(
        select_vsp_manifest_info(
            &manifest,
            "0.145.0-vsp.11-local",
            VspReleaseChannel::Lab,
            TARGET.to_string(),
        )
        .expect("manifest info for unlisted build"),
        VspManifestInfo {
            latest_release: expected_latest,
            installed_upstream_tag: None,
        }
    );
}

#[test]
fn refreshed_release_notifies_the_current_session_with_the_same_version_action() {
    let previous = version_info(CURRENT_VERSION);
    let action = update_action(LATEST_VERSION);
    let refreshed = UpdateCheckResult {
        info: version_info(LATEST_VERSION),
        update_action: Some(action.clone()),
    };

    assert_eq!(
        refreshed_update_notice(Some(&previous), &refreshed, CURRENT_VERSION),
        Some(UpdateNotice {
            latest_version: LATEST_VERSION.to_string(),
            update_action: Some(action),
        })
    );
}

#[test]
fn refreshed_release_never_attaches_an_action_for_a_different_version() {
    let previous = version_info(CURRENT_VERSION);
    let refreshed = UpdateCheckResult {
        info: version_info(LATEST_VERSION),
        update_action: Some(update_action("0.145.0-vsp.11")),
    };

    assert_eq!(
        refreshed_update_notice(Some(&previous), &refreshed, CURRENT_VERSION),
        Some(UpdateNotice {
            latest_version: LATEST_VERSION.to_string(),
            update_action: None,
        })
    );
}

#[test]
fn unchanged_refresh_does_not_duplicate_the_current_session_notice() {
    let previous = version_info(LATEST_VERSION);
    let refreshed = UpdateCheckResult {
        info: version_info(LATEST_VERSION),
        update_action: Some(update_action(LATEST_VERSION)),
    };

    assert_eq!(
        refreshed_update_notice(Some(&previous), &refreshed, CURRENT_VERSION),
        None
    );
}

#[test]
fn refreshed_upstream_tag_notifies_the_current_session_when_vsp_is_current() {
    let mut previous = version_info(CURRENT_VERSION);
    previous.latest_upstream_tag = Some("rust-v0.145.0-alpha.2".to_string());
    let refreshed = UpdateCheckResult {
        info: version_info(CURRENT_VERSION),
        update_action: None,
    };

    assert_eq!(
        refreshed_upstream_sync_notice(Some(&previous), &refreshed, CURRENT_VERSION),
        Some(UpstreamSyncInfo {
            latest_tag: "rust-v0.145.0-alpha.3".to_string(),
            baseline_tag: "rust-v0.145.0-alpha.2".to_string(),
            release_url: "https://github.com/openai/codex/releases/tag/rust-v0.145.0-alpha.3"
                .to_string(),
        })
    );
}

#[test]
fn refreshed_upstream_tag_is_not_repeated_or_shown_ahead_of_a_vsp_update() {
    let current = version_info(CURRENT_VERSION);
    let current_refresh = UpdateCheckResult {
        info: current.clone(),
        update_action: None,
    };
    assert_eq!(
        refreshed_upstream_sync_notice(Some(&current), &current_refresh, CURRENT_VERSION),
        None
    );

    let vsp_update_refresh = UpdateCheckResult {
        info: version_info(LATEST_VERSION),
        update_action: Some(update_action(LATEST_VERSION)),
    };
    assert_eq!(
        refreshed_upstream_sync_notice(Some(&current), &vsp_update_refresh, CURRENT_VERSION),
        None
    );
}

#[test]
fn vsp_current_session_update_notice_snapshot() {
    let event = UpdateNotice {
        latest_version: LATEST_VERSION.to_string(),
        update_action: Some(update_action(LATEST_VERSION)),
    }
    .into_event();
    let AppEvent::InsertHistoryCell(cell) = event else {
        panic!("update notice should use InsertHistoryCell");
    };
    let rendered = cell
        .display_lines(/*width*/ 88)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    insta::assert_snapshot!(rendered);
}

fn manifest_with_two_releases() -> VspReleaseManifest {
    VspReleaseManifest::from_json(
        &serde_json::json!({
            "schema": 1,
            "minimum_updater": "0.134.0-vsp.6",
            "releases": [
                release_json(CURRENT_VERSION, "rust-v0.145.0-alpha.2"),
                release_json(LATEST_VERSION, "rust-v0.145.0-alpha.3")
            ]
        })
        .to_string(),
    )
    .expect("test manifest")
}

fn release_json(version: &str, upstream_tag: &str) -> serde_json::Value {
    serde_json::json!({
        "version": version,
        "channel": "lab",
        "target": TARGET,
        "upstream_tag": upstream_tag,
        "archive_url": format!("https://gitlab.vsplab.cn/vsp/{version}.tar.gz"),
        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "notes_url": format!("https://gitlab.vsplab.cn/vsp/{version}")
    })
}

fn update_action(version: &str) -> UpdateAction {
    let release = VspSelectedRelease::new(
        version,
        VspReleaseChannel::Lab,
        TARGET,
        &format!("https://gitlab.vsplab.cn/vsp/{version}.tar.gz"),
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        &format!("https://gitlab.vsplab.cn/vsp/{version}"),
    )
    .expect("selected release");
    UpdateAction::VspStandaloneManifest(
        crate::update_action::VspStandaloneManifestUpdateAction::new(MANIFEST_URL, release)
            .expect("VSP update action"),
    )
}
