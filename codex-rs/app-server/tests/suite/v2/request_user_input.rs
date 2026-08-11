use anyhow::Context;
use anyhow::Result;
use app_test_support::TestAppServer;
use app_test_support::create_final_assistant_message_sse_response;
use app_test_support::create_mock_responses_server_sequence;
use app_test_support::create_mock_responses_server_sequence_unchecked;
use app_test_support::create_request_user_input_sse_response;
use app_test_support::to_response;
use codex_app_server_protocol::ClientInfo;
use codex_app_server_protocol::InitializeCapabilities;
use codex_app_server_protocol::ItemCompletedNotification;
use codex_app_server_protocol::ItemStartedNotification;
use codex_app_server_protocol::JSONRPCMessage;
use codex_app_server_protocol::JSONRPCResponse;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ServerRequest;
use codex_app_server_protocol::ServerRequestResolvedNotification;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::ThreadResumeParams;
use codex_app_server_protocol::ThreadResumeResponse;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::ThreadStatus;
use codex_app_server_protocol::TurnCompletedNotification;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::TurnStatus;
use codex_app_server_protocol::UserInput as V2UserInput;
use codex_protocol::config_types::CollaborationMode;
use codex_protocol::config_types::ModeKind;
use codex_protocol::config_types::Settings;
use codex_protocol::openai_models::ReasoningEffort;
use core_test_support::responses;
use serde_json::Value;
use serde_json::json;
use tokio::time::timeout;

const DEFAULT_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const FALLBACK_PROMPT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

fn create_request_user_input_sse_response_with_auto_resolution(
    call_id: &str,
    auto_resolution_ms: u64,
) -> anyhow::Result<String> {
    let tool_call_arguments = serde_json::to_string(&json!({
        "questions": [{
            "id": "confirm_path",
            "header": "Confirm",
            "question": "Proceed with the plan?",
            "options": [{
                "label": "Yes (Recommended)",
                "description": "Continue the current plan."
            }, {
                "label": "No",
                "description": "Stop and revisit the approach."
            }]
        }],
        "autoResolutionMs": auto_resolution_ms
    }))?;

    Ok(responses::sse(vec![
        responses::ev_response_created("resp-1"),
        responses::ev_function_call(call_id, "request_user_input", &tool_call_arguments),
        responses::ev_completed("resp-1"),
    ]))
}

fn create_request_user_input_sse_response_with_visible_context(
    call_id: &str,
) -> anyhow::Result<String> {
    let tool_call_arguments = serde_json::to_string(&json!({
        "summary": "Confirm the implementation direction",
        "description": "Use the ordinary chat explanation above as the decision context.",
        "questions": [{
            "id": "confirm_path",
            "kind": "singleChoice",
            "header": "Confirm",
            "question": "Proceed with the plan?",
            "options": [{
                "id": "yes",
                "label": "Yes",
                "description": "Continue the current plan."
            }, {
                "id": "no",
                "label": "No",
                "description": "Stop and revisit the approach."
            }]
        }]
    }))?;

    Ok(responses::sse(vec![
        responses::ev_response_created("resp-1"),
        responses::ev_assistant_message(
            "context-msg",
            "Conclusion: proceed with the focused fix. Evidence: tests cover the native Ask route. Answer impact: Yes continues; No reopens the plan. Risks: remaining rollout validation.",
        ),
        responses::ev_function_call(call_id, "request_user_input", &tool_call_arguments),
        responses::ev_completed("resp-1"),
    ]))
}

