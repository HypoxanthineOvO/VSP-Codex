use semver::Version;
use serde_json::Map;
use serde_json::Value;
use sha2::Digest as _;
use sha2::Sha256;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

const INSTALLED_DESCRIPTOR: &str = "contracts/host/v1/installed-release.json";
const MAX_WORKFLOW_COMMANDS: usize = 64;
const MAX_COMMAND_NAME_BYTES: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowSlashRoute {
    command: String,
    arguments: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowCommandPaletteEntry {
    pub command: String,
    pub description: String,
}

impl WorkflowSlashRoute {
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn arguments(&self) -> &str {
        &self.arguments
    }

    pub fn is_hypo_workflow(&self) -> bool {
        true
    }
}

pub fn parse_workflow_slash_command(input: &str) -> Option<WorkflowSlashRoute> {
    let root = explicit_release_root()?;
    parse_workflow_slash_command_from_release(input, &root)
}

pub fn parse_workflow_slash_command_from_release(
    input: &str,
    release_root: &Path,
) -> Option<WorkflowSlashRoute> {
    let trimmed = input.trim();
    let routed = trimmed
        .strip_prefix("/hw:")
        .or_else(|| trimmed.strip_prefix("$hypo-workflow:"))?;
    let mut parts = routed.splitn(2, char::is_whitespace);
    let command = parts.next().unwrap_or_default().to_string();
    let arguments = parts.next().unwrap_or_default().trim().to_string();
    let slash = format!("/hw:{command}");
    if !load_installed_command_manifest(release_root)
        .ok()
        .is_some_and(|commands| commands.iter().any(|candidate| candidate == &slash))
    {
        return None;
    }
    Some(WorkflowSlashRoute { command, arguments })
}

pub fn workflow_command_palette_entries(
    is_workflow_project: bool,
) -> Vec<WorkflowCommandPaletteEntry> {
    let Some(root) = explicit_release_root() else {
        return Vec::new();
    };
    workflow_command_palette_entries_from_release(is_workflow_project, &root)
}

pub fn workflow_command_palette_entries_for_codex_home(
    is_workflow_project: bool,
    codex_home: &Path,
) -> Vec<WorkflowCommandPaletteEntry> {
    let Ok(root) = resolve_installed_release_root(codex_home) else {
        return Vec::new();
    };
    workflow_command_palette_entries_from_release(is_workflow_project, &root)
}

pub fn workflow_command_palette_entries_from_release(
    is_workflow_project: bool,
    release_root: &Path,
) -> Vec<WorkflowCommandPaletteEntry> {
    if !is_workflow_project {
        return Vec::new();
    }
    load_installed_command_manifest(release_root)
        .unwrap_or_default()
        .into_iter()
        .map(|command| WorkflowCommandPaletteEntry {
            description: format!(
                "Hypo-Workflow command from the verified installed release: {command}"
            ),
            command,
        })
        .collect()
}

pub fn workflow_release_diagnostic(codex_home: &Path) -> String {
    match resolve_installed_release_root(codex_home)
        .and_then(|root| load_installed_command_manifest(&root).map(|_| root))
    {
        Ok(root) => format!("Verified Hypo-Workflow release: {}", root.display()),
        Err(error) => format!("Hypo-Workflow release unavailable: {error}"),
    }
}

pub fn workflow_status_summary(is_workflow_project: bool) -> Vec<String> {
    if !is_workflow_project {
        return vec![
            "Workflow project: not detected".to_string(),
            "Install or invoke Hypo-Workflow before using /hw:* commands.".to_string(),
        ];
    }
    vec![
        "Workflow project: detected".to_string(),
        "Host Contract v1: read-only host integration".to_string(),
        "Workflow lifecycle and persistence are owned by the installed Hypo-Workflow plugin"
            .to_string(),
    ]
}

fn resolve_installed_release_root(codex_home: &Path) -> Result<PathBuf, String> {
    if let Some(root) = explicit_release_root() {
        load_installed_command_manifest(&root)?;
        return Ok(root);
    }
    let cache = codex_home.join("plugins/cache");
    let marketplaces =
        fs::read_dir(&cache).map_err(|_| format!("no plugin cache at {}", cache.display()))?;
    let mut candidates = Vec::new();
    for marketplace in marketplaces.filter_map(Result::ok) {
        let plugin_base = marketplace.path().join("hypo-workflow");
        let Ok(versions) = fs::read_dir(plugin_base) else {
            continue;
        };
        for version in versions.filter_map(Result::ok) {
            let root = version.path();
            if !root.is_dir() {
                continue;
            }
            if let Ok((release_version, _)) = validate_installed_descriptor(&root) {
                candidates.push((release_version, root));
            }
        }
    }
    candidates.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
    candidates
        .pop()
        .map(|(_, root)| root)
        .ok_or_else(|| "no verified hypo-workflow plugin is installed".to_string())
}

fn explicit_release_root() -> Option<PathBuf> {
    std::env::var_os("HYPO_WORKFLOW_RELEASE_ROOT")
        .or_else(|| std::env::var_os("HYPO_WORKFLOW_ROOT"))
        .map(PathBuf::from)
}

fn load_installed_command_manifest(root: &Path) -> Result<Vec<String>, String> {
    let (_, command_path) = validate_installed_descriptor(root)?;
    let content = fs::read(&command_path)
        .map_err(|error| format!("read command manifest {}: {error}", command_path.display()))?;
    let descriptor: Value = serde_json::from_slice(
        &fs::read(root.join(INSTALLED_DESCRIPTOR)).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let expected_hash = descriptor["command_manifest"]["sha256"]
        .as_str()
        .ok_or_else(|| "installed descriptor command checksum is missing".to_string())?;
    let actual_hash = format!("{:x}", Sha256::digest(&content));
    if actual_hash != expected_hash {
        return Err("installed command manifest checksum mismatch".to_string());
    }
    let manifest: Value = serde_json::from_slice(&content).map_err(|error| error.to_string())?;
    let object = exact_object(
        &manifest,
        &["schema_version", "contract_version", "root", "commands"],
        "command manifest",
    )?;
    require_literal(object, "schema_version", "1")?;
    require_literal(object, "contract_version", "1")?;
    let root_binding = exact_object(
        object
            .get("root")
            .ok_or_else(|| "command manifest root is missing".to_string())?,
        &["name", "skill", "path"],
        "command manifest root",
    )?;
    require_literal(root_binding, "name", "hypo-workflow")?;
    require_literal(root_binding, "skill", "hypo-workflow:hypo-workflow")?;
    require_literal(root_binding, "path", "SKILL.md")?;
    safe_installed_file(root, "SKILL.md", "root Skill")?;
    let entries = object
        .get("commands")
        .and_then(Value::as_array)
        .ok_or_else(|| "command manifest commands must be an array".to_string())?;
    if entries.is_empty() || entries.len() > MAX_WORKFLOW_COMMANDS {
        return Err(format!(
            "command manifest must expose between 1 and {MAX_WORKFLOW_COMMANDS} commands"
        ));
    }
    let mut commands = Vec::new();
    let mut names = HashSet::new();
    for entry in entries {
        let entry = exact_object(entry, &["name", "slash", "skill", "path"], "command entry")?;
        let name = text(entry, "name")?;
        let slash = text(entry, "slash")?;
        let skill = text(entry, "skill")?;
        let path = text(entry, "path")?;
        let suffix = command_suffix(name)?;
        if slash != format!("/{name}")
            || skill != format!("hypo-workflow:{suffix}")
            || path != format!("skills/{suffix}/SKILL.md")
        {
            return Err(format!("invalid command binding: {name}"));
        }
        if !names.insert(name.to_string()) {
            return Err(format!(
                "command manifest contains duplicate command: {name}"
            ));
        }
        safe_installed_file(root, path, "command Skill")?;
        commands.push(slash.to_string());
    }
    Ok(commands)
}

fn validate_installed_descriptor(root: &Path) -> Result<(Version, PathBuf), String> {
    let descriptor_path = safe_installed_file(root, INSTALLED_DESCRIPTOR, "installed descriptor")?;
    let descriptor: Value = serde_json::from_slice(&fs::read(&descriptor_path).map_err(|_| {
        format!(
            "installed descriptor is missing at {}",
            descriptor_path.display()
        )
    })?)
    .map_err(|error| format!("installed descriptor is invalid: {error}"))?;
    let object = exact_object(
        &descriptor,
        &[
            "schema_version",
            "contract_version",
            "release",
            "command_manifest",
        ],
        "installed descriptor",
    )?;
    require_literal(object, "schema_version", "1")?;
    require_literal(object, "contract_version", "1")?;
    let release = exact_object(
        object
            .get("release")
            .ok_or_else(|| "installed release is missing".to_string())?,
        &["version", "source_commit"],
        "installed release",
    )?;
    let version = Version::parse(text(release, "version")?)
        .map_err(|error| format!("installed release version is invalid: {error}"))?;
    let source_commit = text(release, "source_commit")?;
    if source_commit.len() != 40 || !source_commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("installed release source commit is invalid".to_string());
    }
    let command = exact_object(
        object
            .get("command_manifest")
            .ok_or_else(|| "installed command binding is missing".to_string())?,
        &["path", "sha256"],
        "installed command binding",
    )?;
    let command_path =
        safe_installed_file(root, text(command, "path")?, "installed command manifest")?;
    let checksum = text(command, "sha256")?;
    if checksum.len() != 64 || !checksum.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("installed command checksum is invalid".to_string());
    }
    let plugin: Value = serde_json::from_slice(
        &fs::read(safe_installed_file(
            root,
            ".codex-plugin/plugin.json",
            "Codex plugin manifest",
        )?)
        .map_err(|_| "installed Codex plugin manifest is missing".to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let version_text = version.to_string();
    if plugin.get("name").and_then(Value::as_str) != Some("hypo-workflow")
        || plugin.get("version").and_then(Value::as_str) != Some(version_text.as_str())
    {
        return Err(
            "installed Codex plugin identity does not match release descriptor".to_string(),
        );
    }
    Ok((version, command_path))
}

fn command_suffix(name: &str) -> Result<&str, String> {
    let suffix = name
        .strip_prefix("hw:")
        .filter(|suffix| {
            !suffix.is_empty()
                && suffix.len() <= MAX_COMMAND_NAME_BYTES
                && suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
                && suffix
                    .bytes()
                    .next()
                    .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
                && suffix
                    .bytes()
                    .last()
                    .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
        .ok_or_else(|| format!("invalid command name: {name}"))?;
    Ok(suffix)
}

fn safe_installed_file(root: &Path, relative: &str, label: &str) -> Result<PathBuf, String> {
    let root_metadata = fs::symlink_metadata(root)
        .map_err(|_| format!("installed release root is missing: {}", root.display()))?;
    if !root_metadata.is_dir() || root_metadata.file_type().is_symlink() {
        return Err(format!(
            "installed release root is unsafe: {}",
            root.display()
        ));
    }
    safe_join(root, relative)?;
    let parts = relative.split('/').collect::<Vec<_>>();
    let mut current = root.to_path_buf();
    for (index, part) in parts.iter().enumerate() {
        current.push(part);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|_| format!("installed {label} is missing: {relative}"))?;
        if metadata.file_type().is_symlink()
            || (index + 1 == parts.len() && !metadata.is_file())
            || (index + 1 < parts.len() && !metadata.is_dir())
        {
            return Err(format!("installed {label} is unsafe: {relative}"));
        }
    }
    Ok(current)
}

fn safe_join(root: &Path, relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty()
        || relative.contains('\\')
        || relative.starts_with('/')
        || relative
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err("installed release path is unsafe".to_string());
    }
    Ok(root.join(relative))
}

fn exact_object<'a>(
    value: &'a Value,
    keys: &[&str],
    label: &str,
) -> Result<&'a Map<String, Value>, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{label} must be an object"))?;
    if object.len() != keys.len() || object.keys().any(|key| !keys.contains(&key.as_str())) {
        return Err(format!("{label} contains unknown or missing fields"));
    }
    Ok(object)
}

fn require_literal(object: &Map<String, Value>, key: &str, expected: &str) -> Result<(), String> {
    if object.get(key).and_then(Value::as_str) == Some(expected) {
        Ok(())
    } else {
        Err(format!("{key} is unsupported"))
    }
}

fn text<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a str, String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{key} must be non-empty text"))
}
