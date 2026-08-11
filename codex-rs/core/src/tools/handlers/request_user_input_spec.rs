use codex_protocol::config_types::ModeKind;
use codex_protocol::request_user_input::RequestUserInputArgs;
use codex_protocol::request_user_input::RequestUserInputQuestionKind;
use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use std::collections::BTreeMap;

pub const REQUEST_USER_INPUT_TOOL_NAME: &str = "request_user_input";
pub const MIN_AUTO_RESOLUTION_MS: u64 = 60_000;
pub const MAX_AUTO_RESOLUTION_MS: u64 = 240_000;

pub fn create_request_user_input_tool(description: String) -> ToolSpec {
    let option_props = BTreeMap::from([
        (
            "id".to_string(),
            JsonSchema::string(Some(
                "Optional stable option id. Use snake_case when provided.".to_string(),
            )),
        ),
        (
            "label".to_string(),
            JsonSchema::string(Some("User-facing option label.".to_string())),
        ),
        (
            "description".to_string(),
            JsonSchema::string(Some(
                "One short sentence explaining impact/tradeoff if selected.".to_string(),
            )),
        ),
    ]);

    let options_schema = JsonSchema::array(JsonSchema::object(
            option_props,
            Some(vec!["label".to_string(), "description".to_string()]),
            Some(false.into()),
        ), Some(
            "Options for singleChoice, multiChoice, or ranking questions. Omit for freeText questions."
                .to_string(),
        ));

    let question_props = BTreeMap::from([
        (
            "id".to_string(),
            JsonSchema::string(Some(
                "Stable identifier for mapping answers (snake_case).".to_string(),
            )),
        ),
        (
            "kind".to_string(),
            JsonSchema::string_enum(
                vec![
                    serde_json::json!("singleChoice"),
                    serde_json::json!("multiChoice"),
                    serde_json::json!("freeText"),
                    serde_json::json!("ranking"),
                ],
                Some(
                    "Answer kind: singleChoice, multiChoice, freeText, or ranking. Defaults to singleChoice when options are present."
                        .to_string(),
                ),
            ),
        ),
        (
            "header".to_string(),
            JsonSchema::string(Some(
                "Short header label shown in the UI.".to_string(),
            )),
        ),
        (
            "question".to_string(),
            JsonSchema::string(Some(
                "Single-sentence prompt shown to the user.".to_string(),
            )),
        ),
        (
            "description".to_string(),
            JsonSchema::string(Some(
                "Optional extra context shown near this question.".to_string(),
            )),
        ),
        ("options".to_string(), options_schema),
        (
            "isSecret".to_string(),
            JsonSchema::boolean(Some(
                "Set true only for sensitive freeText answers that native clients must mask."
                    .to_string(),
            )),
        ),
        (
            "workflowDecisionType".to_string(),
            JsonSchema::string(Some(
                "Optional explicit Workflow card metadata for clients that support structured Workflow decisions."
                    .to_string(),
            )),
        ),
        (
            "intentOnly".to_string(),
            JsonSchema::boolean(Some(
                "Optional explicit Workflow metadata. Set true for authorization-intent cards that record intent without granting approval."
                    .to_string(),
            )),
        ),
        (
            "grantsApproval".to_string(),
            JsonSchema::boolean(Some(
                "Optional explicit Workflow metadata. Set false when an authorization-intent card must not grant command, file, or permission approvals."
                    .to_string(),
            )),
        ),
    ]);

    let questions_schema = JsonSchema::array(
        JsonSchema::object(
            question_props,
            Some(vec![
                "id".to_string(),
                "header".to_string(),
                "question".to_string(),
            ]),
            Some(false.into()),
        ),
        Some("Questions to show the user. Ask only when the answer changes what you do next; there is no hard count limit.".to_string()),
    );

    let auto_resolution_ms_schema = JsonSchema::number(Some(format!(
        "Optional auto-resolution window in milliseconds, from {MIN_AUTO_RESOLUTION_MS} to {MAX_AUTO_RESOLUTION_MS}. Include this only when the question is useful but non-blocking and continuing with best judgment is acceptable if the user does not answer; omit it when explicit user input is required before continuing. Use {MIN_AUTO_RESOLUTION_MS} for lightly helpful context and up to {MAX_AUTO_RESOLUTION_MS} when the answer would materially unblock better work."
    )));

    let properties = BTreeMap::from([
        (
            "summary".to_string(),
            JsonSchema::string(Some("Short title for the Ask card.".to_string())),
        ),
        (
            "description".to_string(),
            JsonSchema::string(Some(
                "Brief context explaining why these answers are needed.".to_string(),
            )),
        ),
        ("questions".to_string(), questions_schema),
        ("autoResolutionMs".to_string(), auto_resolution_ms_schema),
    ]);

    ToolSpec::Function(ResponsesApiTool {
        name: REQUEST_USER_INPUT_TOOL_NAME.to_string(),
        description,
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["questions".to_string()]),
            Some(false.into()),
        ),
        output_schema: None,
    })
}

