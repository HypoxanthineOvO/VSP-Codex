use super::*;
use crate::outgoing_message::OutgoingEnvelope;
use crate::outgoing_message::OutgoingMessage;
use codex_app_server_protocol::ToolRequestUserInputParams;
use pretty_assertions::assert_eq;
use serde_json::json;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn connection_removal_preserves_other_request_recipients() {
    let (tx, mut rx) = mpsc::channel::<OutgoingEnvelope>(4);
    let outgoing = Arc::new(OutgoingMessageSender::new(
        tx,
        codex_analytics::AnalyticsEventsClient::disabled(),
    ));
    let conversation_id = ThreadId::new();
    let previous_owner = ConnectionId(1);
    let new_owner = ConnectionId(2);
    let (request_id, mut waiter) = outgoing
        .send_request_to_connections(
            Some(&[previous_owner, new_owner]),
            ServerRequestPayload::ToolRequestUserInput(ToolRequestUserInputParams {
                thread_id: conversation_id.to_string(),
                turn_id: "turn-1".to_string(),
                item_id: "call-1".to_string(),
                summary: None,
                description: None,
                questions: vec![],
                auto_resolution_ms: None,
            }),
            Some(conversation_id),
        )
        .await;
    let _request = rx.recv().await.expect("request should be sent");
    let _request = rx.recv().await.expect("second request should be sent");

    retarget_and_dismiss_request_user_input_after_connection_removal(
        conversation_id,
        previous_owner,
        &ThreadConnectionRemoval {
            was_subscribed: true,
            previous_interactive_connection: Some(previous_owner),
            interactive_connection: Some(new_owner),
            has_connections: true,
        },
        &outgoing,
    )
    .await;

    let resolved = rx
        .recv()
        .await
        .expect("previous owner should receive resolved");
    let OutgoingEnvelope::ToConnection {
        connection_id,
        message:
            OutgoingMessage::AppServerNotification(ServerNotification::ServerRequestResolved(
                notification,
            )),
        ..
    } = resolved
    else {
        panic!("expected a targeted serverRequest/resolved notification");
    };
    assert_eq!(connection_id, previous_owner);
    assert_eq!(
        notification,
        ServerRequestResolvedNotification {
            thread_id: conversation_id.to_string(),
            request_id: request_id.clone(),
        }
    );

    outgoing
        .notify_client_response(
            previous_owner,
            request_id.clone(),
            json!({"answers": {"confirm": {"answers": ["stale"]}}}),
        )
        .await;
    assert_eq!(
        waiter.try_recv(),
        Err(tokio::sync::oneshot::error::TryRecvError::Empty)
    );

    let accepted = json!({"answers": {"confirm": {"answers": ["accepted"]}}});
    outgoing
        .notify_client_response(new_owner, request_id, accepted.clone())
        .await;
    assert_eq!(
        timeout(Duration::from_secs(1), waiter)
            .await
            .expect("new owner response should not time out")
            .expect("waiter should receive a callback"),
        Ok(accepted)
    );
}
