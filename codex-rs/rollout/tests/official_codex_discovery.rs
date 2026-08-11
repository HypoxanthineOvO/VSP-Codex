use std::fs;
use std::path::Path;
use std::path::PathBuf;

use codex_protocol::protocol::SessionSource;
use codex_rollout::official_codex_discovery::classify_field_for_official_discovery;
use codex_rollout::official_codex_discovery::export_sessions_for_official_codex;
use serde::Deserialize;
use serde_json::Value;
use tempfile::TempDir;

const FIXTURE_ID: &str = "11111111-1111-4111-8111-111111111111";
const FIXTURE_FILE_TS: &str = "2026-05-29T10-00-00";
const FIXTURE_RFC3339: &str = "2026-05-29T10:00:00Z";
const SAFE_TITLE: &str = "Safe VSP fixture title";
const SAFE_CWD: &str = "/workspaces/safe-project";

const UNSAFE_NEEDLES: &[&str] = &[
    "vsp-private-provider",
    "private-vsp",
    "private-vsp-gateway",
    "VSP_PRIVATE_API_KEY",
    "vsp-provider-secret-token",
    "sk-vsp-raw-secret-from-rollout",
    "vsp-refresh-token-secret",
    "sk-vsp-env-secret",
    "sk-ant-vsp-env-secret",
    ".pipeline",
    "implementation-private",
    "/hw:resume",
    "trace-private-123",
    "trace-secret",
    "rollout_trace",
    "provider_details",
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
];

#[derive(Debug, Deserialize)]
struct ClassificationCase {
    field: String,
    #[serde(rename = "class")]
    expected_class: String,
}

