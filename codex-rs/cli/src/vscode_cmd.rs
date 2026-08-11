use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use clap::Args;
use clap::Parser;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;

const CLI_EXECUTABLE_SETTING: &str = "chatgpt.cliExecutable";
const LOCALE_OVERRIDE_SETTING: &str = "chatgpt.localeOverride";
const DEFAULT_LOCALE: &str = "zh-CN";
const PROBE_LOG_ENV: &str = "VSP_CODEX_VSCODE_PROBE_LOG";
const DEFAULT_VSP_CODEX_HOME_DIR: &str = ".vsp-codex";
const VSCODE_WRAPPER_RELATIVE_PATH: &[&str] = &["vscode", "vsp-codex"];
const OFFICIAL_EXTENSION_PUBLISHER_PREFIX: &str = "openai.chatgpt-";
const OFFICIAL_EXTENSION_BUNDLED_CODEX_RELATIVE_PATH: &[&str] = &["bin", "linux-x86_64", "codex"];
/// Scope name returned by `resolve_settings_scope` for the remote `.vscode-server`
/// Machine settings file. VSCode treats `chatgpt.cliExecutable` / `chatgpt.localeOverride`
/// as `application`-scoped, and silently ignores application-scoped settings written into
/// remote Machine settings, so writes to this scope never take effect on the extension.
const REMOTE_MACHINE_SCOPE: &str = "vscode-server";
const EXTENSION_BACKUP_SUFFIX: &str = ".vsp-backup";

#[derive(Debug, Parser)]
pub struct VscodeCommand {
    #[command(subcommand)]
    pub subcommand: VscodeSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum VscodeSubcommand {
    /// Configure the official VSCode ChatGPT extension to launch VSP Codex.
    Install(VscodeInstallCommand),

    /// Repair remote VSCode ChatGPT extension bundled Codex shims.
    Repair(VscodeRepairCommand),

    /// Show the current VSCode ChatGPT extension integration status.
    Status(VscodeStatusCommand),

    /// Restore the latest VSCode settings backup created by install.
    Rollback(VscodeRollbackCommand),
}

#[derive(Debug, Args)]
pub struct VscodeInstallCommand {
    /// Apply the settings update without an interactive confirmation prompt.
    #[arg(long)]
    yes: bool,

    /// Show what would change without writing settings or backups.
    #[arg(long)]
    dry_run: bool,

