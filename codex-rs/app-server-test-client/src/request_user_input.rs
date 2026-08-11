use std::collections::HashMap;
use std::io;
use std::io::BufRead;
use std::io::IsTerminal;
use std::io::Write;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use codex_app_server_protocol::ToolRequestUserInputAnswer;
use codex_app_server_protocol::ToolRequestUserInputOption;
use codex_app_server_protocol::ToolRequestUserInputParams;
use codex_app_server_protocol::ToolRequestUserInputQuestionKind;
use codex_app_server_protocol::ToolRequestUserInputResponse;

pub(super) fn prompt_for_answers(
    params: &ToolRequestUserInputParams,
) -> Result<ToolRequestUserInputResponse> {
    let stdin = io::stdin();
    if !stdin.is_terminal() {
        bail!("request_user_input requires an interactive stdin terminal");
    }

    let stdout = io::stdout();
    prompt_for_answers_with(&mut stdin.lock(), &mut stdout.lock(), params)
}

fn prompt_for_answers_with(
    input: &mut impl BufRead,
    output: &mut impl Write,
    params: &ToolRequestUserInputParams,
) -> Result<ToolRequestUserInputResponse> {
    writeln!(
        output,
        "\n[request_user_input for thread {}, turn {}]",
        params.thread_id, params.turn_id
    )?;
    if let Some(auto_resolution_ms) = params.auto_resolution_ms {
        writeln!(
            output,
            "The app-server may auto-resolve this request after {auto_resolution_ms} ms."
        )?;
    }

    let mut answers = HashMap::new();
    for question in &params.questions {
        writeln!(output, "\n{}: {}", question.header, question.question)?;
        let options = question
            .options
            .as_deref()
            .filter(|options| !options.is_empty());
        let answer = if let Some(options) = options {
            let kind = question.kind.clone().unwrap_or_else(|| {
                if question
                    .options
                    .as_ref()
                    .is_some_and(|options| !options.is_empty())
                {
                    ToolRequestUserInputQuestionKind::SingleChoice
                } else {
                    ToolRequestUserInputQuestionKind::FreeText
                }
            });

            for (index, option) in options.iter().enumerate() {
                writeln!(
                    output,
                    "  {}. {} - {}",
                    index + 1,
                    option.label,
                    option.description
                )?;
            }
            if question.is_other {
                writeln!(output, "  o. Other (free-form)")?;
            }

            match kind {
                ToolRequestUserInputQuestionKind::MultiChoice => loop {
                    write!(
                        output,
                        "Choose one or more 1-{} separated by commas: ",
                        options.len()
                    )?;
                    output.flush()?;

                    let indices = read_option_indices(input, options)?;
                    if let Some(indices) = indices {
                        break option_indices_answer(
                            options,
                            &indices,
                            ToolRequestUserInputQuestionKind::MultiChoice,
                        );
                    }

                    writeln!(output, "Invalid selection; try again.")?;
                },
                ToolRequestUserInputQuestionKind::Ranking => loop {
                    write!(
                        output,
                        "Rank 1-{} in order, separated by commas: ",
                        options.len()
                    )?;
                    output.flush()?;

                    let indices = read_option_indices(input, options)?;
                    if let Some(indices) = indices {
                        break option_indices_answer(
                            options,
                            &indices,
                            ToolRequestUserInputQuestionKind::Ranking,
                        );
                    }

                    writeln!(output, "Invalid selection; try again.")?;
                },
                ToolRequestUserInputQuestionKind::SingleChoice
                | ToolRequestUserInputQuestionKind::FreeText => loop {
                    if question.is_other {
                        write!(output, "Choose 1-{} or o: ", options.len())?;
                    } else {
                        write!(output, "Choose 1-{}: ", options.len())?;
                    }
                    output.flush()?;

                    let mut line = String::new();
                    if input
                        .read_line(&mut line)
                        .context("failed to read request_user_input selection")?
                        == 0
                    {
                        bail!("stdin closed while waiting for request_user_input selection");
                    }
                    let selection = line.trim();

                    if let Ok(index) = selection.parse::<usize>()
                        && let Some(option) =
                            index.checked_sub(1).and_then(|index| options.get(index))
                    {
                        break option_answer(option);
                    }

                    if let Some(option) = options
                        .iter()
                        .find(|option| option.label.eq_ignore_ascii_case(selection))
                    {
                        break option_answer(option);
                    }

                    if question.is_other && selection.eq_ignore_ascii_case("o") {
                        write!(output, "Other: ")?;
                        output.flush()?;
                        line.clear();
                        if input
                            .read_line(&mut line)
                            .context("failed to read request_user_input free-form answer")?
                            == 0
                        {
                            bail!(
                                "stdin closed while waiting for request_user_input free-form answer"
                            );
                        }
                        let answer = line.trim();
                        if !answer.is_empty() {
                            let answers = vec![format!("user_note: {answer}")];
                            break ToolRequestUserInputAnswer {
                                selected_option_ids: None,
                                ordered_option_ids: None,
                                free_text: Some(answer.to_string()),
                                legacy_answers: Some(answers.clone()),
                                answers,
                            };
                        }
                    }

                    writeln!(output, "Invalid selection; try again.")?;
                },
            }
        } else {
            loop {
                write!(output, "Answer: ")?;
                output.flush()?;

                let mut line = String::new();
                if input
                    .read_line(&mut line)
                    .context("failed to read request_user_input answer")?
                    == 0
                {
                    bail!("stdin closed while waiting for request_user_input answer");
                }
                let answer = line.trim();
                if !answer.is_empty() {
                    let answers = vec![format!("user_note: {answer}")];
                    break ToolRequestUserInputAnswer {
                        selected_option_ids: None,
                        ordered_option_ids: None,
                        free_text: Some(answer.to_string()),
                        legacy_answers: Some(answers.clone()),
                        answers,
                    };
                }
                writeln!(output, "Answer cannot be empty; try again.")?;
            }
        };

        answers.insert(question.id.clone(), answer);
    }

    Ok(ToolRequestUserInputResponse { answers })
}