#[tokio::test]
async fn export_for_official_codex_strips_private_runtime_and_preserves_resume_metadata()
-> anyhow::Result<()> {
    let vsp_home = TempDir::new()?;
    let official_home = TempDir::new()?;
    write_representative_vsp_home(vsp_home.path())?;

    let export = export_sessions_for_official_codex(vsp_home.path(), official_home.path()).await?;

    assert_eq!(export.sessions.len(), 1);
    let session = &export.sessions[0];
    assert_eq!(session.id, FIXTURE_ID);
    assert_eq!(session.created_at, FIXTURE_RFC3339);
    assert_eq!(session.cwd, PathBuf::from(SAFE_CWD));
    assert_eq!(session.title.as_deref(), Some(SAFE_TITLE));
    assert_eq!(session.source.as_deref(), Some("vscode"));
    assert!(
        session
            .sanitized_artifact_path
            .starts_with(official_home.path()),
        "sanitized artifact must be written under official CODEX_HOME, got {}",
        session.sanitized_artifact_path.display()
    );
    assert!(
        !session.sanitized_artifact_path.starts_with(vsp_home.path()),
        "sanitized artifact must not expose the VSP private home directly"
    );

    let manifest_text = fs::read_to_string(&export.manifest_path)?;
    let artifact_text = fs::read_to_string(&session.sanitized_artifact_path)?;
    let exported_text = format!("{manifest_text}\n{artifact_text}");

    for safe_value in [FIXTURE_ID, FIXTURE_RFC3339, SAFE_CWD, SAFE_TITLE, "vscode"] {
        assert!(
            exported_text.contains(safe_value),
            "official discovery export dropped safe field value {safe_value:?}"
        );
    }

    for unsafe_value in UNSAFE_NEEDLES {
        assert!(
            !exported_text.contains(unsafe_value),
            "official discovery export leaked unsafe value {unsafe_value:?}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn export_for_official_codex_does_not_symlink_or_raw_share_vsp_home() -> anyhow::Result<()> {
    let vsp_home = TempDir::new()?;
    let official_home = TempDir::new()?;
    write_representative_vsp_home(vsp_home.path())?;

    let export = export_sessions_for_official_codex(vsp_home.path(), official_home.path()).await?;

    assert_no_symlink_points_into(official_home.path(), vsp_home.path())?;
    for session in &export.sessions {
        let metadata = fs::symlink_metadata(&session.sanitized_artifact_path)?;
        assert!(
            !metadata.file_type().is_symlink(),
            "sanitized artifact must be a real sanitized file, not a symlink: {}",
            session.sanitized_artifact_path.display()
        );
    }

    let official_sessions = official_home.path().join("sessions");
    if official_sessions.exists() {
        let official_sessions = official_sessions.canonicalize()?;
        assert!(
            !official_sessions.starts_with(vsp_home.path().canonicalize()?),
            "official sessions directory must not be the raw VSP sessions directory"
        );
    }

    Ok(())
}

#[tokio::test]
async fn session_index_thread_name_is_sanitized_before_manifest_title() -> anyhow::Result<()> {
    let vsp_home = TempDir::new()?;
    let official_home = TempDir::new()?;
    write_representative_vsp_home_with_index_title(
        vsp_home.path(),
        "Investigate .pipeline /hw:resume private-vsp sk-index-title-secret",
    )?;

    let export = export_sessions_for_official_codex(vsp_home.path(), official_home.path()).await?;

    assert_eq!(export.sessions.len(), 1);
    let manifest_text = fs::read_to_string(&export.manifest_path)?;
    let exported_title = export.sessions[0].title.as_deref().unwrap_or_default();
    for unsafe_value in [
        ".pipeline",
        "/hw:resume",
        "private-vsp",
        "sk-index-title-secret",
    ] {
        assert!(
            !exported_title.contains(unsafe_value),
            "manifest title leaked unsafe session_index thread_name marker {unsafe_value:?}"
        );
        assert!(
            !manifest_text.contains(unsafe_value),
            "manifest JSON leaked unsafe session_index thread_name marker {unsafe_value:?}"
        );
    }

    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn symlinked_sessions_directory_outside_root_is_not_traversed() -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;

    let vsp_home = TempDir::new()?;
    let official_home = TempDir::new()?;
    let outside = TempDir::new()?;

    let outside_id = "22222222-2222-4222-8222-222222222222";
    let outside_rollout_dir = outside.path().join("2026/05/30");
    fs::create_dir_all(&outside_rollout_dir)?;
    write_rollout_file(
        &outside_rollout_dir.join(format!("rollout-2026-05-30T11-00-00-{outside_id}.jsonl")),
        outside_id,
        "2026-05-30T11:00:00Z",
        "/outside-session-root",
        "Outside symlinked session",
        "Message from outside symlinked session",
    )?;

    let sessions_root = vsp_home.path().join("sessions");
    fs::create_dir_all(&sessions_root)?;
    symlink(outside.path(), sessions_root.join("outside-link"))?;

    let export = export_sessions_for_official_codex(vsp_home.path(), official_home.path()).await?;

    assert!(
        export.sessions.is_empty(),
        "symlinked directory outside sessions root must not be traversed/exported: {:?}",
        export
            .sessions
            .iter()
            .map(|session| (&session.id, &session.sanitized_artifact_path))
            .collect::<Vec<_>>()
    );

    Ok(())
}

#[tokio::test]
async fn sanitized_artifact_is_readable_by_rollout_thread_reader() -> anyhow::Result<()> {
    let vsp_home = TempDir::new()?;
    let official_home = TempDir::new()?;
    write_representative_vsp_home(vsp_home.path())?;

    let export = export_sessions_for_official_codex(vsp_home.path(), official_home.path()).await?;

    assert_eq!(export.sessions.len(), 1);
    let session = &export.sessions[0];
    let item =
        codex_rollout::read_thread_item_from_rollout(session.sanitized_artifact_path.clone())
            .await
            .expect("sanitized artifact should parse through normal rollout reader");

    assert_eq!(
        item.thread_id.as_ref().map(ToString::to_string).as_deref(),
        Some(FIXTURE_ID)
    );
    assert_eq!(item.cwd.as_deref(), Some(Path::new(SAFE_CWD)));
    assert_eq!(
        item.preview.as_deref(),
        Some("List the files for the safe project")
    );
    assert_eq!(item.source, Some(SessionSource::VSCode));

    Ok(())
}

#[tokio::test]
async fn duplicate_source_basenames_do_not_overwrite_sanitized_artifacts() -> anyhow::Result<()> {
    let vsp_home = TempDir::new()?;
    let official_home = TempDir::new()?;
    let file_name = "rollout-2026-05-30T12-00-00-duplicate-basename.jsonl";
    let first_id = "33333333-3333-4333-8333-333333333333";
    let second_id = "44444444-4444-4444-8444-444444444444";
    let first_dir = vsp_home.path().join("sessions/nested/one");
    let second_dir = vsp_home.path().join("sessions/nested/two");
    fs::create_dir_all(&first_dir)?;
    fs::create_dir_all(&second_dir)?;
    write_rollout_file(
        &first_dir.join(file_name),
        first_id,
        "2026-05-30T12:00:00Z",
        "/workspaces/collision-one",
        "Collision one",
        "Preview from collision one",
    )?;
    write_rollout_file(
        &second_dir.join(file_name),
        second_id,
        "2026-05-30T12:00:01Z",
        "/workspaces/collision-two",
        "Collision two",
        "Preview from collision two",
    )?;

    let export = export_sessions_for_official_codex(vsp_home.path(), official_home.path()).await?;

    assert_eq!(export.sessions.len(), 2);
    assert_ne!(
        export.sessions[0].sanitized_artifact_path, export.sessions[1].sanitized_artifact_path,
        "same-basename source rollouts must get distinct sanitized artifact paths"
    );
    let exported_texts = export
        .sessions
        .iter()
        .map(|session| fs::read_to_string(&session.sanitized_artifact_path))
        .collect::<Result<Vec<_>, _>>()?;
    assert!(
        exported_texts.iter().any(|text| text.contains(first_id)),
        "first duplicate-basename session was overwritten or dropped"
    );
    assert!(
        exported_texts.iter().any(|text| text.contains(second_id)),
        "second duplicate-basename session was overwritten or dropped"
    );

    Ok(())
}

#[test]
fn field_classification_fixture_locks_provider_auth_workflow_fields() -> anyhow::Result<()> {
    let cases: Vec<ClassificationCase> = serde_json::from_str(include_str!(
        "fixtures/official_codex_discovery/field_classification.json"
    ))?;

    for case in cases {
        let actual = classify_field_for_official_discovery(&case.field);
        assert_eq!(
            format!("{actual:?}").to_ascii_lowercase(),
            case.expected_class,
            "unexpected official-discovery classification for {}",
            case.field
        );
    }

    Ok(())
}

fn write_representative_vsp_home(vsp_home: &Path) -> anyhow::Result<()> {
    write_representative_vsp_home_with_index_title(vsp_home, SAFE_TITLE)
}

fn write_representative_vsp_home_with_index_title(
    vsp_home: &Path,
    index_title: &str,
) -> anyhow::Result<()> {
    let rollout_dir = vsp_home.join("sessions/2026/05/29");
    fs::create_dir_all(&rollout_dir)?;
    fs::write(
        rollout_dir.join(format!("rollout-{FIXTURE_FILE_TS}-{FIXTURE_ID}.jsonl")),
        include_str!("fixtures/official_codex_discovery/vsp_rollout_with_private_metadata.jsonl"),
    )?;
    fs::write(
        vsp_home.join("session_index.jsonl"),
        format!(
            "{}\n",
            serde_json::json!({
                "id": FIXTURE_ID,
                "thread_name": index_title,
                "updated_at": "2026-05-29T10:01:00Z"
            })
        ),
    )?;
    fs::write(
        vsp_home.join("config.toml"),
        r#"
model_provider = "vsp-private-provider"

[model_providers.vsp-private-provider]
name = "Private VSP Gateway"
base_url = "https://private-vsp-gateway.invalid/v1"
env_key = "VSP_PRIVATE_API_KEY"
"#,
    )?;
    fs::write(
        vsp_home.join("auth.json"),
        r#"{"openai_api_key":"sk-vsp-auth-json-secret","refresh_token":"vsp-auth-refresh-secret"}"#,
    )?;
    let pipeline_dir = vsp_home.join(".pipeline");
    fs::create_dir_all(&pipeline_dir)?;
    fs::write(
        pipeline_dir.join("state.yaml"),
        "cycle: C8\nmilestone: M4\ncurrent_step: implementation-private\n",
    )?;
    Ok(())
}

fn write_rollout_file(
    path: &Path,
    id: &str,
    timestamp: &str,
    cwd: &str,
    title: &str,
    preview: &str,
) -> anyhow::Result<()> {
    let text = format!(
        "{}\n{}\n",
        serde_json::json!({
            "timestamp": timestamp,
            "type": "session_meta",
            "payload": {
                "id": id,
                "timestamp": timestamp,
                "cwd": cwd,
                "originator": "vsp-codex",
                "cli_version": "0.0.0-test",
                "source": "vscode",
                "model_provider": "test-provider",
                "base_instructions": null,
                "title": title
            }
        }),
        serde_json::json!({
            "timestamp": timestamp,
            "type": "event_msg",
            "payload": {
                "type": "user_message",
                "message": preview,
                "kind": "plain"
            }
        })
    );
    fs::write(path, text)?;
    Ok(())
}

fn assert_no_symlink_points_into(root: &Path, forbidden_target_root: &Path) -> anyhow::Result<()> {
    let forbidden_target_root = forbidden_target_root.canonicalize()?;
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let metadata = fs::symlink_metadata(&entry_path)?;
            if metadata.file_type().is_symlink() {
                let target = fs::read_link(&entry_path)?;
                let absolute_target = if target.is_absolute() {
                    target
                } else {
                    entry_path
                        .parent()
                        .expect("symlink has parent")
                        .join(target)
                };
                if let Ok(canonical_target) = absolute_target.canonicalize() {
                    assert!(
                        !canonical_target.starts_with(&forbidden_target_root),
                        "official export must not symlink {} to raw VSP path {}",
                        entry_path.display(),
                        canonical_target.display()
                    );
                }
                continue;
            }
            if metadata.is_dir() {
                stack.push(entry_path);
            }
        }
    }
    Ok(())
}

#[test]
fn fixture_contains_expected_private_values_so_sanitizer_assertions_are_meaningful() {
    let fixture: Vec<Value> =
        include_str!("fixtures/official_codex_discovery/vsp_rollout_with_private_metadata.jsonl")
            .lines()
            .map(|line| serde_json::from_str(line).expect("fixture line is valid JSON"))
            .collect();
    let raw_fixture = serde_json::to_string(&fixture).expect("fixture serializes");

    for unsafe_value in UNSAFE_NEEDLES {
        if *unsafe_value == ".pipeline" {
            continue;
        }
        assert!(
            raw_fixture.contains(unsafe_value),
            "fixture should contain unsafe value {unsafe_value:?}"
        );
    }
}
