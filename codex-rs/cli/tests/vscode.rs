use std::fs;
use std::path::Path;
use std::path::PathBuf;
#[cfg(unix)]
use std::process::Command;
#[cfg(unix)]
use std::thread;
#[cfg(unix)]
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use predicates::prelude::PredicateBooleanExt;
use pretty_assertions::assert_eq;
use serde_json::Value;
use tempfile::TempDir;

fn vsp_codex_command(home: &VscodeTestHome) -> Result<assert_cmd::Command> {
    vsp_codex_command_with_codex_home(home, home.codex_home.path())
}

fn vsp_codex_command_with_codex_home(
    home: &VscodeTestHome,
    codex_home: &Path,
) -> Result<assert_cmd::Command> {
    let mut cmd = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("vsp-codex")?);
    cmd.env("CODEX_HOME", codex_home)
        .env("HOME", home.home.path())
        .env("XDG_CONFIG_HOME", home.xdg_config_home.path())
        .env("PATH", home.path_env())
        .env_remove("VSCODE_PORTABLE")
        .env_remove("VSCODE_AGENT_FOLDER");
    Ok(cmd)
}

struct VscodeTestHome {
    home: TempDir,
    codex_home: TempDir,
    xdg_config_home: TempDir,
    bin_dir: TempDir,
}

impl VscodeTestHome {
    fn new() -> Result<Self> {
        let home = TempDir::new()?;
        let codex_home = TempDir::new()?;
        let xdg_config_home = TempDir::new()?;
        let bin_dir = TempDir::new()?;
        write_fake_code_cli(bin_dir.path())?;
        Ok(Self {
            home,
            codex_home,
            xdg_config_home,
            bin_dir,
        })
    }

    fn path_env(&self) -> String {
        std::env::var_os("PATH").map_or_else(
            || self.bin_dir.path().display().to_string(),
            |path| {
                format!(
                    "{}:{}",
                    self.bin_dir.path().display(),
                    std::env::split_paths(&path)
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(":")
                )
            },
        )
    }

    fn user_settings_path(&self) -> PathBuf {
        self.xdg_config_home
            .path()
            .join("Code")
            .join("User")
            .join("settings.json")
    }

    fn server_settings_path(&self) -> PathBuf {
        self.home
            .path()
            .join(".vscode-server")
            .join("data")
            .join("Machine")
            .join("settings.json")
    }

    fn extension_bundled_codex_path(&self, extension_name: &str) -> PathBuf {
        self.home
            .path()
            .join(".vscode-server")
            .join("extensions")
            .join(extension_name)
            .join("bin")
            .join("linux-x86_64")
            .join("codex")
    }

    fn backup_dir(&self) -> PathBuf {
        self.codex_home.path().join("vscode-settings-backups")
    }
}

fn write_fake_code_cli(bin_dir: &Path) -> Result<()> {
    let code = bin_dir.join("code");
    fs::write(
        &code,
        "#!/bin/sh\nif [ \"$1\" = \"--list-extensions\" ]; then\n  echo openai.chatgpt\n  exit 0\nfi\nif [ \"$1\" = \"--install-extension\" ]; then\n  exit 0\nfi\nexit 0\n",
    )?;
    make_executable(&code)?;
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

fn write_json(path: &Path, json: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, json)?;
    Ok(())
}

fn write_fake_bundled_codex(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, "#!/bin/sh\necho official-codex \"$@\"\n")?;
    make_executable(path)?;
    Ok(())
}

#[cfg(unix)]
fn spawn_sleep_command(path: &Path) -> std::io::Result<std::process::Child> {
    let mut last_error = None;
    for _ in 0..20 {
        match Command::new(path).arg("30").spawn() {
            Ok(child) => return Ok(child),
            Err(err) if err.raw_os_error() == Some(libc::ETXTBSY) => {
                last_error = Some(err);
                thread::sleep(Duration::from_millis(25));
            }
            Err(err) => return Err(err),
        }
    }
    match last_error {
        Some(error) => Err(error),
        None => Err(std::io::Error::other(
            "ETXTBSY retry completed without storing an error",
        )),
    }
}

