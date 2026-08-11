use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;

use crate::SESSIONS_SUBDIR;

const EXPORT_DIR: &str = "official_codex_discovery";
const MANIFEST_FILE: &str = "manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficialDiscoveryFieldClass {
    Safe,
    Private,
    Secret,
}

#[derive(Debug, Clone, Serialize)]
pub struct OfficialCodexExport {
    pub manifest_path: PathBuf,
    pub sessions: Vec<OfficialCodexSessionExport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OfficialCodexSessionExport {
    pub id: String,
    pub created_at: String,
    pub cwd: PathBuf,
    pub title: Option<String>,
    pub source: Option<String>,
    pub sanitized_artifact_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct SessionIndexEntry {
    id: String,
    thread_name: Option<String>,
}

#[derive(Debug)]
struct RolloutCandidate {
    path: PathBuf,
    relative_path: PathBuf,
    id: String,
    created_at: String,
}

#[derive(Debug, Default)]
struct SanitizedSessionMeta {
    id: Option<String>,
    created_at: Option<String>,
    cwd: Option<PathBuf>,
    title: Option<String>,
    source: Option<String>,
}

#[derive(Serialize)]
struct OfficialCodexManifest<'a> {
    version: u32,
    strategy: &'static str,
    sessions: &'a [OfficialCodexSessionExport],
}

pub fn classify_field_for_official_discovery(field_path: &str) -> OfficialDiscoveryFieldClass {
    let normalized = field_path.to_ascii_lowercase();
    let parts: Vec<&str> = normalized
        .split(['.', '[', ']'])
        .filter(|part| !part.is_empty())
        .collect();

    if parts
        .iter()
        .any(|part| matches_secret_key(part) || matches_secret_value_name(part))
    {
        return OfficialDiscoveryFieldClass::Secret;
    }

    if parts.iter().any(|part| {
        matches!(
            *part,
            "auth"
                | "authorization"
                | "config"
                | "env"
                | "provider_details"
                | "provider"
                | "providers"
                | "model_provider"
                | "model_providers"
                | "base_url"
                | "env_key"
                | "workflow"
                | "pipeline"
                | "rollout_trace"
                | "environment"
                | "current_step"
                | "safe_resume_command"
                | "parent_trace_id"
                | "span_attributes"
        )
    }) || parts.iter().any(|part| part.contains("private"))
    {
        return OfficialDiscoveryFieldClass::Private;
    }

    OfficialDiscoveryFieldClass::Safe
}

pub async fn export_sessions_for_official_codex(
    vsp_codex_home: &Path,
    official_codex_home: &Path,
) -> anyhow::Result<OfficialCodexExport> {
    let export_root = official_codex_home.join(EXPORT_DIR);
    let artifacts_root = export_root.join(SESSIONS_SUBDIR);
    tokio::fs::create_dir_all(&artifacts_root)
        .await
        .with_context(|| {
            format!(
                "create official Codex export dir {}",
                artifacts_root.display()
            )
        })?;

    let index_names = read_session_index_names(vsp_codex_home).await?;
    let mut candidates = collect_rollout_candidates(&vsp_codex_home.join(SESSIONS_SUBDIR)).await?;
    candidates.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| right.id.cmp(&left.id))
    });

    let mut sessions = Vec::new();
    for candidate in candidates {
        let sanitized_artifact_path = artifact_path_for_candidate(&artifacts_root, &candidate)?;
        let meta = sanitize_rollout_file(
            &candidate.path,
            &sanitized_artifact_path,
            index_names.get(&candidate.id),
        )
        .await
        .with_context(|| format!("sanitize rollout {}", candidate.path.display()))?;

        let id = meta.id.unwrap_or(candidate.id);
        let created_at = meta.created_at.unwrap_or(candidate.created_at);
        let Some(cwd) = meta.cwd else {
            continue;
        };

        sessions.push(OfficialCodexSessionExport {
            id,
            created_at,
            cwd,
            title: meta.title,
            source: meta.source,
            sanitized_artifact_path,
        });
    }

    let manifest_path = export_root.join(MANIFEST_FILE);
    let manifest = OfficialCodexManifest {
        version: 1,
        strategy: "sanitized_vsp_session_export",
        sessions: &sessions,
    };
    let manifest_text = serde_json::to_string_pretty(&manifest)?;
    tokio::fs::write(&manifest_path, format!("{manifest_text}\n"))
        .await
        .with_context(|| format!("write official Codex manifest {}", manifest_path.display()))?;

    Ok(OfficialCodexExport {
        manifest_path,
        sessions,
    })
}

