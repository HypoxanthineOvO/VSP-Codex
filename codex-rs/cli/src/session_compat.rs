use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use clap::ValueEnum;

const SESSION_COMPAT_RESOURCE: &str = "vsp-session-compat.py";
const SESSION_COMPAT_MARKER: &str = "session-compat-v1.json";

#[derive(Debug, Args)]
pub(crate) struct SessionCompatCommand {
    /// Inspect compatibility or migrate historical VSP sessions.
    #[arg(value_enum, default_value_t = SessionCompatAction::Apply)]
    action: SessionCompatAction,

    /// Official Codex home containing the shared sessions.
    #[arg(long)]
    official_home: Option<PathBuf>,

    /// VSP Codex home containing its independent state database.
    #[arg(long)]
    vsp_home: Option<PathBuf>,

    /// Directory in which to create the database backup and migration manifest.
    #[arg(long)]
    backup_root: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum SessionCompatAction {
    Status,
    Apply,
}

impl SessionCompatAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Apply => "apply",
        }
    }
}

pub(crate) fn run(command: SessionCompatCommand) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        let _ = command;
        anyhow::bail!("session compatibility migration is not yet available on Windows");
    }

    #[cfg(not(windows))]
    {
        let script = codex_install_context::InstallContext::current()
            .bundled_resource(SESSION_COMPAT_RESOURCE)
            .context(
                "This VSP Codex install does not include the session compatibility resource. Reinstall or update VSP Codex first.",
            )?;
        let python =
            which::which("python3").context("python3 is required for session migration")?;
        let mut process = std::process::Command::new(python);
        process.arg(script.as_path()).arg(command.action.as_str());
        if let Some(path) = command.official_home {
            process.arg("--official-home").arg(path);
        }
        if let Some(path) = command.vsp_home {
            process.arg("--vsp-home").arg(path);
        }
        if let Some(path) = command.backup_root {
            process.arg("--backup-root").arg(path);
        }
        let status = process
            .status()
            .context("run session compatibility migration")?;
        if !status.success() {
            anyhow::bail!("session compatibility migration failed with status {status}");
        }
        Ok(())
    }
}

pub(crate) fn run_automatic_if_needed(vsp_home: &std::path::Path) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        let _ = vsp_home;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        if vsp_home.join(SESSION_COMPAT_MARKER).is_file()
            || !vsp_home.join("state_5.sqlite").is_file()
        {
            return Ok(());
        }
        let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
            return Ok(());
        };
        let official_home = std::env::var_os("VSP_CODEX_OFFICIAL_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".codex"));
        if !official_home.join("state_5.sqlite").is_file() {
            return Ok(());
        }
        let Some(script) = codex_install_context::InstallContext::current()
            .bundled_resource(SESSION_COMPAT_RESOURCE)
        else {
            return Ok(());
        };
        let python =
            which::which("python3").context("python3 is required for session migration")?;
        let output = std::process::Command::new(python)
            .arg(script.as_path())
            .arg("apply")
            .arg("--official-home")
            .arg(&official_home)
            .arg("--vsp-home")
            .arg(vsp_home)
            .output()
            .context("run automatic session compatibility migration")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("automatic session migration failed: {}", stderr.trim());
        }
        if !vsp_home.join(SESSION_COMPAT_MARKER).is_file() {
            anyhow::bail!("automatic session migration completed without writing its marker");
        }
        Ok(())
    }
}