fn create_request_user_input_sse_response_with_card_context(
    call_id: &str,
) -> anyhow::Result<String> {
    let tool_call_arguments = serde_json::to_string(&json!({
        "summary": "Workflow decision required",
        "description": "Review the report summary before choosing the next Workflow step.",
        "questions": [{
            "id": "confirm_path",
            "kind": "singleChoice",
            "header": "Confirm",
            "question": "Proceed with the plan?",
            "options": [{
                "id": "yes",
                "label": "Yes",
                "description": "Continue the current plan."
            }, {
                "id": "no",
                "label": "No",
                "description": "Stop and revisit the approach."
            }]
        }]
    }))?;

    Ok(responses::sse(vec![
        responses::ev_response_created("resp-1"),
        responses::ev_function_call(call_id, "request_user_input", &tool_call_arguments),
        responses::ev_completed("resp-1"),
    ]))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_user_input_round_trip() -> Result<()> {
    let codex_home = tempfile::TempDir::new()?;
    let responses = vec![
        create_request_user_input_sse_response_with_auto_resolution(
            "call1", /*auto_resolution_ms*/ 60_000,
        )?,
        create_final_assistant_message_sse_response("done")?,
    ];
    let server = create_mock_responses_server_sequence(responses).await;
    create_config_toml(codex_home.path(), &server.uri())?;

    let mut mcp = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build()
        .await?;
    let initialized = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.initialize_with_capabilities(
            ClientInfo {
                name: "codex-app-server-tests".to_string(),
                title: None,
                version: "0.1.0".to_string(),
            },
            Some(InitializeCapabilities {
                experimental_api: true,
                native_ask_ui: true,
                ..Default::default()
            }),
        ),
    )
    .await??;
    let JSONRPCMessage::Response(_) = initialized else {
        panic!("expected initialize response");
    };

    let thread_start_id = mcp
        .send_thread_start_request_with_auto_env(ThreadStartParams {
            model: Some("mock-model".to_string()),
            ..Default::default()
        })
        .await?;
    let thread_start_resp: JSONRPCResponse = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(thread_start_id)),
    )
    .await??;
    let ThreadStartResponse { thread, .. } = to_response(thread_start_resp)?;

    let turn_start_id = mcp
        .send_turn_start_request(TurnStartParams {
            thread_id: thread.id.clone(),
            client_user_message_id: None,
            input: vec![V2UserInput::Text {
                text: "ask something".to_string(),
                text_elements: Vec::new(),
            }],
            model: Some("mock-model".to_string()),
            effort: Some(ReasoningEffort::Medium),
            collaboration_mode: Some(CollaborationMode {
                mode: ModeKind::Plan,
                settings: Settings {
                    model: "mock-model".to_string(),
                    reasoning_effort: Some(ReasoningEffort::Medium),
                    developer_instructions: None,
                },
            }),
            ..Default::default()
        })
        .await?;
    let turn_start_resp: JSONRPCResponse = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(turn_start_id)),
    )
    .await??;
    let TurnStartResponse { turn, .. } = to_response(turn_start_resp)?;

    let server_req = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_request_message(),
    )
    .await??;
    let ServerRequest::ToolRequestUserInput { request_id, params } = server_req else {
        panic!("expected ToolRequestUserInput request, got: {server_req:?}");
    };

    assert_eq!(params.thread_id, thread.id);
    assert_eq!(params.turn_id, turn.id);
    assert_eq!(params.item_id, "call1");
    assert_eq!(params.questions.len(), 1);
    assert_eq!(params.auto_resolution_ms, Some(60_000));
    let resolved_request_id = request_id.clone();

    mcp.send_response(
        request_id,
        serde_json::json!({
            "answers": {
                "confirm_path": { "answers": ["yes"] }
            }
        }),
    )
    .await?;
    let mut saw_resolved = false;
    loop {
        let message = timeout(DEFAULT_READ_TIMEOUT, mcp.read_next_message()).await??;
        let JSONRPCMessage::Notification(notification) = message else {
            continue;
        };
        match notification.method.as_str() {
            "serverRequest/resolved" => {
                let resolved: ServerRequestResolvedNotification = serde_json::from_value(
                    notification
                        .params
                        .clone()
                        .expect("serverRequest/resolved params"),
                )?;
                assert_eq!(resolved.thread_id, thread.id);
                assert_eq!(resolved.request_id, resolved_request_id);
                saw_resolved = true;
            }
            "turn/completed" => {
                assert!(saw_resolved, "serverRequest/resolved should arrive first");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_user_input_native_ask_ui_false_uses_readable_fallback_and_completes() -> Result<()>
{
    let mut harness = RequestUserInputHarness::new().await?;
    harness
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: false,
            ..Default::default()
        }))
        .await?;

    let thread = harness.start_thread().await?;
    let turn = harness.start_ask_turn(&thread.id).await?;
    let fallback_text = harness.read_fallback_agent_message().await?;
    assert_readable_fallback_prompt(&fallback_text);
    harness
        .read_fallback_turn_completed(&thread.id, &turn.id)
        .await?;
    assert_no_native_ask_request_buffered(&mut harness.mcp).await?;

    harness.answer_fallback(&thread.id, "yes").await?;
    harness
        .read_fallback_answer_resolved_then_turn_completed(&thread.id, &turn.id)
        .await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_user_input_without_native_ask_ui_capability_uses_readable_fallback_and_completes()
-> Result<()> {
    let mut harness = RequestUserInputHarness::new().await?;
    harness
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            ..Default::default()
        }))
        .await?;

    let thread = harness.start_thread().await?;
    let turn = harness.start_ask_turn(&thread.id).await?;
    let fallback_text = harness.read_fallback_agent_message().await?;
    assert_readable_fallback_prompt(&fallback_text);
    harness
        .read_fallback_turn_completed(&thread.id, &turn.id)
        .await?;
    assert_no_native_ask_request_buffered(&mut harness.mcp).await?;

    harness.answer_fallback(&thread.id, "yes").await?;
    harness
        .read_fallback_answer_resolved_then_turn_completed(&thread.id, &turn.id)
        .await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_user_input_vscode_named_client_without_declared_native_ask_ui_uses_native_ask()