fn option_answer(option: &ToolRequestUserInputOption) -> ToolRequestUserInputAnswer {
    let answers = vec![option.label.clone()];
    ToolRequestUserInputAnswer {
        selected_option_ids: option.id.clone().map(|id| vec![id]),
        ordered_option_ids: None,
        free_text: None,
        legacy_answers: Some(answers.clone()),
        answers,
    }
}

fn option_indices_answer(
    options: &[ToolRequestUserInputOption],
    indices: &[usize],
    kind: ToolRequestUserInputQuestionKind,
) -> ToolRequestUserInputAnswer {
    let ids = indices
        .iter()
        .filter_map(|index| options.get(*index))
        .map(|option| option.id.clone().unwrap_or_else(|| option.label.clone()))
        .collect::<Vec<_>>();
    let answers = indices
        .iter()
        .filter_map(|index| options.get(*index))
        .map(|option| option.label.clone())
        .collect::<Vec<_>>();
    match kind {
        ToolRequestUserInputQuestionKind::MultiChoice => ToolRequestUserInputAnswer {
            selected_option_ids: (!ids.is_empty()).then_some(ids),
            ordered_option_ids: None,
            free_text: None,
            legacy_answers: Some(answers.clone()),
            answers,
        },
        ToolRequestUserInputQuestionKind::Ranking => ToolRequestUserInputAnswer {
            selected_option_ids: None,
            ordered_option_ids: (!ids.is_empty()).then_some(ids),
            free_text: None,
            legacy_answers: Some(answers.clone()),
            answers,
        },
        ToolRequestUserInputQuestionKind::SingleChoice
        | ToolRequestUserInputQuestionKind::FreeText => ToolRequestUserInputAnswer {
            selected_option_ids: None,
            ordered_option_ids: None,
            free_text: None,
            legacy_answers: None,
            answers: Vec::new(),
        },
    }
}

fn read_option_indices(
    input: &mut impl BufRead,
    options: &[ToolRequestUserInputOption],
) -> Result<Option<Vec<usize>>> {
    let mut line = String::new();
    if input
        .read_line(&mut line)
        .context("failed to read request_user_input selection")?
        == 0
    {
        bail!("stdin closed while waiting for request_user_input selection");
    }
    let mut indices = Vec::new();
    for token in line.trim().split([',', '，']) {
        let token = token.trim();
        if token.is_empty() {
            return Ok(None);
        }
        let index = if let Ok(number) = token.parse::<usize>() {
            let Some(index) = number.checked_sub(1).filter(|index| *index < options.len()) else {
                return Ok(None);
            };
            index
        } else {
            let Some(index) = options
                .iter()
                .position(|option| option.label.eq_ignore_ascii_case(token))
            else {
                return Ok(None);
            };
            index
        };
        indices.push(index);
    }
    Ok((!indices.is_empty()).then_some(indices))
}

#[cfg(test)]
#[path = "request_user_input_tests.rs"]
mod tests;
