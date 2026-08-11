use super::*;
use crate::legacy_core::config::AgentRoleConfig;
use pretty_assertions::assert_eq;

const SAFETY_BUFFERING_HEADER_TEXT: &str =
    "Our systems are thinking a bit more about this request before responding.";

fn thread_settings_for_test(
    model: &str,
    thread_id: ThreadId,
) -> codex_app_server_protocol::ThreadSettingsUpdatedNotification {
    codex_app_server_protocol::ThreadSettingsUpdatedNotification {
        thread_id: thread_id.to_string(),
        thread_settings: codex_app_server_protocol::ThreadSettings {
            cwd: test_path_buf("/tmp/thread-settings").abs(),
            approval_policy: AskForApproval::OnRequest,
            approvals_reviewer: codex_app_server_protocol::ApprovalsReviewer::AutoReview,
            sandbox_policy: codex_app_server_protocol::SandboxPolicy::ReadOnly {
                network_access: false,
            },
            active_permission_profile: Some(
                codex_app_server_protocol::ActivePermissionProfile::read_only(),
            ),
            model: model.to_string(),
            model_provider: "openai".to_string(),
            service_tier: Some(ServiceTier::Fast.request_value().to_string()),
            effort: Some(ReasoningEffortConfig::High),
            summary: None,
            collaboration_mode: CollaborationMode {
                mode: ModeKind::Plan,
                settings: codex_protocol::config_types::Settings {
                    model: model.to_string(),
                    reasoning_effort: Some(ReasoningEffortConfig::High),
                    developer_instructions: None,
                },
            },
            multi_agent_mode: Default::default(),
            personality: Some(Personality::Pragmatic),
        },
    }
}

fn configured_thread_session(thread_id: ThreadId) -> crate::session_state::ThreadSessionState {
    crate::session_state::ThreadSessionState {
        thread_id,
        forked_from_id: None,
        fork_parent_title: None,
        thread_name: None,
        model: "gpt-5.2".to_string(),
        model_provider_id: "openai".to_string(),
        service_tier: None,
        approval_policy: AskForApproval::Never,
        approvals_reviewer: ApprovalsReviewer::User,
        permission_profile: PermissionProfile::read_only(),
        active_permission_profile: None,
        cwd: test_path_buf("/tmp/thread-settings").abs(),
        runtime_workspace_roots: vec![test_path_buf("/tmp/thread-settings").abs()],
        instruction_source_paths: Vec::new(),
        reasoning_effort: None,
        collaboration_mode: None,
        personality: None,
        message_history: None,
        network_proxy: None,
        rollout_path: None,
    }
}

fn start_safety_buffering_test_turn(
    chat: &mut ChatWidget,
    op_rx: &mut tokio::sync::mpsc::UnboundedReceiver<Op>,
) -> (ThreadId, &'static str, Op) {
    let thread_id = ThreadId::new();
    let turn_id = "turn-safety-buffering";
    chat.thread_id = Some(thread_id);
    chat.submit_user_message(UserMessage::from("Explain the request"));
    let turn = next_submit_op(op_rx);
    assert_matches!(&turn, Op::UserTurn { .. });
    chat.record_safety_buffering_turn(turn_id.to_string(), &turn);
    chat.handle_server_notification(
        ServerNotification::TurnStarted(TurnStartedNotification {
            thread_id: thread_id.to_string(),
            turn: AppServerTurn {
                id: turn_id.to_string(),
                items_view: codex_app_server_protocol::TurnItemsView::Full,
                items: Vec::new(),
                status: AppServerTurnStatus::InProgress,
                error: None,
                started_at: Some(0),
                completed_at: None,
                duration_ms: None,
            },
        }),
        /*replay_kind*/ None,
    );
    (thread_id, turn_id, turn)
}

fn safety_buffering_notification(
    thread_id: ThreadId,
    turn_id: &str,
    faster_model: Option<&str>,
) -> ModelSafetyBufferingUpdatedNotification {
    ModelSafetyBufferingUpdatedNotification {
        thread_id: thread_id.to_string(),
        turn_id: turn_id.to_string(),
        model: "current-model".to_string(),
        use_cases: Vec::new(),
        reasons: Vec::new(),
        show_buffering_ui: true,
        faster_model: faster_model.map(str::to_string),
    }
}

