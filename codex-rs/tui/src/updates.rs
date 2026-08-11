#![cfg(not(debug_assertions))]

use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use crate::history_cell::UpdateAvailableHistoryCell;
use crate::history_cell::UpstreamSyncAvailableHistoryCell;
use crate::legacy_core::config::Config;
use crate::npm_registry;
use crate::npm_registry::NpmPackageInfo;
use crate::update_action;
use crate::update_action::UpdateAction;
use crate::update_versions::DEFAULT_VSP_RELEASE_MANIFEST_URL;
use crate::update_versions::VspReleaseChannel;
use crate::update_versions::VspReleaseManifest;
use crate::update_versions::VspReleaseQuery;
use crate::update_versions::VspReleaseSource;
use crate::update_versions::VspSelectedRelease;
use crate::update_versions::extract_version_from_latest_tag;
use crate::update_versions::is_newer;
use crate::update_versions::is_newer_upstream_tag;
use crate::update_versions::is_source_build_version;
use crate::update_versions::latest_upstream_tag;
use crate::update_versions::vsp_release_channel_from_environment_or_install_context;
use crate::update_versions::vsp_release_source_from_environment_or_install_context;
use crate::update_versions::vsp_release_target_from_environment_or_install_context;
use crate::updates_cache::VersionInfo;
use crate::updates_cache::read_version_info;
use crate::updates_cache::version_filepath;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use codex_login::default_client::create_client;
use serde::Deserialize;
use std::path::Path;

use crate::version::CODEX_CLI_VERSION;

pub(crate) use crate::updates_cache::dismiss_version;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UpstreamSyncInfo {
    pub(crate) latest_tag: String,
    pub(crate) baseline_tag: String,
    pub(crate) release_url: String,
}

impl UpstreamSyncInfo {
    fn into_event(self) -> AppEvent {
        AppEvent::InsertHistoryCell(Box::new(UpstreamSyncAvailableHistoryCell::new(
            self.latest_tag,
            self.baseline_tag,
            self.release_url,
        )))
    }
}

#[derive(Clone, Copy)]
enum VspRefreshOutcome {
    Complete,
    Partial,
}

