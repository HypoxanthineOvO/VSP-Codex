use super::*;
use codex_features::Features;
use codex_protocol::config_types::ModeKind;
use codex_protocol::request_user_input::RequestUserInputArgs;
use codex_protocol::request_user_input::RequestUserInputQuestion;
use codex_protocol::request_user_input::RequestUserInputQuestionKind;
use codex_protocol::request_user_input::RequestUserInputQuestionOption;
use codex_tools::request_user_input_available_modes;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

fn default_available_modes() -> Vec<ModeKind> {
    request_user_input_available_modes(&Features::with_defaults())
}

fn normalize_args(value: Value) -> Result<RequestUserInputArgs, String> {
    let args: RequestUserInputArgs =
        serde_json::from_value(value).expect("request_user_input args should deserialize");
    normalize_request_user_input_args(args)
}

#[test]
fn request_user_input_tool_includes_extended_ask_schema() {
    let ToolSpec::Function(tool) =
        create_request_user_input_tool("Ask the user to choose.".to_string())
    else {
        panic!("request_user_input should be a function tool");
    };
    assert_eq!(tool.name, "request_user_input");
    assert_eq!(tool.description, "Ask the user to choose.");

    let schema = serde_json::to_value(&tool.parameters).expect("schema serializes");
    assert_eq!(schema["required"], json!(["questions"]));

    let properties = schema["properties"]
        .as_object()
        .expect("top-level properties");
    for field in ["summary", "description", "questions", "autoResolutionMs"] {
        assert!(properties.contains_key(field), "missing field {field}");
    }

    let questions = &properties["questions"];
    let questions_description = questions["description"]
        .as_str()
        .expect("questions description");
    assert!(
        !questions_description.contains("one to three"),
        "schema should not advertise a hard per-turn question limit: {questions_description}"
    );

    let question_properties = questions["items"]["properties"]
        .as_object()
        .expect("question properties");
    assert_eq!(
        question_properties["kind"]["enum"],
        json!(["singleChoice", "multiChoice", "freeText", "ranking"])
    );
    for field in [
        "id",
        "kind",
        "header",
        "question",
        "description",
        "options",
        "isSecret",
        "workflowDecisionType",
        "intentOnly",
        "grantsApproval",
    ] {
        assert!(
            question_properties.contains_key(field),
            "missing question field {field}"
        );
    }
}

#[test]
fn request_user_input_normalization_allows_free_text_without_options() {
    let normalized = normalize_args(json!({
        "summary": "Collect context",
        "description": "The response will guide the plan.",
        "questions": [{
            "id": "notes",
            "kind": "freeText",
            "header": "Notes",
            "question": "What else should I know?",
            "description": "Include any constraints that matter."
        }]
    }))
    .expect("freeText questions should not require options");

    assert_eq!(normalized.summary, Some("Collect context".to_string()));
    assert_eq!(
        normalized.description,
        Some("The response will guide the plan.".to_string())
    );
    assert_eq!(normalized.questions[0].options, None);
}

#[test]
fn request_user_input_normalization_requires_options_for_choice_and_ranking_kinds() {
    for (kind, question_id) in [
        ("singleChoice", "deploy_target"),
        ("multiChoice", "approvers"),
        ("ranking", "rollback_order"),
    ] {
        let err = normalize_args(json!({
            "questions": [{
                "id": question_id,
                "kind": kind,
                "header": "Pick",
                "question": "Choose an option."
            }]
        }))
        .expect_err("choice and ranking questions should require options");
        assert_eq!(
            err,
            format!(
                "request_user_input question `{question_id}` with kind `{kind}` requires non-empty options"
            )
        );
    }
}

#[test]
fn request_user_input_normalization_preserves_legacy_questions_with_options() {
    let normalized = normalize_args(json!({
        "questions": [{
            "id": "legacy_choice",
            "header": "Legacy",
            "question": "Pick one.",
            "options": [{
                "id": "yes",
                "label": "Yes",
                "description": "Proceed."
            }]
        }]
    }))
    .expect("legacy questions without kind should remain compatible");

    assert_eq!(
        normalized.questions[0].kind,
        Some(RequestUserInputQuestionKind::SingleChoice)
    );
    assert_eq!(
        normalized.questions[0].options.as_ref().map(Vec::len),
        Some(1)
    );
}

