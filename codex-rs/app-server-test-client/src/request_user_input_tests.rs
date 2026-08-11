use std::collections::HashMap;
use std::io::Cursor;

use codex_app_server_protocol::ToolRequestUserInputAnswer;
use codex_app_server_protocol::ToolRequestUserInputOption;
use codex_app_server_protocol::ToolRequestUserInputParams;
use codex_app_server_protocol::ToolRequestUserInputQuestion;
use codex_app_server_protocol::ToolRequestUserInputQuestionKind;
use codex_app_server_protocol::ToolRequestUserInputResponse;
use pretty_assertions::assert_eq;

use super::prompt_for_answers_with;

#[test]
fn collects_option_and_free_form_answers() {
    let params = ToolRequestUserInputParams {
        thread_id: "thread-1".to_string(),
        turn_id: "turn-1".to_string(),
        item_id: "item-1".to_string(),
        summary: None,
        description: None,
        questions: vec![
            ToolRequestUserInputQuestion {
                id: "target".to_string(),
                kind: Some(ToolRequestUserInputQuestionKind::SingleChoice),
                header: "Target".to_string(),
                question: "Which target?".to_string(),
                description: None,
                is_other: true,
                is_secret: false,
                options: Some(vec![
                    ToolRequestUserInputOption {
                        id: Some("core".to_string()),
                        label: "Core".to_string(),
                        description: "Inspect core".to_string(),
                    },
                    ToolRequestUserInputOption {
                        id: Some("tui".to_string()),
                        label: "TUI".to_string(),
                        description: "Inspect TUI".to_string(),
                    },
                ]),
                workflow_decision_type: None,
                intent_only: None,
                grants_approval: None,
            },
            ToolRequestUserInputQuestion {
                id: "details".to_string(),
                kind: Some(ToolRequestUserInputQuestionKind::FreeText),
                header: "Details".to_string(),
                question: "Anything else?".to_string(),
                description: None,
                is_other: true,
                is_secret: false,
                options: None,
                workflow_decision_type: None,
                intent_only: None,
                grants_approval: None,
            },
        ],
        auto_resolution_ms: Some(60_000),
    };
    let mut input = Cursor::new(b"2\ninclude snapshots\n");
    let mut output = Vec::new();

    let response = prompt_for_answers_with(&mut input, &mut output, &params).unwrap();

    assert_eq!(
        response,
        ToolRequestUserInputResponse {
            answers: HashMap::from([
                (
                    "target".to_string(),
                    ToolRequestUserInputAnswer {
                        selected_option_ids: Some(vec!["tui".to_string()]),
                        ordered_option_ids: None,
                        free_text: None,
                        legacy_answers: Some(vec!["TUI".to_string()]),
                        answers: vec!["TUI".to_string()],
                    },
                ),
                (
                    "details".to_string(),
                    ToolRequestUserInputAnswer {
                        selected_option_ids: None,
                        ordered_option_ids: None,
                        free_text: Some("include snapshots".to_string()),
                        legacy_answers: Some(vec!["user_note: include snapshots".to_string()]),
                        answers: vec!["user_note: include snapshots".to_string()],
                    },
                ),
            ]),
        }
    );
    assert_eq!(
        String::from_utf8(output).unwrap(),
        concat!(
            "\n[request_user_input for thread thread-1, turn turn-1]\n",
            "The app-server may auto-resolve this request after 60000 ms.\n",
            "\nTarget: Which target?\n",
            "  1. Core - Inspect core\n",
            "  2. TUI - Inspect TUI\n",
            "  o. Other (free-form)\n",
            "Choose 1-2 or o: ",
            "\nDetails: Anything else?\n",
            "Answer: ",
        )
    );
}

#[test]
fn retries_invalid_selection_and_collects_other_answer() {
    let params = ToolRequestUserInputParams {
        thread_id: "thread-1".to_string(),
        turn_id: "turn-1".to_string(),
        item_id: "item-1".to_string(),
        summary: None,
        description: None,
        questions: vec![ToolRequestUserInputQuestion {
            id: "target".to_string(),
            kind: Some(ToolRequestUserInputQuestionKind::SingleChoice),
            header: "Target".to_string(),
            question: "Which target?".to_string(),
            description: None,
            is_other: true,
            is_secret: false,
            options: Some(vec![ToolRequestUserInputOption {
                id: None,
                label: "Core".to_string(),
                description: "Inspect core".to_string(),
            }]),
            workflow_decision_type: None,
            intent_only: None,
            grants_approval: None,
        }],
        auto_resolution_ms: None,
    };
    let mut input = Cursor::new(b"9\no\nSDK wrapper\n");
    let mut output = Vec::new();

    let response = prompt_for_answers_with(&mut input, &mut output, &params).unwrap();

    assert_eq!(
        response,
        ToolRequestUserInputResponse {
            answers: HashMap::from([(
                "target".to_string(),
                ToolRequestUserInputAnswer {
                    selected_option_ids: None,
                    ordered_option_ids: None,
                    free_text: Some("SDK wrapper".to_string()),
                    legacy_answers: Some(vec!["user_note: SDK wrapper".to_string()]),
                    answers: vec!["user_note: SDK wrapper".to_string()],
                },
            )]),
        }
    );
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("Invalid selection; try again."));
}

