//! Persistent Plan/SubPlan panel rendering for the main chat surface.

use super::*;
use crate::wrapping::RtOptions;
use crate::wrapping::adaptive_wrap_line;
use codex_protocol::plan_tool::PlanNode;
use codex_protocol::plan_tool::PlanState;
use codex_protocol::plan_tool::PlanSubItem;
use codex_protocol::plan_tool::StepStatus;
use ratatui::text::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlanPanelMode {
    Hidden,
    Expanded,
    Collapsed,
}

impl ChatWidget {
    pub(super) fn plan_panel_mode(&self) -> PlanPanelMode {
        let Some(state) = &self.transcript.latest_plan_state else {
            return PlanPanelMode::Hidden;
        };
        if state.items.is_empty() {
            return PlanPanelMode::Hidden;
        }
        if self.plan_panel_expanded || self.bottom_pane.is_task_running() {
            PlanPanelMode::Expanded
        } else {
            PlanPanelMode::Collapsed
        }
    }

    pub(super) fn toggle_plan_panel_expanded(&mut self) {
        if self.transcript.latest_plan_state.is_none() {
            return;
        }
        self.plan_panel_expanded = !self.plan_panel_expanded;
        self.request_redraw();
    }
}

pub(super) struct PlanPanelRenderable<'a> {
    state: &'a PlanState,
    mode: PlanPanelMode,
}

impl<'a> PlanPanelRenderable<'a> {
    pub(super) fn new(state: &'a PlanState, mode: PlanPanelMode) -> Self {
        Self { state, mode }
    }

    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        match self.mode {
            PlanPanelMode::Hidden => Vec::new(),
            PlanPanelMode::Collapsed => collapsed_lines(self.state, width),
            PlanPanelMode::Expanded => expanded_lines(self.state, width),
        }
    }
}

impl Renderable for PlanPanelRenderable<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let lines = self.lines(area.width);
        let area = Rect {
            height: area.height.min(lines.len().try_into().unwrap_or(u16::MAX)),
            ..area
        };
        Paragraph::new(lines).render(area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        self.lines(width).len().try_into().unwrap_or(u16::MAX)
    }
}

fn collapsed_lines(state: &PlanState, width: u16) -> Vec<Line<'static>> {
    let progress = progress_label(state).text;
    let title = state
        .title
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Plan");
    let (completed, total) = root_progress_counts(state);
    if completed == total && total > 0 {
        let text = format!("{title}: all milestones completed ({completed}/{total})");
        return wrap_panel_line(vec!["• ".dim(), text.into()], width, "", "  ");
    }
    let active = active_root(state).or_else(|| state.items.first());
    let active_title = active
        .map(|item| item.title.as_str())
        .unwrap_or("No active item");
    let active_subplan = active.and_then(active_subplan_title);
    let detail = match active_subplan {
        Some(subplan) => format!("{active_title} / {subplan}"),
        None => active_title.to_string(),
    };
    let text = format!("{title}: {detail} ({progress})");
    wrap_panel_line(vec!["• ".dim(), text.into()], width, "", "  ")
}

fn expanded_lines(state: &PlanState, width: u16) -> Vec<Line<'static>> {
    let progress = progress_label(state);
    let progress_text = if progress.is_split {
        format!("({})", progress.text)
    } else {
        progress.text
    };
    let title = state
        .title
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Plan");
    let mut lines = wrap_panel_line(
        vec![
            "• ".dim(),
            title.to_string().bold(),
            " ".into(),
            progress_text.dim(),
        ],
        width,
        "",
        "  ",
    );
    if let Some(summary) = state
        .summary
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        lines.extend(wrap_panel_line(
            vec![summary.to_string().dim()],
            width,
            "  ",
            "  ",
        ));
    }
    let active = active_root(state);
    for item in &state.items {
        lines.extend(root_item_lines(
            item,
            width,
            active.is_some_and(|active| std::ptr::eq(active, item)),
        ));
    }
    lines
}

