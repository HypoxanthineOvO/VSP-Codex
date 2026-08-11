use serde_json::Value;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

const EXPECTED_COMMANDS: &[&str] = &[
    "hw:accept",
    "hw:cycle",
    "hw:experiment",
    "hw:goal",
    "hw:guide",
    "hw:init",
    "hw:maintain",
    "hw:plan",
    "hw:reject",
    "hw:resume",
];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("Codex-VSP repository root")
        .to_path_buf()
}

fn hypo_workflow_root() -> PathBuf {
    std::env::var_os("HYPO_WORKFLOW_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            repository_root()
                .parent()
                .expect("shared workspace parent")
                .join("Hypo-Workflow")
        })
}

fn read(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("read {}: {error}", path.as_ref().display()))
}

#[test]
#[ignore = "manual cross-repository contract; requires HYPO_WORKFLOW_ROOT or a sibling checkout"]
fn workflow_command_surface_comes_from_the_release_manifest() {
    let contract = hypo_workflow_root().join("contracts/host/v1");
    let manifest: Value = serde_json::from_str(&read(contract.join("command-manifest.json")))
        .expect("parse Host Contract command manifest");
    let mut actual = manifest["commands"]
        .as_array()
        .expect("commands array")
        .iter()
        .map(|entry| entry["name"].as_str().expect("command name"))
        .collect::<Vec<_>>();
    actual.sort_unstable();

    assert_eq!(actual, EXPECTED_COMMANDS);

    let routing = read(repository_root().join("codex-rs/tui/src/workflow_slash_routing.rs"));
    assert!(routing.contains("installed-release.json"));
    assert!(routing.contains("Sha256::digest"));
    assert!(!routing.contains("join(\"Hypo-Workflow\")"));
    for retired in [
        "/hw:status",
        "/hw:start",
        "/hw:stop",
        "/hw:audit",
        "/hw:quality",
    ] {
        assert!(
            !routing.contains(&format!("command: \"{retired}\"")),
            "retired or internal command {retired} must not remain hard-coded"
        );
    }
}

#[test]
fn host_projection_reader_is_json_only_and_has_no_write_authority() {
    let source = read(repository_root().join("codex-rs/core/src/host_contract.rs"));
    assert!(source.contains(".pipeline/runtime/host-status-v1.json"));
    assert!(source.contains("contract_version"));
    for forbidden in [
        "fs::write",
        "std::fs::write",
        "OpenOptions",
        "atomic_write",
        "write_all",
        "create_dir_all",
    ] {
        assert!(
            !source.contains(forbidden),
            "Host Contract consumer must remain read-only: found {forbidden}"
        );
    }
}

#[test]
#[ignore = "manual cross-repository contract; requires HYPO_WORKFLOW_ROOT or a sibling checkout"]
fn host_projection_reader_rejects_shared_sensitive_and_nested_unknown_fields() {
    use codex_core::host_contract::HostContractStatus;
    use codex_core::host_contract::read_host_status;

    let fixture_root = hypo_workflow_root().join("contracts/host/v1/fixtures");
    let temp = tempfile::tempdir().expect("temporary workspace");
    let runtime = temp.path().join(".pipeline/runtime");
    fs::create_dir_all(&runtime).expect("create runtime directory");
    fs::copy(
        fixture_root.join("host-status.current.json"),
        runtime.join("host-status-v1.json"),
    )
    .expect("copy current projection");
    assert_eq!(
        read_host_status(temp.path()).status,
        HostContractStatus::Current
    );

    fs::copy(
        fixture_root.join("host-status.secret-rejected.json"),
        runtime.join("host-status-v1.json"),
    )
    .expect("copy sensitive projection");
    assert!(matches!(
        read_host_status(temp.path()).status,
        HostContractStatus::Invalid(_)
    ));

    let mut nested: Value =
        serde_json::from_str(&read(fixture_root.join("host-status.current.json")))
            .expect("parse current projection");
    nested["workspace"]["private_runtime"] = Value::String("must fail closed".to_string());
    fs::write(
        runtime.join("host-status-v1.json"),
        serde_json::to_vec(&nested).expect("serialize nested projection"),
    )
    .expect("write nested projection");
    assert!(matches!(
        read_host_status(temp.path()).status,
        HostContractStatus::Invalid(_)
    ));
}