#[tokio::test]
async fn safety_buffering_offers_one_retry_with_app_wording() {
    let (mut chat, mut rx, mut op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let (thread_id, turn_id, _) = start_safety_buffering_test_turn(&mut chat, &mut op_rx);

    let notification = safety_buffering_notification(thread_id, turn_id, Some("faster-model"));
    chat.handle_server_notification(
        ServerNotification::ModelSafetyBufferingUpdated(notification.clone()),
        /*replay_kind*/ None,
    );
    chat.handle_server_notification(
        ServerNotification::ModelSafetyBufferingUpdated(notification),
        /*replay_kind*/ None,
    );

    let popup = render_bottom_popup(&chat, /*width*/ 80);
    assert_chatwidget_snapshot!("safety_buffering_retry_prompt", popup);

    chat.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    chat.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let opened_url = loop {
        match rx.try_recv() {
            Ok(AppEvent::OpenUrlInBrowser { url }) => break url,
            Ok(_) => continue,
            Err(err) => panic!("expected learn-more URL event: {err}"),
        }
    };
    assert_eq!(opened_url, "https://help.openai.com/en/articles/20001326");
    assert!(render_bottom_popup(&chat, /*width*/ 80).contains(SAFETY_BUFFERING_HEADER_TEXT));

    chat.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    chat.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let (event_thread_id, event_turn_id, model, turn) = loop {
        match rx.try_recv() {
            Ok(AppEvent::RetrySafetyBufferedTurn {
                thread_id,
                turn_id,
                model,
                turn,
            }) => break (thread_id, turn_id, model, turn),
            Ok(_) => continue,
            Err(err) => panic!("expected safety-buffering retry event: {err}"),
        }
    };
    assert_eq!(event_thread_id, thread_id);
    assert_eq!(event_turn_id, turn_id);
    assert_eq!(model, "faster-model");
    assert_matches!(turn, Op::UserTurn { .. });
    assert!(
        !render_bottom_popup(&chat, /*width*/ 80)
            .contains("Press enter to confirm or esc to go back")
    );
}

#[tokio::test]
async fn safety_buffering_remains_visible_until_turn_completes() {
    let (mut chat, _rx, mut op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let (thread_id, turn_id, _) = start_safety_buffering_test_turn(&mut chat, &mut op_rx);
    chat.handle_server_notification(
        ServerNotification::ModelSafetyBufferingUpdated(safety_buffering_notification(
            thread_id,
            turn_id,
            Some("faster-model"),
        )),
        /*replay_kind*/ None,
    );
    assert!(chat.can_retry_safety_buffered_turn(turn_id));

    chat.on_agent_message_delta("Visible response".to_string());

    assert!(!chat.can_retry_safety_buffered_turn(turn_id));
    assert!(render_bottom_popup(&chat, /*width*/ 80).contains(SAFETY_BUFFERING_HEADER_TEXT));

    handle_turn_completed(&mut chat, turn_id, /*duration_ms*/ None);

    assert!(!render_bottom_popup(&chat, /*width*/ 80).contains(SAFETY_BUFFERING_HEADER_TEXT));
}

#[tokio::test]
async fn safety_buffering_without_retry_shows_short_app_message() {
    let (mut chat, _rx, mut op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let (thread_id, turn_id, turn) = start_safety_buffering_test_turn(&mut chat, &mut op_rx);

    chat.handle_server_notification(
        ServerNotification::ModelSafetyBufferingUpdated(safety_buffering_notification(
            thread_id, turn_id, /*faster_model*/ None,
        )),
        /*replay_kind*/ None,
    );

    let render_popup = |chat: &ChatWidget| {
        normalize_snapshot_paths(render_bottom_popup(chat, /*width*/ 80))
    };
    let popup = render_popup(&chat);
    assert_chatwidget_snapshot!("safety_buffering_status_without_retry", popup,);

    let notification = safety_buffering_notification(thread_id, turn_id, Some("faster-model"));
    chat.record_safety_buffering_turn("other-turn".to_string(), &turn);
    chat.handle_server_notification(
        ServerNotification::ModelSafetyBufferingUpdated(notification.clone()),
        /*replay_kind*/ None,
    );
    assert_eq!(render_popup(&chat), popup);

    chat.record_safety_buffering_turn(turn_id.to_string(), &turn);
    chat.handle_server_notification(
        ServerNotification::ModelSafetyBufferingUpdated(notification),
        Some(ReplayKind::ThreadSnapshot),
    );
    assert_eq!(render_popup(&chat), popup);

    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(
        !render_bottom_popup(&chat, /*width*/ 80)
            .contains("Press enter to confirm or esc to go back")
    );
}

#[tokio::test]
async fn safety_buffering_ignores_hidden_stale_and_historical_updates() {
    let (mut chat, _rx, mut op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let (thread_id, turn_id, _) = start_safety_buffering_test_turn(&mut chat, &mut op_rx);

    let mut hidden = safety_buffering_notification(thread_id, turn_id, Some("faster-model"));
    hidden.show_buffering_ui = false;
    chat.handle_server_notification(
        ServerNotification::ModelSafetyBufferingUpdated(hidden),
        /*replay_kind*/ None,
    );
    chat.handle_server_notification(
        ServerNotification::ModelSafetyBufferingUpdated(safety_buffering_notification(
            thread_id,
            "stale-turn",
            Some("faster-model"),
        )),
        /*replay_kind*/ None,
    );
    chat.handle_server_notification(
        ServerNotification::ModelSafetyBufferingUpdated(safety_buffering_notification(
            thread_id,
            turn_id,
            Some("faster-model"),
        )),
        Some(ReplayKind::ResumeInitialMessages),
    );
    assert!(!render_bottom_popup(&chat, /*width*/ 80).contains(SAFETY_BUFFERING_HEADER_TEXT));

    let mut hidden = safety_buffering_notification(thread_id, turn_id, Some("faster-model"));
    chat.handle_server_notification(
        ServerNotification::ModelSafetyBufferingUpdated(hidden.clone()),
        /*replay_kind*/ None,
    );
    assert!(render_bottom_popup(&chat, /*width*/ 80).contains(SAFETY_BUFFERING_HEADER_TEXT));
    hidden.show_buffering_ui = false;
    chat.handle_server_notification(
        ServerNotification::ModelSafetyBufferingUpdated(hidden),
        /*replay_kind*/ None,
    );

    assert_eq!(
        chat.bottom_pane
            .status_widget()
            .expect("status indicator should be visible")
            .details(),
        None
    );
    assert!(!render_bottom_popup(&chat, /*width*/ 80).contains(SAFETY_BUFFERING_HEADER_TEXT));
}

#[tokio::test]
async fn invalid_url_elicitation_is_declined() {
    let (mut chat, _app_event_tx, mut rx, _op_rx) = make_chatwidget_manual_with_sender().await;
    let visible_thread_id = ThreadId::new();
    let request_thread_id = ThreadId::new();
    chat.thread_id = Some(visible_thread_id);

    chat.handle_elicitation_request_now(
        codex_app_server_protocol::RequestId::Integer(9),
        codex_app_server_protocol::McpServerElicitationRequestParams {
            thread_id: request_thread_id.to_string(),
            turn_id: Some("turn-auth".to_string()),
            server_name: "payments".to_string(),
            request: codex_app_server_protocol::McpServerElicitationRequest::Url {
                meta: None,
                message: "Review the payment details to continue.".to_string(),
                url: "http://payments.example/checkout/123".to_string(),
                elicitation_id: "payment-123".to_string(),
            },
        },
    );

    assert_matches!(
        rx.try_recv(),
        Ok(AppEvent::SubmitThreadOp {
            thread_id: op_thread_id,
            op: Op::ResolveElicitation {
                server_name,
                request_id: codex_app_server_protocol::RequestId::Integer(9),
                decision: codex_app_server_protocol::McpServerElicitationAction::Decline,
                content: None,
                meta: None,
            },
        }) if op_thread_id == request_thread_id && server_name == "payments"
    );
}

#[tokio::test]
async fn thread_settings_updated_updates_visible_state_without_transcript() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(Some("gpt-5.2")).await;
    set_fast_mode_test_catalog(&mut chat);
    let thread_id = ThreadId::new();
    chat.handle_thread_session(configured_thread_session(thread_id));
    let _ = drain_insert_history(&mut rx);

    chat.handle_server_notification(
        ServerNotification::ThreadSettingsUpdated(thread_settings_for_test("gpt-5.4", thread_id)),
        /*replay_kind*/ None,
    );

    assert_eq!(chat.current_model(), "gpt-5.4");
    assert_eq!(
        chat.current_reasoning_effort(),
        Some(ReasoningEffortConfig::High)
    );
    assert_eq!(
        chat.current_service_tier(),
        Some(ServiceTier::Fast.request_value())
    );
    assert_eq!(
        chat.config_ref().permissions.approval_policy.value(),
        AskForApproval::OnRequest.to_core()
    );
    assert_eq!(
        chat.config_ref().approvals_reviewer,
        ApprovalsReviewer::AutoReview
    );
    assert_eq!(
        chat.config_ref()
            .permissions
            .active_permission_profile()
            .expect("active profile")
            .id,
        codex_protocol::models::BUILT_IN_PERMISSION_PROFILE_READ_ONLY
    );
    assert_eq!(chat.config_ref().personality, Some(Personality::Pragmatic));
    assert_eq!(chat.active_collaboration_mode_kind(), ModeKind::Plan);
    assert!(
        drain_insert_history(&mut rx).is_empty(),
        "ThreadSettingsUpdated should not render transcript history"
    );

    chat.handle_server_notification(
        ServerNotification::ThreadSettingsUpdated(thread_settings_for_test(
            "gpt-5.2",
            ThreadId::new(),
        )),
        /*replay_kind*/ None,
    );

    assert_eq!(chat.current_model(), "gpt-5.4");
}

#[tokio::test]
async fn thread_settings_updated_preserves_default_settings_for_plan_mode() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(Some("gpt-5.2")).await;
    let thread_id = ThreadId::new();
    let mut session = configured_thread_session(thread_id);
    session.model = "gpt-default".to_string();
    session.reasoning_effort = Some(ReasoningEffortConfig::Low);
    chat.handle_thread_session(session);
    let _ = drain_insert_history(&mut rx);
    let default_mode = chat.current_collaboration_mode().clone();

    chat.handle_server_notification(
        ServerNotification::ThreadSettingsUpdated(thread_settings_for_test("gpt-plan", thread_id)),
        /*replay_kind*/ None,
    );

    assert_eq!(chat.active_collaboration_mode_kind(), ModeKind::Plan);
    assert_eq!(chat.current_model(), "gpt-plan");
    assert_eq!(
        chat.current_reasoning_effort(),
        Some(ReasoningEffortConfig::High)
    );
    assert_eq!(chat.current_collaboration_mode(), &default_mode);

    let default_mask = collaboration_modes::default_mask(chat.model_catalog.as_ref())
        .expect("expected default collaboration mode");
    chat.set_collaboration_mask(default_mask);

    assert_eq!(chat.active_collaboration_mode_kind(), ModeKind::Default);
    assert_eq!(chat.current_model(), "gpt-default");
    assert_eq!(
        chat.current_reasoning_effort(),
        Some(ReasoningEffortConfig::Low)
    );
}

#[tokio::test]
async fn collab_spawn_end_shows_requested_model_and_effort() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
    let sender_thread_id = ThreadId::new();
    let spawned_thread_id = ThreadId::new();
    chat.set_collab_agent_metadata(
        spawned_thread_id,
        Some("Robie".to_string()),
        Some("explorer".to_string()),
    );

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "call-spawn".to_string(),
                tool: AppServerCollabAgentTool::SpawnAgent,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: Vec::new(),
                prompt: Some("Explore the repo".to_string()),
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::new(),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "call-spawn".to_string(),
                tool: AppServerCollabAgentTool::SpawnAgent,
                status: AppServerCollabAgentToolCallStatus::Completed,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![spawned_thread_id.to_string()],
                prompt: Some("Explore the repo".to_string()),
                model: None,
                reasoning_effort: None,
                agents_states: HashMap::from([(
                    spawned_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::PendingInit,
                        message: None,
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );

    let cells = drain_insert_history(&mut rx);
    let rendered = cells
        .iter()
        .map(|lines| lines_to_single_string(lines))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        rendered.contains("Spawned Robie [explorer] (gpt-5 high)"),
        "expected spawn line to include agent metadata and requested model, got {rendered:?}"
    );
}

#[tokio::test]
async fn live_app_server_user_message_item_completed_does_not_duplicate_rendered_prompt() {
    let (mut chat, mut rx, mut op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.thread_id = Some(ThreadId::new());

    chat.bottom_pane
        .set_composer_text("Hi, are you there?".to_string(), Vec::new(), Vec::new());
    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    match next_submit_op(&mut op_rx) {
        Op::UserTurn { .. } => {}
        other => panic!("expected Op::UserTurn, got {other:?}"),
    }

    let inserted = drain_insert_history(&mut rx);
    assert_eq!(inserted.len(), 1);
    assert!(lines_to_single_string(&inserted[0]).contains("Hi, are you there?"));

    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::UserMessage {
                id: "user-1".to_string(),
                client_id: None,
                content: vec![AppServerUserInput::Text {
                    text: "Hi, are you there?".to_string(),
                    text_elements: Vec::new(),
                }],
            },
        }),
        /*replay_kind*/ None,
    );

    assert!(drain_insert_history(&mut rx).is_empty());
}

