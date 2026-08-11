#[cfg(any(not(debug_assertions), test))]
use codex_install_context::InstallContext;
#[cfg(any(not(debug_assertions), test))]
use codex_install_context::InstallMethod;
#[cfg(any(not(debug_assertions), test))]
use codex_install_context::StandalonePlatform;

/// Update action the CLI should perform after the TUI exits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateAction {
    /// Update via `npm install -g @openai/codex@latest`.
    NpmGlobalLatest,
    /// Update via `bun install -g @openai/codex@latest`.
    BunGlobalLatest,
    /// Update via `pnpm add -g @openai/codex@latest`.
    PnpmGlobalLatest,
    /// Update via `brew upgrade codex`.
    BrewUpgrade,
    /// Update via `curl -fsSL https://chatgpt.com/codex/install.sh | CODEX_NON_INTERACTIVE=1 sh`.
    StandaloneUnix,
    /// Update via `$env:CODEX_NON_INTERACTIVE=1; irm https://chatgpt.com/codex/install.ps1 | iex`.
    StandaloneWindows,
    /// Update a VSP standalone install from manifest-selected archive metadata.
    #[cfg(any(not(debug_assertions), test))]
    VspStandaloneManifest(VspStandaloneManifestUpdateAction),
}

#[cfg(any(not(debug_assertions), test))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VspStandaloneManifestUpdateAction {
    manifest_url: String,
    channel: crate::update_versions::VspReleaseChannel,
    target: String,
    version: String,
    archive_url: String,
    sha256: String,
    notes_url: String,
}

#[cfg(any(not(debug_assertions), test))]
impl VspStandaloneManifestUpdateAction {
    pub fn new(
        manifest_url: &str,
        release: crate::update_versions::VspSelectedRelease,
    ) -> anyhow::Result<Self> {
        crate::update_versions::VspReleaseSource::from_manifest_url(manifest_url)?;
        Ok(Self {
            manifest_url: manifest_url.to_string(),
            channel: release.channel(),
            target: release.target().to_string(),
            version: release.version().to_string(),
            archive_url: release.archive_url().to_string(),
            sha256: release.sha256().to_string(),
            notes_url: release.notes_url().to_string(),
        })
    }

    pub fn manifest_url(&self) -> &str {
        &self.manifest_url
    }

