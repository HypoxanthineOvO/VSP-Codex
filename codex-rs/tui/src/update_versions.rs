use serde::Deserialize;
use sha2::Digest as _;
use sha2::Sha256;
use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use url::Url;

pub const DEFAULT_VSP_RELEASE_MANIFEST_URL: &str =
    "https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/.pipeline/release/vsp-release-manifest.json";
pub const VSP_RELEASE_MANIFEST_URL_ENV: &str = "VSP_CODEX_RELEASE_MANIFEST_URL";
pub const VSP_RELEASE_CHANNEL_ENV: &str = "VSP_CODEX_RELEASE_CHANNEL";

pub(crate) fn is_newer(latest: &str, current: &str) -> Option<bool> {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => Some(l > c),
        _ => None,
    }
}

pub(crate) fn extract_version_from_latest_tag(latest_tag_name: &str) -> anyhow::Result<String> {
    latest_tag_name
        .strip_prefix("rust-v")
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse latest tag name '{latest_tag_name}'"))
}

pub(crate) fn is_source_build_version(version: &str) -> bool {
    parse_version(version).is_some_and(|(major, minor, patch, revision)| {
        major == 0 && minor == 0 && patch == 0 && revision.is_source_revision()
    })
}

pub(crate) fn is_newer_upstream_tag(latest: &str, baseline: &str) -> Option<bool> {
    Some(parse_upstream_tag(latest)? > parse_upstream_tag(baseline)?)
}

pub(crate) fn latest_upstream_tag<'a>(tags: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    tags.into_iter()
        .filter_map(|tag| parse_upstream_tag(tag).map(|version| (tag, version)))
        .max_by_key(|(_, version)| *version)
        .map(|(tag, _)| tag)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum VspReleaseChannel {
    Stable,
    Alpha,
    #[default]
    Lab,
    Other(String),
}

impl VspReleaseChannel {
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "stable" => Self::Stable,
            "alpha" => Self::Alpha,
            "lab" => Self::Lab,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn from_environment_or_default() -> Self {
        std::env::var(VSP_RELEASE_CHANNEL_ENV)
            .ok()
            .map(|value| Self::parse(&value))
            .unwrap_or_default()
    }

    pub fn as_str(&self) -> Cow<'_, str> {
        match self {
            Self::Stable => Cow::Borrowed("stable"),
            Self::Alpha => Cow::Borrowed("alpha"),
            Self::Lab => Cow::Borrowed("lab"),
            Self::Other(value) => Cow::Borrowed(value.as_str()),
        }
    }
}

impl<'de> Deserialize<'de> for VspReleaseChannel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::parse(&value))
    }
}

#[derive(Clone, Debug)]
pub struct VspReleaseQuery {
    pub current_version: String,
    pub channel: VspReleaseChannel,
    pub target: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VspReleaseManifest {
    schema: u64,
    minimum_updater: String,
    upstream_tag: Option<String>,
    releases: Vec<VspReleaseManifestRelease>,
}

#[derive(Clone, Debug, Deserialize)]
struct VspReleaseManifestRelease {
    version: String,
    channel: VspReleaseChannel,
    target: String,
    upstream_tag: Option<String>,
    archive_url: String,
    sha256: String,
    notes_url: String,
}

impl VspReleaseManifest {
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let manifest: Self = serde_json::from_str(json)?;
        if manifest.schema == 0 {
            anyhow::bail!("VSP release manifest schema must be non-zero");
        }
        Ok(manifest)
    }

    pub fn minimum_updater(&self) -> &str {
        &self.minimum_updater
    }

    pub fn upstream_tag(&self) -> Option<&str> {
        self.upstream_tag.as_deref()
    }

    pub fn select(&self, query: VspReleaseQuery) -> Option<VspSelectedRelease> {
        let VspReleaseQuery {
            current_version,
            channel,
            target,
        } = query;
        self.select_latest_compatible(&channel, &target)
            .filter(|release| is_newer(release.version(), &current_version).unwrap_or(false))
    }