async fn read_session_index_names(
    vsp_codex_home: &Path,
) -> anyhow::Result<HashMap<String, String>> {
    let path = vsp_codex_home.join("session_index.jsonl");
    let Ok(file) = tokio::fs::File::open(&path).await else {
        return Ok(HashMap::new());
    };
    let mut names = HashMap::new();
    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(entry) = serde_json::from_str::<SessionIndexEntry>(trimmed) else {
            continue;
        };
        if let Some(thread_name) = entry.thread_name
            && !thread_name.trim().is_empty()
        {
            names.insert(entry.id, thread_name);
        }
    }
    Ok(names)
}

async fn collect_rollout_candidates(root: &Path) -> anyhow::Result<Vec<RolloutCandidate>> {
    let root = root.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut candidates = Vec::new();
        collect_rollout_candidates_sync(&root, &root, &mut candidates)?;
        Ok(candidates)
    })
    .await?
}

fn collect_rollout_candidates_sync(
    root: &Path,
    current_dir: &Path,
    candidates: &mut Vec<RolloutCandidate>,
) -> anyhow::Result<()> {
    let Ok(entries) = std::fs::read_dir(current_dir) else {
        return Ok(());
    };
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            collect_rollout_candidates_sync(root, &path, candidates)?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }
        let Some((created_at, id)) = parse_rollout_file_name(path.file_name()) else {
            continue;
        };
        let relative_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        candidates.push(RolloutCandidate {
            path,
            relative_path,
            id,
            created_at,
        });
    }
    Ok(())
}

fn parse_rollout_file_name(file_name: Option<&OsStr>) -> Option<(String, String)> {
    let file_name = file_name?.to_str()?;
    let rest = file_name.strip_prefix("rollout-")?;
    let rest = rest.strip_suffix(".jsonl")?;
    let timestamp = rest.get(..19)?;
    let id = rest.get(20..)?;
    if id.is_empty() {
        return None;
    }
    Some((file_timestamp_to_rfc3339(timestamp)?, id.to_string()))
}

fn file_timestamp_to_rfc3339(timestamp: &str) -> Option<String> {
    let date = timestamp.get(..10)?;
    let time = timestamp.get(11..19)?.replace('-', ":");
    let normalized = format!("{date}T{time}");
    let rfc3339 = format!("{normalized}Z");
    OffsetDateTime::parse(&rfc3339, &Rfc3339)
        .ok()?
        .format(&Rfc3339)
        .ok()
}

fn artifact_path_for_candidate(
    artifacts_root: &Path,
    candidate: &RolloutCandidate,
) -> anyhow::Result<PathBuf> {
    let file_name = candidate
        .path
        .file_name()
        .context("rollout candidate has no file name")?;
    let source_hash = short_path_hash(&candidate.relative_path);
    let parent_path = candidate.relative_path.parent().unwrap_or(Path::new(""));
    let mut destination_dir = artifacts_root.join(source_hash);
    for component in parent_path.components() {
        let Component::Normal(segment) = component else {
            continue;
        };
        destination_dir = destination_dir.join(sanitize_path_component(segment));
    }
    Ok(destination_dir.join(file_name))
}

async fn sanitize_rollout_file(
    source_path: &Path,
    destination_path: &Path,
    indexed_name: Option<&String>,
) -> anyhow::Result<SanitizedSessionMeta> {
    if let Some(parent) = destination_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let input = tokio::fs::File::open(source_path).await?;
    let reader = tokio::io::BufReader::new(input);
    let output = tokio::fs::File::create(destination_path).await?;
    let mut writer = tokio::io::BufWriter::new(output);
    let mut lines = reader.lines();
    let mut meta = SanitizedSessionMeta::default();

    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(mut value) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        sanitize_value(&mut value, "");
        if value.get("type").and_then(Value::as_str) == Some("session_meta") {
            capture_session_meta(&value, indexed_name, &mut meta);
        }
        let line = serde_json::to_string(&value)?;
        writer.write_all(line.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }
    writer.flush().await?;

    Ok(meta)
}

