use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::items::TurnItem;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_utils_output_truncation::TruncationPolicy;
use codex_utils_output_truncation::approx_token_count;
use codex_utils_output_truncation::truncate_text;

use crate::stream_events_utils::raw_assistant_output_text_from_item;

const SUMMARY_CONTEXT_PERCENT: usize = 10;
pub(crate) const SUMMARY_MIN_TOKENS: usize = 128;
const SUMMARY_MAX_TOKENS: usize = 16_384;
const REPLACEMENT_CONTEXT_PERCENT: usize = 30;
const REPLACEMENT_MIN_TOKENS: usize = 4_096;
const REPLACEMENT_MAX_TOKENS: usize = 64_000;

pub(crate) const ACTIVE_CONTINUATION_CONTRACT: &str = "Continuation contract: resume the active task from the preserved user/tool tail. Do not re-answer completed requests or stop at a plan/status recap while executable work remains. Stop only for an explicit pause or required approval/input; missing optional recovery metadata is not a gate.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LocalCompactionBudget {
    pub(crate) summary_tokens: usize,
    pub(crate) replacement_tokens: usize,
}

impl LocalCompactionBudget {
    pub(crate) fn for_context_window(context_window: Option<i64>) -> Self {
        let Some(context_window) = context_window
            .and_then(|value| usize::try_from(value).ok())
            .filter(|value| *value > 0)
        else {
            return Self {
                summary_tokens: SUMMARY_MAX_TOKENS,
                replacement_tokens: REPLACEMENT_MAX_TOKENS,
            };
        };
        let summary_tokens = context_window
            .saturating_mul(SUMMARY_CONTEXT_PERCENT)
            .saturating_div(100)
            .clamp(SUMMARY_MIN_TOKENS, SUMMARY_MAX_TOKENS);
        let replacement_tokens = context_window
            .saturating_mul(REPLACEMENT_CONTEXT_PERCENT)
            .saturating_div(100)
            .clamp(REPLACEMENT_MIN_TOKENS, REPLACEMENT_MAX_TOKENS);
        Self {
            summary_tokens,
            replacement_tokens,
        }
    }

    pub(crate) fn replacement_limit(self, source_tokens: Option<i64>) -> usize {
        let Some(source_tokens) = source_tokens
            .and_then(|value| usize::try_from(value).ok())
            .filter(|value| *value > REPLACEMENT_MIN_TOKENS)
        else {
            return self.replacement_tokens;
        };
        self.replacement_tokens.min(source_tokens.saturating_sub(1))
    }
}

pub(crate) fn active_turn_tail(items: &[ResponseItem], active_turn_id: &str) -> Vec<ResponseItem> {
    let active_user_index = items.iter().position(|item| {
        if item.turn_id() != Some(active_turn_id) {
            return false;
        }
        matches!(
            crate::event_mapping::parse_turn_item(item),
            Some(TurnItem::UserMessage(user))
                if !user.message().starts_with(codex_prompts::SUMMARY_PREFIX)
        )
    });
    active_user_index
        .map(|index| items[index..].to_vec())
        .unwrap_or_default()
}

pub(crate) fn trim_oldest_active_tail_interaction(items: &mut Vec<ResponseItem>) -> bool {
    if items.len() <= 1 {
        return false;
    }

    let removed = items.remove(1);
    let remove_paired_output = match (&removed, items.get(1)) {
        (
            ResponseItem::FunctionCall { call_id, .. },
            Some(ResponseItem::FunctionCallOutput {
                call_id: output_call_id,
                ..
            }),
        )
        | (
            ResponseItem::CustomToolCall { call_id, .. },
            Some(ResponseItem::CustomToolCallOutput {
                call_id: output_call_id,
                ..
            }),
        ) => call_id == output_call_id,
        (
            ResponseItem::ToolSearchCall {
                call_id: Some(call_id),
                ..
            },
            Some(ResponseItem::ToolSearchOutput {
                call_id: Some(output_call_id),
                ..
            }),
        ) => call_id == output_call_id,
        _ => false,
    };
    if remove_paired_output {
        items.remove(1);
    }
    true
}

