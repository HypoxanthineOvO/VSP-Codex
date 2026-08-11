use super::*;
use crate::request_user_input::pending_request_from_rollout_items;
use crate::request_user_input::response_to_core;
use crate::request_user_input_fallback::workflow_surface_fallback_prompt;
use crate::server_request_error::is_turn_transition_server_request_error;
use crate::thread_state::ThreadConnectionRemoval;
use crate::thread_state::resolve_request_user_input_on_thread_listener;
use codex_app_server_protocol::ItemCompletedNotification;
use codex_app_server_protocol::ItemStartedNotification;
use codex_app_server_protocol::ServerRequest;
use codex_app_server_protocol::ServerRequestPayload;
use codex_app_server_protocol::Turn;
use codex_app_server_protocol::TurnCompletedNotification;
use codex_app_server_protocol::TurnItemsView;
use codex_app_server_protocol::TurnStatus;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Clone, Copy)]
pub(super) enum PendingRequestUserInputRecovery {
    LiveOnly,
    RolloutHistory,
}

pub(super) async fn retarget_and_dismiss_request_user_input_after_connection_removal(
    conversation_id: ThreadId,
    removed_connection: ConnectionId,
    removal: &ThreadConnectionRemoval,
    outgoing: &Arc<OutgoingMessageSender>,
) {
    if !removal.was_subscribed {
        return;
    }
    let request_ids = outgoing
        .remove_request_user_input_connection_for_thread(removed_connection, conversation_id)
        .await;
    for request_id in request_ids {
        outgoing
            .send_server_notification_to_connections(
                &[removed_connection],
                ServerNotification::ServerRequestResolved(ServerRequestResolvedNotification {
                    thread_id: conversation_id.to_string(),
                    request_id,
                }),
            )
            .await;
    }
}