fn root_item_lines(item: &PlanNode, width: u16, expand_subplan: bool) -> Vec<Line<'static>> {
    let mut lines = wrap_panel_line(
        vec![
            status_symbol(&item.status),
            " ".into(),
            item.title.clone().into(),
        ],
        width,
        "  ",
        "    ",
    );
    if let Some(summary) = item
        .summary
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        lines.extend(wrap_panel_line(
            vec![summary.to_string().dim()],
            width,
            "    ",
            "    ",
        ));
    }
    if expand_subplan && let Some(subplan) = &item.subplan {
        if let Some(title) = subplan
            .title
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            lines.extend(wrap_panel_line(
                vec!["SubPlan: ".dim(), title.to_string().dim()],
                width,
                "    ",
                "    ",
            ));
        }
        for subitem in &subplan.items {
            lines.extend(sub_item_lines(subitem, width));
        }
    }
    lines
}

fn sub_item_lines(item: &PlanSubItem, width: u16) -> Vec<Line<'static>> {
    let mut lines = wrap_panel_line(
        vec![
            status_symbol(&item.status),
            " ".into(),
            item.title.clone().dim(),
        ],
        width,
        "    ",
        "      ",
    );
    if let Some(summary) = item
        .summary
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        lines.extend(wrap_panel_line(
            vec![summary.to_string().dim()],
            width,
            "      ",
            "      ",
        ));
    }
    lines
}

fn wrap_panel_line(
    spans: Vec<Span<'static>>,
    width: u16,
    initial_indent: &'static str,
    subsequent_indent: &'static str,
) -> Vec<Line<'static>> {
    let opts = RtOptions::new(width.max(1) as usize)
        .initial_indent(initial_indent.into())
        .subsequent_indent(subsequent_indent.into());
    let line = Line::from(spans);
    let wrapped = adaptive_wrap_line(&line, opts);
    let mut lines = Vec::new();
    crate::render::line_utils::push_owned_lines(&wrapped, &mut lines);
    lines
        .into_iter()
        .map(ratatui::prelude::Stylize::dim)
        .collect()
}

fn status_symbol(status: &StepStatus) -> Span<'static> {
    match status {
        StepStatus::Completed => "✓".green(),
        StepStatus::InProgress => "●".cyan(),
        StepStatus::Pending => "○".dim(),
    }
}

struct ProgressLabel {
    text: String,
    is_split: bool,
}

fn progress_label(state: &PlanState) -> ProgressLabel {
    ProgressLabel {
        text: progress_text(state),
        is_split: has_active_subplan_progress(state),
    }
}

pub(super) fn progress_text(state: &PlanState) -> String {
    let root = root_progress_counts(state);
    if let Some(subplan) = active_root(state).and_then(|item| item.subplan.as_ref()) {
        let subplan = subplan_progress_counts(subplan);
        if subplan.1 > 0 {
            return format!("{}/{} | {}/{}", subplan.0, subplan.1, root.0, root.1);
        }
    }
    format!("{}/{}", root.0, root.1)
}

fn has_active_subplan_progress(state: &PlanState) -> bool {
    active_root(state)
        .and_then(|item| item.subplan.as_ref())
        .is_some_and(|subplan| !subplan.items.is_empty())
}

fn root_progress_counts(state: &PlanState) -> (usize, usize) {
    let total = state.items.len();
    let completed = state
        .items
        .iter()
        .filter(|item| matches!(item.status, StepStatus::Completed))
        .count();
    (completed, total)
}

fn subplan_progress_counts(subplan: &codex_protocol::plan_tool::SubPlan) -> (usize, usize) {
    let total = subplan.items.len();
    let completed = subplan
        .items
        .iter()
        .filter(|item| matches!(item.status, StepStatus::Completed))
        .count();
    (completed, total)
}

fn active_root(state: &PlanState) -> Option<&PlanNode> {
    state
        .items
        .iter()
        .find(|item| matches!(item.status, StepStatus::InProgress))
        .or_else(|| {
            state.items.iter().find(|item| {
                item.subplan.as_ref().is_some_and(|subplan| {
                    subplan
                        .items
                        .iter()
                        .any(|subitem| matches!(subitem.status, StepStatus::InProgress))
                })
            })
        })
}