#[derive(Debug)]
struct UpdateCheckResult {
    info: VersionInfo,
    update_action: Option<UpdateAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UpdateNotice {
    latest_version: String,
    update_action: Option<UpdateAction>,
}

impl UpdateNotice {
    fn into_event(self) -> AppEvent {
        AppEvent::InsertHistoryCell(Box::new(UpdateAvailableHistoryCell::new(
            self.latest_version,
            self.update_action,
        )))
    }
}

enum RefreshNotification {
    Silent,
    AppEvents(AppEventSender),
}

pub fn get_upgrade_version(config: &Config) -> Option<String> {
    get_upgrade_version_inner(config, RefreshNotification::Silent)
}

pub(crate) fn get_upgrade_version_with_current_session_notification(
    config: &Config,
    app_event_tx: AppEventSender,
) -> Option<String> {
    get_upgrade_version_inner(config, RefreshNotification::AppEvents(app_event_tx))
}

fn get_upgrade_version_inner(config: &Config, notification: RefreshNotification) -> Option<String> {
    if !config.check_for_update_on_startup || is_source_build_version(CODEX_CLI_VERSION) {
        return None;
    }

    let install_context = codex_install_context::InstallContext::current();
    let vsp_source = vsp_release_source(install_context).and_then(Result::ok);
    if crate::vsp_provider::official_services_disabled() && vsp_source.is_none() {
        return None;
    }
    let action = update_action::get_update_action();
    let version_file = version_filepath(config);
    let info = read_version_info(&version_file).ok();
    let notification = if crate::vsp_provider::official_services_disabled() {
        notification
    } else {
        RefreshNotification::Silent
    };

    if match &info {
        None => true,
        Some(info) => info.last_checked_at < Utc::now() - Duration::hours(20),
    } {
        let previous_info = info.clone();
        tokio::spawn(async move {
            match check_for_update(&version_file, action, vsp_source, install_context).await {
                Ok(result) => {
                    if let RefreshNotification::AppEvents(app_event_tx) = notification {
                        if let Some(notice) = refreshed_update_notice(
                            previous_info.as_ref(),
                            &result,
                            CODEX_CLI_VERSION,
                        ) {
                            app_event_tx.send(notice.into_event());
                        } else if let Some(sync) = refreshed_upstream_sync_notice(
                            previous_info.as_ref(),
                            &result,
                            CODEX_CLI_VERSION,
                        ) {
                            app_event_tx.send(sync.into_event());
                        }
                    }
                }
                Err(err) => tracing::error!("Failed to update version: {err}"),
            }
        });
    }

    info.and_then(|info| {
        if is_newer(&info.latest_version, CODEX_CLI_VERSION).unwrap_or(false) {
            Some(info.latest_version)
        } else {
            None
        }
    })
}

pub(crate) fn get_upstream_sync_available(config: &Config) -> Option<UpstreamSyncInfo> {
    if !config.check_for_update_on_startup
        || is_source_build_version(CODEX_CLI_VERSION)
        || !crate::vsp_provider::official_services_disabled()
    {
        return None;
    }

    let info = read_version_info(&version_filepath(config)).ok()?;
    upstream_sync_info(info, CODEX_CLI_VERSION)
}

fn upstream_sync_info(info: VersionInfo, current_version: &str) -> Option<UpstreamSyncInfo> {
    if is_newer(&info.latest_version, current_version).unwrap_or(false) {
        return None;
    }
    if info.vsp_baseline_version.as_deref() != Some(current_version) {
        return None;
    }
    let latest_tag = info.latest_upstream_tag?;
    let baseline_tag = info.vsp_baseline_upstream_tag?;
    is_newer_upstream_tag(&latest_tag, &baseline_tag)
        .unwrap_or(false)
        .then(|| UpstreamSyncInfo {
            release_url: format!("{UPSTREAM_RELEASE_URL_BASE}/tag/{latest_tag}"),
            latest_tag,
            baseline_tag,
        })
}

fn refreshed_update_notice(
    previous: Option<&VersionInfo>,
    refreshed: &UpdateCheckResult,
    current_version: &str,
) -> Option<UpdateNotice> {
    let latest_version = refreshed.info.latest_version.as_str();
    if is_newer(latest_version, current_version) != Some(true)
        || previous.is_some_and(|previous| {
            is_newer(latest_version, &previous.latest_version) != Some(true)
        })
        || refreshed.info.dismissed_version.as_deref() == Some(latest_version)
    {
        return None;
    }

    let update_action = refreshed
        .update_action
        .clone()
        .filter(|action| action.latest_version() == Some(latest_version));
    Some(UpdateNotice {
        latest_version: latest_version.to_string(),
        update_action,
    })
}

fn refreshed_upstream_sync_notice(
    previous: Option<&VersionInfo>,
    refreshed: &UpdateCheckResult,
    current_version: &str,
) -> Option<UpstreamSyncInfo> {
    let sync = upstream_sync_info(refreshed.info.clone(), current_version)?;
    let already_shown = previous
        .cloned()
        .and_then(|previous| upstream_sync_info(previous, current_version))
        .is_some_and(|previous| previous == sync);
    (!already_shown).then_some(sync)
}

// We use the latest version from the cask if installation is via homebrew - homebrew does not immediately pick up the latest release and can lag behind.
const HOMEBREW_CASK_API_URL: &str = "https://formulae.brew.sh/api/cask/codex.json";
const LATEST_RELEASE_URL: &str = "https://api.github.com/repos/openai/codex/releases/latest";
const UPSTREAM_TAGS_URL: &str = "https://api.github.com/repos/openai/codex/tags?per_page=100";
const UPSTREAM_RELEASE_URL_BASE: &str = "https://github.com/openai/codex/releases";

#[derive(Deserialize, Debug, Clone)]
struct ReleaseInfo {
    tag_name: String,
}

#[derive(Deserialize, Debug, Clone)]
struct TagInfo {
    name: String,
}

#[derive(Deserialize, Debug, Clone)]
struct HomebrewCaskInfo {
    version: String,
}

#[derive(Debug, PartialEq, Eq)]
struct VspManifestInfo {
    latest_release: VspSelectedRelease,
    installed_upstream_tag: Option<String>,
}

async fn check_for_update(
    version_file: &Path,
    action: Option<UpdateAction>,
    vsp_source: Option<VspReleaseSource>,
    install_context: &'static codex_install_context::InstallContext,
) -> anyhow::Result<UpdateCheckResult> {
    let prev_info = read_version_info(version_file).ok();
    let (info, update_action) = if let Some(source) = vsp_source {
        let (manifest_result, upstream_result) = tokio::join!(
            fetch_latest_vsp_manifest_info(&source, install_context),
            fetch_latest_upstream_release_tag(),
        );
        if let (Err(manifest_err), Err(upstream_err)) = (&manifest_result, &upstream_result) {
            return Err(anyhow::anyhow!(
                "VSP manifest and upstream release checks both failed: manifest: {manifest_err}; upstream: {upstream_err}"
            ));
        }
        let refresh_outcome = match (&manifest_result, &upstream_result) {
            (Ok(_), Ok(_)) => VspRefreshOutcome::Complete,
            _ => VspRefreshOutcome::Partial,
        };
        let previous_latest_version = prev_info
            .as_ref()
            .map(|info| info.latest_version.clone())
            .unwrap_or_else(|| CODEX_CLI_VERSION.to_string());
        let (latest_version, vsp_baseline_version, vsp_baseline_upstream_tag, update_action) =
            match manifest_result {
                Ok(manifest) => {
                    let VspManifestInfo {
                        latest_release,
                        installed_upstream_tag,
                    } = manifest;
                    let latest_version = latest_release.version().to_string();
                    let update_action = UpdateAction::from_vsp_manifest_release(
                        install_context,
                        source.manifest_url(),
                        latest_release,
                    )?;
                    let (baseline_version, baseline_tag) = installed_upstream_tag
                        .map(|tag| (Some(CODEX_CLI_VERSION.to_string()), Some(tag)))
                        .unwrap_or_default();
                    (
                        latest_version,
                        baseline_version,
                        baseline_tag,
                        update_action,
                    )
                }
                Err(err) => {
                    tracing::warn!("Failed to refresh VSP release manifest: {err}");
                    let (baseline_version, baseline_tag) = prev_info
                        .as_ref()
                        .filter(|info| {
                            info.vsp_baseline_version.as_deref() == Some(CODEX_CLI_VERSION)
                        })
                        .and_then(|info| {
                            info.vsp_baseline_upstream_tag
                                .clone()
                                .map(|tag| (Some(CODEX_CLI_VERSION.to_string()), Some(tag)))
                        })
                        .unwrap_or_default();
                    (
                        previous_latest_version,
                        baseline_version,
                        baseline_tag,
                        None,
                    )
                }
            };
        let latest_upstream_tag = match upstream_result {
            Ok(tag) => Some(tag),
            Err(err) => {
                tracing::warn!("Failed to refresh official upstream releases: {err}");
                prev_info
                    .as_ref()
                    .and_then(|info| info.latest_upstream_tag.clone())
            }
        };
        (
            VersionInfo {
                latest_version,
                last_checked_at: next_vsp_checked_at(
                    prev_info.as_ref(),
                    refresh_outcome,
                    Utc::now(),
                ),
                dismissed_version: prev_info
                    .as_ref()
                    .and_then(|info| info.dismissed_version.clone()),
                latest_upstream_tag,
                vsp_baseline_version,
                vsp_baseline_upstream_tag,
            },
            update_action,
        )
    } else {
        let latest_version = match action.as_ref() {
            #[cfg(any(not(debug_assertions), test))]
            Some(UpdateAction::VspStandaloneManifest(action)) => action.version().to_string(),
            Some(UpdateAction::BrewUpgrade) => {
                let HomebrewCaskInfo { version } = create_client()
                    .get(HOMEBREW_CASK_API_URL)
                    .send()
                    .await?
                    .error_for_status()?
                    .json::<HomebrewCaskInfo>()
                    .await?;
                version
            }
            Some(UpdateAction::NpmGlobalLatest)
            | Some(UpdateAction::BunGlobalLatest)
            | Some(UpdateAction::PnpmGlobalLatest) => {
                let latest_version = fetch_latest_github_release_version().await?;
                let package_info = create_client()
                    .get(npm_registry::PACKAGE_URL)
                    .send()
                    .await?
                    .error_for_status()?
                    .json::<NpmPackageInfo>()
                    .await?;
                npm_registry::ensure_version_ready(&package_info, &latest_version)?;
                latest_version
            }
            Some(UpdateAction::StandaloneUnix) | Some(UpdateAction::StandaloneWindows) | None => {
                fetch_latest_github_release_version().await?
            }
        };
        (
            VersionInfo {
                latest_version,
                last_checked_at: Utc::now(),
                dismissed_version: prev_info
                    .as_ref()
                    .and_then(|info| info.dismissed_version.clone()),
                latest_upstream_tag: prev_info
                    .as_ref()
                    .and_then(|info| info.latest_upstream_tag.clone()),
                vsp_baseline_version: prev_info
                    .as_ref()
                    .and_then(|info| info.vsp_baseline_version.clone()),
                vsp_baseline_upstream_tag: prev_info
                    .as_ref()
                    .and_then(|info| info.vsp_baseline_upstream_tag.clone()),
            },
            action,
        )
    };

    let json_line = format!("{}\n", serde_json::to_string(&info)?);
    if let Some(parent) = version_file.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(version_file, json_line).await?;
    Ok(UpdateCheckResult {
        info,
        update_action,
    })
}

fn next_vsp_checked_at(
    previous: Option<&VersionInfo>,
    outcome: VspRefreshOutcome,
    now: DateTime<Utc>,
) -> DateTime<Utc> {
    match outcome {
        VspRefreshOutcome::Complete => now,
        VspRefreshOutcome::Partial => previous
            .map(|info| info.last_checked_at)
            .unwrap_or(DateTime::<Utc>::UNIX_EPOCH),
    }
}

async fn fetch_latest_github_release_version() -> anyhow::Result<String> {
    let ReleaseInfo {
        tag_name: latest_tag_name,
    } = create_client()
        .get(LATEST_RELEASE_URL)
        .send()
        .await?
        .error_for_status()?
        .json::<ReleaseInfo>()
        .await?;
    extract_version_from_latest_tag(&latest_tag_name)
}

async fn fetch_latest_upstream_release_tag() -> anyhow::Result<String> {
    let tags = create_client()
        .get(UPSTREAM_TAGS_URL)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<TagInfo>>()
        .await?;
    latest_upstream_tag(tags.iter().map(|tag| tag.name.as_str()))
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("No valid rust-v release tag found in upstream tags"))
}

