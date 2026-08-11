use serde_json::Value;
use sha2::Digest as _;
use sha2::Sha256;
use std::fs;
use std::path::Path;

fn command_entry(suffix: &str) -> Value {
    serde_json::json!({
        "name": format!("hw:{suffix}"),
        "slash": format!("/hw:{suffix}"),
        "skill": format!("hypo-workflow:{suffix}"),
        "path": format!("skills/{suffix}/SKILL.md")
    })
}

fn staged_public_release() -> tempfile::TempDir {
    staged_release(
        [
            "guide",
            "init",
            "goal",
            "plan",
            "cycle",
            "maintain",
            "experiment",
            "resume",
            "accept",
            "reject",
        ]
        .into_iter()
        .map(command_entry)
        .collect(),
    )
}

fn staged_release(commands: Vec<Value>) -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("temporary release");
    let root = temp.path();
    fs::create_dir_all(root.join("contracts/host/v1")).expect("contract directory");
    fs::create_dir_all(root.join(".codex-plugin")).expect("plugin directory");
    fs::write(root.join("SKILL.md"), "# Root Skill\n").expect("root Skill");
    for entry in &commands {
        let path = entry["path"].as_str().expect("command path");
        let destination = root.join(path);
        fs::create_dir_all(destination.parent().expect("Skill parent")).expect("Skill directory");
        fs::write(destination, "# Command Skill\n").expect("command Skill");
    }
    let manifest = serde_json::json!({
        "schema_version": "1",
        "contract_version": "1",
        "root": {
            "name": "hypo-workflow",
            "skill": "hypo-workflow:hypo-workflow",
            "path": "SKILL.md"
        },
        "commands": commands
    });
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).expect("command manifest");
    fs::write(
        root.join("contracts/host/v1/command-manifest.json"),
        &manifest_bytes,
    )
    .expect("write command manifest");
    let checksum = format!("{:x}", Sha256::digest(&manifest_bytes));
    fs::write(
        root.join("contracts/host/v1/installed-release.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "1",
            "contract_version": "1",
            "release": {
                "version": "15.0.0-alpha.1",
                "source_commit": "0123456789abcdef0123456789abcdef01234567"
            },
            "command_manifest": {
                "path": "contracts/host/v1/command-manifest.json",
                "sha256": checksum
            }
        }))
        .expect("release descriptor"),
    )
    .expect("write release descriptor");
    fs::write(
        root.join(".codex-plugin/plugin.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "name": "hypo-workflow",
            "version": "15.0.0-alpha.1"
        }))
        .expect("plugin manifest"),
    )
    .expect("write plugin manifest");
    temp
}

fn parse_from_release(
    input: &str,
    root: &Path,
) -> Option<codex_tui::workflow_slash_routing::WorkflowSlashRoute> {
    codex_tui::workflow_slash_routing::parse_workflow_slash_command_from_release(input, root)
}

#[test]
fn hw_plan_slash_command_is_recognized_as_native_workflow_route() {
    let release = staged_public_release();
    let route = parse_from_release("/hw:plan refine", release.path())
        .expect("/hw:* input should be routed");

    assert_eq!(route.command(), "plan");
    assert_eq!(route.arguments(), "refine");
    assert!(route.is_hypo_workflow());
}

#[test]
fn release_manifest_commands_are_recognized_as_native_workflow_routes() {
    let release = staged_public_release();
    for expected in [
        "accept",
        "cycle",
        "experiment",
        "goal",
        "guide",
        "init",
        "maintain",
        "plan",
        "reject",
        "resume",
    ] {
        let input = format!("/hw:{expected} example");
        let route = parse_from_release(&input, release.path())
            .expect("Host Contract v1 public command should be routed");

        assert_eq!(route.command(), expected);
        assert!(route.is_hypo_workflow());
    }
}

#[test]
fn legacy_internal_and_unknown_hw_commands_are_not_public_routes() {
    let release = staged_public_release();
    for command in [
        "status",
        "start",
        "stop",
        "audit",
        "quality",
        "optimize",
        "rules",
        "patch",
        "does-not-exist",
    ] {
        assert!(
            parse_from_release(&format!("/hw:{command} now"), release.path()).is_none(),
            "{command} is not in the Host Contract v1 public command manifest"
        );
    }
}

#[test]
fn hypo_workflow_dollar_command_is_recognized_as_native_workflow_route() {
    let release = staged_public_release();
    let route = parse_from_release("$hypo-workflow:goal", release.path())
        .expect("$hypo-workflow:* input should be routed");

    assert_eq!(route.command(), "goal");
    assert_eq!(route.arguments(), "");
    assert!(route.is_hypo_workflow());
}

#[test]
fn unrelated_slash_commands_are_not_workflow_routes() {
    let release = staged_public_release();
    for input in [
        "/review foo",
        "/compact",
        "/model gpt-5",
        "/hw",
        "/hw plan",
        "/integration-sync apply",
        "/quality baseline",
        "/optimize loop",
    ] {
        assert!(
            parse_from_release(input, release.path()).is_none(),
            "{input:?} must not be classified as a Hypo-Workflow route"
        );
    }
}