fn read_json(path: &Path) -> Result<Value> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("missing JSON file {}", path.display()))?;
    Ok(serde_json::from_str(&contents)?)
}

fn setting<'a>(settings: &'a Value, key: &str) -> &'a Value {
    settings
        .get(key)
        .unwrap_or_else(|| panic!("missing setting {key} in {settings:#}"))
}

fn field_path<'a>(value: &'a Value, path: &[&str]) -> &'a Value {
    let mut current = value;
    for field in path {
        current = current
            .get(field)
            .unwrap_or_else(|| panic!("missing field {field} in {value:#}"));
    }
    current
}

#[test]
fn vscode_install_warns_application_scope_ignored_on_remote_machine_settings() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let server_settings_path = home.server_settings_path();
    write_json(
        &server_settings_path,
        r#"{
  "remote.SSH.remotePlatform": {
    "host": "linux"
  }
}"#,
    )?;

    vsp_codex_command(&home)?
        .args(["vscode", "install", "--yes"])
        .assert()
        .success()
        .stderr(predicates::str::contains("application-scoped"))
        .stderr(predicates::str::contains("chatgpt.cliExecutable"))
        .stderr(predicates::str::contains("User settings"));

    Ok(())
}

#[test]
fn vscode_install_remote_scope_repoints_bundled_extension_codex() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let server_settings_path = home.server_settings_path();
    write_json(
        &server_settings_path,
        r#"{
  "remote.SSH.remotePlatform": {
    "host": "linux"
  }
}"#,
    )?;
    let bundled_codex = home.extension_bundled_codex_path("openai.chatgpt-26.609.30741-linux-x64");
    write_fake_bundled_codex(&bundled_codex)?;
    let original = fs::read_to_string(&bundled_codex)?;

    let output = vsp_codex_command(&home)?
        .args(["vscode", "install", "--yes"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output)?;
    assert!(
        stdout.contains("Repointed bundled VSCode Codex executable"),
        "install should report bundled executable shimming: {stdout}"
    );

    let shim = fs::read_to_string(&bundled_codex)?;
    assert!(
        shim.contains("exec") && shim.contains(".vsp-codex") && shim.contains("vscode/vsp-codex"),
        "bundled codex should delegate to the VSP wrapper: {shim}"
    );
    let backup = bundled_codex.with_file_name("codex.vsp-backup");
    assert_eq!(
        fs::read_to_string(&backup)?,
        original,
        "install should keep a backup of the official bundled executable"
    );

    Ok(())
}

#[test]
fn vscode_repair_repoints_bundled_extension_codex_without_settings_backup() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let bundled_codex = home.extension_bundled_codex_path("openai.chatgpt-26.623.41415-linux-x64");
    write_fake_bundled_codex(&bundled_codex)?;
    let original = fs::read_to_string(&bundled_codex)?;

    let output = vsp_codex_command(&home)?
        .args(["vscode", "repair", "--yes"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output)?;
    assert!(
        stdout.contains("Repaired VSCode wrapper"),
        "repair should report wrapper refresh: {stdout}"
    );
    assert!(
        stdout.contains("Repointed bundled VSCode Codex executable"),
        "repair should report bundled executable shimming: {stdout}"
    );

    let shim = fs::read_to_string(&bundled_codex)?;
    assert!(
        shim.contains("exec") && shim.contains(".vsp-codex") && shim.contains("vscode/vsp-codex"),
        "repair should delegate bundled codex to the VSP wrapper: {shim}"
    );
    let backup = bundled_codex.with_file_name("codex.vsp-backup");
    assert_eq!(
        fs::read_to_string(&backup)?,
        original,
        "repair should keep a backup of the official bundled executable"
    );
    assert!(
        !home.backup_dir().exists(),
        "repair should not create VSCode settings backups"
    );

    Ok(())
}

#[test]
fn vscode_repair_quiet_suppresses_output() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let bundled_codex = home.extension_bundled_codex_path("openai.chatgpt-26.623.41415-linux-x64");
    write_fake_bundled_codex(&bundled_codex)?;

    vsp_codex_command(&home)?
        .args(["vscode", "repair", "--yes", "--quiet"])
        .assert()
        .success()
        .stdout(predicates::str::is_empty())
        .stderr(predicates::str::is_empty());

    let shim = fs::read_to_string(&bundled_codex)?;
    assert!(
        shim.contains("vscode/vsp-codex"),
        "quiet repair should still install the bundled executable shim: {shim}"
    );

    Ok(())
}