#[tokio::test]
async fn live_app_server_turn_completed_clears_working_status_after_answer_item() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    chat.handle_server_notification(
        ServerNotification::TurnStarted(TurnStartedNotification {
            thread_id: "thread-1".to_string(),
            turn: AppServerTurn {
                id: "turn-1".to_string(),
                items_view: codex_app_server_protocol::TurnItemsView::Full,
                items: Vec::new(),
                status: AppServerTurnStatus::InProgress,
                error: None,
                started_at: Some(0),
                completed_at: None,
                duration_ms: None,
            },
        }),
        /*replay_kind*/ None,
    );

    assert!(chat.bottom_pane.is_task_running());
    let status = chat
        .bottom_pane
        .status_widget()
        .expect("status indicator should be visible");
    assert_eq!(status.header(), "Working");

    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::AgentMessage {
                id: "msg-1".to_string(),
                text: "Yes. What do you need?".to_string(),
                phase: Some(MessagePhase::FinalAnswer),
                memory_citation: None,
            },
        }),
        /*replay_kind*/ None,
    );

    let cells = drain_insert_history(&mut rx);
    assert_eq!(cells.len(), 1);
    assert!(lines_to_single_string(&cells[0]).contains("Yes. What do you need?"));
    assert!(chat.bottom_pane.is_task_running());

    chat.handle_server_notification(
        ServerNotification::TurnCompleted(TurnCompletedNotification {
            thread_id: "thread-1".to_string(),
            last_agent_message: Some("Yes. What do you need?".to_string()),
            turn: AppServerTurn {
                id: "turn-1".to_string(),
                items_view: codex_app_server_protocol::TurnItemsView::Full,
                items: Vec::new(),
                status: AppServerTurnStatus::Completed,
                error: None,
                started_at: None,
                completed_at: Some(0),
                duration_ms: None,
            },
        }),
        /*replay_kind*/ None,
    );

    assert!(!chat.bottom_pane.is_task_running());
    assert!(chat.bottom_pane.status_widget().is_none());
    assert!(drain_insert_history(&mut rx).is_empty());
}

#[tokio::test]
async fn turn_completed_recovers_canonical_answer_when_item_completion_is_missing() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    handle_turn_started(&mut chat, "turn-1");
    while rx.try_recv().is_ok() {}

    handle_agent_message_delta(&mut chat, "Partial streamed answer.\n");
    chat.run_commit_tick();
    while rx.try_recv().is_ok() {}

    let canonical = "Complete canonical answer.\n\nThe tail is present.";
    chat.handle_server_notification(
        ServerNotification::TurnCompleted(TurnCompletedNotification {
            thread_id: "thread-1".to_string(),
            last_agent_message: Some(canonical.to_string()),
            turn: AppServerTurn {
                id: "turn-1".to_string(),
                items_view: codex_app_server_protocol::TurnItemsView::Full,
                items: Vec::new(),
                status: AppServerTurnStatus::Completed,
                error: None,
                started_at: None,
                completed_at: Some(0),
                duration_ms: None,
            },
        }),
        /*replay_kind*/ None,
    );

    let mut consolidation = None;
    while let Ok(event) = rx.try_recv() {
        if let AppEvent::ConsolidateAgentMessage {
            source,
            scrollback_reflow,
            ..
        } = event
        {
            consolidation = Some((source, scrollback_reflow));
        }
    }
    let consolidation =
        consolidation.expect("turn completion should consolidate the canonical fallback");
    assert_eq!(consolidation.0, canonical);
    assert_eq!(
        consolidation.1,
        crate::app_event::ConsolidationScrollbackReflow::RequiredForCanonicalMismatch
    );
    assert_eq!(
        chat.transcript.last_agent_markdown.as_deref(),
        Some(canonical)
    );
}

#[tokio::test]
async fn live_app_server_turn_started_sets_feedback_turn_id() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    chat.handle_server_notification(
        ServerNotification::TurnStarted(TurnStartedNotification {
            thread_id: "thread-1".to_string(),
            turn: AppServerTurn {
                id: "turn-1".to_string(),
                items_view: codex_app_server_protocol::TurnItemsView::Full,
                items: Vec::new(),
                status: AppServerTurnStatus::InProgress,
                error: None,
                started_at: Some(0),
                completed_at: None,
                duration_ms: None,
            },
        }),
        /*replay_kind*/ None,
    );

    chat.open_feedback_note(
        crate::app_event::FeedbackCategory::Bug,
        /*include_logs*/ false,
    );
    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert_matches!(
        rx.try_recv(),
        Ok(AppEvent::SubmitFeedback {
            category: crate::app_event::FeedbackCategory::Bug,
            reason: None,
            turn_id: Some(turn_id),
            include_logs: false,
        }) if turn_id == "turn-1"
    );
}

#[tokio::test]
async fn live_app_server_warning_notification_renders_message() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    chat.handle_server_notification(
        ServerNotification::Warning(WarningNotification {
            thread_id: None,
            message: "Exceeded skills context budget of 2%. All skill descriptions were removed and 2 additional skills were not included in the model-visible skills list.".to_string(),
        }),
        /*replay_kind*/ None,
    );

    let cells = drain_insert_history(&mut rx);
    assert_eq!(cells.len(), 1, "expected one warning history cell");
    let rendered = lines_to_single_string(&cells[0]);
    let normalized = rendered.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        normalized.contains("Exceeded skills context budget of 2%."),
        "expected warning notification message, got {rendered}"
    );
    assert!(
        normalized.contains(
            "All skill descriptions were removed and 2 additional skills were not included in the model-visible skills list."
        ),
        "expected warning guidance, got {rendered}"
    );
}

#[tokio::test]
async fn live_app_server_guardian_warning_notification_renders_message() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    chat.handle_server_notification(
        ServerNotification::GuardianWarning(GuardianWarningNotification {
            thread_id: "thread-1".to_string(),
            message: "Automatic approval review denied the requested action.".to_string(),
        }),
        /*replay_kind*/ None,
    );

    let cells = drain_insert_history(&mut rx);
    assert_eq!(cells.len(), 1, "expected one warning history cell");
    let rendered = lines_to_single_string(&cells[0]);
    assert!(
        rendered.contains("Automatic approval review denied the requested action."),
        "expected guardian warning notification message, got {rendered}"
    );
}

#[tokio::test]
async fn live_app_server_config_warning_prefixes_summary() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    chat.handle_server_notification(
        ServerNotification::ConfigWarning(ConfigWarningNotification {
            summary: "Invalid configuration; using defaults.".to_string(),
            details: None,
            path: None,
            range: None,
        }),
        /*replay_kind*/ None,
    );

    let cells = drain_insert_history(&mut rx);
    assert_eq!(cells.len(), 1, "expected one warning history cell");
    let rendered = lines_to_single_string(&cells[0]);
    assert!(
        rendered.contains("Invalid configuration; using defaults."),
        "expected config warning summary, got {rendered}"
    );
}

#[tokio::test]
async fn live_app_server_file_change_item_started_preserves_changes() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::FileChange {
                id: "patch-1".to_string(),
                changes: vec![FileUpdateChange {
                    path: "foo.txt".to_string(),
                    kind: PatchChangeKind::Add,
                    diff: "hello\n".to_string(),
                }],
                status: AppServerPatchApplyStatus::InProgress,
            },
        }),
        /*replay_kind*/ None,
    );

    let cells = drain_insert_history(&mut rx);
    assert!(!cells.is_empty(), "expected patch history to be rendered");
    let transcript = lines_to_single_string(cells.last().expect("patch cell"));
    assert!(
        transcript.contains("Added foo.txt") || transcript.contains("Edited foo.txt"),
        "expected patch summary to include foo.txt, got: {transcript}"
    );
}

#[tokio::test]
async fn live_app_server_command_execution_strips_shell_wrapper() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let script = r#"python3 -c 'print("Hello, world!")'"#;
    let command =
        shlex::try_join(["/bin/zsh", "-lc", script]).expect("round-trippable shell wrapper");

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CommandExecution {
                id: "cmd-1".to_string(),
                command: command.clone(),
                cwd: test_path_buf("/tmp").abs().into(),
                process_id: None,
                source: AppServerCommandExecutionSource::UserShell,
                status: AppServerCommandExecutionStatus::InProgress,
                command_actions: vec![AppServerCommandAction::Unknown {
                    command: script.to_string(),
                }],
                aggregated_output: None,
                exit_code: None,
                duration_ms: None,
            },
        }),
        /*replay_kind*/ None,
    );
    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::CommandExecution {
                id: "cmd-1".to_string(),
                command,
                cwd: test_path_buf("/tmp").abs().into(),
                process_id: None,
                source: AppServerCommandExecutionSource::UserShell,
                status: AppServerCommandExecutionStatus::Completed,
                command_actions: vec![AppServerCommandAction::Unknown {
                    command: script.to_string(),
                }],
                aggregated_output: Some("Hello, world!\n".to_string()),
                exit_code: Some(0),
                duration_ms: Some(5),
            },
        }),
        /*replay_kind*/ None,
    );

    let cells = drain_insert_history(&mut rx);
    assert_eq!(
        cells.len(),
        1,
        "expected one completed command history cell"
    );
    let blob = lines_to_single_string(cells.first().expect("command cell"));
    assert_chatwidget_snapshot!(
        "live_app_server_command_execution_strips_shell_wrapper",
        blob
    );
}