#[test]
fn plain_text_is_not_a_workflow_route() {
    let release = staged_public_release();
    for input in ["hello", "continue the implementation", ""] {
        assert!(
            parse_from_release(input, release.path()).is_none(),
            "{input:?} must not be classified as a Hypo-Workflow route"
        );
    }
}

#[test]
fn workflow_commands_are_discoverable_only_for_workflow_projects() {
    let release = staged_public_release();
    let workflow_entries =
        codex_tui::workflow_slash_routing::workflow_command_palette_entries_from_release(
            true,
            release.path(),
        );
    let actual = workflow_entries
        .iter()
        .map(|entry| entry.command.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            "/hw:guide",
            "/hw:init",
            "/hw:goal",
            "/hw:plan",
            "/hw:cycle",
            "/hw:maintain",
            "/hw:experiment",
            "/hw:resume",
            "/hw:accept",
            "/hw:reject",
        ],
        "palette must project the Host Contract v1 command manifest exactly"
    );
    assert!(
        workflow_entries
            .iter()
            .all(|entry| entry.description.contains("Hypo-Workflow")),
        "Workflow command descriptions should identify their owner"
    );

    let non_workflow_entries =
        codex_tui::workflow_slash_routing::workflow_command_palette_entries_from_release(
            false,
            release.path(),
        );
    assert!(
        non_workflow_entries.is_empty(),
        "non-Workflow projects should not be cluttered with active /hw:* commands: {non_workflow_entries:?}"
    );
}

#[test]
fn manifest_declared_future_command_is_accepted_without_a_hard_coded_allowlist() {
    let release = staged_release(vec![command_entry("future")]);

    let route = parse_from_release("/hw:future now", release.path())
        .expect("manifest-declared command should be routed");

    assert_eq!(route.command(), "future");
    assert_eq!(route.arguments(), "now");
    assert!(parse_from_release("/hw:undeclared", release.path()).is_none());
}

#[test]
fn duplicate_illegal_unsafe_and_excessive_command_manifests_are_rejected() {
    let duplicate = staged_release(vec![command_entry("goal"), command_entry("goal")]);
    assert!(parse_from_release("/hw:goal", duplicate.path()).is_none());

    let illegal = staged_release(vec![serde_json::json!({
        "name": "hw:Bad_Name",
        "slash": "/hw:Bad_Name",
        "skill": "hypo-workflow:Bad_Name",
        "path": "skills/Bad_Name/SKILL.md"
    })]);
    assert!(parse_from_release("/hw:Bad_Name", illegal.path()).is_none());

    let unsafe_path = staged_release(vec![serde_json::json!({
        "name": "hw:escape",
        "slash": "/hw:escape",
        "skill": "hypo-workflow:escape",
        "path": "skills/escape/../SKILL.md"
    })]);
    assert!(parse_from_release("/hw:escape", unsafe_path.path()).is_none());

    let excessive = staged_release(
        (0..65)
            .map(|index| command_entry(&format!("cmd-{index}")))
            .collect(),
    );
    assert!(
        codex_tui::workflow_slash_routing::workflow_command_palette_entries_from_release(
            true,
            excessive.path(),
        )
        .is_empty()
    );
}

#[cfg(unix)]
#[test]
fn symlinked_command_skill_is_rejected() {
    use std::os::unix::fs::symlink;

    let release = staged_release(vec![command_entry("goal")]);
    let outside = tempfile::tempdir().expect("outside directory");
    let skill = release.path().join("skills/goal/SKILL.md");
    fs::remove_file(&skill).expect("remove staged Skill");
    fs::write(outside.path().join("SKILL.md"), "# Escaped Skill\n").expect("outside Skill");
    symlink(outside.path().join("SKILL.md"), skill).expect("symlink command Skill");

    assert!(parse_from_release("/hw:goal", release.path()).is_none());
}

#[test]
fn workflow_status_summary_reports_read_only_host_contract_without_legacy_claims() {
    let summary = codex_tui::workflow_slash_routing::workflow_status_summary(true).join("\n");

    for expected in [
        "Workflow project: detected",
        "Host Contract v1",
        "read-only",
    ] {
        assert!(
            summary.contains(expected),
            "Workflow status summary missing {expected:?}:\n{summary}"
        );
    }
    for retired_claim in [
        "compact fallback: enabled",
        "ordinary capture: enabled",
        "quality commands: /hw:audit, /hw:quality, /hw:optimize",
    ] {
        assert!(
            !summary.contains(retired_claim),
            "Workflow status must not claim retired host behavior {retired_claim:?}:\n{summary}"
        );
    }

    let non_workflow_summary =
        codex_tui::workflow_slash_routing::workflow_status_summary(false).join("\n");
    assert!(
        non_workflow_summary.contains("Workflow project: not detected"),
        "non-Workflow status should state that Workflow was not detected:\n{non_workflow_summary}"
    );
    assert!(
        !non_workflow_summary.contains("Host Contract v1")
            && !non_workflow_summary.contains("read-only"),
        "non-Workflow status must not claim Workflow runtime features are enabled:\n{non_workflow_summary}"
    );
}
