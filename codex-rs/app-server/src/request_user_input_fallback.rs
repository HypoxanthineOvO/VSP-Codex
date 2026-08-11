use codex_app_server_protocol::ToolRequestUserInputAnswer;
use codex_app_server_protocol::ToolRequestUserInputParams;
use codex_app_server_protocol::ToolRequestUserInputQuestion;
use codex_app_server_protocol::ToolRequestUserInputQuestionKind;
use codex_app_server_protocol::ToolRequestUserInputResponse;
use codex_app_server_protocol::UserInput as V2UserInput;
use codex_app_server_protocol::is_authorization_intent_question;
use codex_app_server_protocol::workflow_decision_type_for_question;
use std::collections::HashMap;

use crate::workflow_markdown_surface::render_request_input_workflow_markdown_surface;

const LETTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub(crate) fn fallback_prompt(params: &ToolRequestUserInputParams) -> String {
    let mut lines = Vec::new();
    if is_workflow_decision_prompt(params) {
        lines.push("## Workflow Decision".to_string());
        lines.push(String::new());
        lines.push(
            "Reply with the numbered answer below to continue. This client does not support the native Ask UI."
                .to_string(),
        );
    } else {
        lines.push(
            "Reply with a numbered answer to continue. This client does not support the native Ask UI."
                .to_string(),
        );
    }
    if let Some(summary) = params
        .summary
        .as_deref()
        .filter(|summary| !summary.is_empty())
    {
        lines.push(String::new());
        lines.push(summary.to_string());
    }
    if let Some(description) = params
        .description
        .as_deref()
        .filter(|description| !description.is_empty())
    {
        lines.push(description.to_string());
    }
    lines.push(String::new());
    lines.push("**Answer controls**".to_string());
    lines.push(String::new());
    for (question_idx, question) in params.questions.iter().enumerate() {
        let number = question_idx + 1;
        let kind = question_kind(question);
        if let Some(workflow_decision_type) = workflow_decision_type_for_question(question) {
            lines.push(format!(
                "Workflow decision: {}",
                workflow_decision_label(workflow_decision_type)
            ));
            if is_authorization_intent_question(question) {
                lines.push(
                    "Authorization intent only; this does not grant command, file, or permission approvals."
                        .to_string(),
                );
                lines.push("Existing approval mechanisms still apply.".to_string());
            }
        }
        match kind {
            ToolRequestUserInputQuestionKind::SingleChoice => {
                lines.push(format!("{number}. {}", question.question));
                append_options(
                    &mut lines,
                    question,
                    is_workflow_decision_question(question),
                );
            }
            ToolRequestUserInputQuestionKind::MultiChoice => {
                lines.push(format!("{number}. {}", question.question));
                append_options(
                    &mut lines,
                    question,
                    is_workflow_decision_question(question),
                );
                lines.push("   Select one or more options, separated by commas.".to_string());
            }
            ToolRequestUserInputQuestionKind::FreeText => {
                lines.push(format!("{number}. {}", question.question));
            }
            ToolRequestUserInputQuestionKind::Ranking => {
                lines.push(format!("{number}. {}", question.question));
                append_options(
                    &mut lines,
                    question,
                    is_workflow_decision_question(question),
                );
                lines.push(
                    "   Rank the options in priority order, for example: B, A, C".to_string(),
                );
            }
        }
        if let Some(description) = question
            .description
            .as_deref()
            .filter(|description| !description.is_empty())
        {
            lines.push(format!("   {description}"));
        }
        lines.push(String::new());
    }
    lines.push("Reply in this format:".to_string());
    for question_idx in 0..params.questions.len() {
        lines.push(format!("{}. ...", question_idx + 1));
    }
    lines.join("\n")
}

pub(crate) fn workflow_surface_fallback_prompt(params: &ToolRequestUserInputParams) -> String {
    render_request_input_workflow_markdown_surface(fallback_prompt(params))
}

pub(crate) fn parse_fallback_response(
    params: &ToolRequestUserInputParams,
    input: &[V2UserInput],
) -> Option<ToolRequestUserInputResponse> {
    let text = user_input_text(input);
    if text.trim().is_empty() {
        return None;
    }

    let numbered = parse_numbered_answers(&text);
    let mut answers = HashMap::new();
    for (idx, question) in params.questions.iter().enumerate() {
        let raw = numbered
            .get(&(idx + 1))
            .cloned()
            .unwrap_or_else(|| text.trim().to_string());
        let answer = answer_from_text(question, raw.trim());
        answers.insert(question.id.clone(), answer);
    }
    Some(ToolRequestUserInputResponse { answers })
}

