use codex_app_server_protocol::ToolRequestUserInputOption;
use codex_app_server_protocol::ToolRequestUserInputParams;
use codex_app_server_protocol::ToolRequestUserInputQuestion;
use codex_app_server_protocol::ToolRequestUserInputQuestionKind;
use codex_app_server_protocol::ToolRequestUserInputResponse;
use codex_protocol::ThreadId;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::RequestUserInputEvent;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::TurnAbortReason;
use codex_protocol::request_user_input::RequestUserInputAnswer as CoreRequestUserInputAnswer;
use codex_protocol::request_user_input::RequestUserInputQuestionKind as CoreRequestUserInputQuestionKind;
use codex_protocol::request_user_input::RequestUserInputResponse as CoreRequestUserInputResponse;
use std::collections::HashMap;

pub(crate) fn params_from_event(
    conversation_id: ThreadId,
    request: RequestUserInputEvent,
) -> ToolRequestUserInputParams {
    let questions = request
        .questions
        .into_iter()
        .map(|question| ToolRequestUserInputQuestion {
            id: question.id,
            kind: question.kind.map(convert_question_kind),
            header: question.header,
            question: question.question,
            description: question.description,
            is_other: question.is_other,
            is_secret: question.is_secret,
            options: question.options.map(|options| {
                options
                    .into_iter()
                    .map(|option| ToolRequestUserInputOption {
                        id: option.id,
                        label: option.label,
                        description: option.description,
                    })
                    .collect()
            }),
            workflow_decision_type: question.workflow_decision_type,
            intent_only: question.intent_only,
            grants_approval: question.grants_approval,
        })
        .collect();
    ToolRequestUserInputParams {
        thread_id: conversation_id.to_string(),
        turn_id: request.turn_id,
        item_id: request.call_id,
        summary: request.summary,
        description: request.description,
        questions,
        auto_resolution_ms: request.auto_resolution_ms,
    }
}

pub(crate) fn response_to_core(
    response: ToolRequestUserInputResponse,
) -> CoreRequestUserInputResponse {
    CoreRequestUserInputResponse {
        answers: response
            .answers
            .into_iter()
            .map(|(id, answer)| {
                (
                    id,
                    CoreRequestUserInputAnswer {
                        selected_option_ids: answer.selected_option_ids,
                        ordered_option_ids: answer.ordered_option_ids,
                        free_text: answer.free_text,
                        legacy_answers: answer.legacy_answers,
                        answers: answer.answers,
                    },
                )
            })
            .collect(),
    }
}

pub(crate) fn response_from_value(value: serde_json::Value) -> ToolRequestUserInputResponse {
    serde_json::from_value::<ToolRequestUserInputResponse>(value).unwrap_or_else(|err| {
        tracing::error!("failed to deserialize ToolRequestUserInputResponse: {err}");
        ToolRequestUserInputResponse {
            answers: HashMap::new(),
        }
    })
}

pub(crate) fn pending_request_from_rollout_items(
    conversation_id: ThreadId,
    items: &[RolloutItem],
) -> Option<(String, ToolRequestUserInputParams)> {
    let mut pending = None;
    for item in items {
        let RolloutItem::EventMsg(event) = item else {
            continue;
        };
        match event {
            EventMsg::RequestUserInput(request) => {
                pending = Some((
                    request.turn_id.clone(),
                    params_from_event(conversation_id, request.clone()),
                ));
            }
            EventMsg::TurnComplete(_) => {
                pending = None;
            }
            EventMsg::TurnAborted(aborted) if aborted.reason != TurnAbortReason::Interrupted => {
                pending = None;
            }
            _ => {}
        }
    }
    pending
}

fn convert_question_kind(
    value: CoreRequestUserInputQuestionKind,
) -> ToolRequestUserInputQuestionKind {
    match value {
        CoreRequestUserInputQuestionKind::SingleChoice => {
            ToolRequestUserInputQuestionKind::SingleChoice
        }
        CoreRequestUserInputQuestionKind::MultiChoice => {
            ToolRequestUserInputQuestionKind::MultiChoice
        }
        CoreRequestUserInputQuestionKind::FreeText => ToolRequestUserInputQuestionKind::FreeText,
        CoreRequestUserInputQuestionKind::Ranking => ToolRequestUserInputQuestionKind::Ranking,
    }
}