#[tokio::test]
async fn collab_lifecycle_history_uses_same_fallback_names_as_active_panel() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.config.agent_nickname_candidates = Some(vec!["娄鑫".to_string(), "李超凡".to_string()]);
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000100").expect("valid thread id");
    let first_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000100").expect("valid thread id");
    let second_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000101").expect("valid thread id");

    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "spawn-first".to_string(),
                tool: AppServerCollabAgentTool::SpawnAgent,
                status: AppServerCollabAgentToolCallStatus::Completed,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![first_thread_id.to_string()],
                prompt: Some("Check lifecycle labels".to_string()),
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([(
                    first_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::Running,
                        message: None,
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-both".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![
                    first_thread_id.to_string(),
                    second_thread_id.to_string(),
                ],
                prompt: None,
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([
                    (
                        first_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Running,
                            message: None,
                        },
                    ),
                    (
                        second_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Running,
                            message: None,
                        },
                    ),
                ]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.force_active_subagent_elapsed_zero_for_test();
    let active_panel = chat
        .active_subagent_panel_lines_for_test(/*width*/ 100)
        .join("\n");
    assert!(
        active_panel.contains("娄鑫 · GPT 5 high · 运行中 · 0s")
            && active_panel.contains("李超凡 · GPT 5 high · 运行中 · 0s"),
        "active panel should lease configured fallback names, got:\n{active_panel}"
    );

    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-first".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::Completed,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![first_thread_id.to_string()],
                prompt: None,
                model: None,
                reasoning_effort: None,
                agents_states: HashMap::from([(
                    first_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::Completed,
                        message: Some("Done".to_string()),
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "close-first".to_string(),
                tool: AppServerCollabAgentTool::CloseAgent,
                status: AppServerCollabAgentToolCallStatus::Completed,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![first_thread_id.to_string()],
                prompt: None,
                model: None,
                reasoning_effort: None,
                agents_states: HashMap::new(),
            },
        }),
        /*replay_kind*/ None,
    );

    let rendered = drain_insert_history(&mut rx)
        .into_iter()
        .map(|lines| lines_to_single_string(&lines))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        rendered.contains("Spawned 娄鑫 (gpt-5 high)")
            && rendered.contains("Waiting for 2 agents")
            && rendered.contains("娄鑫: Completed - Done")
            && rendered.contains("Closed 娄鑫"),
        "lifecycle rows should use the same display-name resolver as the active panel, got:\n{rendered}"
    );
    assert!(
        !rendered.contains(&first_thread_id.to_string())
            && !rendered.contains(&second_thread_id.to_string()),
        "raw thread ids should not be primary lifecycle labels when fallback names exist, got:\n{rendered}"
    );
}

#[tokio::test]
async fn active_subagent_name_leases_avoid_collisions_and_reuse_released_names() {
    let (mut chat, _rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.config.agent_nickname_candidates = Some(vec!["娄鑫".to_string(), "李超凡".to_string()]);
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000100").expect("valid thread id");
    let first_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000100").expect("valid thread id");
    let second_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000102").expect("valid thread id");
    let third_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000104").expect("valid thread id");

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-first-two".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![
                    first_thread_id.to_string(),
                    second_thread_id.to_string(),
                ],
                prompt: None,
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([
                    (
                        first_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Running,
                            message: None,
                        },
                    ),
                    (
                        second_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Running,
                            message: None,
                        },
                    ),
                ]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.force_active_subagent_elapsed_zero_for_test();
    let active_panel = chat
        .active_subagent_panel_lines_for_test(/*width*/ 100)
        .join("\n");
    assert!(
        active_panel.contains("娄鑫 · GPT 5 high · 运行中 · 0s")
            && active_panel.contains("李超凡 · GPT 5 high · 运行中 · 0s"),
        "active leases should avoid collisions while candidates are available, got:\n{active_panel}"
    );

    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-first-terminal".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::Completed,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![
                    first_thread_id.to_string(),
                    second_thread_id.to_string(),
                ],
                prompt: None,
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([
                    (
                        first_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Completed,
                            message: Some("done".to_string()),
                        },
                    ),
                    (
                        second_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Running,
                            message: None,
                        },
                    ),
                ]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-second-third".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![
                    second_thread_id.to_string(),
                    third_thread_id.to_string(),
                ],
                prompt: None,
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([
                    (
                        second_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Running,
                            message: None,
                        },
                    ),
                    (
                        third_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Running,
                            message: None,
                        },
                    ),
                ]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.force_active_subagent_elapsed_zero_for_test();
    let active_panel_after_reuse = chat
        .active_subagent_panel_lines_for_test(/*width*/ 100)
        .join("\n");
    assert!(
        active_panel_after_reuse.contains("李超凡 · GPT 5 high · 运行中 · 0s")
            && active_panel_after_reuse.contains("娄鑫 · GPT 5 high · 运行中 · 0s"),
        "terminal agents should release names for deterministic reuse, got:\n{active_panel_after_reuse}"
    );
}

#[tokio::test]
async fn active_subagent_fallback_avoids_active_explicit_nickname_collision() {
    let (mut chat, _rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.config.agent_nickname_candidates = Some(vec!["娄鑫".to_string(), "李超凡".to_string()]);
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000100").expect("valid thread id");
    let explicit_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000106").expect("valid thread id");
    let fallback_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000108").expect("valid thread id");
    chat.set_collab_agent_metadata(
        explicit_thread_id,
        Some("娄鑫".to_string()),
        Some("audit".to_string()),
    );

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-explicit-and-fallback".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![
                    explicit_thread_id.to_string(),
                    fallback_thread_id.to_string(),
                ],
                prompt: None,
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([
                    (
                        explicit_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Running,
                            message: None,
                        },
                    ),
                    (
                        fallback_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Running,
                            message: None,
                        },
                    ),
                ]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.force_active_subagent_elapsed_zero_for_test();

    let active_panel = chat
        .active_subagent_panel_lines_for_test(/*width*/ 100)
        .join("\n");
    assert!(
        active_panel.contains("娄鑫 [audit] · GPT 5 high · 运行中 · 0s")
            && active_panel.contains("李超凡 · GPT 5 high · 运行中 · 0s"),
        "fallback should skip a candidate already used by an active explicit nickname, got:\n{active_panel}"
    );
    assert!(
        !active_panel.contains("娄鑫 · GPT 5 high · 运行中 · 0s"),
        "fallback row should not collide with the active explicit nickname, got:\n{active_panel}"
    );
}

#[tokio::test]
async fn live_app_server_collab_wait_items_render_history() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000001").expect("valid thread id");
    let receiver_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc1cc").expect("valid thread id");
    let other_receiver_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b96db334332d").expect("valid thread id");
    chat.set_collab_agent_metadata(
        receiver_thread_id,
        Some("Robie".to_string()),
        Some("explorer".to_string()),
    );
    chat.set_collab_agent_metadata(
        other_receiver_thread_id,
        Some("Ada".to_string()),
        Some("reviewer".to_string()),
    );

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-1".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![
                    receiver_thread_id.to_string(),
                    other_receiver_thread_id.to_string(),
                ],
                prompt: None,
                model: None,
                reasoning_effort: None,
                agents_states: HashMap::new(),
            },
        }),
        /*replay_kind*/ None,
    );

    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-1".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::Completed,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![
                    receiver_thread_id.to_string(),
                    other_receiver_thread_id.to_string(),
                ],
                prompt: None,
                model: None,
                reasoning_effort: None,
                agents_states: HashMap::from([
                    (
                        receiver_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Completed,
                            message: Some("Done".to_string()),
                        },
                    ),
                    (
                        other_receiver_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Running,
                            message: None,
                        },
                    ),
                ]),
            },
        }),
        /*replay_kind*/ None,
    );

    let combined = drain_insert_history(&mut rx)
        .into_iter()
        .map(|lines| lines_to_single_string(&lines))
        .collect::<Vec<_>>()
        .join("\n");
    assert_chatwidget_snapshot!("app_server_collab_wait_items_render_history", combined);
}