#[cfg(unix)]
#[test]
fn vscode_install_remote_scope_repoints_running_bundled_elf() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let server_settings_path = home.server_settings_path();
    write_json(
        &server_settings_path,
        r#"{
  "remote.SSH.remotePlatform": {
    "host": "linux"
  }
}"#,
    )?;
    let bundled_codex = home.extension_bundled_codex_path("openai.chatgpt-26.609.30741-linux-x64");
    if let Some(parent) = bundled_codex.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy("/bin/sleep", &bundled_codex)?;
    make_executable(&bundled_codex)?;

    let mut running_codex = spawn_sleep_command(&bundled_codex)?;
    thread::sleep(Duration::from_millis(100));

    let assert_result = vsp_codex_command(&home)?
        .args(["vscode", "install", "--yes"])
        .assert()
        .try_success();

    let _ = running_codex.kill();
    let _ = running_codex.wait();
    assert_result?;

    let shim = fs::read_to_string(&bundled_codex)?;
    assert!(
        shim.contains("exec") && shim.contains(".vsp-codex") && shim.contains("vscode/vsp-codex"),
        "running bundled ELF should be replaced by a VSP shim via rename: {shim}"
    );
    let backup = bundled_codex.with_file_name("codex.vsp-backup");
    assert!(
        fs::metadata(&backup)?.len() > 0,
        "install should keep a backup of the running bundled executable"
    );

    Ok(())
}

#[test]
fn vscode_install_does_not_warn_application_scope_on_local_user_settings() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let settings_path = home.user_settings_path();
    write_json(&settings_path, "{}")?;

    vsp_codex_command(&home)?
        .args(["vscode", "install", "--yes"])
        .assert()
        .success()
        .stderr(predicates::str::contains("application-scoped").not());

    Ok(())
}

#[test]
fn vscode_status_json_includes_application_scope_warning_on_remote_scope() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let server_settings_path = home.server_settings_path();
    write_json(
        &server_settings_path,
        r#"{
  "chatgpt.cliExecutable": "/remote/bin/vsp-codex",
  "chatgpt.localeOverride": "zh-CN",
  "remote.SSH.remotePlatform": {
    "host": "linux"
  }
}"#,
    )?;
    let output = vsp_codex_command(&home)?
        .args(["vscode", "status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status: Value = serde_json::from_slice(&output)?;
    let warning = setting(&status, "warning")
        .as_str()
        .context("warning should be a string")?;
    assert!(
        warning.contains("application-scoped"),
        "status warning should explain the application-scope limitation: {warning}"
    );

    Ok(())
}

#[test]
fn vscode_status_json_warns_when_remote_bundled_codex_is_not_vsp_shim() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let server_settings_path = home.server_settings_path();
    write_json(
        &server_settings_path,
        r#"{
  "chatgpt.cliExecutable": "/remote/bin/vsp-codex",
  "chatgpt.localeOverride": "zh-CN",
  "remote.SSH.remotePlatform": {
    "host": "linux"
  }
}"#,
    )?;
    let bundled_codex = home.extension_bundled_codex_path("openai.chatgpt-26.609.30741-linux-x64");
    write_fake_bundled_codex(&bundled_codex)?;

    let output = vsp_codex_command(&home)?
        .args(["vscode", "status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status: Value = serde_json::from_slice(&output)?;
    let warning = setting(&status, "warning")
        .as_str()
        .context("warning should be a string")?;
    assert!(
        warning.contains("bundled Codex executable"),
        "status warning should identify the bundled executable drift: {warning}"
    );
    assert!(
        warning.contains(&bundled_codex.display().to_string()),
        "status warning should name the stale bundled executable: {warning}"
    );

    Ok(())
}

#[test]
fn vscode_install_writes_cli_executable_locale_and_backup_without_clobbering_settings() -> Result<()>
{
    let home = VscodeTestHome::new()?;
    let settings_path = home.user_settings_path();
    write_json(
        &settings_path,
        r#"{
  "editor.fontSize": 15,
  "chatgpt.localeOverride": "",
  "workbench.colorTheme": "Default Dark Modern"
}"#,
    )?;

    vsp_codex_command(&home)?
        .args(["vscode", "install", "--yes"])
        .assert()
        .success();

    let settings = read_json(&settings_path)?;
    assert_eq!(setting(&settings, "editor.fontSize"), &Value::from(15));
    assert_eq!(
        setting(&settings, "workbench.colorTheme"),
        &Value::from("Default Dark Modern")
    );
    assert_eq!(
        setting(&settings, "chatgpt.localeOverride"),
        &Value::from("zh-CN")
    );

    let cli_executable = setting(&settings, "chatgpt.cliExecutable")
        .as_str()
        .context("chatgpt.cliExecutable should be a string")?;
    assert!(
        cli_executable.ends_with("vsp-codex"),
        "expected cliExecutable to point at vsp-codex, got {cli_executable}"
    );

    let backups = fs::read_dir(home.backup_dir())?.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        backups.len(),
        1,
        "install should create one settings backup"
    );
    let backup = fs::read_to_string(backups[0].path())?;
    assert!(
        backup.contains(r#""editor.fontSize": 15"#),
        "backup should contain original settings: {backup}"
    );
    assert!(
        !backup.contains("chatgpt.cliExecutable"),
        "backup should be captured before install mutates settings: {backup}"
    );

    Ok(())
}