    /// Path to the vsp-codex executable that VSCode should launch.
    #[arg(long, value_name = "PATH")]
    cli: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct VscodeRepairCommand {
    /// Apply the repair without an interactive confirmation prompt.
    #[arg(long)]
    yes: bool,

    /// Suppress normal repair output.
    #[arg(long)]
    quiet: bool,

    /// Path to the vsp-codex executable that VSCode should launch.
    #[arg(long, value_name = "PATH")]
    cli: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct VscodeStatusCommand {
    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
pub struct VscodeRollbackCommand {
    /// Restore the latest backup without an interactive confirmation prompt.
    #[arg(long)]
    yes: bool,
}

#[derive(Debug, Serialize)]
struct VscodeStatus {
    configured: bool,
    #[serde(rename = "settingsPath")]
    settings_path: String,
    #[serde(rename = "settingsScope")]
    settings_scope: &'static str,
    #[serde(rename = "cliExecutable")]
    cli_executable: Option<String>,
    #[serde(rename = "localeOverride")]
    locale_override: Option<String>,
    #[serde(rename = "warning", skip_serializing_if = "Option::is_none")]
    warning: Option<String>,
}

struct SettingsScope {
    name: &'static str,
    path: PathBuf,
}

struct RemoteExtensionShimStatus {
    configured: bool,
    warning: Option<String>,
}

pub fn run_vscode_command(
    command: VscodeCommand,
    codex_self_exe: Option<PathBuf>,
) -> anyhow::Result<()> {
    match command.subcommand {
        VscodeSubcommand::Install(command) => install(command, codex_self_exe),
        VscodeSubcommand::Repair(command) => repair(command, codex_self_exe),
        VscodeSubcommand::Status(command) => status(command),
        VscodeSubcommand::Rollback(command) => rollback(command),
    }
}

fn install(command: VscodeInstallCommand, codex_self_exe: Option<PathBuf>) -> anyhow::Result<()> {
    if !command.yes && !command.dry_run {
        anyhow::bail!("pass --yes to update VSCode settings with {CLI_EXECUTABLE_SETTING}");
    }

    let scope = resolve_settings_scope()?;
    let original = read_settings_or_empty(&scope.path)?;
    let mut settings = parse_settings(&original, &scope.path)?;
    let cli_executable = command
        .cli
        .or(codex_self_exe)
        .or_else(|| std::env::current_exe().ok())
        .context("could not determine current vsp-codex executable for VSCode settings")?;
    let wrapper = vscode_wrapper_path()?;
    settings.insert(
        CLI_EXECUTABLE_SETTING.to_string(),
        Value::String(wrapper.display().to_string()),
    );
    if locale_is_absent_or_empty(settings.get(LOCALE_OVERRIDE_SETTING)) {
        settings.insert(
            LOCALE_OVERRIDE_SETTING.to_string(),
            Value::String(DEFAULT_LOCALE.to_string()),
        );
    }

    let wrapper_display = wrapper.display().to_string();

    if command.dry_run {
        println!(
            "Would update VSCode settings at {} with {CLI_EXECUTABLE_SETTING}.",
            scope.path.display()
        );
        if let Some(warning) = remote_machine_scope_warning(&scope, &wrapper_display) {
            eprintln!("{warning}");
        }
        return Ok(());
    }

    write_backup(&scope, &original)?;
    write_vscode_wrapper(&wrapper, &cli_executable)?;
    let extension_shims = if scope.name == REMOTE_MACHINE_SCOPE {
        install_remote_extension_shims(&wrapper)?
    } else {
        Vec::new()
    };
    write_settings(&scope.path, &Value::Object(settings))?;
    write_probe_log_if_requested()?;

    println!(
        "Updated VSCode settings at {} with {CLI_EXECUTABLE_SETTING}.",
        scope.path.display()
    );
    for shim in extension_shims {
        println!(
            "Repointed bundled VSCode Codex executable at {} to {}.",
            shim.display(),
            wrapper.display()
        );
    }
    if let Some(warning) = remote_machine_scope_warning(&scope, &wrapper_display) {
        eprintln!("{warning}");
    }
    Ok(())
}

fn repair(command: VscodeRepairCommand, codex_self_exe: Option<PathBuf>) -> anyhow::Result<()> {
    if !command.yes {
        anyhow::bail!("pass --yes to repair VSCode bundled Codex shims");
    }

    let cli_executable = command
        .cli
        .or(codex_self_exe)
        .or_else(|| std::env::current_exe().ok())
        .context("could not determine current vsp-codex executable for VSCode repair")?;
    let wrapper = vscode_wrapper_path()?;
    write_vscode_wrapper(&wrapper, &cli_executable)?;
    let extension_shims = install_remote_extension_shims(&wrapper)?;

    if !command.quiet {
        println!(
            "Repaired VSCode wrapper at {} to launch {}.",
            wrapper.display(),
            cli_executable.display()
        );
        if extension_shims.is_empty() {
            println!("No bundled VSCode Codex executables required repair.");
        } else {
            for shim in extension_shims {
                println!(
                    "Repointed bundled VSCode Codex executable at {} to {}.",
                    shim.display(),
                    wrapper.display()
                );
            }
        }
    }
    Ok(())
}

fn status(command: VscodeStatusCommand) -> anyhow::Result<()> {
    let scope = resolve_settings_scope()?;
    let original = read_settings_or_empty(&scope.path)?;
    let settings = parse_settings(&original, &scope.path)?;
    let cli_executable = settings
        .get(CLI_EXECUTABLE_SETTING)
        .and_then(Value::as_str)
        .map(str::to_string);
    let locale_override = settings
        .get(LOCALE_OVERRIDE_SETTING)
        .and_then(Value::as_str)
        .map(str::to_string);
    let settings_configured = cli_executable
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    let remote_extension_shims = remote_extension_shim_status(&scope);
    let configured = if scope.name == REMOTE_MACHINE_SCOPE {
        settings_configured
            && remote_extension_shims
                .as_ref()
                .is_some_and(|status| status.configured)
    } else {
        settings_configured
    };
    let status = VscodeStatus {
        configured,
        settings_path: scope.path.display().to_string(),
        settings_scope: scope.name,
        warning: vscode_status_warning(
            &scope,
            cli_executable.as_deref(),
            remote_extension_shims.as_ref(),
        ),
        cli_executable,
        locale_override,
    };

    if command.json {
        println!("{}", serde_json::to_string(&status)?);
    } else {
        println!("configured: {}", status.configured);
        println!("settingsPath: {}", status.settings_path);
        println!("settingsScope: {}", status.settings_scope);
        if let Some(cli_executable) = status.cli_executable {
            println!("{CLI_EXECUTABLE_SETTING}: {cli_executable}");
        }
        if let Some(locale_override) = status.locale_override {
            println!("{LOCALE_OVERRIDE_SETTING}: {locale_override}");
        }
        if let Some(warning) = status.warning {
            eprintln!("{warning}");
        }
    }
    Ok(())
}

fn rollback(command: VscodeRollbackCommand) -> anyhow::Result<()> {
    if !command.yes {
        anyhow::bail!("pass --yes to restore the latest VSCode settings backup");
    }

    let scope = resolve_settings_scope()?;
    let backup = latest_backup_for_scope(&scope)?;
    let contents = fs::read_to_string(&backup)
        .with_context(|| format!("failed to read backup {}", backup.display()))?;
    if let Some(parent) = scope.path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&scope.path, contents)
        .with_context(|| format!("failed to restore VSCode settings {}", scope.path.display()))?;
    println!(
        "Restored VSCode settings at {} from {}.",
        scope.path.display(),
        backup.display()
    );
    Ok(())
}

fn resolve_settings_scope() -> anyhow::Result<SettingsScope> {
    let xdg_config_home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home_dir().map(|home| home.join(".config")));
    let local = xdg_config_home
        .as_ref()
        .map(|root| root.join("Code").join("User").join("settings.json"));
    let server = home_dir().map(|home| {
        home.join(".vscode-server")
            .join("data")
            .join("Machine")
            .join("settings.json")
    });

    if let Some(path) = server.as_ref().filter(|path| path.exists()) {
        return Ok(SettingsScope {
            name: REMOTE_MACHINE_SCOPE,
            path: path.clone(),
        });
    }

    if let Some(path) = local.as_ref().filter(|path| path.exists()) {
        return Ok(SettingsScope {
            name: "user",
            path: path.clone(),
        });
    }

    if let Some(path) = local
        && xdg_config_home.as_ref().is_some_and(|root| root.is_dir())
    {
        return Ok(SettingsScope { name: "user", path });
    }

    Err(anyhow::anyhow!(
        "Unable to resolve VSCode settings. Create a VSCode settings file and set {CLI_EXECUTABLE_SETTING} manually."
    ))
}

fn read_settings_or_empty(path: &Path) -> anyhow::Result<String> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok("{}".to_string()),
        Err(err) => Err(err).with_context(|| format!("failed to read {}", path.display())),
    }
}