#[tokio::test]
async fn active_subagents_ctrl_o_opens_detail_popup_and_esc_closes() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000003").expect("valid thread id");
    let receiver_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc1cd").expect("valid thread id");
    chat.set_collab_agent_metadata(
        receiver_thread_id,
        Some("Robie".to_string()),
        Some("explorer".to_string()),
    );

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-1".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![receiver_thread_id.to_string()],
                prompt: Some("Inspect the recent app-server changes".to_string()),
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([(
                    receiver_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::Running,
                        message: Some("Reading protocol fixtures".to_string()),
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.force_active_subagent_elapsed_zero_for_test();

    let compact = chat
        .active_subagent_panel_lines_for_test(/*width*/ 100)
        .join("\n");
    assert!(
        compact.contains("Robie [explorer] · GPT 5 high · 运行中 · 0s"),
        "expected compact active subagent row to show nickname plus role, got:\n{compact}"
    );

    chat.handle_key_event(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL));

    let popup = render_bottom_popup(&chat, /*width*/ 100);
    assert!(
        popup.contains("Active Subagents") && popup.contains("Robie [explorer]"),
        "expected Ctrl+O details to show nickname plus role, got:\n{popup}"
    );
    assert_chatwidget_snapshot!("active_subagents_ctrl_o_detail_popup", popup);

    chat.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

    assert!(chat.no_modal_or_popup_active());
}

#[tokio::test]
async fn v2_subagents_render_hierarchy_and_enter_opens_selected_thread() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let research_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc201").expect("valid thread id");
    let tests_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc202").expect("valid thread id");
    let ui_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc203").expect("valid thread id");

    for (thread_id, path) in [
        (ui_thread_id, "/root/ui"),
        (tests_thread_id, "/root/research/tests"),
        (research_thread_id, "/root/research"),
    ] {
        chat.handle_server_notification(
            ServerNotification::ItemStarted(ItemStartedNotification {
                thread_id: "root-thread".to_string(),
                turn_id: "root-turn".to_string(),
                started_at_ms: 0,
                item: AppServerThreadItem::SubAgentActivity {
                    id: format!("activity-{thread_id}"),
                    kind: codex_app_server_protocol::SubAgentActivityKind::Started,
                    agent_thread_id: thread_id.to_string(),
                    agent_path: path.to_string(),
                },
            }),
            /*replay_kind*/ None,
        );
    }
    chat.set_v2_subagent_runtime(
        research_thread_id,
        "gpt-5.6".to_string(),
        Some(ReasoningEffortConfig::High),
        Some("梳理 V1/V2 tool routing 和状态事件".to_string()),
    );
    chat.set_v2_subagent_runtime(
        tests_thread_id,
        "gpt-5.6-terra".to_string(),
        Some(ReasoningEffortConfig::Medium),
        Some("等待新的回归测试任务".to_string()),
    );
    chat.set_v2_subagent_runtime(
        ui_thread_id,
        "gpt-5.6".to_string(),
        Some(ReasoningEffortConfig::High),
        Some("设计 V2 Subagents 状态面板".to_string()),
    );
    chat.set_v2_subagent_running(tests_thread_id, false);
    chat.force_active_subagent_elapsed_zero_for_test();

    let panel = chat
        .active_subagent_panel_lines_for_test(/*width*/ 100)
        .join("\n");
    assert!(panel.contains("● Agents  2 active · 1 idle"));
    assert!(panel.contains("├─ ● research"));
    assert!(panel.contains("GPT 5.6 High"));
    assert!(panel.contains("运行中 · 0s"));
    assert!(panel.contains("○ tests"));
    assert!(panel.contains("└─ ● ui"));
    assert!(!panel.contains("completed · 已折叠"));

    while rx.try_recv().is_ok() {}
    chat.handle_key_event(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL));
    let popup = render_bottom_popup(&chat, /*width*/ 100);
    assert!(popup.contains("research") && popup.contains("path: /root/research"));

    chat.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let selected_thread = std::iter::from_fn(|| rx.try_recv().ok()).find_map(|event| match event {
        AppEvent::SelectAgentThread(thread_id) => Some(thread_id),
        _ => None,
    });
    assert_eq!(selected_thread, Some(research_thread_id));
}

fn c7_m4_markdown_prompt() -> String {
    [
        "# Prompt Investigation",
        "",
        "Please inspect the subagent viewer contract before changing code.",
        "",
        "- Preserve lifecycle rows as short previews only.",
        "- Render the full prompt body in the explicit viewer.",
        "- Keep the `src/chatwidget/active_subagents.rs` path literal.",
        "",
        "```rust",
        "fn prompt_viewer_marker() {",
        "    println!(\"prompt code fence should render as code\");",
        "}",
        "```",
        "",
        "The full prompt continues past preview limits so the viewer must retain this tail marker:",
        "C7_M4_FULL_PROMPT_TAIL_MARKER_VISIBLE_ONLY_IN_VIEWER.",
    ]
    .join("\n")
}

fn c7_m4_markdown_report() -> String {
    [
        "# Report Summary",
        "",
        "The subagent found the relevant UI entry points.",
        "",
        "- Active Subagents owns the Ctrl+O entry point.",
        "- The viewer should expose original and translated modes.",
        "- Inline evidence such as `render_bottom_popup` should stay readable.",
        "",
        "```text",
        "report block should render without leaking raw fence markers",
        "```",
        "",
        "The full report also continues beyond preview limits with this tail marker:",
        "C7_M4_FULL_REPORT_TAIL_MARKER_VISIBLE_ONLY_IN_VIEWER.",
    ]
    .join("\n")
}

fn install_active_subagent_with_markdown_prompt_and_report(chat: &mut ChatWidget) {
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000400").expect("valid thread id");
    let receiver_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc4c7").expect("valid thread id");
    chat.set_collab_agent_metadata(
        receiver_thread_id,
        Some("Robie".to_string()),
        Some("explorer".to_string()),
    );

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-markdown-viewer".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![receiver_thread_id.to_string()],
                prompt: Some(c7_m4_markdown_prompt()),
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([(
                    receiver_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::Running,
                        message: Some(c7_m4_markdown_report()),
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.force_active_subagent_elapsed_zero_for_test();
}

fn has_original_and_translated_labels(rendered: &str) -> bool {
    (rendered.contains("Original") || rendered.contains("原文"))
        && (rendered.contains("Translated") || rendered.contains("翻译"))
}

#[tokio::test]
async fn c7_m4_active_subagents_ctrl_o_viewer_exposes_full_prompt_and_report_markdown() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    install_active_subagent_with_markdown_prompt_and_report(&mut chat);

    let compact = chat
        .active_subagent_panel_lines_for_test(/*width*/ 100)
        .join("\n");
    assert!(
        compact.contains("Robie [explorer] · GPT 5 high · 运行中 · 0s"),
        "active row should remain a concise status summary, got:\n{compact}"
    );
    assert!(
        !compact.contains("C7_M4_FULL_PROMPT_TAIL_MARKER_VISIBLE_ONLY_IN_VIEWER")
            && !compact.contains("C7_M4_FULL_REPORT_TAIL_MARKER_VISIBLE_ONLY_IN_VIEWER"),
        "active row should not leak full prompt/report tails, got:\n{compact}"
    );

    chat.handle_key_event(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL));
    let popup = render_bottom_popup(&chat, /*width*/ 120);

    assert!(
        popup.contains("Prompt Investigation")
            && popup.contains("Report Summary")
            && popup.contains("Preserve lifecycle rows as short previews only.")
            && popup.contains("Active Subagents owns the Ctrl+O entry point.")
            && popup.contains("prompt_viewer_marker")
            && popup.contains("report block should render without leaking raw fence markers")
            && popup.contains("C7_M4_FULL_PROMPT_TAIL_MARKER_VISIBLE_ONLY_IN_VIEWER")
            && popup.contains("C7_M4_FULL_REPORT_TAIL_MARKER_VISIBLE_ONLY_IN_VIEWER"),
        "viewer should render full prompt/report Markdown content, including tail markers beyond preview limits, got:\n{popup}"
    );
    assert!(
        popup.contains("Prompt") && popup.contains("Report"),
        "Ctrl+O viewer should expose explicit Prompt and Report surfaces, got:\n{popup}"
    );
    assert!(
        has_original_and_translated_labels(&popup),
        "viewer should expose original/translated mode labels without requiring external translation, got:\n{popup}"
    );
    let popup_dense: String = popup.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        !popup_dense.contains("未生成机器翻译；请以上方原文为准。"),
        "viewer must no longer hardcode the dead 'translation not generated' placeholder; prompt/report are now wired to the runtime translator, got:\n{popup}"
    );
    assert!(
        popup_dense.contains("正在生成机器翻译"),
        "without a reachable translation service the viewer should surface the pending-translation state for the wired translator, got:\n{popup}"
    );
    assert!(
        !popup.contains("```"),
        "viewer should render Markdown code blocks instead of exposing raw fence markers, got:\n{popup}"
    );
}

