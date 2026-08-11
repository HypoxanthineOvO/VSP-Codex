#![cfg(not(windows))]

use std::fs;

use anyhow::Result;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::TempDir;

#[test]
fn session_compat_runs_bundled_migration_with_selected_homes() -> Result<()> {
    let root = TempDir::new()?;
    let package_dir = root.path().join("package");
    let bin_dir = package_dir.join("bin");
    let resources_dir = package_dir.join("codex-resources");
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&resources_dir)?;
    fs::write(package_dir.join("codex-package.json"), "{}\n")?;

    let installed_binary = bin_dir.join("vsp-codex");
    fs::copy(
        codex_utils_cargo_bin::cargo_bin("vsp-codex")?,
        &installed_binary,
    )?;
    fs::write(
        resources_dir.join("vsp-session-compat.py"),
        "import json, sys\nprint(json.dumps(sys.argv[1:]))\n",
    )?;

    let codex_home = root.path().join("vsp-home");
    let official_home = root.path().join("official-home");
    fs::create_dir_all(&codex_home)?;
    fs::create_dir_all(&official_home)?;

    assert_cmd::Command::new(installed_binary)
        .env("CODEX_HOME", &codex_home)
        .args([
            "session-compat",
            "status",
            "--official-home",
            official_home.to_str().expect("utf-8 official home"),
            "--vsp-home",
            codex_home.to_str().expect("utf-8 VSP home"),
        ])
        .assert()
        .success()
        .stdout(contains("status"))
        .stdout(contains(
            official_home.to_str().expect("utf-8 official home"),
        ))
        .stdout(contains(codex_home.to_str().expect("utf-8 VSP home")));

    Ok(())
}

#[test]
fn automatic_migration_is_silent_before_protocol_capable_commands() -> Result<()> {
    let root = TempDir::new()?;
    let package_dir = root.path().join("package");
    let bin_dir = package_dir.join("bin");
    let resources_dir = package_dir.join("codex-resources");
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&resources_dir)?;
    fs::write(package_dir.join("codex-package.json"), "{}\n")?;

    let installed_binary = bin_dir.join("vsp-codex");
    fs::copy(
        codex_utils_cargo_bin::cargo_bin("vsp-codex")?,
        &installed_binary,
    )?;
    fs::write(
        resources_dir.join("vsp-session-compat.py"),
        r#"import sys
from pathlib import Path

args = sys.argv[1:]
vsp_home = Path(args[args.index("--vsp-home") + 1])
(vsp_home / "session-compat-v1.json").write_text("{}\n", encoding="utf-8")
print("migration output must stay captured")
"#,
    )?;

    let home = root.path().join("home");
    let official_home = home.join(".codex");
    let codex_home = root.path().join("vsp-home");
    fs::create_dir_all(&official_home)?;
    fs::create_dir_all(&codex_home)?;
    fs::write(official_home.join("state_5.sqlite"), [])?;
    fs::write(codex_home.join("state_5.sqlite"), [])?;

    assert_cmd::Command::new(installed_binary)
        .env("HOME", &home)
        .env("CODEX_HOME", &codex_home)
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(contains("migration output must stay captured").not());
    assert!(codex_home.join("session-compat-v1.json").is_file());

    Ok(())
}