    pub fn select_latest_compatible(
        &self,
        channel: &VspReleaseChannel,
        target: &str,
    ) -> Option<VspSelectedRelease> {
        self.releases
            .iter()
            .filter(|release| &release.channel == channel)
            .filter(|release| release.target == target)
            .filter_map(|release| parse_version(&release.version).map(|version| (release, version)))
            .max_by_key(|(_, version)| version.clone())
            .map(|(release, _)| release)
            .and_then(|release| self.selected_release(release))
    }

    pub fn select_installed(&self, query: &VspReleaseQuery) -> Option<VspSelectedRelease> {
        self.releases
            .iter()
            .find(|release| {
                release.version == query.current_version
                    && release.channel == query.channel
                    && release.target == query.target
            })
            .and_then(|release| self.selected_release(release))
    }

    fn selected_release(&self, release: &VspReleaseManifestRelease) -> Option<VspSelectedRelease> {
        let legacy_upstream_tag = (release.upstream_tag.is_none()
            && self
                .releases
                .iter()
                .all(|candidate| candidate.upstream_tag.is_none())
            && self
                .releases
                .iter()
                .all(|candidate| candidate.version == release.version))
        .then_some(self.upstream_tag.as_deref())
        .flatten();
        let mut selected = VspSelectedRelease::new(
            &release.version,
            release.channel.clone(),
            &release.target,
            &release.archive_url,
            &release.sha256,
            &release.notes_url,
        )
        .ok()?;
        selected.upstream_tag = release
            .upstream_tag
            .as_deref()
            .or(legacy_upstream_tag)
            .map(str::to_string);
        Some(selected)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VspSelectedRelease {
    version: String,
    channel: VspReleaseChannel,
    target: String,
    upstream_tag: Option<String>,
    archive_url: String,
    sha256: String,
    notes_url: String,
}

impl VspSelectedRelease {
    pub fn new(
        version: &str,
        channel: VspReleaseChannel,
        target: &str,
        archive_url: &str,
        sha256: &str,
        notes_url: &str,
    ) -> anyhow::Result<Self> {
        if version.trim().is_empty() {
            anyhow::bail!("VSP selected release version is empty");
        }
        if target.trim().is_empty() {
            anyhow::bail!("VSP selected release target is empty");
        }
        if archive_url.trim().is_empty() {
            anyhow::bail!("VSP selected release archive URL is empty");
        }
        if sha256.trim().is_empty() {
            anyhow::bail!("VSP selected release SHA-256 is empty");
        }
        if notes_url.trim().is_empty() {
            anyhow::bail!("VSP selected release notes URL is empty");
        }
        Url::parse(archive_url)?;
        Url::parse(notes_url)?;
        reject_official_update_endpoint(archive_url, "archive URL")?;
        reject_official_update_endpoint(notes_url, "notes URL")?;
        validate_sha256_hex(sha256)?;
        Ok(Self {
            version: version.to_string(),
            channel,
            target: target.to_string(),
            upstream_tag: None,
            archive_url: archive_url.to_string(),
            sha256: sha256.to_string(),
            notes_url: notes_url.to_string(),
        })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn channel(&self) -> VspReleaseChannel {
        self.channel.clone()
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn upstream_tag(&self) -> Option<&str> {
        self.upstream_tag.as_deref()
    }

    pub fn archive_url(&self) -> &str {
        &self.archive_url
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    pub fn notes_url(&self) -> &str {
        &self.notes_url
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VspReleaseSource {
    manifest_url: String,
}

impl VspReleaseSource {
    pub fn from_manifest_url(manifest_url: &str) -> anyhow::Result<Self> {
        Url::parse(manifest_url)?;
        if is_official_update_endpoint(manifest_url) {
            anyhow::bail!(
                "VSP release manifest must not point at official OpenAI update endpoints"
            );
        }
        Ok(Self {
            manifest_url: manifest_url.to_string(),
        })
    }

    pub fn from_environment() -> Option<anyhow::Result<Self>> {
        std::env::var(VSP_RELEASE_MANIFEST_URL_ENV)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|value| Self::from_manifest_url(value.trim()))
    }

    pub fn from_install_context(
        context: &codex_install_context::InstallContext,
    ) -> Option<anyhow::Result<Self>> {
        context
            .package_layout
            .as_ref()
            .and_then(|layout| layout.vsp_release.as_ref())
            .map(|release| Self::from_manifest_url(release.manifest_url.trim()))
    }

    pub fn manifest_url(&self) -> &str {
        &self.manifest_url
    }

    pub fn network_endpoints(&self) -> Vec<String> {
        vec![self.manifest_url.clone()]
    }
}

pub fn vsp_release_channel_from_install_context(
    context: &codex_install_context::InstallContext,
) -> Option<VspReleaseChannel> {
    context
        .package_layout
        .as_ref()
        .and_then(|layout| layout.vsp_release.as_ref())
        .map(|release| VspReleaseChannel::parse(&release.channel))
}

pub fn vsp_release_target_from_install_context(
    context: &codex_install_context::InstallContext,
) -> Option<String> {
    context
        .package_layout
        .as_ref()
        .and_then(|layout| layout.vsp_release.as_ref())
        .and_then(|release| release.target.as_ref())
        .map(|target| target.trim().to_string())
        .filter(|target| !target.is_empty())
}

pub fn vsp_release_source_from_environment_or_install_context(
    context: &codex_install_context::InstallContext,
) -> Option<anyhow::Result<VspReleaseSource>> {
    VspReleaseSource::from_environment().or_else(|| VspReleaseSource::from_install_context(context))
}

pub fn vsp_release_channel_from_environment_or_install_context(
    context: &codex_install_context::InstallContext,
) -> VspReleaseChannel {
    std::env::var(VSP_RELEASE_CHANNEL_ENV)
        .ok()
        .map(|value| VspReleaseChannel::parse(&value))
        .or_else(|| vsp_release_channel_from_install_context(context))
        .unwrap_or_default()
}

pub fn vsp_release_target_from_environment_or_install_context(
    context: &codex_install_context::InstallContext,
) -> String {
    std::env::var("VSP_CODEX_RELEASE_TARGET")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| vsp_release_target_from_install_context(context))
        .unwrap_or_else(current_vsp_release_target)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VspStandaloneUpdateErrorKind {
    ChecksumMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VspStandaloneUpdateError {
    kind: VspStandaloneUpdateErrorKind,
    would_switch_current: bool,
}

impl VspStandaloneUpdateError {
    pub fn kind(&self) -> VspStandaloneUpdateErrorKind {
        self.kind
    }

    pub fn would_switch_current(&self) -> bool {
        self.would_switch_current
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VspStandaloneUpdatePlan {
    release: VspSelectedRelease,
    releases_dir: PathBuf,
    current_path: PathBuf,
}

impl VspStandaloneUpdatePlan {
    pub fn verify_archive_bytes(
        release: &VspSelectedRelease,
        archive_bytes: &[u8],
        releases_dir: &Path,
        current_path: &Path,
    ) -> Result<Self, VspStandaloneUpdateError> {
        let digest = Sha256::digest(archive_bytes);
        let actual = format!("{digest:x}");
        if !actual.eq_ignore_ascii_case(release.sha256().trim()) {
            return Err(VspStandaloneUpdateError {
                kind: VspStandaloneUpdateErrorKind::ChecksumMismatch,
                would_switch_current: false,
            });
        }
        Ok(Self {
            release: release.clone(),
            releases_dir: releases_dir.to_path_buf(),
            current_path: current_path.to_path_buf(),
        })
    }

    pub fn release(&self) -> &VspSelectedRelease {
        &self.release
    }

    pub fn releases_dir(&self) -> &Path {
        &self.releases_dir
    }

    pub fn current_path(&self) -> &Path {
        &self.current_path
    }
}

pub fn current_vsp_release_target() -> String {
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        "arm64" => "aarch64",
        other => other,
    };
    let os = match (std::env::consts::OS, arch) {
        ("macos", _) => "apple-darwin",
        ("linux", "x86_64") => "unknown-linux-gnu",
        ("linux", _) => "unknown-linux-musl",
        ("windows", _) => "pc-windows-msvc",
        (other, _) => other,
    };
    format!("{arch}-{os}")
}

pub fn reject_official_update_endpoint(value: &str, field_name: &str) -> anyhow::Result<()> {
    if is_official_update_endpoint(value) {
        anyhow::bail!(
            "VSP release {field_name} must not point at official OpenAI update endpoints"
        );
    }
    Ok(())
}

fn validate_sha256_hex(value: &str) -> anyhow::Result<()> {
    let digest = value.trim();
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        anyhow::bail!("VSP selected release SHA-256 must be a 64-character hex digest");
    }
    Ok(())
}

fn is_official_update_endpoint(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("api.github.com/repos/openai/codex/releases")
        || lower.contains("github.com/openai/codex/releases")
        || lower.contains("chatgpt.com/codex/install.sh")
        || lower.contains("chatgpt.com/codex/install.ps1")
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum VspPrerelease {
    Alpha(Vec<u64>),
    Stable,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct VspRevision {
    components: Vec<u64>,
    prerelease: VspPrerelease,
}

impl VspRevision {
    fn stable(components: Vec<u64>) -> Self {
        Self {
            components,
            prerelease: VspPrerelease::Stable,
        }
    }

    fn is_source_revision(&self) -> bool {
        self.prerelease == VspPrerelease::Stable
            && self.components.iter().all(|component| *component == 0)
    }
}

fn parse_version(v: &str) -> Option<(u64, u64, u64, VspRevision)> {
    let version = v
        .trim()
        .strip_prefix("rust-v")
        .unwrap_or(v.trim())
        .strip_prefix('v')
        .unwrap_or_else(|| v.trim().strip_prefix("rust-v").unwrap_or(v.trim()));
    if let Some((core, revision)) = version.split_once("-vsp.") {
        let (maj, min, pat) = parse_core_version(core)?;
        return Some((maj, min, pat, parse_vsp_revision(revision)?));
    }
    if let Some((maj, min, pat)) = parse_core_version(version) {
        return Some((maj, min, pat, VspRevision::stable(Vec::new())));
    }
    let revision = parse_vsp_revision(version)?;
    Some((0, 0, 0, revision))
}

fn parse_core_version(core: &str) -> Option<(u64, u64, u64)> {
    let mut iter = core.split('.');
    let maj = iter.next()?.parse::<u64>().ok()?;
    let min = iter.next()?.parse::<u64>().ok()?;
    let pat = iter.next()?.parse::<u64>().ok()?;
    if iter.next().is_some() {
        return None;
    }
    Some((maj, min, pat))
}

fn parse_vsp_revision(value: &str) -> Option<VspRevision> {
    let (components, prerelease) = match value.split_once("-alpha.") {
        Some((components, alpha)) => (
            parse_numeric_components(components)?,
            VspPrerelease::Alpha(parse_numeric_components(alpha)?),
        ),
        None => (parse_numeric_components(value)?, VspPrerelease::Stable),
    };
    Some(VspRevision {
        components,
        prerelease,
    })
}

fn parse_numeric_components(value: &str) -> Option<Vec<u64>> {
    let components = value
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    (!components.is_empty()).then_some(components)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum UpstreamPrerelease {
    Alpha(u64),
    Beta(u64),
    ReleaseCandidate(u64),
    Stable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct UpstreamVersion {
    major: u64,
    minor: u64,
    patch: u64,
    prerelease: UpstreamPrerelease,
}

fn parse_upstream_tag(tag: &str) -> Option<UpstreamVersion> {
    let version = tag.trim().strip_prefix("rust-v")?;
    let (core, prerelease) = match version.split_once('-') {
        Some((core, suffix)) => {
            let (kind, revision) = suffix.split_once('.')?;
            let revision = revision.parse::<u64>().ok()?;
            let prerelease = match kind {
                "alpha" => UpstreamPrerelease::Alpha(revision),
                "beta" => UpstreamPrerelease::Beta(revision),
                "rc" => UpstreamPrerelease::ReleaseCandidate(revision),
                _ => return None,
            };
            (core, prerelease)
        }
        None => (version, UpstreamPrerelease::Stable),
    };
    let mut components = core.split('.');
    let major = components.next()?.parse::<u64>().ok()?;
    let minor = components.next()?.parse::<u64>().ok()?;
    let patch = components.next()?.parse::<u64>().ok()?;
    if components.next().is_some() {
        return None;
    }
    Some(UpstreamVersion {
        major,
        minor,
        patch,
        prerelease,
    })
}

#[cfg(test)]
#[path = "update_versions_vsp_tests.rs"]
mod vsp_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn extracts_version_from_latest_tag() {
        assert_eq!(
            extract_version_from_latest_tag("rust-v1.5.0").expect("failed to parse version"),
            "1.5.0"
        );
    }

    #[test]
    fn latest_tag_without_prefix_is_invalid() {
        assert!(extract_version_from_latest_tag("v1.5.0").is_err());
    }

    #[test]
    fn prerelease_version_is_not_considered_newer() {
        assert_eq!(is_newer("0.11.0-beta.1", "0.11.0"), None);
        assert_eq!(is_newer("1.0.0-rc.1", "1.0.0"), None);
    }

    #[test]
    fn plain_semver_comparisons_work() {
        assert_eq!(is_newer("0.11.1", "0.11.0"), Some(true));
        assert_eq!(is_newer("0.11.0", "0.11.1"), Some(false));
        assert_eq!(is_newer("1.0.0", "0.9.9"), Some(true));
        assert_eq!(is_newer("0.9.9", "1.0.0"), Some(false));
    }

    #[test]
    fn source_build_version_is_not_checked() {
        assert!(is_source_build_version("0.0.0"));
        assert!(!is_source_build_version("0.1.0"));
    }

    #[test]
    fn whitespace_is_ignored() {
        assert_eq!(
            parse_version(" 1.2.3 \n"),
            Some((1, 2, 3, VspRevision::stable(Vec::new())))
        );
        assert_eq!(is_newer(" 1.2.3 ", "1.2.2"), Some(true));
    }

    #[test]
    fn vsp_revision_comparisons_support_patch_segments() {
        assert_eq!(
            parse_version("v0.140.0-vsp.8.2"),
            Some((0, 140, 0, VspRevision::stable(vec![8, 2])))
        );
        assert_eq!(is_newer("0.140.0-vsp.8.2", "0.140.0-vsp.8.1"), Some(true));
        assert_eq!(is_newer("0.145.0-vsp.9", "0.140.0-vsp.8.2"), Some(true));
        assert_eq!(is_newer("0.140.0-vsp.8.2", "0.145.0-vsp.9"), Some(false));
    }

    #[test]
    fn release_version_parser_orders_two_part_prereleases_before_stable() {
        for version in ["9.5", "11.0-alpha.1", "11.0", "11.1", "11.0-alpha.2"] {
            assert!(
                parse_version(version).is_some(),
                "version should parse: {version}"
            );
        }

        for (lower, higher) in [
            ("9.5", "11.0-alpha.1"),
            ("11.0-alpha.1", "11.0"),
            ("11.0", "11.1"),
            ("11.0-alpha.1", "11.0-alpha.2"),
        ] {
            assert_eq!(
                is_newer(higher, lower),
                Some(true),
                "expected {lower} < {higher}"
            );
        }
    }

    #[test]
    fn full_vsp_alpha_version_orders_within_the_release_line() {
        let alpha_1 = "0.145.0-vsp.11.0-alpha.1";
        let alpha_2 = "0.145.0-vsp.11.0-alpha.2";
        let stable = "0.145.0-vsp.11.0";

        assert_eq!(is_newer(alpha_1, "0.145.0-vsp.9.5"), Some(true));
        assert_eq!(is_newer(alpha_2, alpha_1), Some(true));
        assert_eq!(is_newer(stable, alpha_1), Some(true));
        assert_eq!(is_newer(alpha_1, "0.145.0-vsp.10.3"), Some(true));
    }

    #[test]
    fn manifest_keeps_alpha_opt_in_separate_from_the_stable_channel() {
        let manifest = VspReleaseManifest::from_json(
            r#"{
                "schema": 1,
                "minimum_updater": "0.145.0-vsp.9.5",
                "releases": [
                    {
                        "version": "0.145.0-vsp.9.5",
                        "channel": "lab",
                        "target": "x86_64-unknown-linux-gnu",
                        "archive_url": "https://releases.vsp.example/vsp-codex-9.5.tar.gz",
                        "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                        "notes_url": "https://releases.vsp.example/vsp-codex-9.5"
                    },
                    {
                        "version": "0.145.0-vsp.11.0-alpha.1",
                        "channel": "alpha",
                        "target": "x86_64-unknown-linux-gnu",
                        "archive_url": "https://releases.vsp.example/vsp-codex-11.0-alpha.1.tar.gz",
                        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "notes_url": "https://releases.vsp.example/vsp-codex-11.0-alpha.1"
                    }
                ]
            }"#,
        )
        .expect("valid VSP alpha release manifest");

        let target = "x86_64-unknown-linux-gnu".to_string();
        let lab_release = manifest
            .select_latest_compatible(&VspReleaseChannel::Lab, &target)
            .expect("default lab clients should remain on the existing line");
        assert_eq!(lab_release.version(), "0.145.0-vsp.9.5");

        let selected = manifest
            .select(VspReleaseQuery {
                current_version: "0.145.0-vsp.9.5".to_string(),
                channel: VspReleaseChannel::Alpha,
                target: target.clone(),
            })
            .expect("9.5 clients should see the reset alpha");
        assert_eq!(selected.version(), "0.145.0-vsp.11.0-alpha.1");
        assert_eq!(selected.channel(), VspReleaseChannel::Alpha);

        assert_eq!(
            manifest.select(VspReleaseQuery {
                current_version: "0.145.0-vsp.11.0-alpha.1".to_string(),
                channel: VspReleaseChannel::Alpha,
                target,
            }),
            None,
            "the installed alpha must not select itself as an update",
        );
    }

    #[test]
    fn vsp_manifest_selects_feature_revision_after_patch_revision() {
        let manifest = VspReleaseManifest::from_json(
            r#"{
                "schema": 1,
                "minimum_updater": "0.134.0-vsp.6",
                "releases": [
                    {
                        "version": "0.140.0-vsp.8.2",
                        "channel": "lab",
                        "target": "x86_64-unknown-linux-gnu",
                        "archive_url": "https://releases.vsp.example/vsp-codex-8.2.tar.gz",
                        "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "notes_url": "https://releases.vsp.example/vsp-codex-8.2"
                    },
                    {
                        "version": "0.145.0-vsp.9",
                        "channel": "lab",
                        "target": "x86_64-unknown-linux-gnu",
                        "archive_url": "https://releases.vsp.example/vsp-codex-9.tar.gz",
                        "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                        "notes_url": "https://releases.vsp.example/vsp-codex-9"
                    }
                ]
            }"#,
        )
        .expect("valid VSP release manifest");

        let selected = manifest
            .select(VspReleaseQuery {
                current_version: "0.140.0-vsp.8.2".to_string(),
                channel: VspReleaseChannel::Lab,
                target: "x86_64-unknown-linux-gnu".to_string(),
            })
            .expect("vsp.9 should be selected for a vsp.8.2 client");

        assert_eq!(selected.version(), "0.145.0-vsp.9");
    }

    #[test]
    fn c7_m7_vsp_manifest_parses_required_metadata_and_selects_channel_target() {
        let manifest = r#"
        {
          "schema": 1,
          "minimum_updater": "0.4.0",
          "releases": [
            {
              "version": "0.42.0",
              "channel": "stable",
              "target": "x86_64-unknown-linux-musl",
              "archive_url": "https://releases.vsp.example/vsp-codex/0.42.0/vsp-codex-x86_64-unknown-linux-musl.tar.gz",
              "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
              "notes_url": "https://releases.vsp.example/vsp-codex/0.42.0/notes.md"
            },
            {
              "version": "0.43.0",
              "channel": "lab",
              "target": "x86_64-unknown-linux-musl",
              "archive_url": "https://gitlab.vsp.example/releases/vsp-codex/0.43.0/vsp-codex-x86_64-unknown-linux-musl.tar.gz",
              "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
              "notes_url": "https://gitlab.vsp.example/releases/vsp-codex/0.43.0/notes.md"
            },
            {
              "version": "0.42.0",
              "channel": "stable",
              "target": "aarch64-apple-darwin",
              "archive_url": "https://releases.vsp.example/vsp-codex/0.42.0/vsp-codex-aarch64-apple-darwin.tar.gz",
              "sha256": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
              "notes_url": "https://releases.vsp.example/vsp-codex/0.42.0/notes.md"
            }
          ]
        }
        "#;

        let manifest = VspReleaseManifest::from_json(manifest).expect("valid VSP manifest");
        assert_eq!(manifest.minimum_updater(), "0.4.0");

        let selected = manifest
            .select(VspReleaseQuery {
                current_version: "0.41.0".to_string(),
                channel: VspReleaseChannel::Stable,
                target: "x86_64-unknown-linux-musl".to_string(),
            })
            .expect("stable linux release should be selected");

        assert_eq!(selected.version(), "0.42.0");
        assert_eq!(selected.channel(), VspReleaseChannel::Stable);
        assert_eq!(selected.target(), "x86_64-unknown-linux-musl");
        assert_eq!(
            selected.archive_url(),
            "https://releases.vsp.example/vsp-codex/0.42.0/vsp-codex-x86_64-unknown-linux-musl.tar.gz"
        );
        assert_eq!(
            selected.sha256(),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(
            selected.notes_url(),
            "https://releases.vsp.example/vsp-codex/0.42.0/notes.md"
        );
    }

    #[test]
    fn c7_m7_vsp_update_source_never_uses_official_openai_endpoints() {
        let source = VspReleaseSource::from_manifest_url(
            "https://releases.vsp.example/vsp-codex/manifest.json",
        )
        .expect("valid manifest URL");
        let endpoints = source.network_endpoints();
        let endpoints = endpoints.join("\n");

        for forbidden in [
            "api.github.com/repos/openai/codex/releases",
            "github.com/openai/codex/releases",
            "chatgpt.com/codex/install.sh",
            "chatgpt.com/codex/install.ps1",
        ] {
            assert!(
                !endpoints.contains(forbidden),
                "VSP update source must not call official endpoint {forbidden}; endpoints:\n{endpoints}"
            );
        }
        assert!(
            endpoints.contains("releases.vsp.example"),
            "VSP update source should derive network calls from the configured manifest URL; endpoints:\n{endpoints}"
        );
    }

    #[test]
    fn c7_m7_manifest_selected_urls_reject_official_update_endpoints() {
        for forbidden_url in [
            "https://api.github.com/repos/openai/codex/releases/latest",
            "https://github.com/openai/codex/releases/download/rust-v0.42.0/codex.tar.gz",
            "https://chatgpt.com/codex/install.sh",
            "https://chatgpt.com/codex/install.ps1",
        ] {
            let archive_err = VspSelectedRelease::new(
                "0.42.0",
                VspReleaseChannel::Stable,
                "x86_64-unknown-linux-musl",
                forbidden_url,
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "https://releases.vsp.example/vsp-codex/0.42.0/notes.md",
            )
            .expect_err("official archive endpoint must be rejected");
            assert!(
                archive_err
                    .to_string()
                    .contains("official OpenAI update endpoints"),
                "unexpected archive error for {forbidden_url}: {archive_err}"
            );

            let notes_err = VspSelectedRelease::new(
                "0.42.0",
                VspReleaseChannel::Stable,
                "x86_64-unknown-linux-musl",
                "https://releases.vsp.example/vsp-codex/0.42.0/vsp-codex.tar.gz",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                forbidden_url,
            )
            .expect_err("official notes endpoint must be rejected");
            assert!(
                notes_err
                    .to_string()
                    .contains("official OpenAI update endpoints"),
                "unexpected notes error for {forbidden_url}: {notes_err}"
            );
        }
    }

    #[test]
    fn c7_m7_checksum_mismatch_rejects_archive_without_switching_current() {
        let release = VspSelectedRelease::new(
            "0.42.0",
            VspReleaseChannel::Stable,
            "x86_64-unknown-linux-musl",
            "file:///tmp/vsp-codex-0.42.0.tar.gz",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "https://releases.vsp.example/vsp-codex/0.42.0/notes.md",
        )
        .expect("valid selected release");

        let err = VspStandaloneUpdatePlan::verify_archive_bytes(
            &release,
            b"archive bytes with a different digest",
            &PathBuf::from("/tmp/vsp/releases"),
            &PathBuf::from("/tmp/vsp/current"),
        )
        .expect_err("checksum mismatch must reject the archive");

        assert_eq!(err.kind(), VspStandaloneUpdateErrorKind::ChecksumMismatch);
        assert!(
            !err.would_switch_current(),
            "checksum mismatch must not atomically switch the standalone current pointer"
        );
    }
}