async fn fetch_latest_vsp_manifest_info(
    source: &VspReleaseSource,
    install_context: &codex_install_context::InstallContext,
) -> anyhow::Result<VspManifestInfo> {
    let manifest = fetch_vsp_manifest(source).await?;
    select_vsp_manifest_info(
        &manifest,
        CODEX_CLI_VERSION,
        vsp_release_channel(install_context),
        vsp_release_target(install_context),
    )
}

fn select_vsp_manifest_info(
    manifest: &VspReleaseManifest,
    current_version: &str,
    channel: VspReleaseChannel,
    target: String,
) -> anyhow::Result<VspManifestInfo> {
    let query = VspReleaseQuery {
        current_version: current_version.to_string(),
        channel,
        target,
    };
    let latest_release = manifest
        .select_latest_compatible(&query.channel, &query.target)
        .ok_or_else(|| {
            anyhow::anyhow!("No compatible VSP release matched the current channel/target")
        })?;
    Ok(VspManifestInfo {
        latest_release,
        installed_upstream_tag: manifest
            .select_installed(&query)
            .and_then(|release| release.upstream_tag().map(str::to_string)),
    })
}

async fn fetch_vsp_manifest(source: &VspReleaseSource) -> anyhow::Result<VspReleaseManifest> {
    let manifest_json = create_client()
        .get(source.manifest_url())
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    VspReleaseManifest::from_json(&manifest_json)
}

