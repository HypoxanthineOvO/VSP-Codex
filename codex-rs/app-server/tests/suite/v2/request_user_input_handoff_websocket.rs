use super::connection_handling_websocket::DEFAULT_READ_TIMEOUT;
use super::connection_handling_websocket::WsClient;
use super::connection_handling_websocket::connect_websocket;
use super::connection_handling_websocket::create_config_toml;
use super::connection_handling_websocket::read_jsonrpc_message;
use super::connection_handling_websocket::read_response_for_id;
use super::connection_handling_websocket::send_request;
use super::connection_handling_websocket::spawn_websocket_server;
use super::request_user_input_websocket::ask_turn_params;
use super::request_user_input_websocket::initialize_ask_client;
use super::request_user_input_websocket::question_parts;
use super::request_user_input_websocket::read_response_and_question;
use super::request_user_input_websocket::read_turn_completed_without_second_question;
use super::request_user_input_websocket::send_question_response;
use anyhow::Context;
use anyhow::Result;
use app_test_support::create_final_assistant_message_sse_response;
use app_test_support::create_mock_responses_server_sequence_unchecked;
use app_test_support::create_request_user_input_sse_response;
use app_test_support::to_response;
use codex_app_server_protocol::JSONRPCMessage;
use codex_app_server_protocol::ServerRequest;
use codex_app_server_protocol::ThreadResumeParams;
use codex_app_server_protocol::ThreadResumeResponse;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::ThreadUnsubscribeParams;
use codex_app_server_protocol::ThreadUnsubscribeResponse;
use codex_app_server_protocol::ThreadUnsubscribeStatus;
use codex_app_server_protocol::TurnCompletedNotification;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::TurnStatus;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use tokio::time::timeout;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn unsubscribing_a_late_joiner_preserves_the_pending_question() -> Result<()> {
    let server = create_mock_responses_server_sequence_unchecked(vec![
        create_request_user_input_sse_response("question-call")?,
        create_final_assistant_message_sse_response("done")?,
    ])
    .await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri(), "never")?;

    let (mut process, bind_addr) = spawn_websocket_server(codex_home.path()).await?;
    let mut client_a = connect_websocket(bind_addr).await?;
    initialize_ask_client(
        &mut client_a,
        /*id*/ 1,
        "owner-a",
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
        read_response_and_question(&mut client_a, /*response_id*/ 3).await?;
    let turn: TurnStartResponse = to_response(turn_response)?;
    let (request_id, params) = question_parts(original_request)?;

    let mut client_b = connect_websocket(bind_addr).await?;
    initialize_ask_client(
        &mut client_b,
        /*id*/ 4,
        "owner-b",
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
    let (_resume_response, replayed) =
        read_response_and_question(&mut client_b, /*response_id*/ 5).await?;
    assert_eq!(
        question_parts(replayed)?,
        (request_id.clone(), params.clone())
    );

    send_request(
        &mut client_b,
        "thread/unsubscribe",
        /*id*/ 6,
        Some(serde_json::to_value(ThreadUnsubscribeParams {
            thread_id: started.thread.id.clone(),
        })?),
    )
    .await?;
    let unsubscribed: ThreadUnsubscribeResponse =
        to_response(read_response_for_id(&mut client_b, /*id*/ 6).await?)?;
    assert_eq!(unsubscribed.status, ThreadUnsubscribeStatus::Unsubscribed);

    send_question_response(&mut client_a, request_id, "answer-after-peer-unsubscribe").await?;
    let completed = read_turn_completed_without_second_question(
        &mut client_a,
        &started.thread.id,
        &turn.turn.id,
    )
    .await?;
    assert_eq!(completed.turn.status, TurnStatus::Completed);

    process.kill().await.context("failed to stop app-server")?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn running_resume_does_not_restore_question_after_answer_was_accepted() -> Result<()> {
    let server = create_mock_responses_server_sequence_unchecked(vec![
        create_request_user_input_sse_response("question-call")?,
        create_final_assistant_message_sse_response("done")?,
    ])
    .await;
    let codex_home = TempDir::new()?;
    create_config_toml(codex_home.path(), &server.uri(), "never")?;

    let (mut process, bind_addr) = spawn_websocket_server(codex_home.path()).await?;
    let mut client_a = connect_websocket(bind_addr).await?;
    initialize_ask_client(
        &mut client_a,
        /*id*/ 1,
        "answer-owner",
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
    let (turn_response, question) =
        read_response_and_question(&mut client_a, /*response_id*/ 3).await?;
    let turn: TurnStartResponse = to_response(turn_response)?;
    let (request_id, _) = question_parts(question)?;
    send_question_response(&mut client_a, request_id, "accepted-before-resume").await?;

    let mut client_b = connect_websocket(bind_addr).await?;
    initialize_ask_client(
        &mut client_b,
        /*id*/ 4,
        "late-resume",
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
    let (resumed, completion_status) = read_resume_and_completion_without_question(
        &mut client_b,
        /*response_id*/ 5,
        &started.thread.id,
        &turn.turn.id,
    )
    .await?;
    assert_eq!(resumed.thread.id, started.thread.id);
    assert_eq!(completion_status, TurnStatus::Completed);

    process.kill().await.context("failed to stop app-server")?;
    Ok(())
}

async fn read_resume_and_completion_without_question(
    stream: &mut WsClient,
    response_id: i64,
    thread_id: &str,
    turn_id: &str,
) -> Result<(ThreadResumeResponse, TurnStatus)> {
    timeout(DEFAULT_READ_TIMEOUT, async {
        let target_id = codex_app_server_protocol::RequestId::Integer(response_id);
        let mut response = None;
        let mut completion_status = None;
        loop {
            match read_jsonrpc_message(stream).await? {
                JSONRPCMessage::Response(candidate) if candidate.id == target_id => {
                    let candidate: ThreadResumeResponse = to_response(candidate)?;
                    completion_status = candidate
                        .thread
                        .turns
                        .iter()
                        .find(|turn| turn.id == turn_id)
                        .and_then(|turn| {
                            (turn.status == TurnStatus::Completed).then_some(turn.status.clone())
                        })
                        .or(completion_status);
                    response = Some(candidate);
                }
                JSONRPCMessage::Error(error) if error.id == target_id => {
                    anyhow::bail!("thread/resume failed: {:?}", error.error);
                }
                JSONRPCMessage::Request(request) => {
                    let request: ServerRequest = request.try_into()?;
                    if matches!(request, ServerRequest::ToolRequestUserInput { .. }) {
                        anyhow::bail!(
                            "running resume restored an already answered Ask: {request:?}"
                        );
                    }
                }
                JSONRPCMessage::Notification(notification)
                    if notification.method == "turn/completed" =>
                {
                    let candidate: TurnCompletedNotification = serde_json::from_value(
                        notification
                            .params
                            .context("turn/completed must include params")?,
                    )?;
                    if candidate.thread_id == thread_id && candidate.turn.id == turn_id {
                        completion_status = Some(candidate.turn.status);
                    }
                }
                _ => {}
            }
            if let (Some(response), Some(completion_status)) =
                (response.take(), completion_status.take())
            {
                return Ok::<_, anyhow::Error>((response, completion_status));
            }
        }
    })
    .await
    .context("timed out waiting for thread/resume response and turn completion")?
}