#[test]
fn collects_multi_choice_selected_option_ids() {
    let params = ToolRequestUserInputParams {
        thread_id: "thread-1".to_string(),
        turn_id: "turn-1".to_string(),
        item_id: "item-1".to_string(),
        summary: None,
        description: None,
        questions: vec![ToolRequestUserInputQuestion {
            id: "areas".to_string(),
            kind: Some(ToolRequestUserInputQuestionKind::MultiChoice),
            header: "Areas".to_string(),
            question: "Which areas need review?".to_string(),
            description: None,
            is_other: false,
            is_secret: false,
            options: Some(vec![
                ToolRequestUserInputOption {
                    id: Some("core".to_string()),
                    label: "Core".to_string(),
                    description: "Inspect core".to_string(),
                },
                ToolRequestUserInputOption {
                    id: Some("tui".to_string()),
                    label: "TUI".to_string(),
                    description: "Inspect TUI".to_string(),
                },
                ToolRequestUserInputOption {
                    id: Some("docs".to_string()),
                    label: "Docs".to_string(),
                    description: "Inspect docs".to_string(),
                },
            ]),
            workflow_decision_type: None,
            intent_only: None,
            grants_approval: None,
        }],
        auto_resolution_ms: None,
    };
    let mut input = Cursor::new(b"1,3\n");
    let mut output = Vec::new();

    let response = prompt_for_answers_with(&mut input, &mut output, &params).unwrap();

    assert_eq!(
        response,
        ToolRequestUserInputResponse {
            answers: HashMap::from([(
                "areas".to_string(),
                ToolRequestUserInputAnswer {
                    selected_option_ids: Some(vec!["core".to_string(), "docs".to_string()]),
                    ordered_option_ids: None,
                    free_text: None,
                    legacy_answers: Some(vec!["Core".to_string(), "Docs".to_string()]),
                    answers: vec!["Core".to_string(), "Docs".to_string()],
                },
            )]),
        }
    );
}

#[test]
fn collects_ranking_ordered_option_ids() {
    let params = ToolRequestUserInputParams {
        thread_id: "thread-1".to_string(),
        turn_id: "turn-1".to_string(),
        item_id: "item-1".to_string(),
        summary: None,
        description: None,
        questions: vec![ToolRequestUserInputQuestion {
            id: "priority".to_string(),
            kind: Some(ToolRequestUserInputQuestionKind::Ranking),
            header: "Priority".to_string(),
            question: "Rank the priority order.".to_string(),
            description: None,
            is_other: false,
            is_secret: false,
            options: Some(vec![
                ToolRequestUserInputOption {
                    id: Some("core".to_string()),
                    label: "Core".to_string(),
                    description: "Inspect core".to_string(),
                },
                ToolRequestUserInputOption {
                    id: Some("tui".to_string()),
                    label: "TUI".to_string(),
                    description: "Inspect TUI".to_string(),
                },
                ToolRequestUserInputOption {
                    id: Some("release".to_string()),
                    label: "Release".to_string(),
                    description: "Inspect release".to_string(),
                },
            ]),
            workflow_decision_type: None,
            intent_only: None,
            grants_approval: None,
        }],
        auto_resolution_ms: None,
    };
    let mut input = Cursor::new(b"3,1,2\n");
    let mut output = Vec::new();

    let response = prompt_for_answers_with(&mut input, &mut output, &params).unwrap();

    assert_eq!(
        response,
        ToolRequestUserInputResponse {
            answers: HashMap::from([(
                "priority".to_string(),
                ToolRequestUserInputAnswer {
                    selected_option_ids: None,
                    ordered_option_ids: Some(vec![
                        "release".to_string(),
                        "core".to_string(),
                        "tui".to_string(),
                    ]),
                    free_text: None,
                    legacy_answers: Some(vec![
                        "Release".to_string(),
                        "Core".to_string(),
                        "TUI".to_string(),
                    ]),
                    answers: vec!["Release".to_string(), "Core".to_string(), "TUI".to_string(),],
                },
            )]),
        }
    );
}