/// Returns the latest version to show in a popup, if it should be shown.
/// This respects the user's dismissal choice for the current latest version.
pub fn get_upgrade_version_for_popup(config: &Config) -> Option<String> {
    if !config.check_for_update_on_startup || is_source_build_version(CODEX_CLI_VERSION) {
        return None;
    }
    if crate::vsp_provider::official_services_disabled()
        && vsp_release_source(codex_install_context::InstallContext::current()).is_none()
    {
        return None;
    }

    let version_file = version_filepath(config);
    let latest = get_upgrade_version(config)?;
    // If the user dismissed this exact version previously, do not show the popup.
    if let Ok(info) = read_version_info(&version_file)
        && info.dismissed_version.as_deref() == Some(latest.as_str())
    {
        return None;
    }
    Some(latest)
}

#[cfg(not(debug_assertions))]
pub(crate) async fn select_vsp_update_action_from_environment(
    context: &codex_install_context::InstallContext,
) -> anyhow::Result<Option<UpdateAction>> {
    let Some(source) = vsp_release_source(context).transpose()? else {
        return Ok(None);
    };
    select_vsp_update_action(context, &source).await
}

#[cfg(not(debug_assertions))]
pub(crate) async fn select_vsp_update_action_for_version_from_environment(
    context: &codex_install_context::InstallContext,
    version: &str,
) -> anyhow::Result<Option<UpdateAction>> {
    let Some(source) = vsp_release_source(context).transpose()? else {
        return Ok(None);
    };
    let manifest = fetch_vsp_manifest(&source).await?;
    let query = VspReleaseQuery {
        current_version: version.to_string(),
        channel: vsp_release_channel(context),
        target: vsp_release_target(context),
    };
    let Some(selected) = manifest.select_installed(&query) else {
        return Ok(None);
    };
    UpdateAction::from_vsp_manifest_release(context, source.manifest_url(), selected)
}