pub(crate) fn summary_from_attempt(items: &[ResponseItem]) -> CodexResult<String> {
    items
        .iter()
        .rev()
        .find_map(raw_assistant_output_text_from_item)
        .map(|summary| summary.trim().to_string())
        .filter(|summary| !summary.is_empty())
        .ok_or_else(|| {
            CodexErr::Stream(
                "local compaction completed without non-empty assistant summary text".into(),
                None,
            )
        })
}

pub(crate) fn bounded_summary_text(
    summary_suffix: &str,
    max_tokens: usize,
    preserve_active_continuation: bool,
) -> CodexResult<String> {
    let checkpoint_header = checkpoint_header(preserve_active_continuation);
    let header_tokens = approx_token_count(&checkpoint_header);
    if max_tokens <= header_tokens {
        return Err(CodexErr::Stream(
            "local compaction summary budget is too small for its checkpoint header".into(),
            None,
        ));
    }

    let mut suffix_budget = max_tokens.saturating_sub(header_tokens);
    loop {
        let suffix = truncate_text(summary_suffix, TruncationPolicy::Tokens(suffix_budget));
        let summary = format!("{checkpoint_header}{suffix}");
        let tokens = approx_token_count(&summary);
        if tokens <= max_tokens {
            return Ok(summary);
        }
        let overflow = tokens.saturating_sub(max_tokens).max(1);
        let next_budget = suffix_budget.saturating_sub(overflow);
        if next_budget == suffix_budget || next_budget == 0 {
            return Err(CodexErr::Stream(
                "local compaction could not fit the summary within its hard token budget".into(),
                None,
            ));
        }
        suffix_budget = next_budget;
    }
}

pub(crate) fn minimum_summary_tokens(preserve_active_continuation: bool) -> usize {
    approx_token_count(&checkpoint_header(preserve_active_continuation))
        .saturating_add(SUMMARY_MIN_TOKENS)
}

fn checkpoint_header(preserve_active_continuation: bool) -> String {
    if preserve_active_continuation {
        format!(
            "{}\n{ACTIVE_CONTINUATION_CONTRACT}\n\n",
            codex_prompts::SUMMARY_PREFIX
        )
    } else {
        format!("{}\n", codex_prompts::SUMMARY_PREFIX)
    }
}

pub(crate) fn truncate_to_token_budget(text: &str, max_tokens: usize) -> String {
    if approx_token_count(text) <= max_tokens {
        return text.to_string();
    }
    let mut truncation_tokens = max_tokens;
    loop {
        let truncated = truncate_text(text, TruncationPolicy::Tokens(truncation_tokens));
        let tokens = approx_token_count(&truncated);
        if tokens <= max_tokens {
            return truncated;
        }
        let overflow = tokens.saturating_sub(max_tokens).max(1);
        let next_tokens = truncation_tokens.saturating_sub(overflow);
        if next_tokens == truncation_tokens || next_tokens == 0 {
            return String::new();
        }
        truncation_tokens = next_tokens;
    }
}

pub(crate) fn build_replacement_history(
    summary_text: &str,
    active_tail: &[ResponseItem],
    summary_turn_id: &str,
) -> Vec<ResponseItem> {
    let mut summary = ResponseItem::Message {
        id: None,
        role: "user".to_string(),
        content: vec![ContentItem::InputText {
            text: summary_text.to_string(),
        }],
        phase: None,
        internal_chat_message_metadata_passthrough: None,
    };
    summary.set_turn_id_if_missing(summary_turn_id);

    let mut replacement = Vec::with_capacity(active_tail.len().saturating_add(1));
    replacement.push(summary);
    replacement.extend_from_slice(active_tail);
    replacement
}

#[cfg(test)]
#[path = "compact_local_tests.rs"]
mod tests;