#[tokio::test]
async fn c7_m4_active_subagents_ctrl_o_viewer_preserves_core_content_at_narrow_width() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    install_active_subagent_with_markdown_prompt_and_report(&mut chat);

    chat.handle_key_event(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL));
    let narrow_popup = render_bottom_popup(&chat, /*width*/ 48);

    assert!(
        narrow_popup.contains("Active Subagents")
            && narrow_popup.contains("Robie")
            && narrow_popup.contains("Prompt")
            && narrow_popup.contains("Report"),
        "narrow Ctrl+O popup/viewer should retain the agent identity plus Prompt/Report entry points, got:\n{narrow_popup}"
    );
    assert!(
        narrow_popup.contains("Prompt Investigation")
            && narrow_popup.contains("Report Summary")
            && narrow_popup.contains("C7_M4_FULL_PROMPT_TAIL_MARKER_VISIBLE_ONLY_IN_VIEWER")
            && narrow_popup.contains("C7_M4_FULL_REPORT_TAIL_MARKER_VISIBLE_ONLY_IN_VIEWER"),
        "narrow viewer should not drop the core rendered heading or full-content tail markers, got:\n{narrow_popup}"
    );
    assert!(
        has_original_and_translated_labels(&narrow_popup),
        "narrow viewer should retain original/translated mode labels, got:\n{narrow_popup}"
    );
}

#[tokio::test]
async fn c7_m4_active_subagents_ctrl_o_viewer_tracks_selected_row() {
    fn prompt_with_marker(name: &str, marker: &str) -> String {
        format!(
            "# Prompt for {name}\n\n{}\n\n{marker}",
            "Preview filler keeps this full prompt marker past the compact row summary. ".repeat(4)
        )
    }

    fn report_with_marker(name: &str, marker: &str) -> String {
        format!(
            "# Report for {name}\n\n{}\n\n{marker}",
            "Preview filler keeps this full report marker past the compact row summary. ".repeat(4)
        )
    }

    fn start_active_subagent(
        chat: &mut ChatWidget,
        sender_thread_id: ThreadId,
        receiver_thread_id: ThreadId,
        wait_id: &str,
        nickname: &str,
        role: &str,
        prompt_marker: &str,
        report_marker: &str,
    ) {
        chat.set_collab_agent_metadata(
            receiver_thread_id,
            Some(nickname.to_string()),
            Some(role.to_string()),
        );

        chat.handle_server_notification(
            ServerNotification::ItemStarted(ItemStartedNotification {
                thread_id: "thread-1".to_string(),
                turn_id: format!("turn-{wait_id}"),
                started_at_ms: 0,
                item: AppServerThreadItem::CollabAgentToolCall {
                    id: wait_id.to_string(),
                    tool: AppServerCollabAgentTool::Wait,
                    status: AppServerCollabAgentToolCallStatus::InProgress,
                    sender_thread_id: sender_thread_id.to_string(),
                    receiver_thread_ids: vec![receiver_thread_id.to_string()],
                    prompt: Some(prompt_with_marker(nickname, prompt_marker)),
                    model: Some("gpt-5".to_string()),
                    reasoning_effort: Some(ReasoningEffortConfig::High),
                    agents_states: HashMap::from([(
                        receiver_thread_id.to_string(),
                        AppServerCollabAgentState {
                            status: AppServerCollabAgentStatus::Running,
                            message: Some(report_with_marker(nickname, report_marker)),
                        },
                    )]),
                },
            }),
            /*replay_kind*/ None,
        );
    }

    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000420").expect("valid thread id");
    let robie_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc420").expect("valid thread id");
    let ada_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc421").expect("valid thread id");
    let robie_prompt_marker = "C7_M4_ROBIE_FULL_PROMPT_MARKER_ONLY_IN_VIEWER";
    let robie_report_marker = "C7_M4_ROBIE_FULL_REPORT_MARKER_ONLY_IN_VIEWER";
    let ada_prompt_marker = "C7_M4_ADA_FULL_PROMPT_MARKER_ONLY_IN_VIEWER";
    let ada_report_marker = "C7_M4_ADA_FULL_REPORT_MARKER_ONLY_IN_VIEWER";

    start_active_subagent(
        &mut chat,
        sender_thread_id,
        robie_thread_id,
        "wait-robie-selected-row",
        "Robie",
        "explorer",
        robie_prompt_marker,
        robie_report_marker,
    );
    start_active_subagent(
        &mut chat,
        sender_thread_id,
        ada_thread_id,
        "wait-ada-selected-row",
        "Ada",
        "worker",
        ada_prompt_marker,
        ada_report_marker,
    );
    chat.force_active_subagent_elapsed_zero_for_test();

    chat.handle_key_event(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL));
    chat.handle_key_event(KeyEvent::from(KeyCode::Down));
    let popup = render_bottom_popup(&chat, /*width*/ 140);

    assert!(
        popup.contains("› 2. Ada [worker]"),
        "Down should select the second active subagent row before rendering the viewer, got:\n{popup}"
    );
    assert!(
        popup.contains(ada_prompt_marker) && popup.contains(ada_report_marker),
        "selected-row viewer should render Ada's full prompt/report markers, got:\n{popup}"
    );
    assert!(
        !popup.contains(robie_prompt_marker) && !popup.contains(robie_report_marker),
        "selected-row viewer should not render Robie's full prompt/report markers after selecting Ada, got:\n{popup}"
    );
}

#[tokio::test]
async fn c7_m4_collab_lifecycle_rows_keep_prompt_and_report_concise() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000401").expect("valid thread id");
    let receiver_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc4c8").expect("valid thread id");
    chat.set_collab_agent_metadata(
        receiver_thread_id,
        Some("Robie".to_string()),
        Some("explorer".to_string()),
    );

    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "spawn-markdown-viewer".to_string(),
                tool: AppServerCollabAgentTool::SpawnAgent,
                status: AppServerCollabAgentToolCallStatus::Completed,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![receiver_thread_id.to_string()],
                prompt: Some(c7_m4_markdown_prompt()),
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([(
                    receiver_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::Running,
                        message: None,
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-markdown-viewer".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::Completed,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![receiver_thread_id.to_string()],
                prompt: None,
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([(
                    receiver_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::Completed,
                        message: Some(c7_m4_markdown_report()),
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );

    let rendered = drain_insert_history(&mut rx)
        .into_iter()
        .map(|lines| lines_to_single_string(&lines))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        rendered.contains("Spawned Robie [explorer] (gpt-5 high)")
            && rendered.contains("Finished waiting")
            && rendered.contains("Robie [explorer]: Completed"),
        "expected concise lifecycle rows for spawn/wait events, got:\n{rendered}"
    );
    assert!(
        !rendered.contains("C7_M4_FULL_PROMPT_TAIL_MARKER_VISIBLE_ONLY_IN_VIEWER")
            && !rendered.contains("C7_M4_FULL_REPORT_TAIL_MARKER_VISIBLE_ONLY_IN_VIEWER")
            && !rendered.contains("```rust")
            && !rendered.contains("```text"),
        "ordinary lifecycle rows should not leak full prompt/report Markdown, got:\n{rendered}"
    );
}

#[tokio::test]
async fn active_subagents_refresh_existing_row_when_metadata_arrives_late() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000004").expect("valid thread id");
    let receiver_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc1ce").expect("valid thread id");

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-1".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![receiver_thread_id.to_string()],
                prompt: None,
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([(
                    receiver_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::Running,
                        message: None,
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );

    assert!(
        !chat
            .active_subagent_panel_lines_for_test(/*width*/ 100)
            .join("\n")
            .contains("019cff70"),
        "row should not expose thread id before metadata arrives"
    );

    chat.set_collab_agent_metadata(
        receiver_thread_id,
        Some("杨超".to_string()),
        Some("audit".to_string()),
    );

    let rendered = chat
        .active_subagent_panel_lines_for_test(/*width*/ 100)
        .join("\n");
    assert!(
        rendered.contains("杨超 [audit] · GPT 5 high · 运行中 · 0s"),
        "existing active row should refresh after metadata arrives, got:\n{rendered}"
    );
}

#[tokio::test]
async fn collab_history_wait_uses_fallback_name_before_metadata_arrives() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.config.agent_nickname_candidates = Some(vec!["陈阳宾".to_string()]);
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000009").expect("valid thread id");
    let receiver_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc1d3").expect("valid thread id");

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-history-fallback".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![receiver_thread_id.to_string()],
                prompt: None,
                model: None,
                reasoning_effort: None,
                agents_states: HashMap::new(),
            },
        }),
        /*replay_kind*/ None,
    );

    let rendered = drain_insert_history(&mut rx)
        .into_iter()
        .map(|lines| lines_to_single_string(&lines))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        rendered.contains("Waiting for 陈阳宾"),
        "history row should use fallback nickname before metadata arrives, got:\n{rendered}"
    );
    assert!(
        !rendered.contains(&receiver_thread_id.to_string()),
        "history row should not expose the raw subagent UUID, got:\n{rendered}"
    );
}