    pub fn channel(&self) -> crate::update_versions::VspReleaseChannel {
        self.channel.clone()
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn version(&self) -> &str {
        &self.version
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

impl UpdateAction {
    #[cfg(any(not(debug_assertions), test))]
    pub(crate) fn from_install_context(context: &InstallContext) -> Option<Self> {
        match &context.method {
            InstallMethod::Npm => Some(UpdateAction::NpmGlobalLatest),
            InstallMethod::Bun => Some(UpdateAction::BunGlobalLatest),
            InstallMethod::Pnpm => Some(UpdateAction::PnpmGlobalLatest),
            InstallMethod::Brew => Some(UpdateAction::BrewUpgrade),
            InstallMethod::Standalone { platform, .. } => Some(match platform {
                StandalonePlatform::Unix => UpdateAction::StandaloneUnix,
                StandalonePlatform::Windows => UpdateAction::StandaloneWindows,
            }),
            InstallMethod::Other => None,
        }
    }

    #[cfg(any(not(debug_assertions), test))]
    pub(crate) fn from_vsp_manifest_release(
        context: &InstallContext,
        manifest_url: &str,
        release: crate::update_versions::VspSelectedRelease,
    ) -> anyhow::Result<Option<Self>> {
        if !matches!(context.method, InstallMethod::Standalone { .. }) {
            return Ok(None);
        }
        let action = VspStandaloneManifestUpdateAction::new(manifest_url, release)?;
        Ok(Some(UpdateAction::VspStandaloneManifest(action)))
    }

    /// Returns the list of command-line arguments for invoking the update.
    pub fn command_args(&self) -> (&'static str, Vec<String>) {
        match self {
            UpdateAction::NpmGlobalLatest => (
                "npm",
                vec!["install".into(), "-g".into(), "@openai/codex".into()],
            ),
            UpdateAction::BunGlobalLatest => (
                "bun",
                vec!["install".into(), "-g".into(), "@openai/codex".into()],
            ),
            UpdateAction::PnpmGlobalLatest => (
                "pnpm",
                vec!["add".into(), "-g".into(), "@openai/codex".into()],
            ),
            UpdateAction::BrewUpgrade => (
                "brew",
                vec!["upgrade".into(), "--cask".into(), "codex".into()],
            ),
            UpdateAction::StandaloneUnix => (
                "sh",
                vec![
                    "-c".into(),
                    "curl -fsSL https://chatgpt.com/codex/install.sh | CODEX_NON_INTERACTIVE=1 sh"
                        .into(),
                ],
            ),
            UpdateAction::StandaloneWindows => (
                "powershell",
                vec![
                    "-ExecutionPolicy".into(),
                    "Bypass".into(),
                    "-c".into(),
                    "$env:CODEX_NON_INTERACTIVE=1; irm https://chatgpt.com/codex/install.ps1 | iex"
                        .into(),
                ],
            ),
            #[cfg(any(not(debug_assertions), test))]
            UpdateAction::VspStandaloneManifest(action) => (
                "vsp-codex",
                vec![
                    "update".into(),
                    "--from-vsp-manifest".into(),
                    "--manifest-url".into(),
                    action.manifest_url().into(),
                    "--channel".into(),
                    action.channel().as_str().into_owned(),
                    "--target".into(),
                    action.target().into(),
                    "--version".into(),
                    action.version().into(),
                    "--archive-url".into(),
                    action.archive_url().into(),
                    "--sha256".into(),
                    action.sha256().into(),
                    "--notes-url".into(),
                    action.notes_url().into(),
                ],
            ),
        }
    }

    /// Returns string representation of the command-line arguments for invoking the update.
    pub fn command_str(&self) -> String {
        let (command, args) = self.command_args();
        shlex::try_join(std::iter::once(command).chain(args.iter().map(String::as_str)))
            .unwrap_or_else(|_| format!("{command} {}", args.join(" ")))
    }

    pub fn latest_version(&self) -> Option<&str> {
        match self {
            #[cfg(any(not(debug_assertions), test))]
            Self::VspStandaloneManifest(action) => Some(action.version()),
            _ => None,
        }
    }

    pub fn release_notes_url(&self) -> Option<&str> {
        match self {
            #[cfg(any(not(debug_assertions), test))]
            Self::VspStandaloneManifest(action) => Some(action.notes_url()),
            _ => None,
        }
    }

    pub fn is_vsp_manifest_update(&self) -> bool {
        match self {
            #[cfg(any(not(debug_assertions), test))]
            Self::VspStandaloneManifest(_) => true,
            _ => false,
        }
    }
}

#[cfg(not(debug_assertions))]
pub fn get_update_action() -> Option<UpdateAction> {
    UpdateAction::from_install_context(InstallContext::current())
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use pretty_assertions::assert_eq;

    #[test]
    fn maps_install_context_to_update_action() {
        let native_release_dir =
            AbsolutePathBuf::from_absolute_path(std::env::temp_dir().join("native-release"))
                .expect("temp dir path should be absolute");

        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Other,
                package_layout: None,
            }),
            None
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Npm,
                package_layout: None,
            }),
            Some(UpdateAction::NpmGlobalLatest)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Bun,
                package_layout: None,
            }),
            Some(UpdateAction::BunGlobalLatest)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Pnpm,
                package_layout: None,
            }),
            Some(UpdateAction::PnpmGlobalLatest)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Brew,
                package_layout: None,
            }),
            Some(UpdateAction::BrewUpgrade)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Standalone {
                    platform: StandalonePlatform::Unix,
                    release_dir: native_release_dir.clone(),
                    resources_dir: Some(native_release_dir.join("codex-resources")),
                },
                package_layout: None,
            }),
            Some(UpdateAction::StandaloneUnix)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Standalone {
                    platform: StandalonePlatform::Windows,
                    release_dir: native_release_dir.clone(),
                    resources_dir: Some(native_release_dir.join("codex-resources")),
                },
                package_layout: None,
            }),
            Some(UpdateAction::StandaloneWindows)
        );
    }

    #[test]
    fn standalone_update_commands_rerun_latest_installer() {
        assert_eq!(
            UpdateAction::StandaloneUnix.command_args(),
            (
                "sh",
                vec![
                    "-c".into(),
                    "curl -fsSL https://chatgpt.com/codex/install.sh | CODEX_NON_INTERACTIVE=1 sh"
                        .into()
                ],
            )
        );

        assert_eq!(
            UpdateAction::StandaloneWindows.command_args(),
            (
                "powershell",
                vec![
                    "-ExecutionPolicy".into(),
                    "Bypass".into(),
                    "-c".into(),
                    "$env:CODEX_NON_INTERACTIVE=1; irm https://chatgpt.com/codex/install.ps1 | iex"
                        .into()
                ],
            )
        );
    }

    #[test]
    fn c7_m7_vsp_standalone_update_action_uses_manifest_archive_not_official_installers() {
        let native_release_dir =
            AbsolutePathBuf::from_absolute_path(std::env::temp_dir().join("native-release"))
                .expect("temp dir path should be absolute");
        let standalone_context = InstallContext {
            method: InstallMethod::Standalone {
                platform: StandalonePlatform::Unix,
                release_dir: native_release_dir.clone(),
                resources_dir: Some(native_release_dir.join("codex-resources")),
            },
            package_layout: None,
        };
        let non_standalone_context = InstallContext {
            method: InstallMethod::Other,
            package_layout: None,
        };
        let selected_release = crate::update_versions::VspSelectedRelease::new(
            "0.42.0",
            crate::update_versions::VspReleaseChannel::Stable,
            "x86_64-unknown-linux-musl",
            "https://releases.vsp.example/vsp-codex/0.42.0/vsp-codex-x86_64-unknown-linux-musl.tar.gz",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "https://releases.vsp.example/vsp-codex/0.42.0/notes.md",
        )
        .expect("valid selected release");

        let action = UpdateAction::from_vsp_manifest_release(
            &standalone_context,
            "https://releases.vsp.example/vsp-codex/manifest.json",
            selected_release.clone(),
        )
        .expect("valid VSP update action")
        .expect("standalone install should expose VSP archive updater");
        let command = action.command_str();
        for forbidden in [
            "chatgpt.com/codex/install.sh",
            "chatgpt.com/codex/install.ps1",
            "github.com/openai/codex",
            "@openai/codex",
        ] {
            assert!(
                !command.contains(forbidden),
                "VSP standalone update action must not use official updater endpoint {forbidden}; command: {command}"
            );
        }
        assert!(
            command.contains("releases.vsp.example") || command.contains("manifest"),
            "VSP standalone update action should be driven by the selected manifest archive; command: {command}"
        );

        assert_eq!(
            UpdateAction::from_vsp_manifest_release(
                &non_standalone_context,
                "https://releases.vsp.example/vsp-codex/manifest.json",
                selected_release,
            )
            .expect("valid non-standalone VSP update action"),
            None,
            "non-standalone installs must not expose the standalone release updater"
        );
    }
}