-> Result<()> {
    let mut harness = RequestUserInputHarness::new().await?;
    harness
        .initialize_client(
            "VS Code",
            Some(InitializeCapabilities {
                experimental_api: true,
                native_ask_ui: false,
                ..Default::default()
            }),
        )
        .await?;

    let thread = harness.start_thread().await?;
    let turn = harness.start_default_ask_turn(&thread.id).await?;
    let server_req = read_next_request_user_input_request(&mut harness.mcp).await?;
    let ServerRequest::ToolRequestUserInput { request_id, params } = server_req else {
        panic!("VS Code should receive native ToolRequestUserInput, got: {server_req:?}");
    };
    assert_eq!(params.thread_id, thread.id);
    assert_eq!(params.turn_id, turn.id);
    assert_eq!(params.item_id, "call1");

    harness
        .mcp
        .send_response(
            request_id,
            serde_json::json!({
                "answers": {
                    "confirm_path": { "answers": ["yes"] }
                }
            }),
        )
        .await?;
    harness.read_turn_completed(&thread.id, &turn.id).await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_user_input_preserves_visible_context_before_native_ask() -> Result<()> {
    let mut harness = RequestUserInputHarness::new_with_responses(vec![
        create_request_user_input_sse_response_with_visible_context("call-with-context")?,
    ])
    .await?;
    harness
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: true,
            ..Default::default()
        }))
        .await?;

    let thread = harness.start_thread().await?;
    let turn = harness.start_ask_turn(&thread.id).await?;

    let (id, text) = read_agent_message_notification_before_request(&mut harness.mcp).await?;
    assert_eq!(id, "context-msg");
    assert!(
        text.contains("Conclusion: proceed with the focused fix")
            && text.contains("Answer impact: Yes continues; No reopens the plan"),
        "native Ask should be preceded by the model's visible decision context, got:\n{text}"
    );

    let server_req = read_next_request_user_input_request(&mut harness.mcp).await?;
    let ServerRequest::ToolRequestUserInput { params, .. } = server_req else {
        panic!("expected ToolRequestUserInput after visible context, got: {server_req:?}");
    };
    assert_eq!(params.thread_id, thread.id);
    assert_eq!(params.turn_id, turn.id);
    assert_eq!(params.item_id, "call-with-context");

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_user_input_native_ask_ui_does_not_emit_synthetic_question_context_message()
-> Result<()> {
    let mut harness = RequestUserInputHarness::new_with_responses(vec![
        create_request_user_input_sse_response_with_card_context("call-card-context")?,
    ])
    .await?;
    harness
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: true,
            ..Default::default()
        }))
        .await?;

    let thread = harness.start_thread().await?;
    let turn = harness.start_ask_turn(&thread.id).await?;

    let server_req = read_next_request_user_input_request(&mut harness.mcp).await?;
    let ServerRequest::ToolRequestUserInput { params, .. } = server_req else {
        panic!(
            "expected ToolRequestUserInput without synthetic app-server context, got: {server_req:?}"
        );
    };
    assert_eq!(params.thread_id, thread.id);
    assert_eq!(params.turn_id, turn.id);
    assert_eq!(params.item_id, "call-card-context");

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_user_input_official_vscode_client_uses_native_ask_ui_without_declared_capabilities()
-> Result<()> {
    assert_official_popup_capable_client_uses_native_ask_ui("codex_vscode").await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_user_input_vscode_title_client_uses_native_ask_ui_without_declared_capabilities()
-> Result<()> {
    assert_official_popup_capable_client_uses_native_ask_ui("VS Code").await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_user_input_official_chatgpt_extension_client_uses_native_ask_ui_without_declared_capabilities()
-> Result<()> {
    assert_official_popup_capable_client_uses_native_ask_ui("openai.chatgpt").await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_user_input_desktop_remote_client_uses_native_ask_ui_without_declared_capabilities()
-> Result<()> {
    assert_official_popup_capable_client_uses_native_ask_ui("codex_chatgpt_desktop_remote").await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_user_input_running_resume_replays_pending_ask_only_to_native_ask_ui_clients()
-> Result<()> {
    let mut primary = RequestUserInputHarness::new_pending_only().await?;
    primary
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: true,
            ..Default::default()
        }))
        .await?;
    let thread = primary.start_thread().await?;
    let _turn = primary.start_ask_turn(&thread.id).await?;

    let original_request = timeout(
        DEFAULT_READ_TIMEOUT,
        primary.mcp.read_stream_until_request_message(),
    )
    .await??;
    let ServerRequest::ToolRequestUserInput { .. } = &original_request else {
        panic!("expected ToolRequestUserInput request, got: {original_request:?}");
    };

    let mut native_resume = TestAppServer::builder()
        .with_codex_home(primary.codex_home.path())
        .build()
        .await?;
    initialize_mcp(
        &mut native_resume,
        Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: true,
            ..Default::default()
        }),
    )
    .await?;
    let resumed_thread = resume_thread(&mut native_resume, &thread.id).await?;
    assert_eq!(resumed_thread.id, thread.id);
    assert_ne!(resumed_thread.status, ThreadStatus::NotLoaded);
    let replayed_request = timeout(
        DEFAULT_READ_TIMEOUT,
        native_resume.read_stream_until_request_message(),
    )
    .await??;
    assert_matching_request_user_input_replay(&replayed_request, &original_request);

    let mut unsupported_resume = TestAppServer::builder()
        .with_codex_home(primary.codex_home.path())
        .build()
        .await?;
    initialize_mcp(
        &mut unsupported_resume,
        Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: false,
            ..Default::default()
        }),
    )
    .await?;
    let unsupported_resumed_thread = resume_thread(&mut unsupported_resume, &thread.id).await?;
    assert_eq!(unsupported_resumed_thread.id, thread.id);
    assert_ne!(unsupported_resumed_thread.status, ThreadStatus::NotLoaded);

    assert_no_native_ask_request_buffered(&mut unsupported_resume).await?;
    let fallback_text = read_fallback_agent_message(&mut unsupported_resume).await?;
    assert_readable_fallback_prompt(&fallback_text);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_user_input_cold_resume_answer_completes_in_the_new_process() -> Result<()> {
    let mut primary = RequestUserInputHarness::new().await?;
    primary
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: true,
            ..Default::default()
        }))
        .await?;
    let thread = primary.start_thread().await?;
    let turn = primary.start_ask_turn(&thread.id).await?;
    let original_request = timeout(
        DEFAULT_READ_TIMEOUT,
        primary.mcp.read_stream_until_request_message(),
    )
    .await??;

    let mut resumed = TestAppServer::builder()
        .with_codex_home(primary.codex_home.path())
        .build()
        .await?;
    initialize_mcp(
        &mut resumed,
        Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: true,
            ..Default::default()
        }),
    )
    .await?;
    let resumed_thread = resume_thread(&mut resumed, &thread.id).await?;
    assert_eq!(resumed_thread.id, thread.id);
    let replayed_request = timeout(
        DEFAULT_READ_TIMEOUT,
        resumed.read_stream_until_request_message(),
    )
    .await??;
    assert_matching_request_user_input_replay(&replayed_request, &original_request);
    let ServerRequest::ToolRequestUserInput { request_id, .. } = replayed_request else {
        panic!("expected ToolRequestUserInput request");
    };
    resumed
        .send_response(
            request_id,
            json!({
                "answers": {
                    "confirm_path": { "answers": ["yes"] }
                }
            }),
        )
        .await?;

    let completed = timeout(
        DEFAULT_READ_TIMEOUT,
        resumed.read_stream_until_notification_message("turn/completed"),
    )
    .await??;
    let completed: TurnCompletedNotification = serde_json::from_value(
        completed
            .params
            .context("turn/completed must include params")?,
    )?;
    assert_eq!(completed.thread_id, thread.id);
    assert_ne!(completed.turn.id, turn.id);
    assert_eq!(completed.turn.status, TurnStatus::Completed);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn workflow_required_decisions_with_native_ask_ui_carry_card_metadata() -> Result<()> {
    let mut harness = RequestUserInputHarness::new_with_responses(vec![
        create_workflow_decision_request_user_input_sse_response("workflow-call")?,
    ])
    .await?;
    harness
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: true,
            ..Default::default()
        }))
        .await?;

    let thread = harness.start_thread().await?;
    let turn = harness.start_ask_turn(&thread.id).await?;
    let server_req = timeout(
        DEFAULT_READ_TIMEOUT,
        harness.mcp.read_stream_until_request_message(),
    )
    .await??;
    let ServerRequest::ToolRequestUserInput { params, .. } = server_req else {
        panic!(
            "expected native item/tool/requestUserInput for Workflow decision, got: {server_req:?}"
        );
    };

    assert_eq!(params.thread_id, thread.id);
    assert_eq!(params.turn_id, turn.id);
    assert_eq!(params.item_id, "workflow-call");

    let params_json = serde_json::to_value(&params)?;
    let questions = params_json["questions"]
        .as_array()
        .expect("questions should serialize as array");
    let decision_types: Vec<&str> = questions
        .iter()
        .filter_map(|question| question["workflowDecisionType"].as_str())
        .collect();
    assert_eq!(
        decision_types,
        vec![
            "plan_confirmation",
            "phase_transition",
            "scope_selection",
            "revision_rejection",
            "authorization_intent",
            "release_update_confirmation",
        ],
        "native Ask UI payload should preserve Workflow card metadata for every required decision type; got {params_json:#}"
    );

    let authorization = questions
        .iter()
        .find(|question| question["workflowDecisionType"] == "authorization_intent")
        .expect("authorization-intent Workflow card");
    assert_eq!(
        authorization["intentOnly"], true,
        "authorization-intent must be marked as intent-only in native metadata: {authorization:#}"
    );
    assert_eq!(
        authorization["grantsApproval"], false,
        "authorization-intent must not grant command/file/permission approvals: {authorization:#}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn workflow_explicit_metadata_survives_native_ask_ui_for_non_reserved_question_id()
-> Result<()> {
    let mut harness = RequestUserInputHarness::new_with_responses(vec![
        create_explicit_workflow_metadata_non_reserved_sse_response("workflow-call")?,
    ])
    .await?;
    harness
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: true,
            ..Default::default()
        }))
        .await?;

    let thread = harness.start_thread().await?;
    let turn = harness.start_ask_turn(&thread.id).await?;
    let server_req = timeout(
        DEFAULT_READ_TIMEOUT,
        harness.mcp.read_stream_until_request_message(),
    )
    .await??;
    let ServerRequest::ToolRequestUserInput { params, .. } = server_req else {
        panic!(
            "expected native item/tool/requestUserInput for explicit Workflow metadata, got: {server_req:?}"
        );
    };

    assert_eq!(params.thread_id, thread.id);
    assert_eq!(params.turn_id, turn.id);
    assert_eq!(params.item_id, "workflow-call");

    let params_json = serde_json::to_value(&params)?;
    let question = &params_json["questions"][0];
    assert_eq!(question["id"], "confirm_release");
    assert_eq!(
        question["workflowDecisionType"], "release_update_confirmation",
        "native Ask UI must preserve explicit Workflow metadata from tool args even when the question id is not reserved; got {params_json:#}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn reserved_question_id_without_explicit_metadata_stays_non_workflow_in_native_ask_ui()
-> Result<()> {
    let mut harness = RequestUserInputHarness::new_with_responses(vec![
        create_reserved_id_without_workflow_metadata_sse_response("plain-call")?,
    ])
    .await?;
    harness
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: true,
            ..Default::default()
        }))
        .await?;

    let thread = harness.start_thread().await?;
    let turn = harness.start_ask_turn(&thread.id).await?;
    let server_req = timeout(
        DEFAULT_READ_TIMEOUT,
        harness.mcp.read_stream_until_request_message(),
    )
    .await??;
    let ServerRequest::ToolRequestUserInput { params, .. } = server_req else {
        panic!(
            "expected native item/tool/requestUserInput for non-Workflow question, got: {server_req:?}"
        );
    };

    assert_eq!(params.thread_id, thread.id);
    assert_eq!(params.turn_id, turn.id);
    assert_eq!(params.item_id, "plain-call");

    let params_json = serde_json::to_value(&params)?;
    let question = &params_json["questions"][0];
    assert_eq!(question["id"], "authorization_intent");
    for field in ["workflowDecisionType", "intentOnly", "grantsApproval"] {
        assert!(
            question.get(field).is_none(),
            "native Ask UI must not synthesize {field} for a non-Workflow question just because its id is reserved; got {params_json:#}"
        );
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn workflow_required_decisions_without_native_ask_ui_render_polished_markdown_cards()
-> Result<()> {
    let mut harness = RequestUserInputHarness::new_with_responses(vec![
        create_workflow_decision_request_user_input_sse_response("workflow-call")?,
    ])
    .await?;
    harness
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: false,
            ..Default::default()
        }))
        .await?;

    let thread = harness.start_thread().await?;
    let _turn = harness.start_ask_turn(&thread.id).await?;
    let fallback_text = harness.read_fallback_agent_message().await?;
    assert_no_native_ask_request_buffered(&mut harness.mcp).await?;

    for expected in [
        "## Workflow Surface",
        "### Request Input",
        "Request Input fallback for plugin clients without native Ask UI.",
    ] {
        assert!(
            fallback_text.contains(expected),
            "live plugin-facing request input fallback should be wrapped in the M6 Workflow Markdown surface and contain {expected:?}; got:\n{fallback_text}"
        );
    }

    for expected in [
        "## Workflow Decision",
        "**Answer controls**",
        "Workflow decision: plan confirmation",
        "Workflow decision: phase transition",
        "Workflow decision: scope selection",
        "Workflow decision: revision/rejection",
        "Workflow decision: authorization-intent",
        "Workflow decision: release/update confirmation",
        "1. Approve plan",
        "2. Request revision",
        "3. Cancel",
    ] {
        assert!(
            fallback_text.contains(expected),
            "polished Markdown fallback should contain {expected:?}; got:\n{fallback_text}"
        );
    }

    for forbidden in [
        "ToolRequestUserInputParams",
        "ServerRequest",
        "workflowDecisionType",
        "[single_choice]",
        "[multi_choice]",
        "[free_text]",
        "[ranking]",
    ] {
        assert!(
            !fallback_text.contains(forbidden),
            "fallback must be user-facing Markdown, not raw protocol/debug text {forbidden:?}; got:\n{fallback_text}"
        );
    }

    assert!(
        fallback_text.contains(
            "Authorization intent only; this does not grant command, file, or permission approvals."
        ),
        "authorization-intent fallback must visibly separate intent capture from real approval grants; got:\n{fallback_text}"
    );
    assert!(
        fallback_text.contains("Existing approval mechanisms still apply."),
        "authorization-intent fallback must keep real approval mechanisms explicit; got:\n{fallback_text}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fallback_markdown_uses_explicit_workflow_metadata_not_reserved_question_ids() -> Result<()>
{
    let mut explicit = RequestUserInputHarness::new_with_responses(vec![
        create_explicit_workflow_metadata_non_reserved_sse_response("workflow-call")?,
    ])
    .await?;
    explicit
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: false,
            ..Default::default()
        }))
        .await?;
    let explicit_thread = explicit.start_thread().await?;
    let _explicit_turn = explicit.start_ask_turn(&explicit_thread.id).await?;
    let explicit_fallback = explicit.read_fallback_agent_message().await?;
    assert!(
        explicit_fallback.contains("Workflow decision: release/update confirmation"),
        "fallback Markdown must use explicit Workflow metadata for non-reserved ids; got:\n{explicit_fallback}"
    );

    let mut plain = RequestUserInputHarness::new_with_responses(vec![
        create_reserved_id_without_workflow_metadata_sse_response("plain-call")?,
    ])
    .await?;
    plain
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: false,
            ..Default::default()
        }))
        .await?;
    let plain_thread = plain.start_thread().await?;
    let _plain_turn = plain.start_ask_turn(&plain_thread.id).await?;
    let plain_fallback = plain.read_fallback_agent_message().await?;
    assert!(
        !plain_fallback.contains("Workflow decision: authorization-intent"),
        "fallback Markdown must not label a non-Workflow question from a reserved id alone; got:\n{plain_fallback}"
    );
    assert!(
        !plain_fallback.contains("Authorization intent only"),
        "fallback Markdown must not show authorization-intent safety copy without explicit Workflow metadata; got:\n{plain_fallback}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ordinary_non_workflow_clarification_remains_agent_message_even_with_native_ask_ui()
-> Result<()> {
    let mut harness = RequestUserInputHarness::new_with_responses(vec![
        create_final_assistant_message_sse_response("Could you clarify which file you mean?")?,
    ])
    .await?;
    harness
        .initialize(Some(InitializeCapabilities {
            experimental_api: true,
            native_ask_ui: true,
            ..Default::default()
        }))
        .await?;

    let thread = harness.start_thread().await?;
    let _turn = harness.start_ask_turn(&thread.id).await?;
    let agent_text = harness.read_completed_agent_message().await?;

    assert_eq!(agent_text, "Could you clarify which file you mean?");
    assert_no_native_ask_request_buffered(&mut harness.mcp).await?;

    Ok(())
}

struct RequestUserInputHarness {
    codex_home: tempfile::TempDir,
    _server: wiremock::MockServer,
    mcp: TestAppServer,
}

impl RequestUserInputHarness {
    async fn new_with_responses(responses: Vec<String>) -> Result<Self> {
        let codex_home = tempfile::TempDir::new()?;
        let server = create_mock_responses_server_sequence_unchecked(responses).await;
        create_config_toml(codex_home.path(), &server.uri())?;
        let mcp = TestAppServer::builder()
            .with_codex_home(codex_home.path())
            .build()
            .await?;
        Ok(Self {
            codex_home,
            _server: server,
            mcp,
        })
    }

    async fn new() -> Result<Self> {
        let codex_home = tempfile::TempDir::new()?;
        let responses = vec![
            create_request_user_input_sse_response("call1")?,
            create_final_assistant_message_sse_response("done")?,
        ];
        let server = create_mock_responses_server_sequence(responses).await;
        create_config_toml(codex_home.path(), &server.uri())?;
        let mcp = TestAppServer::builder()
            .with_codex_home(codex_home.path())
            .build()
            .await?;
        Ok(Self {
            codex_home,
            _server: server,
            mcp,
        })
    }

    async fn new_pending_only() -> Result<Self> {
        let codex_home = tempfile::TempDir::new()?;
        let responses = vec![create_request_user_input_sse_response("call1")?];
        let server = create_mock_responses_server_sequence_unchecked(responses).await;
        create_config_toml(codex_home.path(), &server.uri())?;
        let mcp = TestAppServer::builder()
            .with_codex_home(codex_home.path())
            .build()
            .await?;
        Ok(Self {
            codex_home,
            _server: server,
            mcp,
        })
    }

    async fn initialize(&mut self, capabilities: Option<InitializeCapabilities>) -> Result<()> {
        initialize_mcp(&mut self.mcp, capabilities).await
    }

    async fn initialize_client(
        &mut self,
        client_name: &str,
        capabilities: Option<InitializeCapabilities>,
    ) -> Result<()> {
        initialize_mcp_with_client_name(&mut self.mcp, client_name, capabilities).await
    }

    async fn start_thread(&mut self) -> Result<codex_app_server_protocol::Thread> {
        let thread_start_id = self
            .mcp
            .send_thread_start_request(ThreadStartParams {
                model: Some("mock-model".to_string()),
                ..Default::default()
            })
            .await?;
        let thread_start_resp: JSONRPCResponse = timeout(
            DEFAULT_READ_TIMEOUT,
            self.mcp
                .read_stream_until_response_message(RequestId::Integer(thread_start_id)),
        )
        .await??;
        let ThreadStartResponse { thread, .. } = to_response(thread_start_resp)?;
        Ok(thread)
    }

    async fn start_ask_turn(&mut self, thread_id: &str) -> Result<codex_app_server_protocol::Turn> {
        let turn_start_id = self
            .mcp
            .send_turn_start_request(ask_turn_params(thread_id))
            .await?;
        let turn_start_resp: JSONRPCResponse = timeout(
            DEFAULT_READ_TIMEOUT,
            self.mcp
                .read_stream_until_response_message(RequestId::Integer(turn_start_id)),
        )
        .await??;
        let TurnStartResponse { turn, .. } = to_response(turn_start_resp)?;
        Ok(turn)
    }

    async fn start_default_ask_turn(
        &mut self,
        thread_id: &str,
    ) -> Result<codex_app_server_protocol::Turn> {
        let turn_start_id = self
            .mcp
            .send_turn_start_request(default_ask_turn_params(thread_id))
            .await?;
        let turn_start_resp: JSONRPCResponse = timeout(
            DEFAULT_READ_TIMEOUT,
            self.mcp
                .read_stream_until_response_message(RequestId::Integer(turn_start_id)),
        )
        .await??;
        let TurnStartResponse { turn, .. } = to_response(turn_start_resp)?;
        Ok(turn)
    }

    async fn read_fallback_agent_message(&mut self) -> Result<String> {
        let notification = timeout(FALLBACK_PROMPT_TIMEOUT, async {
            loop {
                let notification = self
                    .mcp
                    .read_stream_until_notification_message("item/started")
                    .await?;
                let Some(params) = notification.params.clone() else {
                    anyhow::bail!("missing item/started params");
                };
                let started: ItemStartedNotification = serde_json::from_value(params)?;
                if let ThreadItem::AgentMessage { text, .. } = started.item {
                    return Ok::<String, anyhow::Error>(text);
                }
            }
        })
        .await??;
        Ok(notification)
    }

    async fn read_completed_agent_message(&mut self) -> Result<String> {
        let notification = timeout(FALLBACK_PROMPT_TIMEOUT, async {
            loop {
                let notification = self
                    .mcp
                    .read_stream_until_notification_message("item/completed")
                    .await?;
                let Some(params) = notification.params.clone() else {
                    anyhow::bail!("missing item/completed params");
                };
                let completed: ItemCompletedNotification = serde_json::from_value(params)?;
                if let ThreadItem::AgentMessage { text, .. } = completed.item {
                    return Ok::<String, anyhow::Error>(text);
                }
            }
        })
        .await??;
        Ok(notification)
    }

    async fn read_fallback_turn_completed(&mut self, thread_id: &str, turn_id: &str) -> Result<()> {
        timeout(FALLBACK_PROMPT_TIMEOUT, async {
            loop {
                let notification = self
                    .mcp
                    .read_stream_until_notification_message("turn/completed")
                    .await?;
                let Some(params) = notification.params.clone() else {
                    anyhow::bail!("missing turn/completed params");
                };
                let completed: TurnCompletedNotification = serde_json::from_value(params)?;
                if completed.thread_id == thread_id && completed.turn.id == turn_id {
                    assert_eq!(
                        completed.turn.status,
                        codex_app_server_protocol::TurnStatus::Completed
                    );
                    return Ok::<(), anyhow::Error>(());
                }
            }
        })
        .await??;
        Ok(())
    }

    async fn answer_fallback(&mut self, thread_id: &str, text: &str) -> Result<()> {
        let turn_start_id = self
            .mcp
            .send_turn_start_request(TurnStartParams {
                thread_id: thread_id.to_string(),
                input: vec![V2UserInput::Text {
                    text: text.to_string(),
                    text_elements: Vec::new(),
                }],
                ..Default::default()
            })
            .await?;
        timeout(
            DEFAULT_READ_TIMEOUT,
            self.mcp
                .read_stream_until_response_message(RequestId::Integer(turn_start_id)),
        )
        .await??;
        Ok(())
    }

    async fn read_fallback_answer_resolved_then_turn_completed(
        &mut self,
        thread_id: &str,
        turn_id: &str,
    ) -> Result<()> {
        let mut saw_resolved = false;
        timeout(DEFAULT_READ_TIMEOUT, async {
            loop {
                let message = self.mcp.read_next_message().await?;
                let JSONRPCMessage::Notification(notification) = message else {
                    continue;
                };
                match notification.method.as_str() {
                    "serverRequest/resolved" => {
                        let resolved: ServerRequestResolvedNotification = serde_json::from_value(
                            notification
                                .params
                                .clone()
                                .ok_or_else(|| anyhow::anyhow!("missing serverRequest/resolved params"))?,
                        )?;
                        if resolved.thread_id == thread_id {
                            saw_resolved = true;
                        }
                    }
                    "turn/completed" => {
                        let Some(params) = notification.params.clone() else {
                            anyhow::bail!("missing turn/completed params");
                        };
                        let completed: TurnCompletedNotification = serde_json::from_value(params)?;
                        if completed.thread_id == thread_id && completed.turn.id == turn_id {
                            assert!(
                                saw_resolved,
                                "fallback answer must resolve the hidden request before the original turn completes"
                            );
                            assert_eq!(
                                completed.turn.status,
                                codex_app_server_protocol::TurnStatus::Completed
                            );
                            return Ok::<(), anyhow::Error>(());
                        }
                    }
                    _ => {}
                }
            }
        })
        .await??;
        Ok(())
    }

    async fn read_turn_completed(&mut self, thread_id: &str, turn_id: &str) -> Result<()> {
        timeout(DEFAULT_READ_TIMEOUT, async {
            loop {
                let notification = self
                    .mcp
                    .read_stream_until_notification_message("turn/completed")
                    .await?;
                let Some(params) = notification.params else {
                    anyhow::bail!("missing turn/completed params");
                };
                if turn_completed_matches(&params, thread_id, turn_id) {
                    return Ok::<(), anyhow::Error>(());
                }
            }
        })
        .await??;
        Ok(())
    }
}

async fn initialize_mcp(
    mcp: &mut TestAppServer,
    capabilities: Option<InitializeCapabilities>,
) -> Result<()> {
    initialize_mcp_with_client_name(mcp, "codex-app-server-tests", capabilities).await
}

async fn initialize_mcp_with_client_name(
    mcp: &mut TestAppServer,
    client_name: &str,
    capabilities: Option<InitializeCapabilities>,
) -> Result<()> {
    let initialized = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.initialize_with_capabilities(
            ClientInfo {
                name: client_name.to_string(),
                title: None,
                version: "0.1.0".to_string(),
            },
            capabilities,
        ),
    )
    .await??;
    let JSONRPCMessage::Response(_) = initialized else {
        panic!("expected initialize response");
    };
    Ok(())
}