fn capture_session_meta(
    value: &Value,
    indexed_name: Option<&String>,
    meta: &mut SanitizedSessionMeta,
) {
    let Some(payload) = value.get("payload") else {
        return;
    };
    if meta.id.is_none() {
        meta.id = payload
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string);
    }
    if meta.created_at.is_none() {
        meta.created_at = payload
            .get("timestamp")
            .or_else(|| value.get("timestamp"))
            .and_then(Value::as_str)
            .map(ToString::to_string);
    }
    if meta.cwd.is_none() {
        meta.cwd = payload
            .get("cwd")
            .and_then(Value::as_str)
            .map(PathBuf::from);
    }
    if meta.source.is_none() {
        meta.source = payload
            .get("source")
            .and_then(Value::as_str)
            .map(ToString::to_string);
    }
    if meta.title.is_none() {
        meta.title = indexed_name
            .and_then(|name| sanitize_export_title(name))
            .or_else(|| {
                payload
                    .get("title")
                    .or_else(|| payload.get("name"))
                    .and_then(Value::as_str)
                    .and_then(sanitize_export_title)
            });
    }
}

fn sanitize_export_title(title: &str) -> Option<String> {
    let trimmed = title.trim();
    if trimmed.is_empty() || looks_like_secret_value(trimmed) {
        return None;
    }
    Some(trimmed.to_string())
}

fn sanitize_value(value: &mut Value, path: &str) {
    match value {
        Value::Object(map) => sanitize_object(map, path),
        Value::Array(items) => {
            for item in items {
                sanitize_value(item, path);
            }
        }
        Value::String(text) => {
            if looks_like_secret_value(text) {
                text.clear();
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn sanitize_object(map: &mut Map<String, Value>, path: &str) {
    let root_is_session_meta =
        path.is_empty() && map.get("type").and_then(Value::as_str) == Some("session_meta");
    map.retain(|key, value| {
        let child_path = if path.is_empty() {
            key.to_string()
        } else {
            format!("{path}.{key}")
        };
        let class = classify_field_for_official_discovery(&child_path);
        let keep = if path == "session_meta" {
            is_safe_session_meta_payload_key(key) && class == OfficialDiscoveryFieldClass::Safe
        } else {
            class == OfficialDiscoveryFieldClass::Safe
        };
        if keep {
            if root_is_session_meta && key == "payload" {
                sanitize_value(value, "session_meta");
            } else {
                sanitize_value(value, &child_path);
            }
        }
        keep
    });
}

fn is_safe_session_meta_payload_key(key: &str) -> bool {
    matches!(
        key,
        "id" | "timestamp"
            | "cwd"
            | "originator"
            | "cli_version"
            | "source"
            | "title"
            | "name"
            | "summary"
    )
}

fn matches_secret_key(part: &str) -> bool {
    matches!(
        part,
        "api_key"
            | "apikey"
            | "openai_api_key"
            | "anthropic_api_key"
            | "authorization"
            | "refresh_token"
            | "access_token"
            | "id_token"
            | "auth_header"
            | "secret"
            | "token"
            | "password"
            | "credential"
            | "credentials"
    )
}

fn matches_secret_value_name(part: &str) -> bool {
    part.ends_with("_api_key")
        || part.ends_with("_token")
        || part.contains("secret")
        || part.contains("password")
}

fn looks_like_secret_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("bearer ")
        || lower.contains("api_key")
        || lower.contains("refresh-token")
        || lower.contains("refresh_token")
        || lower.contains(".pipeline")
        || lower.contains("/hw:")
        || lower.contains("rollout_trace")
        || lower.contains("provider_details")
        || lower.contains("model_provider")
        || lower.contains("provider")
        || lower.contains("private")
        || lower.contains("trace-private")
        || lower.contains("trace-secret")
}

fn sanitize_path_component(component: &OsStr) -> String {
    let raw = component.to_string_lossy();
    let sanitized: String = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() || looks_like_secret_value(&sanitized) {
        "component".to_string()
    } else {
        sanitized
    }
}

fn short_path_hash(path: &Path) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for component in path.components() {
        let Component::Normal(segment) = component else {
            continue;
        };
        for byte in segment.as_encoded_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= u64::from(b'/');
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
