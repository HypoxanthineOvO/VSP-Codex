use super::*;
use pretty_assertions::assert_eq;

const CURRENT_VSP_VERSION: &str = "0.145.0-vsp.9";
const CURRENT_UPSTREAM_TAG: &str = "rust-v0.145.0-alpha.4";
const LINUX_GNU_TARGET: &str = "x86_64-unknown-linux-gnu";

#[test]
fn built_in_vsp_manifest_uses_gitlab_not_official_openai_endpoints() {
    let source = VspReleaseSource::from_manifest_url(DEFAULT_VSP_RELEASE_MANIFEST_URL)
        .expect("built-in VSP manifest URL should be valid");
    let endpoints = source.network_endpoints().join("\n");

    assert!(
        endpoints.contains("gitlab.vsplab.cn/heyx/Codex"),
        "built-in updater must use the VSP GitLab release source: {endpoints}"
    );
    for forbidden in [
        "api.github.com/repos/openai/codex/releases",
        "github.com/openai/codex/releases",
        "chatgpt.com/codex/install.sh",
        "chatgpt.com/codex/install.ps1",
    ] {
        assert!(
            !endpoints.contains(forbidden),
            "built-in VSP updater must not use official endpoint {forbidden}: {endpoints}"
        );
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn default_channel_and_linux_target_match_published_vsp_manifest() {
    let manifest = release_manifest(CURRENT_UPSTREAM_TAG);
    let selected = manifest
        .select_latest_compatible(&VspReleaseChannel::default(), &current_vsp_release_target())
        .expect("default VSP query should match the published lab GNU/Linux release");

    assert_eq!(selected.channel(), VspReleaseChannel::Lab);
    assert_eq!(selected.target(), LINUX_GNU_TARGET);
    assert_eq!(selected.version(), CURRENT_VSP_VERSION);
}

#[test]
fn upstream_tag_comparison_orders_prereleases_and_stable() {
    assert_eq!(
        is_newer_upstream_tag("rust-v0.145.0-alpha.3", "rust-v0.145.0-alpha.2"),
        Some(true)
    );
    assert_eq!(
        is_newer_upstream_tag("rust-v0.145.0", "rust-v0.145.0-alpha.3"),
        Some(true)
    );
    assert_eq!(
        is_newer_upstream_tag("rust-v0.145.0-alpha.1", "rust-v0.145.0-alpha.2"),
        Some(false)
    );
    assert_eq!(
        is_newer_upstream_tag("rust-v0.144.0", "rust-v0.145.0-alpha.2"),
        Some(false)
    );
}

#[test]
fn latest_upstream_tag_selects_highest_version_from_github_tags() {
    let tags = [
        "rust-v0.145.0-alpha.1",
        "rust-v0.144.1",
        "rust-v0.145.0-alpha.2",
    ];

    assert_eq!(latest_upstream_tag(tags), Some("rust-v0.145.0-alpha.2"));
}

#[test]
fn manifest_keeps_upstream_baseline_and_current_release_metadata() {
    let manifest = release_manifest(CURRENT_UPSTREAM_TAG);
    assert_eq!(manifest.upstream_tag(), Some(CURRENT_UPSTREAM_TAG));

    let query = VspReleaseQuery {
        current_version: CURRENT_VSP_VERSION.to_string(),
        channel: VspReleaseChannel::Lab,
        target: LINUX_GNU_TARGET.to_string(),
    };
    assert_eq!(manifest.select(query.clone()), None);

    let compatible = manifest
        .select_latest_compatible(&query.channel, &query.target)
        .expect("current clients still need release metadata for upstream drift checks");
    assert_eq!(compatible.version(), CURRENT_VSP_VERSION);
    assert_eq!(compatible.channel(), VspReleaseChannel::Lab);
    assert_eq!(compatible.target(), LINUX_GNU_TARGET);
    assert_eq!(compatible.upstream_tag(), Some(CURRENT_UPSTREAM_TAG));
    assert_eq!(manifest.select_installed(&query), Some(compatible));
}

#[test]
fn per_release_upstream_tag_does_not_leak_to_legacy_targets() {
    let manifest = VspReleaseManifest::from_json(
        &serde_json::json!({
            "schema": 1,
            "minimum_updater": "0.134.0-vsp.6",
            "upstream_tag": CURRENT_UPSTREAM_TAG,
            "releases": [
                {
                    "version": CURRENT_VSP_VERSION,
                    "channel": "lab",
                    "target": LINUX_GNU_TARGET,
                    "upstream_tag": CURRENT_UPSTREAM_TAG,
                    "archive_url": "https://gitlab.vsplab.cn/vsp9.tar.gz",
                    "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "notes_url": "https://gitlab.vsplab.cn/vsp9"
                },
                {
                    "version": "0.140.0-vsp.8.2",
                    "channel": "lab",
                    "target": "aarch64-unknown-linux-musl",
                    "archive_url": "https://gitlab.vsplab.cn/vsp8.tar.gz",
                    "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                    "notes_url": "https://gitlab.vsplab.cn/vsp8"
                }
            ]
        })
        .to_string(),
    )
    .expect("mixed-generation manifest should parse");

    let selected = manifest
        .select_latest_compatible(&VspReleaseChannel::Lab, "aarch64-unknown-linux-musl")
        .expect("legacy target should remain selectable");
    assert_eq!(selected.upstream_tag(), None);
}

#[test]
fn latest_release_and_exact_installed_release_are_selected_independently() {
    let manifest = VspReleaseManifest::from_json(
        &serde_json::json!({
            "schema": 1,
            "minimum_updater": "0.134.0-vsp.6",
            "releases": [
                release_json("0.145.0-vsp.9", "rust-v0.145.0-alpha.2", "vsp9"),
                release_json("0.145.0-vsp.10", "rust-v0.145.0-alpha.3", "vsp10")
            ]
        })
        .to_string(),
    )
    .expect("multi-release manifest should parse");
    let installed_query = VspReleaseQuery {
        current_version: "0.145.0-vsp.9".to_string(),
        channel: VspReleaseChannel::Lab,
        target: LINUX_GNU_TARGET.to_string(),
    };

    assert_eq!(
        manifest
            .select_latest_compatible(&installed_query.channel, &installed_query.target)
            .map(|release| (
                release.version().to_string(),
                release.upstream_tag().map(str::to_string),
            )),
        Some((
            "0.145.0-vsp.10".to_string(),
            Some("rust-v0.145.0-alpha.3".to_string()),
        ))
    );
    assert_eq!(
        manifest.select_installed(&installed_query).map(|release| (
            release.version().to_string(),
            release.upstream_tag().map(str::to_string),
        )),
        Some((
            "0.145.0-vsp.9".to_string(),
            Some("rust-v0.145.0-alpha.2".to_string()),
        ))
    );

    let unlisted_query = VspReleaseQuery {
        current_version: "0.145.0-vsp.11-local".to_string(),
        ..installed_query
    };
    assert_eq!(manifest.select_installed(&unlisted_query), None);
}

#[test]
fn ambiguous_legacy_manifest_top_level_tag_is_not_assigned_to_an_installed_release() {
    let manifest = VspReleaseManifest::from_json(
        &serde_json::json!({
            "schema": 1,
            "minimum_updater": "0.134.0-vsp.6",
            "upstream_tag": "rust-v0.145.0-alpha.3",
            "releases": [
                legacy_release_json("0.145.0-vsp.9", "vsp9"),
                legacy_release_json("0.145.0-vsp.10", "vsp10")
            ]
        })
        .to_string(),
    )
    .expect("legacy multi-release manifest should parse");
    let query = VspReleaseQuery {
        current_version: "0.145.0-vsp.9".to_string(),
        channel: VspReleaseChannel::Lab,
        target: LINUX_GNU_TARGET.to_string(),
    };

    assert_eq!(
        manifest
            .select_installed(&query)
            .and_then(|release| release.upstream_tag().map(str::to_string)),
        None
    );
}

fn release_json(version: &str, upstream_tag: &str, slug: &str) -> serde_json::Value {
    let mut release = legacy_release_json(version, slug);
    release["upstream_tag"] = upstream_tag.into();
    release
}

fn legacy_release_json(version: &str, slug: &str) -> serde_json::Value {
    serde_json::json!({
        "version": version,
        "channel": "lab",
        "target": LINUX_GNU_TARGET,
        "archive_url": format!("https://gitlab.vsplab.cn/{slug}.tar.gz"),
        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "notes_url": format!("https://gitlab.vsplab.cn/{slug}")
    })
}

fn release_manifest(upstream_tag: &str) -> VspReleaseManifest {
    VspReleaseManifest::from_json(
        &serde_json::json!({
            "schema": 1,
            "minimum_updater": "0.134.0-vsp.6",
            "upstream_tag": upstream_tag,
            "releases": [{
                "version": CURRENT_VSP_VERSION,
                "channel": "lab",
                "target": LINUX_GNU_TARGET,
                "upstream_tag": upstream_tag,
                "archive_url": "https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.9/downloads/vsp-codex-preview-linux-x64-gnu.tar.gz",
                "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "notes_url": "https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.9"
            }]
        })
        .to_string(),
    )
    .expect("test release manifest should parse")
}