pub(crate) fn recovered_answer_prompt(
    params: &ToolRequestUserInputParams,
    response: &ToolRequestUserInputResponse,
) -> String {
    let mut lines = Vec::new();
    lines.push(
        "我恢复了一个挂起的 Ask 请求，但原进程的等待状态已经不可用。请把下面这些用户答案作为继续当前任务的上下文。"
            .to_string(),
    );
    if let Some(summary) = params
        .summary
        .as_deref()
        .filter(|summary| !summary.is_empty())
    {
        lines.push(String::new());
        lines.push(format!("Ask 摘要：{summary}"));
    }
    for (idx, question) in params.questions.iter().enumerate() {
        let answer = response.answers.get(&question.id);
        lines.push(String::new());
        lines.push(format!("{}. {}", idx + 1, question.question));
        if question.is_secret {
            lines.push("答案：••••••".to_string());
        } else {
            lines.push(format!(
                "答案：{}",
                answer
                    .map(answer_display_text)
                    .filter(|text| !text.is_empty())
                    .unwrap_or_else(|| "(empty)".to_string())
            ));
        }
    }
    lines.join("\n")
}

fn user_input_text(input: &[V2UserInput]) -> String {
    input
        .iter()
        .filter_map(|item| match item {
            V2UserInput::Text { text, .. } => Some(text.as_str()),
            V2UserInput::Image { .. }
            | V2UserInput::LocalImage { .. }
            | V2UserInput::Skill { .. }
            | V2UserInput::Mention { .. } => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn answer_display_text(answer: &ToolRequestUserInputAnswer) -> String {
    if let Some(free_text) = answer.free_text.as_deref()
        && !free_text.is_empty()
    {
        return free_text.to_string();
    }
    if let Some(ordered) = answer.ordered_option_ids.as_ref()
        && !ordered.is_empty()
    {
        return ordered.join(", ");
    }
    if let Some(selected) = answer.selected_option_ids.as_ref()
        && !selected.is_empty()
    {
        return selected.join(", ");
    }
    if !answer.answers.is_empty() {
        return answer.answers.join(", ");
    }
    answer
        .legacy_answers
        .as_ref()
        .map(|answers| answers.join(", "))
        .unwrap_or_default()
}

fn append_options(
    lines: &mut Vec<String>,
    question: &ToolRequestUserInputQuestion,
    use_numbers: bool,
) {
    let Some(options) = question.options.as_ref() else {
        return;
    };
    for (idx, option) in options.iter().enumerate() {
        let marker = if use_numbers {
            (idx + 1).to_string()
        } else {
            option_letter(idx).to_string()
        };
        if option.description.is_empty() {
            lines.push(format!("   {marker}. {}", option.label));
        } else {
            lines.push(format!(
                "   {marker}. {} - {}",
                option.label, option.description
            ));
        }
    }
}

fn is_workflow_decision_question(question: &ToolRequestUserInputQuestion) -> bool {
    workflow_decision_type_for_question(question).is_some()
}

fn is_workflow_decision_prompt(params: &ToolRequestUserInputParams) -> bool {
    params
        .questions
        .iter()
        .any(|question| workflow_decision_type_for_question(question).is_some())
}

fn workflow_decision_label(decision_type: &str) -> &'static str {
    match decision_type {
        "plan_confirmation" => "plan confirmation",
        "phase_transition" => "phase transition",
        "scope_selection" => "scope selection",
        "revision_rejection" => "revision/rejection",
        "authorization_intent" => "authorization-intent",
        "release_update_confirmation" => "release/update confirmation",
        _ => "workflow decision",
    }
}

fn parse_numbered_answers(text: &str) -> HashMap<usize, String> {
    let mut answers = HashMap::new();
    let mut current_number = None;
    let mut current_text = String::new();
    for line in text.lines() {
        if let Some((number, rest)) = parse_numbered_line(line) {
            if let Some(existing_number) = current_number.replace(number) {
                answers.insert(existing_number, current_text.trim().to_string());
            }
            current_text = rest.trim().to_string();
        } else if current_number.is_some() {
            if !current_text.is_empty() {
                current_text.push('\n');
            }
            current_text.push_str(line.trim());
        }
    }
    if let Some(number) = current_number {
        answers.insert(number, current_text.trim().to_string());
    }
    answers
}

fn parse_numbered_line(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    let digit_len = trimmed
        .chars()
        .take_while(char::is_ascii_digit)
        .map(char::len_utf8)
        .sum::<usize>();
    if digit_len == 0 {
        return None;
    }
    let rest = &trimmed[digit_len..];
    let rest = rest
        .strip_prefix('.')
        .or_else(|| rest.strip_prefix(')'))
        .or_else(|| rest.strip_prefix('：'))
        .or_else(|| rest.strip_prefix(':'))?;
    let number = trimmed[..digit_len].parse::<usize>().ok()?;
    Some((number, rest))
}

fn answer_from_text(
    question: &ToolRequestUserInputQuestion,
    text: &str,
) -> ToolRequestUserInputAnswer {
    let kind = question_kind(question);
    let labels = labels_from_text(question, text);
    let ids = labels
        .iter()
        .filter_map(|label| option_id_for_label(question, label))
        .collect::<Vec<_>>();
    let legacy_answers = if labels.is_empty() {
        (!text.is_empty()).then(|| vec![text.to_string()])
    } else {
        Some(labels.clone())
    };
    match kind {
        ToolRequestUserInputQuestionKind::SingleChoice => ToolRequestUserInputAnswer {
            selected_option_ids: (!ids.is_empty()).then(|| ids.into_iter().take(1).collect()),
            ordered_option_ids: None,
            free_text: None,
            legacy_answers,
            answers: labels,
        },
        ToolRequestUserInputQuestionKind::MultiChoice => ToolRequestUserInputAnswer {
            selected_option_ids: (!ids.is_empty()).then_some(ids),
            ordered_option_ids: None,
            free_text: None,
            legacy_answers,
            answers: labels,
        },
        ToolRequestUserInputQuestionKind::Ranking => ToolRequestUserInputAnswer {
            selected_option_ids: None,
            ordered_option_ids: (!ids.is_empty()).then_some(ids),
            free_text: None,
            legacy_answers,
            answers: labels,
        },
        ToolRequestUserInputQuestionKind::FreeText => ToolRequestUserInputAnswer {
            selected_option_ids: None,
            ordered_option_ids: None,
            free_text: (!text.is_empty()).then(|| text.to_string()),
            legacy_answers,
            answers: if text.is_empty() {
                Vec::new()
            } else {
                vec![text.to_string()]
            },
        },
    }
}

fn labels_from_text(question: &ToolRequestUserInputQuestion, text: &str) -> Vec<String> {
    text.split([',', '，', '\n'])
        .filter_map(|part| label_from_token(question, part.trim()))
        .collect()
}

fn label_from_token(question: &ToolRequestUserInputQuestion, token: &str) -> Option<String> {
    if token.is_empty() {
        return None;
    }
    let options = question.options.as_ref()?;
    if token.len() == 1 {
        let ch = token.as_bytes()[0].to_ascii_uppercase();
        if let Some(idx) = LETTERS.iter().position(|letter| *letter == ch) {
            return options.get(idx).map(|option| option.label.clone());
        }
    }
    options
        .iter()
        .find(|option| option.label.eq_ignore_ascii_case(token))
        .map(|option| option.label.clone())
}

fn option_id_for_label(question: &ToolRequestUserInputQuestion, label: &str) -> Option<String> {
    question.options.as_ref()?.iter().find_map(|option| {
        option
            .label
            .eq_ignore_ascii_case(label)
            .then(|| option.id.clone().unwrap_or_else(|| option.label.clone()))
    })
}

fn option_letter(idx: usize) -> char {
    LETTERS.get(idx).copied().map(char::from).unwrap_or('?')
}

fn question_kind(question: &ToolRequestUserInputQuestion) -> ToolRequestUserInputQuestionKind {
    question.kind.clone().unwrap_or_else(|| {
        if question
            .options
            .as_ref()
            .is_some_and(|options| !options.is_empty())
        {
            ToolRequestUserInputQuestionKind::SingleChoice
        } else {
            ToolRequestUserInputQuestionKind::FreeText
        }
    })
}