async fn assert_official_popup_capable_client_uses_native_ask_ui(client_name: &str) -> Result<()> {
    let mut harness = RequestUserInputHarness::new().await?;
    harness
        .initialize_client(
            client_name,
            Some(InitializeCapabilities {
                experimental_api: false,
                native_ask_ui: false,
                ..Default::default()
            }),
        )
        .await?;

    let thread = harness.start_thread().await?;
    let turn = harness.start_default_ask_turn(&thread.id).await?;
    let server_req = timeout(
        DEFAULT_READ_TIMEOUT,
        harness.mcp.read_stream_until_request_message(),
    )
    .await??;
    let ServerRequest::ToolRequestUserInput { request_id, params } = server_req else {
        panic!(
            "official popup-capable client {client_name} should receive native item/tool/requestUserInput, got: {server_req:?}"
        );
    };

    assert_eq!(params.thread_id, thread.id);
    assert_eq!(params.turn_id, turn.id);
    assert_eq!(params.item_id, "call1");

    harness
        .mcp
        .send_response(
            request_id,
            serde_json::json!({
                "answers": {
                    "confirm_path": { "answers": ["yes"] }
                }
            }),
        )
        .await?;
    harness.read_turn_completed(&thread.id, &turn.id).await?;

    Ok(())
}

