use super::*;
use codex_protocol::models::FunctionCallOutputBody;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::InternalChatMessageMetadataPassthrough;
use pretty_assertions::assert_eq;

fn message(role: &str, text: &str, turn_id: &str) -> ResponseItem {
    let content = if role == "assistant" {
        ContentItem::OutputText {
            text: text.to_string(),
        }
    } else {
        ContentItem::InputText {
            text: text.to_string(),
        }
    };
    ResponseItem::Message {
        id: None,
        role: role.to_string(),
        content: vec![content],
        phase: None,
        internal_chat_message_metadata_passthrough: Some(InternalChatMessageMetadataPassthrough {
            turn_id: Some(turn_id.to_string()),
        }),
    }
}

#[test]
fn active_tail_starts_at_the_current_real_user_and_keeps_tool_output() {
    let items = vec![
        message("user", "completed request", "old-turn"),
        message("assistant", "completed answer", "old-turn"),
        message("user", "active request", "active-turn"),
        ResponseItem::FunctionCallOutput {
            id: None,
            call_id: "call-1".to_string(),
            output: FunctionCallOutputPayload {
                body: FunctionCallOutputBody::Text("tool result".to_string()),
                success: Some(true),
            },
            internal_chat_message_metadata_passthrough: Some(
                InternalChatMessageMetadataPassthrough {
                    turn_id: Some("active-turn".to_string()),
                },
            ),
        },
    ];

    assert_eq!(active_turn_tail(&items, "active-turn"), items[2..]);
}

#[test]
fn trimming_active_tail_preserves_user_and_newest_context() {
    let mut items = vec![
        message("user", "active request", "active-turn"),
        ResponseItem::FunctionCall {
            id: None,
            name: "old_tool".to_string(),
            namespace: None,
            arguments: "{}".to_string(),
            call_id: "old-call".to_string(),
            internal_chat_message_metadata_passthrough: None,
        },
        ResponseItem::FunctionCallOutput {
            id: None,
            call_id: "old-call".to_string(),
            output: FunctionCallOutputPayload {
                body: FunctionCallOutputBody::Text("old output".to_string()),
                success: Some(true),
            },
            internal_chat_message_metadata_passthrough: None,
        },
        message("assistant", "newest context", "active-turn"),
    ];
    let expected = vec![items[0].clone(), items[3].clone()];

    assert!(trim_oldest_active_tail_interaction(&mut items));
    assert_eq!(items, expected);
}

#[test]
fn summary_validation_is_scoped_to_attempt_items() {
    let prior = message("assistant", "stale answer", "old-turn");
    assert!(summary_from_attempt(&[prior]).is_ok());
    assert!(summary_from_attempt(&[]).is_err());
}

#[test]
fn budgets_are_relative_and_absolutely_capped() {
    assert_eq!(
        LocalCompactionBudget::for_context_window(Some(200_000)),
        LocalCompactionBudget {
            summary_tokens: 16_384,
            replacement_tokens: 60_000,
        }
    );
    assert_eq!(
        LocalCompactionBudget::for_context_window(Some(1_000_000)),
        LocalCompactionBudget {
            summary_tokens: 16_384,
            replacement_tokens: 64_000,
        }
    );
}

#[test]
fn bounded_summary_never_exceeds_the_requested_budget() {
    let summary =
        bounded_summary_text(&"detail ".repeat(4_000), 128, false).expect("bounded summary");
    assert!(approx_token_count(&summary) <= 128);
}

#[test]
fn repeated_bounded_summaries_do_not_grow_between_generations() {
    let oversized = "summary detail ".repeat(4_000);
    let first = bounded_summary_text(&oversized, 128, false).expect("first bounded summary");
    let second = bounded_summary_text(&format!("{first}\n{oversized}"), 128, false)
        .expect("second bounded summary");

    assert!(approx_token_count(&first) <= 128);
    assert!(approx_token_count(&second) <= 128);
}

#[test]
fn active_continuation_contract_is_bounded_and_conditional() {
    let active = bounded_summary_text("current progress", 256, true).expect("active checkpoint");
    let completed =
        bounded_summary_text("completed progress", 256, false).expect("completed checkpoint");

    assert!(active.contains(ACTIVE_CONTINUATION_CONTRACT));
    assert!(!completed.contains(ACTIVE_CONTINUATION_CONTRACT));
    assert!(approx_token_count(&active) <= 256);
}

#[test]
fn active_continuation_summary_floor_includes_checkpoint_header() {
    let minimum = minimum_summary_tokens(true);
    let summary = bounded_summary_text("current progress", minimum, true)
        .expect("active checkpoint should fit its computed minimum");

    assert!(minimum > SUMMARY_MIN_TOKENS);
    assert!(summary.contains(ACTIVE_CONTINUATION_CONTRACT));
    assert!(approx_token_count(&summary) <= minimum);
}

#[test]
fn long_sources_must_contract_below_their_current_occupancy() {
    let budget = LocalCompactionBudget::for_context_window(Some(200_000));

    assert_eq!(budget.replacement_limit(Some(100_000)), 60_000);
    assert_eq!(budget.replacement_limit(Some(50_000)), 49_999);
}