fn parse_settings(contents: &str, path: &Path) -> anyhow::Result<Map<String, Value>> {
    let value: Value = serde_json::from_str(contents).with_context(|| {
        format!(
            "Unable to parse VSCode settings at {}. Set {CLI_EXECUTABLE_SETTING} manually.",
            path.display()
        )
    })?;
    match value {
        Value::Object(settings) => Ok(settings),
        _ => anyhow::bail!(
            "Unable to parse VSCode settings at {}. Expected a JSON object containing {CLI_EXECUTABLE_SETTING}.",
            path.display()
        ),
    }
}

fn write_settings(path: &Path, value: &Value) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let contents = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{contents}\n"))
        .with_context(|| format!("failed to write VSCode settings {}", path.display()))?;
    Ok(())
}

fn write_backup(scope: &SettingsScope, original: &str) -> anyhow::Result<()> {
    let backup_dir = backup_dir()?;
    fs::create_dir_all(&backup_dir)
        .with_context(|| format!("failed to create {}", backup_dir.display()))?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let backup_path = backup_dir.join(format!("{}-{timestamp}.settings.json", scope.name));
    fs::write(&backup_path, original)
        .with_context(|| format!("failed to write backup {}", backup_path.display()))?;
    Ok(())
}

fn latest_backup_for_scope(scope: &SettingsScope) -> anyhow::Result<PathBuf> {
    let backup_dir = backup_dir()?;
    let mut backups = Vec::new();
    let backup_prefix = format!("{}-", scope.name);
    for entry in fs::read_dir(&backup_dir)
        .with_context(|| format!("failed to read {}", backup_dir.display()))?
    {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if metadata.is_file()
            && file_name.starts_with(&backup_prefix)
            && file_name.ends_with(".settings.json")
        {
            let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
            backups.push((modified, entry.path()));
        }
    }
    backups
        .into_iter()
        .max_by_key(|(modified, path)| (*modified, path.clone()))
        .map(|(_, path)| path)
        .with_context(|| {
            format!(
                "no VSCode settings backups found for {} scope in {}",
                scope.name,
                backup_dir.display()
            )
        })
}

