use super::*;
use pretty_assertions::assert_eq;

#[test]
fn parses_version_from_cli_output() {
    assert_eq!(
        parse_binary_version_output("vsp-codex 0.145.0-vsp.9.5\n"),
        Some("0.145.0-vsp.9.5".to_string())
    );
    assert_eq!(parse_binary_version_output("\n"), None);
}

#[cfg(unix)]
#[test]
fn requires_metadata_and_binary_versions_to_match() -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;

    let temp = tempfile::TempDir::new()?;
    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    let binary = bin_dir.join("vsp-codex");
    std::fs::write(
        &binary,
        "#!/bin/sh\nprintf 'vsp-codex 0.145.0-vsp.9.5\\n'\n",
    )?;
    let mut permissions = std::fs::metadata(&binary)?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&binary, permissions)?;
    std::fs::write(
        temp.path().join("codex-package.json"),
        r#"{"version":"0.145.0-vsp.9.5"}"#,
    )?;

    assert!(release_matches_expected_version(
        temp.path(),
        "0.145.0-vsp.9.5"
    ));
    assert!(!release_matches_expected_version(
        temp.path(),
        "0.145.0-vsp.9.4"
    ));

    Ok(())
}