pub fn request_user_input_unavailable_message(
    mode: ModeKind,
    available_modes: &[ModeKind],
) -> Option<String> {
    if available_modes.contains(&mode) {
        None
    } else {
        let mode_name = mode.display_name();
        Some(format!(
            "request_user_input is unavailable in {mode_name} mode"
        ))
    }
}

pub fn normalize_request_user_input_args(
    mut args: RequestUserInputArgs,
) -> Result<RequestUserInputArgs, String> {
    for question in &mut args.questions {
        let legacy_question_without_kind = question.kind.is_none();
        let kind = question.kind.clone().unwrap_or_else(|| {
            if question.options.as_ref().is_some_and(Vec::is_empty) || question.options.is_none() {
                RequestUserInputQuestionKind::FreeText
            } else {
                RequestUserInputQuestionKind::SingleChoice
            }
        });
        let has_options = question
            .options
            .as_ref()
            .is_some_and(|options| !options.is_empty());

        match kind {
            RequestUserInputQuestionKind::SingleChoice
            | RequestUserInputQuestionKind::MultiChoice
            | RequestUserInputQuestionKind::Ranking => {
                if !has_options {
                    let kind_name = match kind {
                        RequestUserInputQuestionKind::SingleChoice => "singleChoice",
                        RequestUserInputQuestionKind::MultiChoice => "multiChoice",
                        RequestUserInputQuestionKind::FreeText => "freeText",
                        RequestUserInputQuestionKind::Ranking => "ranking",
                    };
                    return Err(format!(
                        "request_user_input question `{}` with kind `{kind_name}` requires non-empty options",
                        question.id
                    ));
                }
                if legacy_question_without_kind {
                    question.is_other = true;
                }
            }
            RequestUserInputQuestionKind::FreeText => {
                if question.options.as_ref().is_some_and(Vec::is_empty) {
                    question.options = None;
                }
            }
        }

        question.kind = Some(kind);
    }

    if let Some(auto_resolution_ms) = args.auto_resolution_ms {
        let clamped_auto_resolution_ms =
            auto_resolution_ms.clamp(MIN_AUTO_RESOLUTION_MS, MAX_AUTO_RESOLUTION_MS);
        if clamped_auto_resolution_ms != auto_resolution_ms {
            tracing::warn!(
                auto_resolution_ms,
                clamped_auto_resolution_ms,
                "clamped request_user_input autoResolutionMs to supported range"
            );
            args.auto_resolution_ms = Some(clamped_auto_resolution_ms);
        }
    }

    Ok(args)
}

pub fn request_user_input_tool_description(available_modes: &[ModeKind]) -> String {
    let availability = if request_user_input_available_in_all_modes(available_modes) {
        "This tool is available in every collaboration mode and only from the root thread."
            .to_string()
    } else {
        let allowed_modes = format_allowed_modes(available_modes);
        format!("This tool is only available in {allowed_modes} and only from the root thread.")
    };
    format!(
        "Ask the user for information you need before proceeding, using a question, brief context, and answer controls. Supports singleChoice, multiChoice, freeText, and ranking questions. Before calling this tool, first send a normal assistant message with the complete decision context already visible to the user: the conclusion or recommendation, relevant evidence or artifact summary, what each available answer changes downstream, and any material risks or follow-up. Do not rely on the Ask card summary, description, option labels, or a file/report path as the explanation; those fields are only a compact control surface. A message such as \"I wrote the report; choose an option\" is not enough. When the question confirms or accepts a report, debug artifact, audit artifact, Patch, Cycle, Milestone result, generated plan, or generated prompt package, include the core report content in chat, or the equivalent core plan content, before opening the gate: conclusion/change summary, technical approach, modified files/modules, test design, validation results, expected result, problems encountered, and risks/follow-up when applicable. The tool call must be the gate after that explanation, not the explanation itself. Use it for meaningful clarification, confirmation gates, acceptance, or workflow choices; do not ask questions whose answers would not change your next action. Set autoResolutionMs, from {MIN_AUTO_RESOLUTION_MS} to {MAX_AUTO_RESOLUTION_MS} milliseconds, only when the question is useful but non-blocking and continuing with best judgment is acceptable if the user does not answer; omit it when explicit user input is required. {availability}"
    )
}

fn request_user_input_available_in_all_modes(available_modes: &[ModeKind]) -> bool {
    [
        ModeKind::Default,
        ModeKind::Plan,
        ModeKind::PairProgramming,
        ModeKind::Execute,
    ]
    .into_iter()
    .all(|mode| available_modes.contains(&mode))
}

fn format_allowed_modes(available_modes: &[ModeKind]) -> String {
    let mode_names: Vec<&str> = available_modes
        .iter()
        .map(|mode| mode.display_name())
        .collect();

    match mode_names.as_slice() {
        [] => "no modes".to_string(),
        [mode] => format!("{mode} mode"),
        [first, second] => format!("{first} or {second} mode"),
        [..] => format!("modes: {}", mode_names.join(",")),
    }
}

#[cfg(test)]
#[path = "request_user_input_spec_tests.rs"]
mod tests;