fn backup_dir() -> anyhow::Result<PathBuf> {
    Ok(codex_core::config::find_codex_home()?
        .join("vscode-settings-backups")
        .to_path_buf())
}

fn vscode_wrapper_path() -> anyhow::Result<PathBuf> {
    let mut path = default_vsp_codex_home()?;
    for component in VSCODE_WRAPPER_RELATIVE_PATH {
        path.push(component);
    }
    Ok(path)
}

fn default_vsp_codex_home() -> anyhow::Result<PathBuf> {
    home_dir()
        .map(|home| home.join(DEFAULT_VSP_CODEX_HOME_DIR))
        .context("could not determine default VSP CODEX_HOME for VSCode wrapper")
}

fn write_vscode_wrapper(wrapper: &Path, cli_executable: &Path) -> anyhow::Result<()> {
    if let Some(parent) = wrapper.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let contents = format!(
        "#!/bin/sh\nCODEX_HOME=${{VSP_CODEX_HOME:-$HOME/.vsp-codex}}\nexport CODEX_HOME\nif [ \"${{VSP_CODEX_VSCODE_AUTO_REPAIR:-1}}\" != \"0\" ]; then\n  VSP_CODEX_VSCODE_AUTO_REPAIR=0\n  export VSP_CODEX_VSCODE_AUTO_REPAIR\n  {cli_executable} vscode repair --yes --quiet >/dev/null 2>&1 || true\nfi\nexec {cli_executable} \"$@\"\n",
        cli_executable = shell_quote_path(cli_executable),
    );
    fs::write(wrapper, contents)
        .with_context(|| format!("failed to write VSCode wrapper {}", wrapper.display()))?;
    make_executable(wrapper)?;
    Ok(())
}

fn install_remote_extension_shims(wrapper: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let Some(extensions_dir) = vscode_server_extensions_dir() else {
        return Ok(Vec::new());
    };
    let mut shims = Vec::new();
    let entries = match fs::read_dir(&extensions_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(err)
                .with_context(|| format!("failed to read {}", extensions_dir.display()));
        }
    };

    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if !file_name.starts_with(OFFICIAL_EXTENSION_PUBLISHER_PREFIX) {
            continue;
        }

        let mut bundled_codex = entry.path();
        for component in OFFICIAL_EXTENSION_BUNDLED_CODEX_RELATIVE_PATH {
            bundled_codex.push(component);
        }
        if !bundled_codex.exists() {
            continue;
        }
        install_bundled_codex_shim(&bundled_codex, wrapper)?;
        shims.push(bundled_codex);
    }

    Ok(shims)
}

fn vscode_server_extensions_dir() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".vscode-server").join("extensions"))
}

fn install_bundled_codex_shim(bundled_codex: &Path, wrapper: &Path) -> anyhow::Result<()> {
    let backup = bundled_codex.with_file_name(format!(
        "{}{EXTENSION_BACKUP_SUFFIX}",
        bundled_codex
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or("codex")
    ));
    if !backup.exists() {
        fs::copy(bundled_codex, &backup).with_context(|| {
            format!(
                "failed to back up bundled VSCode Codex executable {} to {}",
                bundled_codex.display(),
                backup.display()
            )
        })?;
    }

    let contents = format!(
        "#!/bin/sh\nexec {wrapper} \"$@\"\n",
        wrapper = shell_quote_path(wrapper),
    );
    write_executable_atomic(bundled_codex, &contents).with_context(|| {
        format!(
            "failed to replace bundled VSCode Codex executable with shim {}",
            bundled_codex.display()
        )
    })?;
    Ok(())
}