#[test]
fn request_user_input_normalization_does_not_enforce_question_count_limits() {
    let questions: Vec<Value> = (0..5)
        .map(|index| {
            json!({
                "id": format!("question_{index}"),
                "kind": "singleChoice",
                "header": format!("Question {index} with a deliberately long header"),
                "question": "Pick one.",
                "options": [
                    {"id": "first", "label": "First", "description": "First option."},
                    {"id": "second", "label": "Second", "description": "Second option."}
                ]
            })
        })
        .collect();

    let normalized = normalize_args(json!({ "questions": questions }))
        .expect("normalization should not enforce UI display limits");
    assert_eq!(normalized.questions.len(), 5);
}

#[test]
fn normalize_request_user_input_args_clamps_out_of_range_auto_resolution_ms() {
    let args = RequestUserInputArgs {
        summary: None,
        description: None,
        questions: vec![RequestUserInputQuestion {
            id: "confirm".to_string(),
            kind: None,
            header: "Confirm".to_string(),
            question: "Proceed?".to_string(),
            description: None,
            is_other: false,
            is_secret: false,
            options: Some(vec![RequestUserInputQuestionOption {
                id: None,
                label: "Yes (Recommended)".to_string(),
                description: "Continue.".to_string(),
            }]),
            workflow_decision_type: None,
            intent_only: None,
            grants_approval: None,
        }],
        auto_resolution_ms: Some(MIN_AUTO_RESOLUTION_MS - 1),
    };

    assert_eq!(
        normalize_request_user_input_args(args.clone()),
        Ok(RequestUserInputArgs {
            questions: vec![RequestUserInputQuestion {
                is_other: true,
                kind: Some(RequestUserInputQuestionKind::SingleChoice),
                ..args.questions[0].clone()
            }],
            auto_resolution_ms: Some(MIN_AUTO_RESOLUTION_MS),
            ..args.clone()
        })
    );
    assert_eq!(
        normalize_request_user_input_args(RequestUserInputArgs {
            auto_resolution_ms: Some(MAX_AUTO_RESOLUTION_MS + 1),
            ..args.clone()
        }),
        Ok(RequestUserInputArgs {
            questions: vec![RequestUserInputQuestion {
                is_other: true,
                kind: Some(RequestUserInputQuestionKind::SingleChoice),
                ..args.questions[0].clone()
            }],
            auto_resolution_ms: Some(MAX_AUTO_RESOLUTION_MS),
            ..args
        })
    );
}

#[test]
fn request_user_input_unavailable_messages_allow_all_collaboration_modes() {
    for mode in [
        ModeKind::Default,
        ModeKind::Plan,
        ModeKind::Execute,
        ModeKind::PairProgramming,
    ] {
        assert_eq!(
            request_user_input_unavailable_message(mode, &default_available_modes()),
            None
        );
    }
}

#[test]
fn request_user_input_tool_description_mentions_extended_contract() {
    let description = request_user_input_tool_description(&default_available_modes());
    assert!(description.contains("singleChoice, multiChoice, freeText, and ranking"));
    assert!(description.contains("autoResolutionMs"));
    assert!(description.contains("explicit user input is required"));
    assert!(description.contains("available in every collaboration mode"));
}

#[test]
fn request_user_input_tool_description_requires_visible_context_before_tool_call() {
    let description = request_user_input_tool_description(&default_available_modes());
    let lower = description.to_ascii_lowercase();

    for expected in [
        "before calling this tool",
        "normal assistant message",
        "complete decision context",
        "already visible to the user",
        "conclusion or recommendation",
        "what each available answer changes downstream",
        "do not rely on the ask card summary",
        "file/report path",
        "i wrote the report",
        "generated plan",
        "generated prompt package",
        "core report content in chat",
        "technical approach",
        "validation results",
        "risks/follow-up",
        "the tool call must be the gate after that explanation",
    ] {
        assert!(
            lower.contains(expected),
            "request_user_input description must require visible pre-card context; missing {expected:?} in:\n{description}"
        );
    }
}
