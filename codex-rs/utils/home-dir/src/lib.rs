use codex_utils_absolute_path::AbsolutePathBuf;
use dirs::home_dir;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// Returns the path to the Codex configuration directory, which can be
/// specified by the `CODEX_HOME` environment variable. If not set, defaults to
/// `~/.vsp-codex`.
///
/// - If `CODEX_HOME` is set, the value must exist and be a directory. The
///   value will be canonicalized and this function will Err otherwise.
/// - If `CODEX_HOME` is not set, this function does not verify that the
///   directory exists.
pub fn find_codex_home() -> std::io::Result<AbsolutePathBuf> {
    let codex_home_env = std::env::var("CODEX_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    find_codex_home_from_env(codex_home_env.as_deref())
}

/// Link VSP's default home to selected assets from sibling `~/.codex`.
///
/// This is intentionally narrow: it only runs for the default `.vsp-codex`
/// home, whether implicit or explicitly selected by a wrapper, never overwrites
/// existing targets, and shares only assets safe to reuse across Codex variants.
pub fn link_default_vsp_codex_home_assets(codex_home: &Path) -> io::Result<()> {
    let codex_home_env = std::env::var_os("CODEX_HOME");
    let default_vsp_home = home_dir().map(|home| home.join(".vsp-codex"));
    link_default_vsp_codex_home_assets_from_env(
        codex_home,
        codex_home_env.as_deref(),
        default_vsp_home.as_deref(),
    )
}

fn link_default_vsp_codex_home_assets_from_env(
    codex_home: &Path,
    codex_home_env: Option<&OsStr>,
    default_vsp_home: Option<&Path>,
) -> io::Result<()> {
    let Some(default_vsp_home) = default_vsp_home else {
        return Ok(());
    };
    if codex_home_env.is_some_and(|value| !value.is_empty() && Path::new(value) != default_vsp_home)
    {
        return Ok(());
    }
    if codex_home != default_vsp_home {
        return Ok(());
    }

    let Some(parent) = codex_home.parent() else {
        return Ok(());
    };
    let sibling_codex_home = parent.join(".codex");
    if !sibling_codex_home.is_dir() {
        return Ok(());
    }

    for asset in ["sessions", "archived_sessions", "session_index.jsonl"] {
        symlink_asset_if_missing(&sibling_codex_home.join(asset), &codex_home.join(asset))?;
    }

    symlink_skill_children_if_missing(
        &sibling_codex_home.join("skills"),
        &codex_home.join("skills"),
    )?;

    for asset in [
        ("plugins/cache", "plugins/cache"),
        ("plugins/data", "plugins/data"),
        (".tmp/plugins", ".tmp/plugins"),
        (".tmp/plugins.sha", ".tmp/plugins.sha"),
        ("vendor_imports/skills", "vendor_imports/skills"),
    ] {
        symlink_asset_if_missing(&sibling_codex_home.join(asset.0), &codex_home.join(asset.1))?;
    }

    Ok(())
}

fn find_codex_home_from_env(codex_home_env: Option<&str>) -> std::io::Result<AbsolutePathBuf> {
    // Honor the `CODEX_HOME` environment variable when it is set to allow users
    // (and tests) to override the default location.
    match codex_home_env {
        Some(val) => {
            let path = PathBuf::from(val);
            let metadata = std::fs::metadata(&path).map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("CODEX_HOME points to {val:?}, but that path does not exist"),
                ),
                _ => std::io::Error::new(
                    err.kind(),
                    format!("failed to read CODEX_HOME {val:?}: {err}"),
                ),
            })?;

            if !metadata.is_dir() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("CODEX_HOME points to {val:?}, but that path is not a directory"),
                ))
            } else {
                let canonical = path.canonicalize().map_err(|err| {
                    std::io::Error::new(
                        err.kind(),
                        format!("failed to canonicalize CODEX_HOME {val:?}: {err}"),
                    )
                })?;
                AbsolutePathBuf::from_absolute_path(canonical)
            }
        }
        None => {
            let mut p = home_dir().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory",
                )
            })?;
            p.push(".vsp-codex");
            AbsolutePathBuf::from_absolute_path(p)
        }
    }
}

fn symlink_skill_children_if_missing(source_skills: &Path, target_skills: &Path) -> io::Result<()> {
    if !source_skills.is_dir() {
        return Ok(());
    }

    fs::create_dir_all(target_skills)?;
    for entry in fs::read_dir(source_skills)? {
        let entry = entry?;
        if entry.file_name() == OsStr::new(".system") {
            continue;
        }
        symlink_asset_if_missing(&entry.path(), &target_skills.join(entry.file_name()))?;
    }

    Ok(())
}

fn symlink_asset_if_missing(source: &Path, target: &Path) -> io::Result<()> {
    let Ok(source_metadata) = fs::metadata(source) else {
        return Ok(());
    };
    if fs::symlink_metadata(target).is_ok() {
        return Ok(());
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    match symlink(source, target, source_metadata.is_dir()) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            if fs::symlink_metadata(target).is_ok() {
                Ok(())
            } else {
                Err(err)
            }
        }
        Err(err) if is_nonfatal_symlink_error(&err) => Ok(()),
        Err(err) => Err(err),
    }
}

fn is_nonfatal_symlink_error(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
    )
}

#[cfg(unix)]
fn symlink(source: &Path, target: &Path, _is_dir: bool) -> io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
fn symlink(source: &Path, target: &Path, is_dir: bool) -> io::Result<()> {
    if is_dir {
        std::os::windows::fs::symlink_dir(source, target)
    } else {
        std::os::windows::fs::symlink_file(source, target)
    }
}

#[cfg(test)]
mod tests {
    use super::find_codex_home_from_env;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use dirs::home_dir;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::io::ErrorKind;
    use tempfile::TempDir;

    #[test]
    fn find_codex_home_env_missing_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let missing = temp_home.path().join("missing-codex-home");
        let missing_str = missing
            .to_str()
            .expect("missing codex home path should be valid utf-8");

        let err = find_codex_home_from_env(Some(missing_str)).expect_err("missing CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("CODEX_HOME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_file_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let file_path = temp_home.path().join("codex-home.txt");
        fs::write(&file_path, "not a directory").expect("write temp file");
        let file_str = file_path
            .to_str()
            .expect("file codex home path should be valid utf-8");

        let err = find_codex_home_from_env(Some(file_str)).expect_err("file CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_valid_directory_canonicalizes() {
        let temp_home = TempDir::new().expect("temp home");
        let temp_str = temp_home
            .path()
            .to_str()
            .expect("temp codex home path should be valid utf-8");

        let resolved = find_codex_home_from_env(Some(temp_str)).expect("valid CODEX_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_codex_home_without_env_uses_vsp_default_home_dir() {
        let resolved =
            find_codex_home_from_env(/*codex_home_env*/ None).expect("default CODEX_HOME");
        let mut expected = home_dir().expect("home dir");
        expected.push(".vsp-codex");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }
}

#[cfg(test)]
mod shared_assets_tests;
