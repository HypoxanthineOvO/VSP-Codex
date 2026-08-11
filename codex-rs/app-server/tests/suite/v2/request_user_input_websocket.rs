use super::connection_handling_websocket::DEFAULT_READ_TIMEOUT;
use super::connection_handling_websocket::WsClient;
use super::connection_handling_websocket::connect_websocket;
use super::connection_handling_websocket::create_config_toml;
use super::connection_handling_websocket::read_error_for_id;
use super::connection_handling_websocket::read_jsonrpc_message;
use super::connection_handling_websocket::read_notification_for_method;
use super::connection_handling_websocket::read_response_for_id;
use super::connection_handling_websocket::send_jsonrpc;
use super::connection_handling_websocket::send_request;
use super::connection_handling_websocket::spawn_websocket_server;
use anyhow::Context;
use anyhow::Result;
use app_test_support::create_final_assistant_message_sse_response;
use app_test_support::create_mock_responses_server_sequence_unchecked;
use app_test_support::create_request_user_input_sse_response;
use app_test_support::to_response;
use codex_app_server_protocol::ClientInfo;
use codex_app_server_protocol::InitializeCapabilities;
use codex_app_server_protocol::InitializeParams;
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
use codex_app_server_protocol::ToolRequestUserInputParams;
use codex_app_server_protocol::TurnCompletedNotification;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::TurnStatus;
use codex_app_server_protocol::UserInput;
use codex_protocol::config_types::CollaborationMode;
use codex_protocol::config_types::ModeKind;
use codex_protocol::config_types::Settings;
use codex_protocol::openai_models::ReasoningEffort;
use pretty_assertions::assert_eq;
use serde_json::Value;
use tempfile::TempDir;
use tokio::time::Duration;
use tokio::time::timeout;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn resumed_native_ask_client_joins_pending_question() -> Result<()> {
    let responses = vec![
        create_request_user_input_sse_response("question-call")?,
        create_final_assistant_message_sse_response("done")?,
    ];
    let server = create_mock_responses_server_sequence_unchecked(responses).await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri(), "never")?;

    let (mut process, bind_addr) = spawn_websocket_server(codex_home.path()).await?;
    let mut client_a = connect_websocket(bind_addr).await?;
    initialize_ask_client(
        &mut client_a,
        /*id*/ 1,
        "native-ask-a",
        /*native_ask_ui*/ true,
    )
    .await?;

    send_request(
        &mut client_a,
        "thread/start",
        /*id*/ 2,
        Some(serde_json::to_value(ThreadStartParams {
            model: Some("mock-model".to_string()),
            ..Default::default()
        })?),
    )
    .await?;
    let started: ThreadStartResponse = to_response(read_response_for_id(&mut client_a, 2).await?)?;

    send_request(
        &mut client_a,
        "turn/start",
        /*id*/ 3,
        Some(serde_json::to_value(ask_turn_params(&started.thread.id))?),
    )
    .await?;
    let (turn_response, original_request) =
        read_response_and_question(&mut client_a, /*response_id*/ 3)
            .await
            .context("client A did not receive the original Question request")?;
    let turn: TurnStartResponse = to_response(turn_response)?;
    let (original_request_id, original_params) = question_parts(original_request)?;

    let mut client_b = connect_websocket(bind_addr).await?;
    initialize_ask_client(
        &mut client_b,
        /*id*/ 4,
        "native-ask-b",
        /*native_ask_ui*/ true,
    )
    .await?;
    send_request(
        &mut client_b,
        "thread/resume",
        /*id*/ 5,
        Some(serde_json::to_value(ThreadResumeParams {
            thread_id: started.thread.id.clone(),
            ..Default::default()
        })?),
    )
    .await?;

    let (resume_response, replayed_request) =
        read_response_and_question(&mut client_b, /*response_id*/ 5)
            .await
            .context("client B did not receive the resumed Question request")?;
    let resumed: ThreadResumeResponse = to_response(resume_response)?;
    assert_eq!(resumed.thread.id, started.thread.id);
    let (replayed_request_id, replayed_params) = question_parts(replayed_request)?;
    assert_eq!(replayed_request_id, original_request_id);
    assert_eq!(replayed_params, original_params);

    send_question_response(
        &mut client_a,
        replayed_request_id.clone(),
        "answer-from-first-terminal",
    )
    .await?;

    let resolved_a = read_notification_for_method(&mut client_a, "serverRequest/resolved").await?;
    let resolved_a: ServerRequestResolvedNotification = serde_json::from_value(
        resolved_a
            .params
            .context("serverRequest/resolved must include params")?,
    )?;
    assert_eq!(resolved_a.thread_id, started.thread.id);
    assert_eq!(resolved_a.request_id, original_request_id);

    let resolved_b = read_notification_for_method(&mut client_b, "serverRequest/resolved").await?;
    let resolved_b: ServerRequestResolvedNotification = serde_json::from_value(
        resolved_b
            .params
            .context("serverRequest/resolved must include params")?,
    )?;
    assert_eq!(resolved_b.thread_id, started.thread.id);
    assert_eq!(resolved_b.request_id, original_request_id);

    send_question_response(
        &mut client_b,
        replayed_request_id,
        "late-answer-from-resumed-terminal",
    )
    .await?;

    let completed = read_turn_completed_without_second_question(
        &mut client_b,
        &started.thread.id,
        &turn.turn.id,
    )
    .await?;
    assert_eq!(completed.turn.status, TurnStatus::Completed);
    assert_no_second_question(&mut client_b).await?;
    assert_no_additional_resolved(&mut client_a, &original_request_id).await?;
    assert_first_terminal_answer_won(&server).await?;

    process
        .kill()
        .await
        .context("failed to stop websocket app-server process")?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn resumed_fallback_client_exclusively_owns_the_pending_answer() -> Result<()> {
    let responses = vec![
        create_request_user_input_sse_response("question-call")?,
        create_final_assistant_message_sse_response("done")?,
    ];
    let server = create_mock_responses_server_sequence_unchecked(responses).await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri(), "never")?;

    let (mut process, bind_addr) = spawn_websocket_server(codex_home.path()).await?;
    let mut client_a = connect_websocket(bind_addr).await?;
    initialize_ask_client(
        &mut client_a,
        /*id*/ 1,
        "fallback-a",
        /*native_ask_ui*/ false,
    )
    .await?;
    send_request(
        &mut client_a,
        "thread/start",
        /*id*/ 2,
        Some(serde_json::to_value(ThreadStartParams {
            model: Some("mock-model".to_string()),
            ..Default::default()
        })?),
    )
    .await?;
    let started: ThreadStartResponse = to_response(read_response_for_id(&mut client_a, 2).await?)?;
    send_request(
        &mut client_a,
        "turn/start",
        /*id*/ 3,
        Some(serde_json::to_value(ask_turn_params(&started.thread.id))?),
    )
    .await?;
    let (_turn_response, fallback_a) = read_response_and_fallback(&mut client_a, /*id*/ 3).await?;
    assert!(fallback_a.contains("Request Input fallback"));

    let mut client_b = connect_websocket(bind_addr).await?;
    initialize_ask_client(
        &mut client_b,
        /*id*/ 4,
        "fallback-b",
        /*native_ask_ui*/ false,
    )
    .await?;
    send_request(
        &mut client_b,
        "thread/resume",
        /*id*/ 5,
        Some(serde_json::to_value(ThreadResumeParams {
            thread_id: started.thread.id.clone(),
            ..Default::default()
        })?),
    )
    .await?;
    let (_resume_response, fallback_b) =
        read_response_and_fallback(&mut client_b, /*id*/ 5).await?;
    assert!(fallback_b.contains("Request Input fallback"));

    send_request(
        &mut client_a,
        "turn/start",
        /*id*/ 6,
        Some(serde_json::to_value(fallback_answer_params(
            &started.thread.id,
        ))?),
    )
    .await?;
    let stale_error = read_error_for_id(&mut client_a, /*id*/ 6).await?;
    assert!(
        stale_error
            .error
            .message
            .contains("belongs to another resumed connection"),
        "stale fallback owner should be rejected: {stale_error:?}"
    );

    send_request(
        &mut client_b,
        "turn/start",
        /*id*/ 7,
        Some(serde_json::to_value(fallback_answer_params(
            &started.thread.id,
        ))?),
    )
    .await?;
    let answer_response: TurnStartResponse =
        to_response(read_response_for_id(&mut client_b, /*id*/ 7).await?)?;
    assert_eq!(answer_response.turn.status, TurnStatus::InProgress);
    let completed = read_turn_completed_without_second_question(
        &mut client_b,
        &started.thread.id,
        &answer_response.turn.id,
    )
    .await?;
    assert_eq!(completed.turn.status, TurnStatus::Completed);

    process
        .kill()
        .await
        .context("failed to stop websocket app-server process")?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn disconnecting_a_late_joiner_preserves_the_pending_question() -> Result<()> {
    let responses = vec![
        create_request_user_input_sse_response("question-call")?,
        create_final_assistant_message_sse_response("done")?,
    ];
    let server = create_mock_responses_server_sequence_unchecked(responses).await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri(), "never")?;

    let (mut process, bind_addr) = spawn_websocket_server(codex_home.path()).await?;
    let mut client_a = connect_websocket(bind_addr).await?;
    initialize_ask_client(
        &mut client_a,
        /*id*/ 1,
        "native-ask-a",
        /*native_ask_ui*/ true,
    )
    .await?;
    send_request(
        &mut client_a,
        "thread/start",
        /*id*/ 2,
        Some(serde_json::to_value(ThreadStartParams {
            model: Some("mock-model".to_string()),
            ..Default::default()
        })?),
    )
    .await?;
    let started: ThreadStartResponse = to_response(read_response_for_id(&mut client_a, 2).await?)?;
    send_request(
        &mut client_a,
        "turn/start",
        /*id*/ 3,
        Some(serde_json::to_value(ask_turn_params(&started.thread.id))?),
    )
    .await?;
    let (turn_response, original_request) =
        read_response_and_question(&mut client_a, /*id*/ 3).await?;
    let turn: TurnStartResponse = to_response(turn_response)?;
    let (original_request_id, original_params) = question_parts(original_request)?;

    let mut client_b = connect_websocket(bind_addr).await?;
    initialize_ask_client(
        &mut client_b,
        /*id*/ 4,
        "native-ask-b",
        /*native_ask_ui*/ true,
    )
    .await?;
    send_request(
        &mut client_b,
        "thread/resume",
        /*id*/ 5,
        Some(serde_json::to_value(ThreadResumeParams {
            thread_id: started.thread.id.clone(),
            ..Default::default()
        })?),
    )
    .await?;
    let (_resume_response, replayed_to_b) =
        read_response_and_question(&mut client_b, /*id*/ 5).await?;
    let (replayed_request_id, replayed_params) = question_parts(replayed_to_b)?;
    assert_eq!(replayed_request_id, original_request_id);
    assert_eq!(replayed_params, original_params);
    client_b.close(None).await?;
    send_question_response(
        &mut client_a,
        original_request_id,
        "answer-after-peer-disconnect",
    )
    .await?;

    let completed = read_turn_completed_without_second_question(
        &mut client_a,
        &started.thread.id,
        &turn.turn.id,
    )
    .await?;
    assert_eq!(completed.turn.status, TurnStatus::Completed);

    process
        .kill()
        .await
        .context("failed to stop websocket app-server process")?;
    Ok(())
}

