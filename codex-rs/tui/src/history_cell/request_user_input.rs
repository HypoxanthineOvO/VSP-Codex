//! Completed request-user-input transcript rendering.

use super::*;

const SECRET_MASK: &str = "••••••";

/// Renders a completed (or interrupted) request_user_input exchange in history.
#[derive(Debug)]
pub(crate) struct RequestUserInputResultCell {
    pub(crate) questions: Vec<ToolRequestUserInputQuestion>,
    pub(crate) answers: HashMap<String, ToolRequestUserInputAnswer>,
    pub(crate) interrupted: bool,
}

impl HistoryCell for RequestUserInputResultCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let width = width.max(1) as usize;
        let total = self.questions.len();
        let answered = self
            .questions
            .iter()
            .filter(|question| {
                self.answers
                    .get(&question.id)
                    .is_some_and(request_user_input_answer_has_content)
            })
            .count();
        let unanswered = total.saturating_sub(answered);

        let mut header = vec![
            Span::styled("•", ask_block_accent()),
            " ".into(),
            "Questions".bold(),
        ];
        header.push(format!(" {answered}/{total} answered").dim());
        if self.interrupted {
            header.push(" (interrupted)".cyan());
        }

        let mut lines: Vec<Line<'static>> = vec![header.into()];

        for question in &self.questions {
            let answer = self.answers.get(&question.id);
            let answer_missing = match answer {
                Some(answer) => !request_user_input_answer_has_content(answer),
                None => true,
            };
            let mut question_lines = wrap_with_prefix(
                &question.question,
                width,
                "  • ".into(),
                "    ".into(),
                Style::default(),
            );
            if answer_missing && let Some(last) = question_lines.last_mut() {
                last.spans.push(" (unanswered)".dim());
            }
            lines.extend(question_lines);

            let Some(answer) =
                answer.filter(|answer| request_user_input_answer_has_content(answer))
            else {
                continue;
            };
            if question.is_secret {
                lines.extend(wrap_with_prefix(
                    SECRET_MASK,
                    width,
                    "    answer: ".dim(),
                    "            ".dim(),
                    Style::default().fg(Color::Cyan),
                ));
                continue;
            }

            let (options, note) = split_request_user_input_answer(answer);

            for option in options {
                lines.extend(wrap_with_prefix(
                    &option,
                    width,
                    "    answer: ".dim(),
                    "            ".dim(),
                    Style::default().fg(Color::Cyan),
                ));
            }
            if let Some(note) = note {
                let (label, continuation, style) = if question.options.is_some() {
                    (
                        "    note: ".dim(),
                        "          ".dim(),
                        Style::default().fg(Color::Cyan),
                    )
                } else {
                    (
                        "    answer: ".dim(),
                        "            ".dim(),
                        Style::default().fg(Color::Cyan),
                    )
                };
                lines.extend(wrap_with_prefix(&note, width, label, continuation, style));
            }
        }

        if self.interrupted && unanswered > 0 {
            let summary = format!("interrupted with {unanswered} unanswered");
            lines.extend(wrap_with_prefix(
                &summary,
                width,
                "  ↳ ".cyan().dim(),
                "    ".dim(),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
            ));
        }

        lines
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        let total = self.questions.len();
        let answered = self
            .questions
            .iter()
            .filter(|question| {
                self.answers
                    .get(&question.id)
                    .is_some_and(request_user_input_answer_has_content)
            })
            .count();
        let mut lines = vec![Line::from(format!("Questions {answered}/{total} answered"))];
        if self.interrupted {
            lines.push(Line::from("(interrupted)"));
        }
        for question in &self.questions {
            lines.push(Line::from(question.question.clone()));
            if let Some(answer) = self
                .answers
                .get(&question.id)
                .filter(|answer| request_user_input_answer_has_content(answer))
            {
                if question.is_secret {
                    lines.push(Line::from(format!("answer: {SECRET_MASK}")));
                } else {
                    let (options, note) = split_request_user_input_answer(answer);
                    lines.extend(
                        options
                            .into_iter()
                            .map(|option| Line::from(format!("answer: {option}"))),
                    );
                    if let Some(note) = note {
                        let label = if question.options.is_some() {
                            "note"
                        } else {
                            "answer"
                        };
                        lines.push(Line::from(format!("{label}: {note}")));
                    }
                }
            } else {
                lines.push(Line::from("(unanswered)"));
            }
        }
        lines
    }
}

/// Wrap a plain string with textwrap and prefix each line, while applying a style to the content.
fn wrap_with_prefix(
    text: &str,
    width: usize,
    initial_prefix: Span<'static>,
    subsequent_prefix: Span<'static>,
    style: Style,
) -> Vec<Line<'static>> {
    let line = Line::from(vec![Span::from(text.to_string()).set_style(style)]);
    let opts = RtOptions::new(width.max(1))
        .initial_indent(Line::from(vec![initial_prefix]))
        .subsequent_indent(Line::from(vec![subsequent_prefix]));
    let wrapped = adaptive_wrap_line(&line, opts);
    let mut out = Vec::new();
    push_owned_lines(&wrapped, &mut out);
    out
}

