use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn missing_user_policy_uses_all_built_in_classes() {
    let temp_dir = tempfile::tempdir().expect("temp dir");

    let policy = load_worker_routing_policy(temp_dir.path())
        .await
        .expect("built-in policy");

    assert_eq!(policy.policy_version, BUILT_IN_POLICY_VERSION);
    assert_eq!(policy.enabled, true);
    assert_eq!(policy.allow_explicit_overrides, true);
    assert_eq!(policy.mechanical.len(), 1);
    assert_eq!(policy.long_context.len(), 1);
    assert_eq!(policy.standard.len(), 1);
    assert_eq!(policy.explore.len(), 2);
    assert_eq!(policy.critical.len(), 1);
    assert_eq!(policy.escalation.len(), 1);
}

#[tokio::test]
async fn user_policy_replaces_only_declared_classes() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    tokio::fs::write(
        temp_dir.path().join(WORKER_ROUTING_FILE),
        r#"[worker_routing]
policy_version = "local-v2"
allow_explicit_overrides = false

[[worker_routing.mechanical]]
id = "local-mechanical"
model_provider = "local"
model = "small"
reasoning_effort = "low"
"#,
    )
    .await
    .expect("write policy");

    let policy = load_worker_routing_policy(temp_dir.path())
        .await
        .expect("overlaid policy");

    assert_eq!(policy.policy_version, "local-v2");
    assert_eq!(policy.allow_explicit_overrides, false);
    assert_eq!(
        policy.mechanical,
        vec![WorkerRoutingCandidate {
            id: "local-mechanical".to_string(),
            model_provider: "local".to_string(),
            model: "small".to_string(),
            reasoning_effort: ReasoningEffort::Low,
            service_tier: None,
            min_context_window: None,
        }]
    );
    assert_eq!(policy.long_context, built_in_policy().long_context);
}

#[tokio::test]
async fn invalid_policy_error_does_not_echo_source_contents() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let secret = "do-not-echo-this-secret";
    tokio::fs::write(
        temp_dir.path().join(WORKER_ROUTING_FILE),
        format!("[worker_routing]\napi_key = \"{secret}\"\n"),
    )
    .await
    .expect("write policy");

    let error = load_worker_routing_policy(temp_dir.path())
        .await
        .expect_err("unknown fields should be rejected");

    assert_eq!(error.contains(secret), false);
    assert_eq!(error.contains("invalid near byte"), true);
}

#[test]
fn compatibility_reports_context_before_service_tier() {
    let mut model_info = codex_models_manager::model_info::model_info_from_slug("gpt-5.6-luna");
    model_info.context_window = Some(200_000);
    model_info.service_tiers.clear();
    let candidate = WorkerRoutingCandidate {
        id: "long".to_string(),
        model_provider: "vsp".to_string(),
        model: "gpt-5.6-luna".to_string(),
        reasoning_effort: ReasoningEffort::Medium,
        service_tier: Some("fast".to_string()),
        min_context_window: Some(300_000),
    };

    assert_eq!(
        incompatible_candidate_reason(&candidate, &model_info),
        Some(WorkerRoutingSkipReason::ContextWindowTooSmall)
    );
}