#[tokio::test]
async fn active_subagents_fallback_uses_configured_nickname_candidates() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.config.agent_nickname_candidates = Some(vec!["陈阳宾".to_string()]);
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000006").expect("valid thread id");
    let receiver_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc1d0").expect("valid thread id");

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-configured-fallback".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![receiver_thread_id.to_string()],
                prompt: None,
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([(
                    receiver_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::Running,
                        message: None,
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.force_active_subagent_elapsed_zero_for_test();

    let rendered = chat
        .active_subagent_panel_lines_for_test(/*width*/ 100)
        .join("\n");
    assert!(
        rendered.contains("陈阳宾 · GPT 5 high · 运行中 · 0s"),
        "fallback active row should use configured nickname pool, got:\n{rendered}"
    );
}

#[tokio::test]
async fn active_subagents_fallback_uses_role_specific_nickname_candidates() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.config.agent_nickname_candidates = Some(vec!["陈阳宾".to_string()]);
    chat.config.agent_roles.insert(
        "audit".to_string(),
        AgentRoleConfig {
            nickname_candidates: Some(vec!["Atlas".to_string()]),
            ..Default::default()
        },
    );
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000007").expect("valid thread id");
    let receiver_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc1d1").expect("valid thread id");
    chat.set_collab_agent_metadata(receiver_thread_id, None, Some("audit".to_string()));

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-role-specific-fallback".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![receiver_thread_id.to_string()],
                prompt: None,
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([(
                    receiver_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::Running,
                        message: None,
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.force_active_subagent_elapsed_zero_for_test();

    let rendered = chat
        .active_subagent_panel_lines_for_test(/*width*/ 100)
        .join("\n");
    assert!(
        rendered.contains("Atlas [audit] · GPT 5 high · 运行中 · 0s"),
        "role-specific fallback should win over top-level nickname pool, got:\n{rendered}"
    );
}

#[tokio::test]
async fn active_subagents_fallback_uses_built_in_role_nickname_candidates() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000008").expect("valid thread id");
    let receiver_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc1d2").expect("valid thread id");
    chat.set_collab_agent_metadata(receiver_thread_id, None, Some("explorer".to_string()));

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-built-in-role-fallback".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![receiver_thread_id.to_string()],
                prompt: None,
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([(
                    receiver_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::Running,
                        message: None,
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );
    chat.force_active_subagent_elapsed_zero_for_test();

    let rendered = chat
        .active_subagent_panel_lines_for_test(/*width*/ 100)
        .join("\n");
    assert!(
        rendered.contains("杨超 [explorer] · GPT 5 high · 运行中 · 0s")
            || rendered.contains("朱昊骋 [explorer] · GPT 5 high · 运行中 · 0s"),
        "built-in role fallback should use explorer nickname pool, got:\n{rendered}"
    );
}

#[tokio::test]
async fn active_subagents_terminal_tool_call_without_agent_state_removes_row() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000005").expect("valid thread id");
    let receiver_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b958ce5dc1cf").expect("valid thread id");

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-1".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![receiver_thread_id.to_string()],
                prompt: None,
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([(
                    receiver_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::Running,
                        message: None,
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );

    assert!(
        !chat
            .active_subagent_panel_lines_for_test(/*width*/ 100)
            .is_empty(),
        "row should appear while agent is running"
    );

    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "wait-1".to_string(),
                tool: AppServerCollabAgentTool::Wait,
                status: AppServerCollabAgentToolCallStatus::Completed,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![receiver_thread_id.to_string()],
                prompt: None,
                model: None,
                reasoning_effort: None,
                agents_states: HashMap::new(),
            },
        }),
        /*replay_kind*/ None,
    );

    assert!(
        chat.active_subagent_panel_lines_for_test(/*width*/ 100)
            .is_empty(),
        "terminal tool calls without agent state should remove stale rows"
    );
}

#[tokio::test]
async fn live_app_server_collab_spawn_completed_renders_requested_model_and_effort() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let sender_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b90000000002").expect("valid thread id");
    let spawned_thread_id =
        ThreadId::from_string("019cff70-2599-75e2-af72-b91781b41a8e").expect("valid thread id");
    chat.set_collab_agent_metadata(
        spawned_thread_id,
        Some("李宇轩".to_string()),
        Some("test".to_string()),
    );

    chat.handle_server_notification(
        ServerNotification::ItemStarted(ItemStartedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            started_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "spawn-1".to_string(),
                tool: AppServerCollabAgentTool::SpawnAgent,
                status: AppServerCollabAgentToolCallStatus::InProgress,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: Vec::new(),
                prompt: Some("Explore the repo".to_string()),
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::new(),
            },
        }),
        /*replay_kind*/ None,
    );

    chat.handle_server_notification(
        ServerNotification::ItemCompleted(ItemCompletedNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            completed_at_ms: 0,
            item: AppServerThreadItem::CollabAgentToolCall {
                id: "spawn-1".to_string(),
                tool: AppServerCollabAgentTool::SpawnAgent,
                status: AppServerCollabAgentToolCallStatus::Completed,
                sender_thread_id: sender_thread_id.to_string(),
                receiver_thread_ids: vec![spawned_thread_id.to_string()],
                prompt: Some("Explore the repo".to_string()),
                model: Some("gpt-5".to_string()),
                reasoning_effort: Some(ReasoningEffortConfig::High),
                agents_states: HashMap::from([(
                    spawned_thread_id.to_string(),
                    AppServerCollabAgentState {
                        status: AppServerCollabAgentStatus::PendingInit,
                        message: None,
                    },
                )]),
            },
        }),
        /*replay_kind*/ None,
    );

    let combined = drain_insert_history(&mut rx)
        .into_iter()
        .map(|lines| lines_to_single_string(&lines))
        .collect::<Vec<_>>()
        .join("\n");
    assert_chatwidget_snapshot!(
        "app_server_collab_spawn_completed_renders_requested_model_and_effort",
        combined
    );
}

#[tokio::test]
async fn live_app_server_failed_turn_does_not_duplicate_error_history() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    chat.handle_server_notification(
        ServerNotification::TurnStarted(TurnStartedNotification {
            thread_id: "thread-1".to_string(),
            turn: AppServerTurn {
                id: "turn-1".to_string(),
                items_view: codex_app_server_protocol::TurnItemsView::Full,
                items: Vec::new(),
                status: AppServerTurnStatus::InProgress,
                error: None,
                started_at: Some(0),
                completed_at: None,
                duration_ms: None,
            },
        }),
        /*replay_kind*/ None,
    );

    chat.handle_server_notification(
        ServerNotification::Error(ErrorNotification {
            error: AppServerTurnError {
                message: "permission denied".to_string(),
                codex_error_info: None,
                additional_details: None,
            },
            will_retry: false,
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
        }),
        /*replay_kind*/ None,
    );

    let first_cells = drain_insert_history(&mut rx);
    assert_eq!(first_cells.len(), 1);
    assert!(lines_to_single_string(&first_cells[0]).contains("permission denied"));

    chat.handle_server_notification(
        ServerNotification::TurnCompleted(TurnCompletedNotification {
            thread_id: "thread-1".to_string(),
            last_agent_message: None,
            turn: AppServerTurn {
                id: "turn-1".to_string(),
                items_view: codex_app_server_protocol::TurnItemsView::Full,
                items: Vec::new(),
                status: AppServerTurnStatus::Failed,
                error: Some(AppServerTurnError {
                    message: "permission denied".to_string(),
                    codex_error_info: None,
                    additional_details: None,
                }),
                started_at: None,
                completed_at: Some(0),
                duration_ms: None,
            },
        }),
        /*replay_kind*/ None,
    );

    assert!(drain_insert_history(&mut rx).is_empty());
    assert!(!chat.bottom_pane.is_task_running());
}

#[tokio::test]
async fn live_app_server_failed_turn_consolidates_streamed_answer() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    handle_turn_started(&mut chat, "turn-1");
    while rx.try_recv().is_ok() {}

    handle_agent_message_delta(&mut chat, "```diff\n+ streamed patch\n```\n");
    chat.run_commit_tick();
    while rx.try_recv().is_ok() {}

    handle_error(
        &mut chat,
        "stream disconnected before completion",
        /*codex_error_info*/ None,
    );

    let mut saw_consolidate = false;
    while let Ok(event) = rx.try_recv() {
        if let AppEvent::ConsolidateAgentMessage { source, .. } = event {
            saw_consolidate = true;
            assert!(
                source.contains("streamed patch"),
                "expected partial stream source to be consolidated, got {source:?}"
            );
        }
    }

    assert!(
        saw_consolidate,
        "failed turn should consolidate streamed cells before clearing the stream controller"
    );
}

