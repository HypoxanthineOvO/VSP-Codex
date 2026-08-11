use pretty_assertions::assert_eq;

use super::executable_identity_from_bytes;
use super::expected_managed_codex_relative_paths;
use super::managed_codex_bin;
use super::parse_codex_version;

#[test]
fn parses_codex_cli_version_output() {
    assert_eq!(
        parse_codex_version("codex 1.2.3\n").expect("version"),
        "1.2.3"
    );
}

#[test]
fn rejects_malformed_codex_cli_version_output() {
    assert!(parse_codex_version("codex\n").is_err());
}

#[test]
fn executable_identity_uses_binary_contents() {
    let old = executable_identity_from_bytes(b"old");
    let same = executable_identity_from_bytes(b"old");
    let new = executable_identity_from_bytes(b"new");

    assert_eq!(old, same);
    assert_ne!(old, new);
}

#[test]
fn managed_codex_bin_prefers_vsp_entrypoint_when_present() {
    let codex_home = tempfile::tempdir().expect("tempdir");
    let current = codex_home
        .path()
        .join("packages")
        .join("standalone")
        .join("current");
    let paths = expected_managed_codex_relative_paths();
    std::fs::create_dir_all(current.join("bin")).expect("create bin");
    std::fs::write(current.join(paths[1]), b"official").expect("write official");
    std::fs::write(current.join(paths[0]), b"vsp").expect("write vsp");

    assert_eq!(managed_codex_bin(codex_home.path()), current.join(paths[0]));
}

#[test]
fn managed_codex_bin_falls_back_to_official_bin_entrypoint_for_upstream_installs() {
    let codex_home = tempfile::tempdir().expect("tempdir");
    let current = codex_home
        .path()
        .join("packages")
        .join("standalone")
        .join("current");
    let paths = expected_managed_codex_relative_paths();
    std::fs::create_dir_all(current.join("bin")).expect("create bin");
    std::fs::write(current.join(paths[1]), b"official").expect("write official");

    assert_eq!(managed_codex_bin(codex_home.path()), current.join(paths[1]));
}

#[test]
fn managed_codex_bin_keeps_flat_layout_fallback_for_legacy_installs() {
    let codex_home = tempfile::tempdir().expect("tempdir");
    let current = codex_home
        .path()
        .join("packages")
        .join("standalone")
        .join("current");
    let paths = expected_managed_codex_relative_paths();
    std::fs::create_dir_all(&current).expect("create current");
    std::fs::write(current.join(paths[3]), b"official").expect("write official");

    assert_eq!(managed_codex_bin(codex_home.path()), current.join(paths[3]));
}

#[test]
fn managed_codex_bin_defaults_to_vsp_bin_entrypoint_when_missing() {
    let codex_home = tempfile::tempdir().expect("tempdir");
    let current = codex_home
        .path()
        .join("packages")
        .join("standalone")
        .join("current");
    let paths = expected_managed_codex_relative_paths();

    assert_eq!(managed_codex_bin(codex_home.path()), current.join(paths[0]));
}