fn active_subplan_title(item: &PlanNode) -> Option<&str> {
    let subplan = item.subplan.as_ref()?;
    let active = subplan
        .items
        .iter()
        .find(|subitem| matches!(subitem.status, StepStatus::InProgress))
        .or_else(|| subplan.items.first())?;
    Some(active.title.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn expanded_plan_panel_renders_subplan() {
        let state = sample_state();
        let panel = PlanPanelRenderable::new(&state, PlanPanelMode::Expanded);

        assert_snapshot!(render_panel(&panel, /*width*/ 72).join("\n"));
    }

    #[test]
    fn collapsed_plan_panel_summarizes_active_subplan() {
        let state = sample_state();
        let panel = PlanPanelRenderable::new(&state, PlanPanelMode::Collapsed);

        assert_snapshot!(render_panel(&panel, /*width*/ 52).join("\n"));
    }

    #[test]
    fn collapsed_plan_panel_summarizes_all_completed_state() {
        let state = completed_state();
        let panel = PlanPanelRenderable::new(&state, PlanPanelMode::Collapsed);
        let rendered = render_panel(&panel, /*width*/ 72).join("\n");

        assert!(
            rendered.contains("Plan: all milestones completed (2/2)"),
            "collapsed completed plan should show a completion summary, got:\n{rendered}"
        );
        assert!(
            !rendered.contains("Prepare failing tests"),
            "collapsed completed plan should not fall back to the first completed item, got:\n{rendered}"
        );
        assert_snapshot!(rendered);
    }

    #[test]
    fn progress_label_splits_active_subplan_and_root_counts_for_any_structured_plan() {
        let state = sample_state();
        let progress = progress_label(&state);

        assert_eq!(progress.text, "1/2 | 1/2");
        assert!(progress.is_split);
    }

    #[test]
    fn progress_label_uses_root_counts_when_no_subplan_is_active() {
        let state = completed_state();
        let progress = progress_label(&state);

        assert_eq!(progress.text, "2/2");
        assert!(!progress.is_split);
    }

    fn sample_state() -> PlanState {
        PlanState {
            id: Some("c3".to_string()),
            title: Some("C3 Ask Tools".to_string()),
            summary: Some("Ship Ask and persistent plan UI.".to_string()),
            items: vec![
                PlanNode {
                    id: Some("m1".to_string()),
                    title: "Ask protocol".to_string(),
                    summary: Some("Structured answers are available.".to_string()),
                    status: StepStatus::Completed,
                    subplan: None,
                },
                PlanNode {
                    id: Some("m2".to_string()),
                    title: "Persistent Plan panel".to_string(),
                    summary: None,
                    status: StepStatus::InProgress,
                    subplan: Some(codex_protocol::plan_tool::SubPlan {
                        title: Some("TUI work".to_string()),
                        summary: None,
                        items: vec![
                            PlanSubItem {
                                id: Some("s1".to_string()),
                                title: "Store structured state".to_string(),
                                summary: None,
                                status: StepStatus::Completed,
                            },
                            PlanSubItem {
                                id: Some("s2".to_string()),
                                title: "Render right-side persistent panel".to_string(),
                                summary: Some("Default idle collapses to one row.".to_string()),
                                status: StepStatus::InProgress,
                            },
                        ],
                    }),
                },
            ],
        }
    }

    fn completed_state() -> PlanState {
        PlanState {
            id: Some("m12".to_string()),
            title: Some("Plan".to_string()),
            summary: Some("All milestone work is complete.".to_string()),
            items: vec![
                PlanNode {
                    id: Some("red".to_string()),
                    title: "Prepare failing tests".to_string(),
                    summary: None,
                    status: StepStatus::Completed,
                    subplan: None,
                },
                PlanNode {
                    id: Some("verify".to_string()),
                    title: "Run focused tests".to_string(),
                    summary: None,
                    status: StepStatus::Completed,
                    subplan: Some(codex_protocol::plan_tool::SubPlan {
                        title: Some("Focused validation".to_string()),
                        summary: None,
                        items: vec![
                            PlanSubItem {
                                id: Some("spawn".to_string()),
                                title: "Subagent display checks".to_string(),
                                summary: None,
                                status: StepStatus::Completed,
                            },
                            PlanSubItem {
                                id: Some("plan".to_string()),
                                title: "Plan collapsed completion check".to_string(),
                                summary: None,
                                status: StepStatus::Completed,
                            },
                        ],
                    }),
                },
            ],
        }
    }

    fn render_panel(panel: &PlanPanelRenderable<'_>, width: u16) -> Vec<String> {
        panel
            .lines(width)
            .into_iter()
            .map(|line| {
                line.spans
                    .into_iter()
                    .map(|span| span.content.into_owned())
                    .collect::<String>()
            })
            .collect()
    }
}