#[cfg(not(debug_assertions))]
async fn select_vsp_update_action(
    context: &codex_install_context::InstallContext,
    source: &VspReleaseSource,
) -> anyhow::Result<Option<UpdateAction>> {
    let manifest = fetch_vsp_manifest(source).await?;
    let channel = vsp_release_channel(context);
    let Some(selected) = manifest.select(VspReleaseQuery {
        current_version: CODEX_CLI_VERSION.to_string(),
        channel,
        target: vsp_release_target(context),
    }) else {
        return Ok(None);
    };
    UpdateAction::from_vsp_manifest_release(context, source.manifest_url(), selected)
}

fn vsp_release_source(
    context: &codex_install_context::InstallContext,
) -> Option<anyhow::Result<VspReleaseSource>> {
    vsp_release_source_from_environment_or_install_context(context).or_else(|| {
        crate::vsp_provider::official_services_disabled()
            .then(|| VspReleaseSource::from_manifest_url(DEFAULT_VSP_RELEASE_MANIFEST_URL))
    })
}

fn vsp_release_channel(context: &codex_install_context::InstallContext) -> VspReleaseChannel {
    vsp_release_channel_from_environment_or_install_context(context)
}

fn vsp_release_target(context: &codex_install_context::InstallContext) -> String {
    vsp_release_target_from_environment_or_install_context(context)
}

#[cfg(test)]
#[path = "updates_vsp_tests.rs"]
mod vsp_tests;
