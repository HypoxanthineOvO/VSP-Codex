use super::is_nonfatal_symlink_error;
use super::link_default_vsp_codex_home_assets_from_env;
use pretty_assertions::assert_eq;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;

fn sibling_homes() -> io::Result<(TempDir, PathBuf, PathBuf)> {
    let root = TempDir::new()?;
    let vsp_home = root.path().join(".vsp-codex");
    let codex_home = root.path().join(".codex");
    Ok((root, vsp_home, codex_home))
}

fn write_file(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}

fn create_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

#[cfg(unix)]
fn assert_symlink_target(path: &Path, expected: &Path) -> io::Result<()> {
    assert_eq!(fs::read_link(path)?, expected);
    Ok(())
}

#[cfg(unix)]
fn assert_not_symlink(path: &Path) -> io::Result<()> {
    assert!(
        !fs::symlink_metadata(path)?.file_type().is_symlink(),
        "{} should not be a symlink",
        path.display()
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn links_selected_legacy_assets_without_config_auth_or_system_skills() -> io::Result<()> {
    let (_root, vsp_home, codex_home) = sibling_homes()?;
    create_dir(&codex_home.join("sessions"))?;
    create_dir(&codex_home.join("archived_sessions"))?;
    write_file(&codex_home.join("session_index.jsonl"), "{\"id\":\"s1\"}\n")?;
    write_file(&codex_home.join("skills/research/SKILL.md"), "research")?;
    write_file(&codex_home.join("skills/.system/system/SKILL.md"), "system")?;
    create_dir(&codex_home.join("plugins/cache/debug/sample/local"))?;
    write_file(&codex_home.join("plugins/data/sample/state.json"), "{}\n")?;
    create_dir(&codex_home.join(".tmp/plugins/plugins/sample"))?;
    write_file(&codex_home.join(".tmp/plugins.sha"), "abc123\n")?;
    write_file(
        &codex_home.join("vendor_imports/skills/imported/SKILL.md"),
        "imported",
    )?;
    write_file(&codex_home.join("config.toml"), "model = \"codex\"\n")?;
    write_file(&codex_home.join("auth.json"), "{}\n")?;
    write_file(&codex_home.join(".credentials.json"), "{}\n")?;
    write_file(&codex_home.join("plugins/installed/marker"), "installed")?;

    link_default_vsp_codex_home_assets_from_env(&vsp_home, None, Some(&vsp_home))?;

    assert_symlink_target(&vsp_home.join("sessions"), &codex_home.join("sessions"))?;
    assert_symlink_target(
        &vsp_home.join("archived_sessions"),
        &codex_home.join("archived_sessions"),
    )?;
    assert_symlink_target(
        &vsp_home.join("session_index.jsonl"),
        &codex_home.join("session_index.jsonl"),
    )?;
    assert_not_symlink(&vsp_home.join("skills"))?;
    assert_symlink_target(
        &vsp_home.join("skills/research"),
        &codex_home.join("skills/research"),
    )?;
    assert!(
        !vsp_home.join("skills/.system").try_exists()?,
        "VSP system skills cache must remain independent"
    );
    assert_symlink_target(
        &vsp_home.join("plugins/cache"),
        &codex_home.join("plugins/cache"),
    )?;
    assert_symlink_target(
        &vsp_home.join("plugins/data"),
        &codex_home.join("plugins/data"),
    )?;
    assert_symlink_target(
        &vsp_home.join(".tmp/plugins"),
        &codex_home.join(".tmp/plugins"),
    )?;
    assert_symlink_target(
        &vsp_home.join(".tmp/plugins.sha"),
        &codex_home.join(".tmp/plugins.sha"),
    )?;
    assert_symlink_target(
        &vsp_home.join("vendor_imports/skills"),
        &codex_home.join("vendor_imports/skills"),
    )?;
    assert!(!vsp_home.join("config.toml").try_exists()?);
    assert!(!vsp_home.join("auth.json").try_exists()?);
    assert!(!vsp_home.join(".credentials.json").try_exists()?);
    assert!(!vsp_home.join("plugins/installed").try_exists()?);
    Ok(())
}

#[cfg(unix)]
#[test]
fn explicit_default_codex_home_env_still_links_shared_sessions() -> io::Result<()> {
    let (_root, vsp_home, codex_home) = sibling_homes()?;
    create_dir(&codex_home.join("sessions"))?;

    link_default_vsp_codex_home_assets_from_env(
        &vsp_home,
        Some(vsp_home.as_os_str()),
        Some(&vsp_home),
    )?;

    assert_symlink_target(&vsp_home.join("sessions"), &codex_home.join("sessions"))?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn does_not_overwrite_existing_vsp_assets() -> io::Result<()> {
    let (_root, vsp_home, codex_home) = sibling_homes()?;
    create_dir(&codex_home.join("sessions"))?;
    write_file(&codex_home.join("session_index.jsonl"), "legacy\n")?;
    write_file(&codex_home.join("skills/research/SKILL.md"), "legacy")?;
    create_dir(&vsp_home.join("sessions"))?;
    write_file(&vsp_home.join("session_index.jsonl"), "vsp\n")?;
    write_file(&vsp_home.join("skills/research/SKILL.md"), "vsp")?;

    link_default_vsp_codex_home_assets_from_env(&vsp_home, None, Some(&vsp_home))?;

    assert_not_symlink(&vsp_home.join("sessions"))?;
    assert_not_symlink(&vsp_home.join("session_index.jsonl"))?;
    assert_not_symlink(&vsp_home.join("skills/research"))?;
    assert_eq!(
        fs::read_to_string(vsp_home.join("session_index.jsonl"))?,
        "vsp\n"
    );
    assert_eq!(
        fs::read_to_string(vsp_home.join("skills/research/SKILL.md"))?,
        "vsp"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn only_runs_for_implicit_default_vsp_home_with_sibling_codex_home() -> io::Result<()> {
    let (_root, vsp_home, codex_home) = sibling_homes()?;
    create_dir(&codex_home.join("sessions"))?;

    link_default_vsp_codex_home_assets_from_env(
        &vsp_home,
        Some(OsStr::new("/tmp/custom")),
        Some(&vsp_home),
    )?;
    assert!(!vsp_home.join("sessions").try_exists()?);

    let custom_home = vsp_home.with_file_name("custom");
    link_default_vsp_codex_home_assets_from_env(&custom_home, None, Some(&vsp_home))?;
    assert!(!custom_home.join("sessions").try_exists()?);

    let missing_sibling_home = vsp_home
        .parent()
        .expect("vsp home has parent")
        .join("isolated")
        .join(".vsp-codex");
    link_default_vsp_codex_home_assets_from_env(
        &missing_sibling_home,
        None,
        Some(&missing_sibling_home),
    )?;
    assert!(!missing_sibling_home.join("sessions").try_exists()?);
    Ok(())
}

#[cfg(unix)]
#[test]
fn does_not_run_when_home_is_not_the_default_vsp_home() -> io::Result<()> {
    let (_root, vsp_home, codex_home) = sibling_homes()?;
    create_dir(&codex_home.join("sessions"))?;
    let configured_default = vsp_home
        .parent()
        .expect("vsp home has parent")
        .join("other-default");

    link_default_vsp_codex_home_assets_from_env(&vsp_home, None, Some(&configured_default))?;

    assert!(!vsp_home.join("sessions").try_exists()?);
    Ok(())
}

#[test]
fn does_not_run_when_default_home_cannot_be_resolved() -> io::Result<()> {
    let (_root, vsp_home, codex_home) = sibling_homes()?;
    create_dir(&codex_home.join("sessions"))?;

    link_default_vsp_codex_home_assets_from_env(&vsp_home, None, None)?;

    assert!(!vsp_home.join("sessions").try_exists()?);
    Ok(())
}

#[cfg(unix)]
#[test]
fn empty_codex_home_env_is_treated_as_unset() -> io::Result<()> {
    let (_root, vsp_home, codex_home) = sibling_homes()?;
    create_dir(&codex_home.join("sessions"))?;

    link_default_vsp_codex_home_assets_from_env(&vsp_home, Some(OsStr::new("")), Some(&vsp_home))?;

    assert_symlink_target(&vsp_home.join("sessions"), &codex_home.join("sessions"))?;
    Ok(())
}

#[test]
fn symlink_permission_failures_are_nonfatal() {
    assert!(is_nonfatal_symlink_error(&io::Error::from(
        io::ErrorKind::PermissionDenied
    )));
    assert!(is_nonfatal_symlink_error(&io::Error::from(
        io::ErrorKind::Unsupported
    )));
    assert!(!is_nonfatal_symlink_error(&io::Error::from(
        io::ErrorKind::InvalidInput
    )));
}