pub(super) async fn handoff_request_user_input_after_connection_removal(
    conversation_id: ThreadId,
    removed_connection: ConnectionId,
    removal: &ThreadConnectionRemoval,
    outgoing: &Arc<OutgoingMessageSender>,
) {
    retarget_and_dismiss_request_user_input_after_connection_removal(
        conversation_id,
        removed_connection,
        removal,
        outgoing,
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn replay_recovered_pending_request_user_input(
    conversation_id: ThreadId,
    conversation: &Arc<CodexThread>,
    connection_id: ConnectionId,
    history_items: &[RolloutItem],
    recovery: PendingRequestUserInputRecovery,
    thread_state_manager: &ThreadStateManager,
    thread_state: &Arc<Mutex<ThreadState>>,
    outgoing: &Arc<OutgoingMessageSender>,
) {
    let pending_request = outgoing
        .pending_requests_for_thread(conversation_id)
        .await
        .into_iter()
        .rev()
        .find_map(|request| match request {
            ServerRequest::ToolRequestUserInput { request_id, params } => {
                Some((request_id, params))
            }
            _ => None,
        });
    let (event_turn_id, params, pending_request_id) =
        if let Some((request_id, params)) = pending_request {
            (params.turn_id.clone(), params, Some(request_id))
        } else {
            let pending_fallback = {
                let state = thread_state.lock().await;
                state.pending_unsupported_ask().cloned()
            };
            if let Some(pending_fallback) = pending_fallback {
                if pending_fallback.request_id.is_some() {
                    thread_state.lock().await.take_pending_unsupported_ask();
                    return;
                }
                (
                    pending_fallback.event_turn_id,
                    pending_fallback.params,
                    None,
                )
            } else {
                if matches!(recovery, PendingRequestUserInputRecovery::LiveOnly) {
                    return;
                }
                let Some((event_turn_id, params)) =
                    pending_request_from_rollout_items(conversation_id, history_items)
                else {
                    return;
                };
                (event_turn_id, params, None)
            }
        };

    if thread_state_manager
        .native_ask_ui_connections_for_thread(conversation_id)
        .await
        .contains(&connection_id)
    {
        if pending_request_id.is_some() {
            thread_state.lock().await.take_pending_unsupported_ask();
            outgoing
                .replay_request_user_input_to_connection_for_thread(connection_id, conversation_id)
                .await;
            return;
        }
        thread_state.lock().await.take_pending_unsupported_ask();
        let (request_id, rx) = outgoing
            .send_request_to_connections(
                Some(&[connection_id]),
                ServerRequestPayload::ToolRequestUserInput(params),
                Some(conversation_id),
            )
            .await;
        let conversation = Arc::clone(conversation);
        let thread_state = Arc::clone(thread_state);
        let response_outgoing = ThreadScopedOutgoingMessageSender::new(
            outgoing.clone(),
            vec![connection_id],
            conversation_id,
        );
        tokio::spawn(async move {
            let response = rx.await;
            resolve_request_user_input_on_thread_listener(
                &thread_state,
                request_id.clone(),
                &response_outgoing,
            )
            .await;
            thread_state
                .lock()
                .await
                .clear_pending_unsupported_ask(&request_id);
            let value = match response {
                Ok(Ok(value)) => value,
                Ok(Err(err)) if is_turn_transition_server_request_error(&err) => return,
                Ok(Err(err)) => {
                    tracing::error!("request failed with client error: {err:?}");
                    return;
                }
                Err(err) => {
                    tracing::error!("request failed: {err:?}");
                    return;
                }
            };
            let text = recovered_answer_prompt_for_core(response_to_core(
                crate::request_user_input::response_from_value(value),
            ));
            if let Err(err) = conversation
                .submit(Op::UserInput {
                    items: vec![CoreInputItem::Text {
                        text,
                        text_elements: Vec::new(),
                    }],
                    final_output_json_schema: None,
                    responsesapi_client_metadata: None,
                    additional_context: Default::default(),
                    thread_settings: Default::default(),
                })
                .await
            {
                tracing::error!("failed to continue recovered request_user_input answer: {err}");
            }
        });
    } else {
        let item = ThreadItem::AgentMessage {
            id: format!("{}-ask-fallback", params.item_id),
            text: workflow_surface_fallback_prompt(&params),
            phase: None,
            memory_citation: None,
        };
        let started = ItemStartedNotification {
            thread_id: conversation_id.to_string(),
            turn_id: event_turn_id.clone(),
            started_at_ms: now_unix_timestamp_ms(),
            item: item.clone(),
        };
        outgoing
            .send_server_notification_to_connections(
                &[connection_id],
                ServerNotification::ItemStarted(started),
            )
            .await;
        let completed = ItemCompletedNotification {
            thread_id: conversation_id.to_string(),
            turn_id: event_turn_id.clone(),
            completed_at_ms: now_unix_timestamp_ms(),
            item,
        };
        outgoing
            .send_server_notification_to_connections(
                &[connection_id],
                ServerNotification::ItemCompleted(completed),
            )
            .await;
        let completed = TurnCompletedNotification {
            thread_id: conversation_id.to_string(),
            last_agent_message: None,
            turn: Turn {
                id: event_turn_id.clone(),
                items: Vec::new(),
                items_view: TurnItemsView::NotLoaded,
                error: None,
                status: TurnStatus::Completed,
                started_at: None,
                completed_at: Some(now_unix_timestamp_ms()),
                duration_ms: None,
            },
        };
        outgoing
            .send_server_notification_to_connections(
                &[connection_id],
                ServerNotification::TurnCompleted(completed),
            )
            .await;
        thread_state.lock().await.set_pending_unsupported_ask(
            pending_request_id,
            event_turn_id,
            params,
        );
    }
}

fn recovered_answer_prompt_for_core(
    response: codex_protocol::request_user_input::RequestUserInputResponse,
) -> String {
    let mut lines = Vec::new();
    lines.push(
        "我恢复了一个挂起的 Ask 请求，但原进程的等待状态已经不可用。请把下面这些用户答案作为继续当前任务的上下文。"
            .to_string(),
    );
    for (question_id, answer) in response.answers {
        let text = answer
            .free_text
            .or_else(|| answer.ordered_option_ids.map(|ids| ids.join(", ")))
            .or_else(|| answer.selected_option_ids.map(|ids| ids.join(", ")))
            .or_else(|| (!answer.answers.is_empty()).then(|| answer.answers.join(", ")))
            .or_else(|| answer.legacy_answers.map(|answers| answers.join(", ")))
            .unwrap_or_default();
        lines.push(format!("{question_id}: {text}"));
    }
    lines.join("\n")
}

fn now_unix_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "request_user_input_handoff_tests.rs"]
mod tests;