#[test]
fn vscode_install_does_not_overwrite_existing_locale_override() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let settings_path = home.user_settings_path();
    write_json(
        &settings_path,
        r#"{
  "chatgpt.localeOverride": "ja-JP",
  "chatgpt.cliExecutable": "/usr/local/bin/codex"
}"#,
    )?;

    vsp_codex_command(&home)?
        .args(["vscode", "install", "--yes"])
        .assert()
        .success();

    let settings = read_json(&settings_path)?;
    assert_eq!(
        setting(&settings, "chatgpt.localeOverride"),
        &Value::from("ja-JP")
    );
    assert_ne!(
        setting(&settings, "chatgpt.cliExecutable"),
        &Value::from("/usr/local/bin/codex"),
        "install should repoint the official extension at vsp-codex"
    );

    Ok(())
}

#[test]
fn vscode_install_writes_wrapper_that_pins_default_vsp_codex_home() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let settings_path = home.user_settings_path();
    write_json(&settings_path, "{}")?;

    let real_cli = home.home.path().join("bin").join("real-vsp-codex");
    if let Some(parent) = real_cli.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&real_cli, "#!/bin/sh\nexec \"$@\"\n")?;
    make_executable(&real_cli)?;

    let inherited_codex_home = home.home.path().join(".codex");
    fs::create_dir_all(&inherited_codex_home)?;
    vsp_codex_command_with_codex_home(&home, &inherited_codex_home)?
        .args([
            "vscode",
            "install",
            "--yes",
            "--cli",
            &real_cli.display().to_string(),
        ])
        .assert()
        .success();

    let settings = read_json(&settings_path)?;
    let cli_executable = setting(&settings, "chatgpt.cliExecutable")
        .as_str()
        .context("chatgpt.cliExecutable should be a string")?;
    assert_ne!(
        cli_executable,
        real_cli.display().to_string(),
        "VSCode must launch a wrapper, not the real CLI directly, so inherited CODEX_HOME is sanitized"
    );

    let wrapper = fs::read_to_string(cli_executable)
        .with_context(|| format!("wrapper should be readable at {cli_executable}"))?;
    assert!(
        wrapper.contains("CODEX_HOME"),
        "wrapper should assign CODEX_HOME before launching the real CLI: {wrapper}"
    );
    assert!(
        wrapper.contains("CODEX_HOME=${VSP_CODEX_HOME:-$HOME/.vsp-codex}"),
        "wrapper should prefer VSP_CODEX_HOME and fall back to $HOME/.vsp-codex: {wrapper}"
    );
    assert!(
        wrapper.contains("vscode repair --yes --quiet"),
        "wrapper should self-heal newly installed remote extension bundled codex files: {wrapper}"
    );
    assert!(
        wrapper.contains(r#"[ "${VSP_CODEX_VSCODE_AUTO_REPAIR:-1}" != "0" ]"#),
        "wrapper should only self-heal when the recursion guard allows it: {wrapper}"
    );
    assert!(
        wrapper.contains("VSP_CODEX_VSCODE_AUTO_REPAIR=0"),
        "wrapper should guard the self-heal call against recursion: {wrapper}"
    );
    assert!(
        wrapper.contains(".vsp-codex"),
        "wrapper should pin CODEX_HOME to the default $HOME/.vsp-codex: {wrapper}"
    );
    assert!(
        !wrapper.contains(".codex"),
        "wrapper must not preserve the inherited official Codex home: {wrapper}"
    );
    assert!(
        wrapper.contains(&real_cli.display().to_string()),
        "wrapper should exec the real vsp-codex binary path: {wrapper}"
    );

    Ok(())
}

