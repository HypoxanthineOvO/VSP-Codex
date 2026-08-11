#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowMarkdownSurface {
    pub title: String,
    pub summary: Option<String>,
    pub plan_progress: Option<PlanProgressSurface>,
    pub subagents: Vec<SubagentMarkdownSurface>,
    pub tool_output_summary: Option<ToolOutputSummarySurface>,
    pub request_input_fallback: Option<RequestInputFallbackMarkdown>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanProgressSurface {
    pub root_title: String,
    pub active_root_title: String,
    pub subplan_title: Option<String>,
    pub subplan_completed: usize,
    pub subplan_total: usize,
    pub root_completed: usize,
    pub root_total: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentMarkdownSurface {
    pub role: String,
    pub prompt_markdown: Option<String>,
    pub report_markdown: Option<String>,
    pub derived_translation_markdown: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolOutputSummarySurface {
    pub command: String,
    pub summary_markdown: String,
    pub raw_output_excerpt: String,
    pub omitted_line_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestInputFallbackMarkdown {
    pub markdown: String,
}

pub fn render_workflow_markdown_surface(surface: WorkflowMarkdownSurface) -> String {
    let mut sections = Vec::new();

    sections.push(format!("## {}", surface.title.trim()));

    if let Some(summary) = non_empty(surface.summary.as_deref()) {
        sections.push(summary.to_string());
    }

    if let Some(plan_progress) = surface.plan_progress.as_ref() {
        sections.push(render_plan_progress(plan_progress));
    }

    for subagent in &surface.subagents {
        sections.push(render_subagent(subagent));
    }

    if let Some(tool_output_summary) = surface.tool_output_summary.as_ref() {
        sections.push(render_tool_output_summary(tool_output_summary));
    }

    append_request_input_fallback(&mut sections, surface.request_input_fallback.as_ref());

    sections.join("\n\n")
}

pub fn render_request_input_workflow_markdown_surface(markdown: String) -> String {
    render_workflow_markdown_surface(WorkflowMarkdownSurface {
        title: "Workflow Surface".to_string(),
        summary: None,
        plan_progress: None,
        subagents: Vec::new(),
        tool_output_summary: None,
        request_input_fallback: Some(RequestInputFallbackMarkdown { markdown }),
    })
}

fn render_plan_progress(progress: &PlanProgressSurface) -> String {
    let subplan_title = progress
        .subplan_title
        .as_deref()
        .and_then(|title| non_empty(Some(title)))
        .unwrap_or(progress.active_root_title.trim());
    format!(
        "Plan progress: {} {}/{} | {} {}/{}",
        subplan_title,
        progress.subplan_completed,
        progress.subplan_total,
        progress.root_title.trim(),
        progress.root_completed,
        progress.root_total
    )
}

fn render_subagent(subagent: &SubagentMarkdownSurface) -> String {
    let mut lines = Vec::new();
    lines.push(format!("### Subagent: {}", subagent.role.trim()));

    append_labeled_markdown(
        &mut lines,
        "Prompt Original",
        subagent.prompt_markdown.as_deref(),
    );
    append_labeled_markdown(
        &mut lines,
        "Report Original",
        subagent.report_markdown.as_deref(),
    );
    append_labeled_markdown(
        &mut lines,
        "Derived Translation",
        subagent.derived_translation_markdown.as_deref(),
    );

    lines.join("\n\n")
}

fn render_tool_output_summary(summary: &ToolOutputSummarySurface) -> String {
    let mut lines = Vec::new();
    lines.push("### Tool Output Summary".to_string());

    if let Some(command) = non_empty(Some(summary.command.as_str())) {
        lines.push(format!("Command: `{command}`"));
    }

    if let Some(summary_markdown) = non_empty(Some(summary.summary_markdown.as_str())) {
        lines.push(summary_markdown.to_string());
    }

    if let Some(raw_output_excerpt) = non_empty(Some(summary.raw_output_excerpt.as_str())) {
        lines.push("Raw output excerpt:".to_string());
        lines.push(fenced("text", raw_output_excerpt));
    }

    if summary.omitted_line_count > 0 {
        lines.push(format!(
            "Omitted {} raw output lines.",
            summary.omitted_line_count
        ));
    }

    lines.join("\n\n")
}

fn append_request_input_fallback(
    sections: &mut Vec<String>,
    request_input_fallback: Option<&RequestInputFallbackMarkdown>,
) {
    let Some(request_input_fallback) = request_input_fallback else {
        return;
    };
    let Some(markdown) = non_empty(Some(request_input_fallback.markdown.as_str())) else {
        return;
    };

    sections.push(
        [
            "### Request Input",
            "",
            "Request Input fallback for plugin clients without native Ask UI.",
            "",
            markdown,
        ]
        .join("\n"),
    );
}

fn append_labeled_markdown(lines: &mut Vec<String>, label: &str, markdown: Option<&str>) {
    let Some(markdown) = non_empty(markdown) else {
        return;
    };

    lines.push(format!("#### {label}"));
    lines.push(strip_duplicate_heading(markdown, label).to_string());
}

fn strip_duplicate_heading<'a>(markdown: &'a str, label: &str) -> &'a str {
    let trimmed = markdown.trim();
    let Some(rest) = trimmed.strip_prefix("#### ") else {
        return trimmed;
    };
    rest.strip_prefix(label)
        .and_then(|after_label| after_label.strip_prefix('\n'))
        .map(str::trim)
        .unwrap_or(trimmed)
}

fn fenced(language: &str, body: &str) -> String {
    format!("```{language}\n{}\n```", body.trim())
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}