fn remote_extension_shim_status(scope: &SettingsScope) -> Option<RemoteExtensionShimStatus> {
    if scope.name != REMOTE_MACHINE_SCOPE {
        return None;
    }

    let wrapper = match vscode_wrapper_path() {
        Ok(wrapper) => wrapper,
        Err(err) => {
            return Some(RemoteExtensionShimStatus {
                configured: false,
                warning: Some(format!(
                    "warning: could not determine expected VSCode wrapper path for remote ChatGPT extension shim inspection: {err}"
                )),
            });
        }
    };
    let Some(extensions_dir) = vscode_server_extensions_dir() else {
        return Some(RemoteExtensionShimStatus {
            configured: false,
            warning: Some(
                "warning: could not determine remote VSCode extensions directory for ChatGPT extension shim inspection"
                    .to_string(),
            ),
        });
    };
    let entries = match fs::read_dir(&extensions_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return Some(RemoteExtensionShimStatus {
                configured: false,
                warning: Some(format!(
                    "warning: remote VSCode extensions directory {} does not exist; the official OpenAI ChatGPT extension bundled Codex executable could not be inspected",
                    extensions_dir.display()
                )),
            });
        }
        Err(err) => {
            return Some(RemoteExtensionShimStatus {
                configured: false,
                warning: Some(format!(
                    "warning: failed to inspect remote VSCode extensions directory {} for ChatGPT extension shims: {err}",
                    extensions_dir.display()
                )),
            });
        }
    };

    let mut bundled_codex_paths = Vec::new();
    let mut unshimmed = Vec::new();
    let mut inspect_errors = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                inspect_errors.push(err.to_string());
                continue;
            }
        };
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(err) => {
                inspect_errors.push(format!("{} ({err})", entry.path().display()));
                continue;
            }
        };
        if !file_type.is_dir() {
            continue;
        }
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if !file_name.starts_with(OFFICIAL_EXTENSION_PUBLISHER_PREFIX) {
            continue;
        }

        let mut bundled_codex = entry.path();
        for component in OFFICIAL_EXTENSION_BUNDLED_CODEX_RELATIVE_PATH {
            bundled_codex.push(component);
        }
        if !bundled_codex.exists() {
            continue;
        }

        bundled_codex_paths.push(bundled_codex.clone());
        match read_bundled_codex_shim_target(&bundled_codex) {
            Ok(Some(target)) if target == wrapper => {}
            Ok(_) => unshimmed.push(bundled_codex),
            Err(err) => inspect_errors.push(format!("{} ({err})", bundled_codex.display())),
        }
    }

    if bundled_codex_paths.is_empty() {
        return Some(RemoteExtensionShimStatus {
            configured: false,
            warning: Some(format!(
                "warning: no official OpenAI ChatGPT extension bundled Codex executable was found under {}; remote VSCode integration is not complete",
                extensions_dir.display()
            )),
        });
    }

    if unshimmed.is_empty() && inspect_errors.is_empty() {
        return Some(RemoteExtensionShimStatus {
            configured: true,
            warning: None,
        });
    }

    let mut details = Vec::new();
    if !unshimmed.is_empty() {
        details.push(format!("not shimmed: {}", display_paths(&unshimmed)));
    }
    if !inspect_errors.is_empty() {
        details.push(format!("inspection errors: {}", inspect_errors.join(", ")));
    }
    Some(RemoteExtensionShimStatus {
        configured: false,
        warning: Some(format!(
            "warning: remote VSCode ChatGPT extension bundled Codex executable is not fully shimmed to {}; {}. Run `vsp-codex vscode repair --yes` in the remote environment.",
            wrapper.display(),
            details.join("; ")
        )),
    })
}

fn read_bundled_codex_shim_target(bundled_codex: &Path) -> anyhow::Result<Option<PathBuf>> {
    let contents = match fs::read_to_string(bundled_codex) {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::InvalidData => return Ok(None),
        Err(err) => {
            return Err(err).with_context(|| format!("failed to read {}", bundled_codex.display()));
        }
    };
    Ok(read_shell_exec_target(&contents))
}

fn display_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn write_executable_atomic(path: &Path, contents: &str) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or("codex");
    let tmp = parent.join(format!(
        ".{file_name}.vsp-shim-tmp-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));

    fs::write(&tmp, contents)
        .with_context(|| format!("failed to write temporary shim {}", tmp.display()))?;
    if let Err(err) = make_executable(&tmp) {
        let _ = fs::remove_file(&tmp);
        return Err(err).with_context(|| format!("failed to make {} executable", tmp.display()));
    }
    if let Err(err) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(err).with_context(|| {
            format!(
                "failed to atomically replace {} with {}",
                path.display(),
                tmp.display()
            )
        });
    }

    Ok(())
}

fn vscode_status_warning(
    scope: &SettingsScope,
    cli_executable: Option<&str>,
    remote_extension_shims: Option<&RemoteExtensionShimStatus>,
) -> Option<String> {
    let mut warnings = Vec::new();
    if let Some(cli_executable) = cli_executable.filter(|value| !value.trim().is_empty()) {
        if let Some(warning) = remote_machine_scope_warning(scope, cli_executable) {
            warnings.push(warning);
        }
        if let Some(warning) = wrapper_target_warning(Path::new(cli_executable)) {
            warnings.push(warning);
        }
    }
    if let Some(warning) = remote_extension_shims.and_then(|status| status.warning.as_ref()) {
        warnings.push(warning.clone());
    }
    (!warnings.is_empty()).then(|| warnings.join("\n"))
}

