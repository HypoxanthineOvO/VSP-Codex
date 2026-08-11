use anyhow::Result;
use codex_core::compact::SUMMARIZATION_PROMPT;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::built_in_model_providers;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use core_test_support::responses;
use core_test_support::responses::ResponsesRequest;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_once;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::future::Future;
use wiremock::MockServer;

const PLAN_CALL_ID: &str = "plan-update-gate-call";
const WORKFLOW_TEST_STACK_SIZE_BYTES: usize = 16 * 1024 * 1024;

fn run_workflow_runtime_test<F>(future: F) -> Result<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    let handle = std::thread::Builder::new()
        .name("plan-update-gate-runtime-test".to_string())
        .stack_size(WORKFLOW_TEST_STACK_SIZE_BYTES)
        .spawn(move || -> Result<()> {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_stack_size(WORKFLOW_TEST_STACK_SIZE_BYTES)
                .enable_all()
                .build()?;
            runtime.block_on(future)
        })?;

    match handle.join() {
        Ok(result) => result,
        Err(_) => Err(anyhow::anyhow!("plan update gate runtime test panicked")),
    }
}

fn active_plan_args() -> String {
    json!({
        "explanation": "Plan update gate coverage",
        "plan": [
            {"step": "Inspect stale plan", "status": "in_progress"},
            {"step": "Report freshness", "status": "pending"}
        ],
    })
    .to_string()
}

async fn seed_active_plan(test: &TestCodex, server: &MockServer) -> Result<()> {
    mount_sse_sequence(
        server,
        vec![
            sse(vec![
                ev_response_created("resp-seed-plan"),
                ev_function_call(PLAN_CALL_ID, "update_plan", &active_plan_args()),
                ev_completed("resp-seed-plan"),
            ]),
            sse(vec![
                ev_response_created("resp-seed-plan-followup"),
                ev_assistant_message("msg-seed-plan", "plan seeded"),
                ev_completed("resp-seed-plan-followup"),
            ]),
        ],
    )
    .await;

    test.submit_turn("start tracked work").await
}

fn plan_update_reminder_texts(request: &ResponsesRequest) -> Vec<String> {
    request
        .message_input_texts("developer")
        .into_iter()
        .chain(request.message_input_texts("user"))
        .filter(|text| is_plan_update_reminder(text))
        .collect()
}

fn is_plan_update_reminder(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("active plan")
        && text.contains("update_plan")
        && (text.contains("refresh") || text.contains("review") || text.contains("stale"))
        && (text.contains("resume")
            || text.contains("compact")
            || text.contains("compaction")
            || text.contains("compressed")
            || text.contains("context"))
}

fn assert_single_plan_update_reminder(request: &ResponsesRequest) {
    let reminders = plan_update_reminder_texts(request);
    assert_eq!(
        reminders.len(),
        1,
        "expected exactly one active Plan refresh reminder that tells the model to call update_plan; request body: {:#?}",
        request.body_json()
    );
}

fn assert_no_plan_update_reminder(request: &ResponsesRequest) {
    let reminders = plan_update_reminder_texts(request);
    assert_eq!(
        reminders,
        Vec::<String>::new(),
        "did not expect an active Plan refresh reminder immediately after a fresh update_plan call; request body: {:#?}",
        request.body_json()
    );
}

fn non_openai_model_provider(server: &MockServer) -> ModelProviderInfo {
    let mut provider = built_in_model_providers(/*openai_base_url*/ None)["openai"].clone();
    provider.name = "OpenAI (plan update gate test)".into();
    provider.base_url = Some(format!("{}/v1", server.uri()));
    provider.supports_websockets = false;
    provider
}

#[test]
fn resume_with_active_plan_injects_update_plan_reminder() -> Result<()> {
    run_workflow_runtime_test(resume_with_active_plan_injects_update_plan_reminder_impl())
}

async fn resume_with_active_plan_injects_update_plan_reminder_impl() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let mut builder = test_codex();
    let initial = builder.build(&server).await?;
    let home = initial.home.clone();
    let rollout_path = initial
        .session_configured
        .rollout_path
        .clone()
        .expect("rollout path");

    seed_active_plan(&initial, &server).await?;

    let resumed_mock = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-resumed"),
            ev_assistant_message("msg-resumed", "resumed work"),
            ev_completed("resp-resumed"),
        ]),
    )
    .await;

    let mut resume_builder = test_codex();
    let resumed = resume_builder.resume(&server, home, rollout_path).await?;
    resumed
        .submit_turn("continue tracked work after resume")
        .await?;

    assert_single_plan_update_reminder(&resumed_mock.single_request());

    Ok(())
}

#[test]
fn compaction_with_active_plan_injects_update_plan_reminder_on_next_turn() -> Result<()> {
    run_workflow_runtime_test(
        compaction_with_active_plan_injects_update_plan_reminder_on_next_turn_impl(),
    )
}

async fn compaction_with_active_plan_injects_update_plan_reminder_on_next_turn_impl() -> Result<()>
{
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let model_provider = non_openai_model_provider(&server);
    let mut builder = test_codex().with_config(move |config| {
        config.model_provider = model_provider;
        config.compact_prompt = Some(SUMMARIZATION_PROMPT.to_string());
    });
    let test = builder.build(&server).await?;

    seed_active_plan(&test, &server).await?;

    mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-compact"),
            ev_assistant_message("msg-compact", "compacted summary"),
            ev_completed("resp-compact"),
        ]),
    )
    .await;
    test.codex.submit(Op::Compact).await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;

    let post_compact_mock = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-post-compact"),
            ev_assistant_message("msg-post-compact", "post compact work"),
            ev_completed("resp-post-compact"),
        ]),
    )
    .await;
    test.submit_turn("continue tracked work after compaction")
        .await?;

    assert_single_plan_update_reminder(&post_compact_mock.single_request());

    Ok(())
}

#[test]
fn fresh_update_plan_followup_does_not_repeat_plan_reminder() -> Result<()> {
    run_workflow_runtime_test(fresh_update_plan_followup_does_not_repeat_plan_reminder_impl())
}

async fn fresh_update_plan_followup_does_not_repeat_plan_reminder_impl() -> Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let mut builder = test_codex();
    let test = builder.build(&server).await?;

    responses::mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-fresh-plan"),
            ev_function_call(PLAN_CALL_ID, "update_plan", &active_plan_args()),
            ev_completed("resp-fresh-plan"),
        ]),
    )
    .await;
    let followup_mock = responses::mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resp-fresh-followup"),
            ev_assistant_message("msg-fresh-followup", "fresh plan acknowledged"),
            ev_completed("resp-fresh-followup"),
        ]),
    )
    .await;

    test.submit_turn("start tracked work").await?;

    assert_no_plan_update_reminder(&followup_mock.single_request());

    Ok(())
}