fn ask_turn_params(thread_id: &str) -> TurnStartParams {
    TurnStartParams {
        thread_id: thread_id.to_string(),
        input: vec![V2UserInput::Text {
            text: "ask something".to_string(),
            text_elements: Vec::new(),
        }],
        model: Some("mock-model".to_string()),
        effort: Some(ReasoningEffort::Medium),
        collaboration_mode: Some(CollaborationMode {
            mode: ModeKind::Plan,
            settings: Settings {
                model: "mock-model".to_string(),
                reasoning_effort: Some(ReasoningEffort::Medium),
                developer_instructions: None,
            },
        }),
        ..Default::default()
    }
}

fn default_ask_turn_params(thread_id: &str) -> TurnStartParams {
    TurnStartParams {
        thread_id: thread_id.to_string(),
        input: vec![V2UserInput::Text {
            text: "ask something".to_string(),
            text_elements: Vec::new(),
        }],
        model: Some("mock-model".to_string()),
        effort: Some(ReasoningEffort::Medium),
        ..Default::default()
    }
}

async fn resume_thread(
    mcp: &mut TestAppServer,
    thread_id: &str,
) -> Result<codex_app_server_protocol::Thread> {
    let resume_id = mcp
        .send_thread_resume_request(ThreadResumeParams {
            thread_id: thread_id.to_string(),
            ..Default::default()
        })
        .await?;
    let resume_resp: JSONRPCResponse = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(resume_id)),
    )
    .await??;
    let ThreadResumeResponse { thread, .. } = to_response(resume_resp)?;
    Ok(thread)
}

