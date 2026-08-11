use codex_core::workflow_routing::find_workflow_root;
use codex_core::workflow_routing::workflow_runtime_guidance;
use pretty_assertions::assert_eq;
use std::fs;

#[test]
fn non_workflow_project_omits_host_guidance() {
    let root = tempfile::tempdir().expect("temporary project");
    assert_eq!(find_workflow_root(root.path()), None);
    assert_eq!(workflow_runtime_guidance(root.path()), None);
}

#[test]
fn current_workspace_receives_thin_read_only_host_guidance() {
    let root = tempfile::tempdir().expect("temporary project");
    fs::create_dir_all(root.path().join(".pipeline/runtime")).expect("pipeline runtime");
    fs::write(
        root.path().join(".pipeline/runtime/host-status-v1.json"),
        r#"{
          "schema_version":"1",
          "contract_version":"1",
          "projection_status":"current",
          "generated_at":"2026-07-13T00:00:00Z",
          "generation":1,
          "workspace":{"format_version":"1","status":"ready"},
          "delivery":null,
          "continuation":null,
          "invalidation":null
        }"#,
    )
    .expect("host status");

    let guidance = workflow_runtime_guidance(root.path()).expect("workflow guidance");
    assert!(guidance.contains("installed plugin"));
    assert!(guidance.contains("VSP-Codex provides host UX only"));
    assert!(guidance.contains("projection is current"));
    assert!(!guidance.contains("/hw:audit"));
    assert!(!guidance.contains("/hw:quality"));
}

#[test]
fn invalidated_projection_fails_closed_without_legacy_fallback() {
    let root = tempfile::tempdir().expect("temporary project");
    fs::create_dir_all(root.path().join(".pipeline/runtime")).expect("pipeline runtime");
    fs::write(
        root.path().join(".pipeline/runtime/host-status-v1.json"),
        r#"{
          "schema_version":"1",
          "contract_version":"1",
          "projection_status":"invalidated",
          "generated_at":"2026-07-13T00:00:00Z",
          "generation":2,
          "workspace":null,
          "delivery":null,
          "continuation":null,
          "invalidation":{"invalidated_at":"2026-07-13T00:00:00Z","reason":"authority_changed"}
        }"#,
    )
    .expect("host status");

    let guidance = workflow_runtime_guidance(root.path()).expect("workflow guidance");
    assert!(guidance.contains("fail closed"));
    assert!(guidance.contains("invalidated"));
}

#[test]
fn legacy_workspace_is_diagnostic_only_and_routes_adoption_to_init() {
    let root = tempfile::tempdir().expect("temporary project");
    fs::create_dir_all(root.path().join(".pipeline")).expect("pipeline");
    fs::write(
        root.path().join(".pipeline/state.yaml"),
        "phase: executing\n",
    )
    .expect("legacy state");

    let guidance = workflow_runtime_guidance(root.path()).expect("workflow guidance");
    assert!(guidance.contains("Legacy Workflow state"));
    assert!(guidance.contains("/hw:init"));
    assert_eq!(
        fs::read_to_string(root.path().join(".pipeline/state.yaml")).expect("legacy state"),
        "phase: executing\n"
    );
}

#[test]
fn semantic_workspace_ignores_invalidated_legacy_projection_without_writing() {
    let root = tempfile::tempdir().expect("temporary project");
    let pipeline = root.path().join(".pipeline");
    fs::create_dir_all(pipeline.join("cycles/C22")).expect("semantic cycle directory");
    fs::create_dir_all(pipeline.join("runtime")).expect("runtime directory");
    fs::write(pipeline.join("INDEX.md"), "# Semantic index\n").expect("semantic index");
    fs::write(pipeline.join("cycles/C22/PLAN.md"), "# C22\n").expect("semantic plan");
    fs::write(
        pipeline.join("runtime/host-status-v1.json"),
        r#"{
          "schema_version":"1",
          "contract_version":"1",
          "projection_status":"invalidated",
          "generated_at":"2026-08-05T00:00:00Z",
          "generation":3,
          "workspace":null,
          "delivery":null,
          "continuation":null,
          "work_items":null,
          "session":null,
          "invalidation":{"invalidated_at":"2026-08-05T00:00:00Z","reason":"semantic_workspace"}
        }"#,
    )
    .expect("invalidated legacy projection");
    let tracked_files = [
        pipeline.join("INDEX.md"),
        pipeline.join("cycles/C22/PLAN.md"),
        pipeline.join("runtime/host-status-v1.json"),
    ];
    let before = tracked_files
        .iter()
        .map(fs::read)
        .collect::<Result<Vec<_>, _>>()
        .expect("tracked workflow bytes");

    let guidance = workflow_runtime_guidance(root.path()).expect("workflow guidance");

    assert!(guidance.contains("semantic index"));
    assert!(guidance.contains("current Cycle"));
    assert!(guidance.contains("installed plugin"));
    assert!(guidance.contains("Host UX only"));
    assert!(guidance.contains("does not require Host Contract v1"));
    assert!(!guidance.contains("fail closed"));
    assert!(!guidance.contains("refresh it"));
    let after = tracked_files
        .iter()
        .map(fs::read)
        .collect::<Result<Vec<_>, _>>()
        .expect("tracked workflow bytes");
    assert_eq!(after, before);
}

#[test]
fn semantic_workspace_without_legacy_projection_uses_semantic_guidance() {
    let root = tempfile::tempdir().expect("temporary project");
    let pipeline = root.path().join(".pipeline");
    fs::create_dir_all(pipeline.join("cycles")).expect("semantic cycles directory");
    fs::write(pipeline.join("INDEX.md"), "# Semantic index\n").expect("semantic index");

    let guidance = workflow_runtime_guidance(root.path()).expect("workflow guidance");

    assert!(guidance.contains("semantic index"));
    assert!(!guidance.contains("No Host Contract projection"));
}

#[cfg(unix)]
#[test]
fn semantic_workspace_rejects_symlinked_index() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("temporary project");
    let outside = tempfile::tempdir().expect("outside directory");
    let pipeline = root.path().join(".pipeline");
    fs::create_dir_all(pipeline.join("cycles")).expect("semantic cycles directory");
    fs::write(outside.path().join("INDEX.md"), "# Escaped index\n").expect("outside index");
    symlink(outside.path().join("INDEX.md"), pipeline.join("INDEX.md"))
        .expect("symlink semantic index");

    let guidance = workflow_runtime_guidance(root.path()).expect("workflow guidance");

    assert!(!guidance.contains("semantic index"));
}