pub(super) fn ask_turn_params(thread_id: &str) -> TurnStartParams {
    TurnStartParams {
        thread_id: thread_id.to_string(),
        input: vec![UserInput::Text {
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

fn fallback_answer_params(thread_id: &str) -> TurnStartParams {
    TurnStartParams {
        thread_id: thread_id.to_string(),
        input: vec![UserInput::Text {
            text: "1. Yes".to_string(),
            text_elements: Vec::new(),
        }],
        ..Default::default()
    }
}

pub(super) async fn send_question_response(
    stream: &mut WsClient,
    request_id: RequestId,
    answer: &str,
) -> Result<()> {
    send_jsonrpc(
        stream,
        JSONRPCMessage::Response(JSONRPCResponse {
            id: request_id,
            result: serde_json::json!({
                "answers": {
                    "confirm_path": { "answers": [answer] }
                }
            }),
        }),
    )
    .await
}

async fn assert_first_terminal_answer_won(server: &wiremock::MockServer) -> Result<()> {
    let requests = server
        .received_requests()
        .await
        .context("failed to read model requests")?;
    let follow_up = requests
        .iter()
        .filter(|request| request.url.path().ends_with("/responses"))
        .nth(1)
        .context("missing model request after Question response")?;
    let body = follow_up
        .body_json::<Value>()
        .context("follow-up model request body should be JSON")?;
    let body = body.to_string();
    assert!(
        body.contains("answer-from-first-terminal"),
        "follow-up request must use the first terminal's answer: {body}"
    );
    assert!(
        !body.contains("late-answer-from-resumed-terminal"),
        "late answer must be ignored after the first valid response: {body}"
    );
    Ok(())
}

pub(super) async fn initialize_ask_client(
    stream: &mut WsClient,
    id: i64,
    client_name: &str,
    native_ask_ui: bool,
) -> Result<()> {
    send_request(
        stream,
        "initialize",
        id,
        Some(serde_json::to_value(InitializeParams {
            client_info: ClientInfo {
                name: client_name.to_string(),
                title: Some("Native Ask WebSocket Test Client".to_string()),
                version: "0.1.0".to_string(),
            },
            capabilities: Some(InitializeCapabilities {
                experimental_api: true,
                native_ask_ui,
                ..Default::default()
            }),
        })?),
    )
    .await?;
    read_response_for_id(stream, id).await?;
    Ok(())
}

pub(super) async fn read_response_and_question(
    stream: &mut WsClient,
    response_id: i64,
) -> Result<(JSONRPCResponse, ServerRequest)> {
    let target_id = RequestId::Integer(response_id);
    timeout(DEFAULT_READ_TIMEOUT, async {
        let mut response = None;
        let mut question = None;
        loop {
            let message = read_jsonrpc_message(stream).await?;
            match message {
                JSONRPCMessage::Response(candidate) if candidate.id == target_id => {
                    response = Some(candidate);
                }
                JSONRPCMessage::Error(error) if error.id == target_id => {
                    anyhow::bail!("request {target_id} failed: {:?}", error.error);
                }
                JSONRPCMessage::Request(request) => {
                    let request: ServerRequest = request.try_into()?;
                    if matches!(request, ServerRequest::ToolRequestUserInput { .. }) {
                        question = Some(request);
                    }
                }
                JSONRPCMessage::Notification(notification)
                    if notification.method == "turn/completed" && question.is_none() =>
                {
                    anyhow::bail!(
                        "turn completed before Question request: {:?}",
                        notification.params
                    );
                }
                _ => {}
            }
            if response.is_some() && question.is_some() {
                return Ok::<_, anyhow::Error>((
                    response.take().expect("response checked above"),
                    question.take().expect("question checked above"),
                ));
            }
        }
    })
    .await
    .context("timed out waiting for response and Question request")?
}

async fn read_response_and_fallback(
    stream: &mut WsClient,
    response_id: i64,
) -> Result<(JSONRPCResponse, String)> {
    let target_id = RequestId::Integer(response_id);
    timeout(DEFAULT_READ_TIMEOUT, async {
        let mut response = None;
        let mut fallback = None;
        loop {
            match read_jsonrpc_message(stream).await? {
                JSONRPCMessage::Response(candidate) if candidate.id == target_id => {
                    response = Some(candidate);
                }
                JSONRPCMessage::Error(error) if error.id == target_id => {
                    anyhow::bail!("request {target_id} failed: {:?}", error.error);
                }
                JSONRPCMessage::Request(request) => {
                    let request: ServerRequest = request.try_into()?;
                    if matches!(request, ServerRequest::ToolRequestUserInput { .. }) {
                        anyhow::bail!(
                            "fallback-only client received native Question request: {request:?}"
                        );
                    }
                }
                JSONRPCMessage::Notification(notification)
                    if notification.method == "item/started" =>
                {
                    let started: ItemStartedNotification = serde_json::from_value(
                        notification
                            .params
                            .context("item/started must include params")?,
                    )?;
                    if let ThreadItem::AgentMessage { text, .. } = started.item
                        && text.contains("Request Input fallback")
                    {
                        fallback = Some(text);
                    }
                }
                _ => {}
            }
            if response.is_some() && fallback.is_some() {
                return Ok::<_, anyhow::Error>((
                    response.take().expect("response checked above"),
                    fallback.take().expect("fallback checked above"),
                ));
            }
        }
    })
    .await
    .context("timed out waiting for response and fallback prompt")?
}

pub(super) fn question_parts(
    request: ServerRequest,
) -> Result<(RequestId, ToolRequestUserInputParams)> {
    let ServerRequest::ToolRequestUserInput { request_id, params } = request else {
        anyhow::bail!("expected ToolRequestUserInput request, got {request:?}");
    };
    Ok((request_id, params))
}

pub(super) async fn read_turn_completed_without_second_question(
    stream: &mut WsClient,
    thread_id: &str,
    turn_id: &str,
) -> Result<TurnCompletedNotification> {
    timeout(DEFAULT_READ_TIMEOUT, async {
        loop {
            match read_jsonrpc_message(stream).await? {
                JSONRPCMessage::Request(request) => {
                    let request: ServerRequest = request.try_into()?;
                    if matches!(request, ServerRequest::ToolRequestUserInput { .. }) {
                        anyhow::bail!("received a second pending Question request: {request:?}");
                    }
                }
                JSONRPCMessage::Notification(notification)
                    if notification.method == "turn/completed" =>
                {
                    let completed: TurnCompletedNotification = serde_json::from_value(
                        notification
                            .params
                            .context("turn/completed must include params")?,
                    )?;
                    if completed.thread_id == thread_id && completed.turn.id == turn_id {
                        return Ok::<_, anyhow::Error>(completed);
                    }
                }
                _ => {}
            }
        }
    })
    .await
    .context("timed out waiting for resumed Question turn to complete")?
}

async fn assert_no_second_question(stream: &mut WsClient) -> Result<()> {
    let second_question = timeout(Duration::from_millis(300), async {
        loop {
            if let JSONRPCMessage::Request(request) = read_jsonrpc_message(stream).await? {
                let request: ServerRequest = request.try_into()?;
                if matches!(request, ServerRequest::ToolRequestUserInput { .. }) {
                    return Ok::<_, anyhow::Error>(request);
                }
            }
        }
    })
    .await;

    match second_question {
        Err(_) => Ok(()),
        Ok(Ok(request)) => anyhow::bail!("received a second pending Question request: {request:?}"),
        Ok(Err(err)) => Err(err),
    }
}

async fn assert_no_additional_resolved(
    stream: &mut WsClient,
    request_id: &RequestId,
) -> Result<()> {
    let duplicate = timeout(Duration::from_millis(300), async {
        loop {
            if let JSONRPCMessage::Notification(notification) = read_jsonrpc_message(stream).await?
                && notification.method == "serverRequest/resolved"
            {
                let resolved: ServerRequestResolvedNotification = serde_json::from_value(
                    notification
                        .params
                        .context("serverRequest/resolved must include params")?,
                )?;
                if &resolved.request_id == request_id {
                    return Ok::<_, anyhow::Error>(resolved);
                }
            }
        }
    })
    .await;

    match duplicate {
        Err(_) => Ok(()),
        Ok(Ok(resolved)) => {
            anyhow::bail!("previous owner received duplicate serverRequest/resolved: {resolved:?}")
        }
        Ok(Err(err)) => Err(err),
    }
}