async fn assert_no_native_ask_request_buffered(mcp: &mut TestAppServer) -> Result<()> {
    let read_request = timeout(
        std::time::Duration::from_millis(300),
        mcp.read_stream_until_request_message(),
    )
    .await;
    match read_request {
        Err(_) => Ok(()),
        Ok(Ok(request)) => {
            assert!(
                !matches!(request, ServerRequest::ToolRequestUserInput { .. }),
                "unsupported client must not receive ToolRequestUserInput: {request:?}"
            );
            Ok(())
        }
        Ok(Err(err)) => Err(err),
    }
}

async fn read_agent_message_notification_before_request(
    mcp: &mut TestAppServer,
) -> Result<(String, String)> {
    timeout(DEFAULT_READ_TIMEOUT, async {
        loop {
            let message = mcp.read_next_message().await?;
            match message {
                JSONRPCMessage::Notification(notification)
                    if notification.method == "item/completed" =>
                {
                    let params = notification.params.clone().ok_or_else(|| {
                        anyhow::anyhow!("missing item/completed params for agent message")
                    })?;
                    let completed: ItemCompletedNotification = serde_json::from_value(params)?;
                    let item = completed.item;
                    if let ThreadItem::AgentMessage { id, text, .. } = item {
                        assert!(
                            !id.ends_with("-ask-context")
                                && !text.contains("## Question Context")
                                && !text.contains("Review this visible context before answering the Question Tool prompt."),
                            "native Ask path must not inject synthetic Question Context messages; got id={id:?}, text:\n{text}"
                        );
                        return Ok::<(String, String), anyhow::Error>((id, text));
                    }
                }
                JSONRPCMessage::Request(request) => {
                    let server_req: ServerRequest = request.try_into()?;
                    if matches!(server_req, ServerRequest::ToolRequestUserInput { .. }) {
                        anyhow::bail!(
                            "native Ask request arrived before the model's visible context: {server_req:?}"
                        );
                    }
                }
                _ => {}
            }
        }
    })
    .await?
}