fn request_user_input_answer_has_content(answer: &ToolRequestUserInputAnswer) -> bool {
    answer
        .selected_option_ids
        .as_ref()
        .is_some_and(|ids| !ids.is_empty())
        || answer
            .ordered_option_ids
            .as_ref()
            .is_some_and(|ids| !ids.is_empty())
        || answer
            .free_text
            .as_ref()
            .is_some_and(|text| !text.is_empty())
        || answer
            .legacy_answers
            .as_ref()
            .is_some_and(|answers| !answers.is_empty())
        || !answer.answers.is_empty()
}

/// Split a request_user_input answer into option labels and an optional freeform note.
/// Notes are encoded as "user_note: <text>" entries in the answers list.
fn split_request_user_input_answer(
    answer: &ToolRequestUserInputAnswer,
) -> (Vec<String>, Option<String>) {
    let mut options = Vec::new();
    let mut note = None;

    let structured_free_text = answer.free_text.as_ref().filter(|text| !text.is_empty());
    if let Some(free_text) = structured_free_text {
        note = Some(free_text.clone());
    }

    if !answer.answers.is_empty() && structured_free_text.is_none() {
        collect_legacy_answer_entries(&answer.answers, &mut options, &mut note);
        return (options, note);
    }

    if structured_free_text.is_none()
        && let Some(legacy_answers) = answer
            .legacy_answers
            .as_ref()
            .filter(|answers| !answers.is_empty())
    {
        collect_legacy_answer_entries(legacy_answers, &mut options, &mut note);
        return (options, note);
    }

    if let Some(ordered_ids) = answer
        .ordered_option_ids
        .as_ref()
        .filter(|ids| !ids.is_empty())
    {
        options.extend(ordered_ids.clone());
    } else if let Some(selected_ids) = answer
        .selected_option_ids
        .as_ref()
        .filter(|ids| !ids.is_empty())
    {
        options.extend(selected_ids.clone());
    }

    (options, note)
}

fn collect_legacy_answer_entries(
    entries: &[String],
    options: &mut Vec<String>,
    note: &mut Option<String>,
) {
    for entry in entries {
        if let Some(note_text) = entry.strip_prefix("user_note: ") {
            *note = Some(note_text.to_string());
        } else {
            options.push(entry.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines_to_string(lines: &[Line<'static>]) -> String {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn secret_question() -> ToolRequestUserInputQuestion {
        ToolRequestUserInputQuestion {
            id: "token".to_string(),
            kind: Some(codex_app_server_protocol::ToolRequestUserInputQuestionKind::FreeText),
            header: "Token".to_string(),
            question: "Paste the temporary token.".to_string(),
            description: Some("This must stay private.".to_string()),
            is_other: false,
            is_secret: true,
            workflow_decision_type: None,
            intent_only: None,
            grants_approval: None,
            options: None,
        }
    }

    fn secret_answer() -> ToolRequestUserInputAnswer {
        ToolRequestUserInputAnswer {
            selected_option_ids: None,
            ordered_option_ids: None,
            free_text: Some("real-token-123".to_string()),
            legacy_answers: Some(vec!["real-token-123".to_string()]),
            answers: vec!["real-token-123".to_string()],
        }
    }

    #[test]
    fn secret_answer_is_dot_masked_in_display_and_raw_lines() {
        let cell = RequestUserInputResultCell {
            questions: vec![secret_question()],
            answers: HashMap::from([("token".to_string(), secret_answer())]),
            interrupted: false,
        };

        let display = lines_to_string(&cell.display_lines(/*width*/ 80));
        let raw = lines_to_string(&cell.raw_lines());

        assert!(!display.contains("real-token-123"));
        assert!(!raw.contains("real-token-123"));
        assert!(display.contains("••••••"));
        assert!(raw.contains("••••••"));
        assert!(!display.contains("******"));
        assert!(!raw.contains("******"));
    }

    #[test]
    fn structured_secret_answer_counts_as_answered() {
        let cell = RequestUserInputResultCell {
            questions: vec![secret_question()],
            answers: HashMap::from([(
                "token".to_string(),
                ToolRequestUserInputAnswer {
                    selected_option_ids: None,
                    ordered_option_ids: None,
                    free_text: Some("real-token-123".to_string()),
                    legacy_answers: None,
                    answers: Vec::new(),
                },
            )]),
            interrupted: false,
        };

        let display = lines_to_string(&cell.display_lines(/*width*/ 80));

        assert!(display.contains("1/1 answered"));
        assert!(!display.contains("(unanswered)"));
    }

    #[test]
    fn free_text_history_prefers_structured_free_text_with_legacy_compatibility() {
        let mut question = secret_question();
        question.is_secret = false;
        let cell = RequestUserInputResultCell {
            questions: vec![question],
            answers: HashMap::from([(
                "token".to_string(),
                ToolRequestUserInputAnswer {
                    selected_option_ids: None,
                    ordered_option_ids: None,
                    free_text: Some("structured text".to_string()),
                    legacy_answers: Some(vec!["legacy text".to_string()]),
                    answers: Vec::new(),
                },
            )]),
            interrupted: false,
        };

        let raw = lines_to_string(&cell.raw_lines());

        assert!(raw.contains("answer: structured text"));
        assert!(!raw.contains("legacy text"));
    }
}