#[tokio::test]
async fn live_app_server_stream_recovery_restores_previous_status_header() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    chat.handle_server_notification(
        ServerNotification::TurnStarted(TurnStartedNotification {
            thread_id: "thread-1".to_string(),
            turn: AppServerTurn {
                id: "turn-1".to_string(),
                items_view: codex_app_server_protocol::TurnItemsView::Full,
                items: Vec::new(),
                status: AppServerTurnStatus::InProgress,
                error: None,
                started_at: Some(0),
                completed_at: None,
                duration_ms: None,
            },
        }),
        /*replay_kind*/ None,
    );
    drain_insert_history(&mut rx);

    chat.handle_server_notification(
        ServerNotification::Error(ErrorNotification {
            error: AppServerTurnError {
                message: "Reconnecting... 1/5".to_string(),
                codex_error_info: Some(CodexErrorInfo::Other),
                additional_details: None,
            },
            will_retry: true,
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
        }),
        /*replay_kind*/ None,
    );
    drain_insert_history(&mut rx);

    chat.handle_server_notification(
        ServerNotification::AgentMessageDelta(
            codex_app_server_protocol::AgentMessageDeltaNotification {
                thread_id: "thread-1".to_string(),
                turn_id: "turn-1".to_string(),
                item_id: "item-1".to_string(),
                delta: "hello".to_string(),
            },
        ),
        /*replay_kind*/ None,
    );

    let status = chat
        .bottom_pane
        .status_widget()
        .expect("status indicator should be visible");
    assert_eq!(status.header(), "Working");
    assert_eq!(status.details(), None);
    assert!(chat.status_state.retry_status_header.is_none());
}

#[tokio::test]
async fn live_app_server_server_overloaded_error_renders_warning() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    chat.handle_server_notification(
        ServerNotification::TurnStarted(TurnStartedNotification {
            thread_id: "thread-1".to_string(),
            turn: AppServerTurn {
                id: "turn-1".to_string(),
                items_view: codex_app_server_protocol::TurnItemsView::Full,
                items: Vec::new(),
                status: AppServerTurnStatus::InProgress,
                error: None,
                started_at: Some(0),
                completed_at: None,
                duration_ms: None,
            },
        }),
        /*replay_kind*/ None,
    );
    drain_insert_history(&mut rx);

    chat.handle_server_notification(
        ServerNotification::Error(ErrorNotification {
            error: AppServerTurnError {
                message: "server overloaded".to_string(),
                codex_error_info: Some(CodexErrorInfo::ServerOverloaded),
                additional_details: None,
            },
            will_retry: false,
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
        }),
        /*replay_kind*/ None,
    );

    let cells = drain_insert_history(&mut rx);
    assert_eq!(cells.len(), 1);
    assert_eq!(lines_to_single_string(&cells[0]), "⚠ server overloaded\n");
    assert!(!chat.bottom_pane.is_task_running());
}

#[tokio::test]
async fn live_app_server_cyber_policy_error_renders_dedicated_notice() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    chat.handle_server_notification(
        ServerNotification::TurnStarted(TurnStartedNotification {
            thread_id: "thread-1".to_string(),
            turn: AppServerTurn {
                id: "turn-1".to_string(),
                items_view: codex_app_server_protocol::TurnItemsView::Full,
                items: Vec::new(),
                status: AppServerTurnStatus::InProgress,
                error: None,
                started_at: Some(0),
                completed_at: None,
                duration_ms: None,
            },
        }),
        /*replay_kind*/ None,
    );
    drain_insert_history(&mut rx);

    chat.handle_server_notification(
        ServerNotification::Error(ErrorNotification {
            error: AppServerTurnError {
                message: "server fallback message".to_string(),
                codex_error_info: Some(CodexErrorInfo::CyberPolicy),
                additional_details: None,
            },
            will_retry: false,
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
        }),
        /*replay_kind*/ None,
    );

    let cells = drain_insert_history(&mut rx);
    assert_eq!(cells.len(), 1);
    let rendered = lines_to_single_string(&cells[0]);
    assert!(rendered.contains("This content can't be shown"));
    assert!(rendered.contains("extra caution with cybersecurity requests"));
    assert!(!rendered.contains("server fallback message"));
    assert!(!chat.bottom_pane.is_task_running());
}

#[tokio::test]
async fn app_server_safety_access_errors_render_dedicated_notice() {
    let legacy_message = "Invalid prompt: we've limited access to this content for safety reasons.";
    let bio_policy_message = "This content was flagged for possible biological risk.";
    let cases = [
        ("legacy plain message", legacy_message.to_string()),
        (
            "legacy JSON message",
            json!({ "error": { "message": legacy_message } }).to_string(),
        ),
        ("bio policy plain message", bio_policy_message.to_string()),
        (
            "bio policy JSON message",
            json!({ "error": { "message": bio_policy_message } }).to_string(),
        ),
        (
            "bio policy code",
            json!({ "error": { "code": "bio_policy", "message": "copy may change" } }).to_string(),
        ),
    ];
    let mut rendered_cases = Vec::new();
    for (case, message) in cases {
        let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
        chat.handle_non_retry_error(message, /*codex_error_info*/ None);

        let cells = drain_insert_history(&mut rx);
        assert_eq!(cells.len(), 1);
        let rendered = lines_to_single_string(&cells[0]);
        assert!(rendered.contains("This content can't be shown"));
        assert!(rendered.contains("biological research"));
        rendered_cases.push((case, rendered));
    }

    let canonical = &rendered_cases[0].1;
    for (case, rendered) in &rendered_cases[1..] {
        assert_eq!(rendered, canonical, "unexpected rendering for {case}");
    }
    insta::assert_snapshot!(
        "app_server_bio_policy_error_renders_dedicated_notice",
        rendered_cases.last().unwrap().1.as_str()
    );
}

#[tokio::test]
async fn live_app_server_model_verification_renders_warning() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    chat.handle_server_notification(
        ServerNotification::ModelVerification(ModelVerificationNotification {
            thread_id: "thread-1".to_string(),
            turn_id: "turn-1".to_string(),
            verifications: vec![AppServerModelVerification::TrustedAccessForCyber],
        }),
        /*replay_kind*/ None,
    );

    let cells = drain_insert_history(&mut rx);
    assert_eq!(cells.len(), 1);
    let rendered = lines_to_single_string(&cells[0]);
    assert!(rendered.contains("multiple flags for possible cybersecurity risk"));
    assert!(rendered.contains("extra safety checks are on"));
    assert!(rendered.contains("Trusted Access for Cyber"));
    assert!(rendered.contains("https://chatgpt.com/cyber"));
}

#[tokio::test]
async fn live_app_server_invalid_thread_name_update_is_ignored() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let thread_id = ThreadId::new();
    chat.thread_id = Some(thread_id);
    chat.thread_name = Some("original name".to_string());

    chat.handle_server_notification(
        ServerNotification::ThreadNameUpdated(
            codex_app_server_protocol::ThreadNameUpdatedNotification {
                thread_id: "not-a-thread-id".to_string(),
                thread_name: Some("bad update".to_string()),
            },
        ),
        /*replay_kind*/ None,
    );

    assert_eq!(chat.thread_id, Some(thread_id));
    assert_eq!(chat.thread_name, Some("original name".to_string()));
}

#[tokio::test]
async fn live_app_server_thread_name_update_shows_resume_hint() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    let thread_id =
        ThreadId::from_string("123e4567-e89b-12d3-a456-426614174000").expect("thread id");
    chat.thread_id = Some(thread_id);

    chat.handle_server_notification(
        ServerNotification::ThreadNameUpdated(
            codex_app_server_protocol::ThreadNameUpdatedNotification {
                thread_id: thread_id.to_string(),
                thread_name: Some("review-fix".to_string()),
            },
        ),
        /*replay_kind*/ None,
    );

    assert_eq!(chat.thread_name, Some("review-fix".to_string()));
    let cells = drain_insert_history(&mut rx);
    assert_eq!(cells.len(), 1);
    let rendered = lines_to_single_string(&cells[0]);
    assert_chatwidget_snapshot!("thread_name_update_resume_hint", rendered);
}

#[tokio::test]
async fn live_app_server_thread_closed_requests_immediate_exit() {
    let (mut chat, mut rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;

    chat.handle_server_notification(
        ServerNotification::ThreadClosed(ThreadClosedNotification {
            thread_id: "thread-1".to_string(),
        }),
        /*replay_kind*/ None,
    );

    assert_matches!(rx.try_recv(), Ok(AppEvent::Exit(ExitMode::Immediate)));
}