async fn read_next_request_user_input_request(mcp: &mut TestAppServer) -> Result<ServerRequest> {
    timeout(DEFAULT_READ_TIMEOUT, async {
        loop {
            let message = mcp.read_next_message().await?;
            if let JSONRPCMessage::Notification(notification) = &message
                && matches!(notification.method.as_str(), "item/started" | "item/completed")
                && let Some(params) = notification.params.clone()
            {
                let item = if notification.method == "item/started" {
                    let started: ItemStartedNotification = serde_json::from_value(params)?;
                    started.item
                } else {
                    let completed: ItemCompletedNotification = serde_json::from_value(params)?;
                    completed.item
                };
                if let ThreadItem::AgentMessage { id, text, .. } = item {
                    assert!(
                        !id.ends_with("-ask-context")
                            && !text.contains("## Question Context")
                            && !text.contains("Review this visible context before answering the Question Tool prompt.")
                            && !text.contains("**Answer impact**"),
                        "native Ask path must not emit synthetic Question Context before the request; got id={id:?}, text:\n{text}"
                    );
                }
            }
            if let JSONRPCMessage::Request(request) = message {
                let server_req: ServerRequest = request.try_into()?;
                if matches!(server_req, ServerRequest::ToolRequestUserInput { .. }) {
                    return Ok::<ServerRequest, anyhow::Error>(server_req);
                }
            }
        }
    })
    .await?
}

async fn read_fallback_agent_message(mcp: &mut TestAppServer) -> Result<String> {
    let notification = timeout(FALLBACK_PROMPT_TIMEOUT, async {
        loop {
            let notification = mcp
                .read_stream_until_notification_message("item/started")
                .await?;
            let Some(params) = notification.params.clone() else {
                anyhow::bail!("missing item/started params");
            };
            let started: ItemStartedNotification = serde_json::from_value(params)?;
            if let ThreadItem::AgentMessage { text, .. } = started.item {
                return Ok::<String, anyhow::Error>(text);
            }
        }
    })
    .await??;
    Ok(notification)
}