#[test]
#[ignore = "manual cross-repository contract; requires HYPO_WORKFLOW_ROOT or a sibling checkout"]
fn host_projection_reader_accepts_plan_work_items_and_session() {
    use codex_core::host_contract::HostContractStatus;
    use codex_core::host_contract::read_host_status;

    let fixture_root = hypo_workflow_root().join("contracts/host/v1/fixtures");
    let temp = tempfile::tempdir().expect("temporary workspace");
    let runtime = temp.path().join(".pipeline/runtime");
    fs::create_dir_all(&runtime).expect("create runtime directory");
    let mut projection: Value =
        serde_json::from_str(&read(fixture_root.join("host-status.current.json")))
            .expect("parse current projection");
    projection["delivery"]["kind"] = Value::String("plan".to_string());
    projection["work_items"] = serde_json::json!([{
        "kind": "delivery",
        "id": "plan-c22",
        "status": "active",
        "placement_ids": ["placement-c22"]
    }]);
    projection["session"] = serde_json::json!({
        "host": "codex",
        "session_id": "session-c22",
        "status": "selected",
        "work_item_ref": {"kind": "delivery", "id": "plan-c22"}
    });
    fs::write(
        runtime.join("host-status-v1.json"),
        serde_json::to_vec(&projection).expect("serialize projection"),
    )
    .expect("write current projection");

    let read = read_host_status(temp.path());
    assert_eq!(read.status, HostContractStatus::Current);
    assert_eq!(read.projection, Some(projection));

    let mut nullable_projection = read.projection.expect("current projection");
    nullable_projection["work_items"] = Value::Null;
    nullable_projection["session"] = Value::Null;
    fs::write(
        runtime.join("host-status-v1.json"),
        serde_json::to_vec(&nullable_projection).expect("serialize nullable projection"),
    )
    .expect("write nullable projection");
    assert_eq!(
        read_host_status(temp.path()).status,
        HostContractStatus::Current
    );
}

#[test]
#[ignore = "manual cross-repository contract; requires HYPO_WORKFLOW_ROOT or a sibling checkout"]
fn host_projection_reader_rejects_duplicate_work_item_placements() {
    use codex_core::host_contract::HostContractStatus;
    use codex_core::host_contract::read_host_status;

    let fixture_root = hypo_workflow_root().join("contracts/host/v1/fixtures");
    let temp = tempfile::tempdir().expect("temporary workspace");
    let runtime = temp.path().join(".pipeline/runtime");
    fs::create_dir_all(&runtime).expect("create runtime directory");
    let mut projection: Value =
        serde_json::from_str(&read(fixture_root.join("host-status.current.json")))
            .expect("parse current projection");
    projection["work_items"] = serde_json::json!([{
        "kind": "experiment",
        "id": "experiment-c22",
        "status": "active",
        "placement_ids": ["duplicate", "duplicate"]
    }]);
    projection["session"] = Value::Null;
    fs::write(
        runtime.join("host-status-v1.json"),
        serde_json::to_vec(&projection).expect("serialize projection"),
    )
    .expect("write current projection");

    assert!(matches!(
        read_host_status(temp.path()).status,
        HostContractStatus::Invalid(_)
    ));
}

#[test]
fn legacy_workflow_writers_and_conversation_capture_are_unreachable() {
    let core_lib = read(repository_root().join("codex-rs/core/src/lib.rs"));
    let session = read(repository_root().join("codex-rs/core/src/session/mod.rs"));

    assert!(!core_lib.contains("pub mod workflow_files"));
    assert!(!core_lib.contains("pub mod workflow_conversation_capture"));
    assert!(!session.contains("ConversationCapture"));
    assert!(!session.contains("maybe_capture_workflow_turn_complete"));
    assert!(!session.contains("WorkflowFiles"));
}

#[test]
fn generic_ask_plan_and_subagent_capabilities_remain_owned_by_codex() {
    let suite = read(repository_root().join("codex-rs/core/tests/suite/mod.rs"));
    for capability in [
        "mod request_user_input;",
        "mod plan_update_gate;",
        "mod spawn_agent_description;",
        "mod subagent_notifications;",
    ] {
        assert!(
            suite.contains(capability),
            "Workflow convergence must not remove generic Codex capability: {capability}"
        );
    }
}
