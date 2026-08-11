use codex_app_server::workflow_markdown_surface::PlanProgressSurface;
use codex_app_server::workflow_markdown_surface::RequestInputFallbackMarkdown;
use codex_app_server::workflow_markdown_surface::SubagentMarkdownSurface;
use codex_app_server::workflow_markdown_surface::ToolOutputSummarySurface;
use codex_app_server::workflow_markdown_surface::WorkflowMarkdownSurface;
use codex_app_server::workflow_markdown_surface::render_workflow_markdown_surface;

#[test]
fn m6_workflow_markdown_surface_is_plugin_readable_without_protocol_clutter() {
    let markdown = render_workflow_markdown_surface(WorkflowMarkdownSurface {
        title: "M6 Markdown Workflow Surface".to_string(),
        summary: Some("Remote SSH plugin fallback surface for Workflow state.".to_string()),
        plan_progress: Some(PlanProgressSurface {
            root_title: "C8 Cycle Plan".to_string(),
            active_root_title: "M6 Markdown Workflow Surface And Remote SSH Acceptance".to_string(),
            subplan_title: Some("M6 test worker".to_string()),
            subplan_completed: 1,
            subplan_total: 4,
            root_completed: 5,
            root_total: 6,
        }),
        subagents: vec![SubagentMarkdownSurface {
            role: "test".to_string(),
            prompt_markdown: Some(
                [
                    "# Test Worker Prompt",
                    "",
                    "Preserve evidence from `.pipeline/reports/020-request-tool-request-input-plugin-ux.md`.",
                    "",
                    "```bash",
                    "cargo test -p codex-app-server --test all request_user_input",
                    "```",
                ]
                .join("\n"),
            ),
            report_markdown: Some(
                [
                    "# Test Worker Report",
                    "",
                    "Derived translation is available, but the original evidence remains authoritative.",
                    "",
                    "```text",
                    "request input fallback rendered as Markdown",
                    "```",
                ]
                .join("\n"),
            ),
            derived_translation_markdown: Some(
                "Derived translation: request input fallback rendered as Markdown.".to_string(),
            ),
        }],
        tool_output_summary: Some(ToolOutputSummarySurface {
            command: "cargo test -p codex-app-server --test all m6_workflow_markdown_surface"
                .to_string(),
            summary_markdown: "Focused app-server contract tests failed at compile time.".to_string(),
            raw_output_excerpt: "error[E0432]: unresolved import".to_string(),
            omitted_line_count: 42,
        }),
        request_input_fallback: Some(RequestInputFallbackMarkdown {
            markdown: [
                "## Workflow Decision",
                "",
                "**Answer controls**",
                "",
                "1. Accept the Remote SSH validation scope.",
            ]
            .join("\n"),
        }),
    });

    for expected in [
        "## M6 Markdown Workflow Surface",
        "Remote SSH plugin fallback surface for Workflow state.",
        "Plan progress: M6 test worker 1/4 | C8 Cycle Plan 5/6",
        "### Subagent: test",
        "#### Prompt Original",
        "#### Report Original",
        "#### Derived Translation",
        "`.pipeline/reports/020-request-tool-request-input-plugin-ux.md`",
        "```bash",
        "cargo test -p codex-app-server --test all request_user_input",
        "```",
        "```text",
        "request input fallback rendered as Markdown",
        "```",
        "### Tool Output Summary",
        "Omitted 42 raw output lines.",
        "## Workflow Decision",
    ] {
        assert!(
            markdown.contains(expected),
            "plugin-visible Markdown should contain {expected:?}; got:\n{markdown}"
        );
    }

    for forbidden in [
        "019cff70-2599-75e2-af72-b958ce5dc4c7",
        "ServerNotification",
        "ThreadItem::",
        "ToolRequestUserInputParams",
        "workflowDecisionType",
        "raw_transcript",
        "机器翻译\n机器翻译",
        "Derived Translation\nDerived Translation",
        "6/10",
    ] {
        assert!(
            !markdown.contains(forbidden),
            "plugin-visible Markdown must avoid raw protocol/debug clutter or duplicated translation blocks {forbidden:?}; got:\n{markdown}"
        );
    }
}

#[test]
fn m6_plan_progress_surface_keeps_subplan_and_root_counts_separate() {
    let markdown = render_workflow_markdown_surface(WorkflowMarkdownSurface {
        title: "Plan Progress".to_string(),
        summary: None,
        plan_progress: Some(PlanProgressSurface {
            root_title: "C8".to_string(),
            active_root_title: "M6".to_string(),
            subplan_title: Some("M6 implement worker".to_string()),
            subplan_completed: 2,
            subplan_total: 5,
            root_completed: 5,
            root_total: 6,
        }),
        subagents: Vec::new(),
        tool_output_summary: None,
        request_input_fallback: None,
    });

    assert!(
        markdown.contains("Plan progress: M6 implement worker 2/5 | C8 5/6"),
        "subplan progress and root plan progress should be displayed as separate counts:\n{markdown}"
    );
    assert!(
        !markdown.contains("7/11"),
        "surface must not mix completed/total counts across root plan and subplan:\n{markdown}"
    );
}