fn assert_matching_request_user_input_replay(replayed: &ServerRequest, original: &ServerRequest) {
    let ServerRequest::ToolRequestUserInput {
        params: replayed_params,
        ..
    } = replayed
    else {
        panic!("expected ToolRequestUserInput request, got: {replayed:?}");
    };
    let ServerRequest::ToolRequestUserInput {
        params: original_params,
        ..
    } = original
    else {
        panic!("expected ToolRequestUserInput request, got: {original:?}");
    };
    assert_eq!(replayed_params, original_params);
}

fn assert_readable_fallback_prompt(text: &str) {
    assert!(
        text.contains("Proceed with the plan?"),
        "fallback prompt should include the question, got: {text}"
    );
    assert!(
        text.contains("Yes") && text.contains("No"),
        "fallback prompt should include options, got: {text}"
    );
}

fn turn_completed_matches(params: &Value, thread_id: &str, turn_id: &str) -> bool {
    params
        .get("threadId")
        .and_then(Value::as_str)
        .is_some_and(|value| value == thread_id)
        && params
            .get("turn")
            .and_then(|turn| turn.get("id"))
            .and_then(Value::as_str)
            .is_some_and(|value| value == turn_id)
}

fn create_config_toml(codex_home: &std::path::Path, server_uri: &str) -> std::io::Result<()> {
    let config_toml = codex_home.join("config.toml");
    std::fs::write(
        config_toml,
        format!(
            r#"
model = "mock-model"
approval_policy = "untrusted"
sandbox_mode = "read-only"

model_provider = "mock_provider"

[model_providers.mock_provider]
name = "Mock provider for test"
base_url = "{server_uri}/v1"
wire_api = "responses"
request_max_retries = 0
stream_max_retries = 0
"#
        ),
    )
}

fn create_workflow_decision_request_user_input_sse_response(call_id: &str) -> Result<String> {
    let tool_call_arguments = serde_json::to_string(&json!({
        "summary": "Workflow decision required",
        "description": "Hypo-Workflow needs structured input before continuing.",
        "questions": [
            workflow_decision_question(
                "plan_confirmation",
                "plan_confirmation",
                "Plan Confirmation",
                "Confirm the current plan.",
                json!({
                    "workflowDecisionType": "plan_confirmation"
                }),
            ),
            workflow_decision_question(
                "phase_transition",
                "phase_transition",
                "Phase Transition",
                "Move to the next Workflow phase?",
                json!({
                    "workflowDecisionType": "phase_transition"
                }),
            ),
            workflow_decision_question(
                "scope_selection",
                "scope_selection",
                "Scope Selection",
                "Choose the implementation scope.",
                json!({
                    "workflowDecisionType": "scope_selection"
                }),
            ),
            workflow_decision_question(
                "revision_rejection",
                "revision_rejection",
                "Revision/Rejection",
                "Accept the revision or reject this work?",
                json!({
                    "workflowDecisionType": "revision_rejection"
                }),
            ),
            workflow_decision_question(
                "authorization_intent",
                "authorization_intent",
                "Authorization Intent",
                "Confirm intent to proceed with this high-impact Workflow action.",
                json!({
                    "workflowDecisionType": "authorization_intent",
                    "intentOnly": true,
                    "grantsApproval": false
                }),
            ),
            workflow_decision_question(
                "release_update_confirmation",
                "release_update_confirmation",
                "Release/Update Confirmation",
                "Choose how to handle this release or update.",
                json!({
                    "workflowDecisionType": "release_update_confirmation"
                }),
            ),
        ]
    }))?;

    Ok(responses::sse(vec![
        responses::ev_response_created("resp-1"),
        responses::ev_function_call(call_id, "request_user_input", &tool_call_arguments),
        responses::ev_completed("resp-1"),
    ]))
}

fn create_explicit_workflow_metadata_non_reserved_sse_response(call_id: &str) -> Result<String> {
    let tool_call_arguments = serde_json::to_string(&json!({
        "summary": "Workflow release confirmation",
        "description": "Confirm release handling before continuing.",
        "questions": [
            workflow_decision_question(
                "confirm_release",
                "confirm_release",
                "Release Confirmation",
                "Choose how to handle this release.",
                json!({
                    "workflowDecisionType": "release_update_confirmation"
                }),
            ),
        ]
    }))?;

    Ok(responses::sse(vec![
        responses::ev_response_created("resp-1"),
        responses::ev_function_call(call_id, "request_user_input", &tool_call_arguments),
        responses::ev_completed("resp-1"),
    ]))
}

fn create_reserved_id_without_workflow_metadata_sse_response(call_id: &str) -> Result<String> {
    let tool_call_arguments = serde_json::to_string(&json!({
        "summary": "Ordinary clarification",
        "description": "This is not a Workflow authorization card.",
        "questions": [
            {
                "id": "authorization_intent",
                "kind": "singleChoice",
                "header": "Naming Collision",
                "question": "This arbitrary question happens to use a reserved-looking id.",
                "options": [
                    {
                        "id": "yes",
                        "label": "Yes",
                        "description": "Continue."
                    },
                    {
                        "id": "no",
                        "label": "No",
                        "description": "Do not continue."
                    }
                ]
            }
        ]
    }))?;

    Ok(responses::sse(vec![
        responses::ev_response_created("resp-1"),
        responses::ev_function_call(call_id, "request_user_input", &tool_call_arguments),
        responses::ev_completed("resp-1"),
    ]))
}

fn workflow_decision_question(
    id: &str,
    option_prefix: &str,
    header: &str,
    question: &str,
    metadata: Value,
) -> Value {
    let mut question = json!({
        "id": id,
        "kind": "singleChoice",
        "header": header,
        "question": question,
        "options": [
            {
                "id": format!("{option_prefix}_approve"),
                "label": "Approve plan",
                "description": "Continue with the proposed Workflow path."
            },
            {
                "id": format!("{option_prefix}_revise"),
                "label": "Request revision",
                "description": "Return to the worker with requested changes."
            },
            {
                "id": format!("{option_prefix}_cancel"),
                "label": "Cancel",
                "description": "Stop this Workflow decision without granting approval."
            }
        ]
    });
    let object = question
        .as_object_mut()
        .expect("workflow decision question should be an object");
    let metadata = metadata
        .as_object()
        .expect("workflow decision metadata should be an object");
    for (key, value) in metadata {
        object.insert(key.clone(), value.clone());
    }
    question
}