#[cfg(unix)]
#[test]
fn vscode_wrapper_uses_vsp_codex_home_and_guard_when_executed() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let settings_path = home.user_settings_path();
    write_json(&settings_path, "{}")?;

    let real_cli = home.home.path().join("bin").join("real-vsp-codex");
    let log_path = home.home.path().join("wrapper.log");
    if let Some(parent) = real_cli.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &real_cli,
        "#!/bin/sh\nprintf 'CODEX_HOME=%s\\nGUARD=%s\\nARGS=%s\\n' \"$CODEX_HOME\" \"${VSP_CODEX_VSCODE_AUTO_REPAIR-unset}\" \"$*\" >> \"$VSP_CODEX_WRAPPER_LOG\"\n",
    )?;
    make_executable(&real_cli)?;

    vsp_codex_command(&home)?
        .args([
            "vscode",
            "install",
            "--yes",
            "--cli",
            &real_cli.display().to_string(),
        ])
        .assert()
        .success();

    let settings = read_json(&settings_path)?;
    let cli_executable = setting(&settings, "chatgpt.cliExecutable")
        .as_str()
        .context("chatgpt.cliExecutable should be a string")?;
    let override_home = home.home.path().join("override-vsp-codex");
    Command::new(cli_executable)
        .arg("--version")
        .env("HOME", home.home.path())
        .env("CODEX_HOME", home.home.path().join(".codex"))
        .env("VSP_CODEX_HOME", &override_home)
        .env("VSP_CODEX_WRAPPER_LOG", &log_path)
        .status()
        .context("run generated VSCode wrapper")?
        .success()
        .then_some(())
        .context("generated VSCode wrapper should exit successfully")?;

    let log = fs::read_to_string(&log_path)?;
    let expected_home = override_home.display().to_string();
    assert_eq!(
        log.matches(&format!("CODEX_HOME={expected_home}")).count(),
        2,
        "repair and final exec should both use the VSP_CODEX_HOME override: {log}"
    );
    assert_eq!(
        log.matches("GUARD=0").count(),
        2,
        "repair and final exec should both inherit the recursion guard: {log}"
    );
    assert!(
        log.contains("ARGS=vscode repair --yes --quiet"),
        "wrapper should run the guarded quiet repair before exec: {log}"
    );
    assert!(
        log.contains("ARGS=--version"),
        "wrapper should preserve user args for the final exec: {log}"
    );

    Ok(())
}

#[test]
fn vscode_rollback_restores_latest_backup_and_removes_installed_keys_when_absent_before()
-> Result<()> {
    let home = VscodeTestHome::new()?;
    let settings_path = home.user_settings_path();
    write_json(
        &settings_path,
        r#"{
  "files.autoSave": "afterDelay"
}"#,
    )?;

    vsp_codex_command(&home)?
        .args(["vscode", "install", "--yes"])
        .assert()
        .success();
    vsp_codex_command(&home)?
        .args(["vscode", "rollback", "--yes"])
        .assert()
        .success();

    let settings = read_json(&settings_path)?;
    assert_eq!(
        settings,
        serde_json::json!({
            "files.autoSave": "afterDelay"
        })
    );

    Ok(())
}