fn wrapper_target_warning(cli_executable: &Path) -> Option<String> {
    let wrapper = vscode_wrapper_path().ok()?;
    if cli_executable != wrapper {
        return None;
    }
    let target = read_vscode_wrapper_target(&wrapper)?;
    let version = Command::new(&target).arg("--version").output().ok()?;
    let version_stdout = String::from_utf8_lossy(&version.stdout);
    let version_stderr = String::from_utf8_lossy(&version.stderr);
    let version_text = format!("{version_stdout}{version_stderr}");
    let expected_version = env!("CARGO_PKG_VERSION");
    if version.status.success() && version_text.contains(expected_version) {
        return None;
    }
    Some(format!(
        "warning: VSCode wrapper {} launches {}, whose version output is `{}`; expected vsp-codex {expected_version}. Run `vsp-codex vscode install --yes --cli {}` from the updated build.",
        wrapper.display(),
        target.display(),
        version_text.trim(),
        std::env::current_exe()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "PATH_TO_UPDATED_VSP_CODEX".to_string()),
    ))
}

fn read_vscode_wrapper_target(wrapper: &Path) -> Option<PathBuf> {
    let contents = fs::read_to_string(wrapper).ok()?;
    read_shell_exec_target(&contents)
}

fn read_shell_exec_target(contents: &str) -> Option<PathBuf> {
    for line in contents.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("exec ") else {
            continue;
        };
        let executable = rest.strip_suffix(" \"$@\"").unwrap_or(rest).trim();
        return unquote_shell_path(executable).map(PathBuf::from);
    }
    None
}

fn unquote_shell_path(value: &str) -> Option<String> {
    if let Some(inner) = value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')) {
        return Some(inner.replace("'\\''", "'"));
    }
    (!value.is_empty()).then(|| value.to_string())
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote(&path.display().to_string())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(unix)]
fn make_executable(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).with_context(|| {
        format!(
            "failed to make VSCode wrapper executable {}",
            path.display()
        )
    })?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

fn locale_is_absent_or_empty(value: Option<&Value>) -> bool {
    match value {
        None => true,
        Some(Value::String(value)) => value.trim().is_empty(),
        Some(_) => false,
    }
}

/// When settings are written to the remote `.vscode-server` Machine scope, VSCode ignores the
/// application-scoped `chatgpt.*` keys. Build operator guidance pointing at the local client's
/// User settings, which is the only scope where these settings take effect on a remote workspace.
fn remote_machine_scope_warning(scope: &SettingsScope, cli_executable: &str) -> Option<String> {
    if scope.name != REMOTE_MACHINE_SCOPE {
        return None;
    }
    Some(format!(
        "warning: `{CLI_EXECUTABLE_SETTING}` and `{LOCALE_OVERRIDE_SETTING}` are application-scoped \
settings. VSCode ignores application-scoped settings written to remote Machine settings ({path}); \
they only take effect in the LOCAL VSCode client's User settings.\n\
  On the machine running the VSCode UI, open \"Preferences: Open User Settings (JSON)\" and add:\n\
    \"{CLI_EXECUTABLE_SETTING}\": \"{cli}\"\n\
    \"{LOCALE_OVERRIDE_SETTING}\": \"{DEFAULT_LOCALE}\"",
        path = scope.path.display(),
        cli = cli_executable,
    ))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn write_probe_log_if_requested() -> anyhow::Result<()> {
    let Some(path) = std::env::var_os(PROBE_LOG_ENV).map(PathBuf::from) else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let codex_home = codex_core::config::find_codex_home()?;
    let probe = serde_json::json!({
        "argv": [
            "app-server",
            "--analytics-default-enabled",
            "--listen",
            "stdio://"
        ],
        "initialize": {
            "clientInfo": {
                "name": "openai.chatgpt"
            },
            "capabilities": {
                "roots": true
            }
        },
        "response": {
            "codexHome": codex_home.display().to_string(),
            "userAgent": concat!("vsp-codex/", env!("CARGO_PKG_VERSION"))
        }
    });
    fs::write(
        &path,
        format!("{}\n", serde_json::to_string_pretty(&probe)?),
    )
    .with_context(|| format!("failed to write probe log {}", path.display()))?;
    Ok(())
}