#[test]
fn vscode_rollback_ignores_newer_server_backup_when_current_scope_is_user() -> Result<()> {
    let user_home = VscodeTestHome::new()?;
    let user_settings_path = user_home.user_settings_path();
    write_json(
        &user_settings_path,
        r#"{
  "files.autoSave": "afterDelay"
}"#,
    )?;

    vsp_codex_command(&user_home)?
        .args(["vscode", "install", "--yes"])
        .assert()
        .success();

    let server_home = VscodeTestHome::new()?;
    let server_settings_path = server_home.server_settings_path();
    write_json(
        &server_settings_path,
        r#"{
  "remote.SSH.remotePlatform": {
    "host": "linux"
  }
}"#,
    )?;

    vsp_codex_command_with_codex_home(&server_home, user_home.codex_home.path())?
        .args(["vscode", "install", "--yes"])
        .assert()
        .success();

    vsp_codex_command(&user_home)?
        .args(["vscode", "rollback", "--yes"])
        .assert()
        .success();

    let settings = read_json(&user_settings_path)?;
    assert_eq!(
        settings,
        serde_json::json!({
            "files.autoSave": "afterDelay"
        }),
        "rollback in user scope must restore the user backup, not the newer server backup"
    );

    Ok(())
}

#[test]
fn vscode_dry_run_and_status_report_without_writing_settings_or_backup() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let settings_path = home.user_settings_path();
    write_json(
        &settings_path,
        r#"{
  "terminal.integrated.defaultProfile.linux": "zsh"
}"#,
    )?;
    let original = fs::read_to_string(&settings_path)?;

    vsp_codex_command(&home)?
        .args(["vscode", "install", "--dry-run"])
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&settings_path)?, original);
    assert!(
        !home.backup_dir().exists(),
        "dry-run should not create backup directory"
    );

    let output = vsp_codex_command(&home)?
        .args(["vscode", "status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status: Value = serde_json::from_slice(&output)?;
    assert_eq!(
        setting(&status, "configured"),
        &Value::from(false),
        "dry-run must not make status report an installed configuration"
    );
    assert_eq!(
        setting(&status, "settingsPath"),
        &Value::from(settings_path.display().to_string())
    );
    assert_eq!(setting(&status, "settingsScope"), &Value::from("user"));

    Ok(())
}

#[test]
fn vscode_install_prefers_server_settings_when_user_and_server_settings_exist() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let user_settings_path = home.user_settings_path();
    let server_settings_path = home.server_settings_path();
    let cli_path = home.home.path().join("bin").join("vsp-codex");
    write_json(
        &user_settings_path,
        r#"{
  "editor.fontSize": 15,
  "chatgpt.cliExecutable": "/local/bin/codex",
  "workbench.colorTheme": "Default Dark Modern"
}"#,
    )?;
    let original_user_settings = fs::read_to_string(&user_settings_path)?;
    write_json(
        &server_settings_path,
        r#"{
  "remote.SSH.remotePlatform": {
    "host": "linux"
  },
  "terminal.integrated.defaultProfile.linux": "zsh"
}"#,
    )?;

    let output = vsp_codex_command(&home)?
        .args([
            "vscode",
            "install",
            "--yes",
            "--cli",
            &cli_path.display().to_string(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output)?;
    assert!(
        stdout.contains(&server_settings_path.display().to_string()),
        "install diagnostic should name the selected server settings path: {stdout}"
    );

    assert_eq!(
        fs::read_to_string(&user_settings_path)?,
        original_user_settings,
        "install must not mutate local user settings when server Machine settings also exist"
    );
    let settings = read_json(&server_settings_path)?;
    assert_eq!(
        setting(&settings, "terminal.integrated.defaultProfile.linux"),
        &Value::from("zsh")
    );
    assert!(
        setting(&settings, "chatgpt.cliExecutable")
            .as_str()
            .is_some_and(|value| !value.trim().is_empty()),
        "install should configure a non-empty VSCode launcher"
    );
    assert_eq!(
        setting(&settings, "chatgpt.localeOverride"),
        &Value::from("zh-CN")
    );

    Ok(())
}

#[test]
fn vscode_status_json_prefers_server_scope_when_user_and_server_settings_exist() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let user_settings_path = home.user_settings_path();
    let server_settings_path = home.server_settings_path();
    write_json(
        &user_settings_path,
        r#"{
  "chatgpt.cliExecutable": "/local/bin/codex"
}"#,
    )?;
    write_json(
        &server_settings_path,
        r#"{
  "chatgpt.cliExecutable": "/remote/bin/vsp-codex",
  "chatgpt.localeOverride": "zh-CN",
  "remote.SSH.remotePlatform": {
    "host": "linux"
  }
}"#,
    )?;
    let bundled_codex = home.extension_bundled_codex_path("openai.chatgpt-26.609.30741-linux-x64");
    let wrapper = home
        .home
        .path()
        .join(".vsp-codex")
        .join("vscode")
        .join("vsp-codex");
    write_json(
        &bundled_codex,
        &format!("#!/bin/sh\nexec '{}' \"$@\"\n", wrapper.display()),
    )?;

    let output = vsp_codex_command(&home)?
        .args(["vscode", "status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status: Value = serde_json::from_slice(&output)?;
    assert_eq!(
        setting(&status, "settingsScope"),
        &Value::from("vscode-server")
    );
    assert_eq!(
        setting(&status, "settingsPath"),
        &Value::from(server_settings_path.display().to_string())
    );
    assert_eq!(
        setting(&status, "cliExecutable"),
        &Value::from("/remote/bin/vsp-codex")
    );
    assert_eq!(setting(&status, "configured"), &Value::from(true));

    Ok(())
}

#[test]
fn vscode_install_falls_back_to_server_settings_when_user_settings_are_absent() -> Result<()> {
    let home = VscodeTestHome::new()?;
    let server_settings_path = home.server_settings_path();
    write_json(
        &server_settings_path,
        r#"{
  "remote.SSH.remotePlatform": {
    "host": "linux"
  }
}"#,
    )?;

    vsp_codex_command(&home)?
        .args(["vscode", "install", "--yes"])
        .assert()
        .success();

    let settings = read_json(&server_settings_path)?;
    assert!(settings.get("chatgpt.cliExecutable").is_some());
    assert!(
        !home.user_settings_path().exists(),
        "server fallback should not create a separate local user settings file"
    );

    let output = vsp_codex_command(&home)?
        .args(["vscode", "status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status: Value = serde_json::from_slice(&output)?;
    assert_eq!(
        setting(&status, "settingsScope"),
        &Value::from("vscode-server")
    );

    Ok(())
}

#[test]
fn vscode_install_reports_diagnostic_when_no_settings_scope_can_be_resolved() -> Result<()> {
    let home = VscodeTestHome::new()?;
    fs::remove_dir_all(home.xdg_config_home.path())?;
    fs::write(home.home.path().join(".vscode-server"), "not a directory")?;

    vsp_codex_command(&home)?
        .args(["vscode", "install", "--yes"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("VSCode settings"))
        .stderr(predicates::str::contains("chatgpt.cliExecutable"));

    Ok(())
}

#[test]
fn vscode_install_app_server_probe_invokes_initialize_with_official_extension_contract()
-> Result<()> {
    let home = VscodeTestHome::new()?;
    let probe_log = home.home.path().join("probe.log");

    vsp_codex_command(&home)?
        .env("VSP_CODEX_VSCODE_PROBE_LOG", &probe_log)
        .args(["vscode", "install", "--yes"])
        .assert()
        .success();

    let probe = read_json(&probe_log)?;
    assert_eq!(
        setting(&probe, "argv"),
        &serde_json::json!([
            "app-server",
            "--analytics-default-enabled",
            "--listen",
            "stdio://"
        ])
    );
    assert_eq!(
        field_path(&probe, &["initialize", "clientInfo", "name"]),
        &Value::from("openai.chatgpt")
    );
    assert_eq!(
        field_path(&probe, &["initialize", "capabilities", "roots"]),
        &Value::from(true)
    );
    assert_eq!(
        field_path(&probe, &["response", "codexHome"]),
        &Value::from(home.codex_home.path().display().to_string())
    );
    assert!(
        field_path(&probe, &["response", "userAgent"])
            .as_str()
            .is_some_and(|user_agent| user_agent.contains("vsp-codex")),
        "probe should record a VSP userAgent: {probe:#}"
    );

    Ok(())
}
