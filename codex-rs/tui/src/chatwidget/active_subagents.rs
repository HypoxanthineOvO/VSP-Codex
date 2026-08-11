//! Active subagent panel state and rendering.

use super::*;
use crate::bottom_pane::SelectionRowDisplay;
use crate::bottom_pane::SideContentWidth;
use crate::goal_display::format_goal_elapsed_seconds;
use crate::hypo_translate::RuntimeTranslator;
use crate::hypo_translate::TranslatedText;
use crate::hypo_translate::TranslationDisplay;
use crate::hypo_translate::TranslationScope;
use crate::hypo_translate::TranslationSource;
use codex_app_server_protocol::CollabAgentState;
use codex_app_server_protocol::CollabAgentStatus;
use codex_app_server_protocol::SubAgentActivityKind;
use codex_protocol::openai_models::ReasoningEffort as ReasoningEffortConfig;
use ratatui::text::Span;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
#[cfg(test)]
use std::time::Duration;

const ACTIVE_SUBAGENT_MAX_ROWS: usize = 3;
const ACTIVE_SUBAGENT_PREVIEW_GRAPHEMES: usize = 120;
// Forces the prompt/report viewer to render stacked below the selection list
// rather than side-by-side. Side-by-side caps the side panel to the list's row
// height (list_selection_view shares vertical space in that mode), which would
// truncate long prompt/report bodies with no scrollback. Stacked layout instead
// adds the viewer's full desired height to the popup, so the complete original +
// translated content always fits.
const ACTIVE_SUBAGENT_VIEWER_SIDE_MIN_WIDTH: u16 = u16::MAX;
const BUILT_IN_FALLBACK_NAMES: &[&str] = &[
    "娄鑫",
    "李萌",
    "杨超",
    "朱昊骋",
    "李超凡",
    "贺云翔",
    "吴浩宇",
    "丁麒涵",
    "李宇轩",
    "李铭杰",
    "闫启航",
    "张雍治",
    "蒋烨",
    "陈泉宇",
    "王珺",
    "沈博宇",
];
#[allow(dead_code)]
const PUBLIC_ANCIENT_FALLBACK_NAMES: &[&str] = &[
    "孔丘",
    "孟轲",
    "墨翟",
    "庄周",
    "屈原",
    "司马迁",
    "张衡",
    "蔡伦",
    "祖冲之",
    "郦道元",
    "李白",
    "杜甫",
    "韩愈",
    "柳宗元",
    "苏轼",
    "沈括",
    "李清照",
    "辛弃疾",
    "关汉卿",
    "徐霞客",
    "宋应星",
    "曹雪芹",
];
const BUILT_IN_SUPERVISOR_FALLBACK_NAMES: &[&str] = &["娄鑫", "李萌"];
const BUILT_IN_EXPLORER_FALLBACK_NAMES: &[&str] = &["杨超", "朱昊骋"];
const BUILT_IN_WORKER_FALLBACK_NAMES: &[&str] = &["娄鑫", "李萌"];

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReleaseBuildSource {
    InternalGitLab,
    ExternalGitHub,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReleaseChannelMetadata {
    source: ReleaseBuildSource,
    internal_lab_names: Vec<String>,
    external_github_names: Vec<String>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl ReleaseChannelMetadata {
    pub(super) fn new(
        source: ReleaseBuildSource,
        internal_lab_names: Vec<String>,
        external_github_names: Vec<String>,
    ) -> Self {
        Self {
            source,
            internal_lab_names,
            external_github_names,
        }
    }

    fn fallback_names(&self) -> &[String] {
        match self.source {
            ReleaseBuildSource::InternalGitLab => &self.internal_lab_names,
            ReleaseBuildSource::ExternalGitHub => &self.external_github_names,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct ActiveSubagentRow {
    thread_id: ThreadId,
    agent_path: Option<String>,
    nickname: Option<String>,
    fallback_nickname: Option<String>,
    role: Option<String>,
    model: Option<String>,
    reasoning_effort: Option<ReasoningEffortConfig>,
    status: CollabAgentStatus,
    message: Option<String>,
    started_at: Instant,
    last_prompt: Option<String>,
    last_tool: CollabAgentTool,
}

struct ActiveSubagentRowUpdate {
    metadata: AgentMetadata,
    spawn_request: Option<multi_agents::SpawnRequestSummary>,
    status: CollabAgentStatus,
    message: Option<String>,
    prompt: Option<String>,
    tool: CollabAgentTool,
}

struct ActiveSubagentRowFallback {
    metadata: AgentMetadata,
}

#[derive(Clone, Debug, Default)]
pub(super) struct SubagentDisplayNameResolver {
    resolved_fallbacks: HashMap<ThreadId, String>,
    active_leases: HashMap<ThreadId, String>,
    release_channel_metadata: Option<ReleaseChannelMetadata>,
}

impl SubagentDisplayNameResolver {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn with_release_channel_metadata(
        mut self,
        metadata: ReleaseChannelMetadata,
    ) -> Self {
        self.release_channel_metadata = Some(metadata);
        self
    }

    pub(super) fn resolve_metadata(
        &mut self,
        config: &Config,
        thread_id: ThreadId,
        mut metadata: AgentMetadata,
    ) -> AgentMetadata {
        if explicit_nickname(metadata.agent_nickname.as_deref()).is_some() {
            return metadata;
        }

        let fallback = self
            .resolved_fallbacks
            .get(&thread_id)
            .cloned()
            .unwrap_or_else(|| self.resolve_fallback(config, thread_id, &metadata));
        metadata.agent_nickname = Some(fallback);
        metadata
    }

    pub(super) fn resolve_active_metadata(
        &mut self,
        config: &Config,
        thread_id: ThreadId,
        mut metadata: AgentMetadata,
    ) -> AgentMetadata {
        if let Some(nickname) = explicit_nickname(metadata.agent_nickname.as_deref()) {
            self.active_leases.insert(thread_id, nickname.to_string());
            return metadata;
        }

        let fallback = self
            .active_leases
            .get(&thread_id)
            .cloned()
            .or_else(|| self.resolved_fallbacks.get(&thread_id).cloned())
            .unwrap_or_else(|| self.lease_fallback(config, thread_id, &metadata));
        self.resolved_fallbacks.insert(thread_id, fallback.clone());
        self.active_leases.insert(thread_id, fallback.clone());
        metadata.agent_nickname = Some(fallback);
        metadata
    }

    pub(super) fn reserve_active_explicit_metadata(
        &mut self,
        thread_id: ThreadId,
        metadata: &AgentMetadata,
    ) {
        if let Some(nickname) = explicit_nickname(metadata.agent_nickname.as_deref()) {
            self.active_leases.insert(thread_id, nickname.to_string());
        }
    }

    pub(super) fn release(&mut self, thread_id: ThreadId) {
        self.active_leases.remove(&thread_id);
    }

    fn resolve_fallback(
        &mut self,
        config: &Config,
        thread_id: ThreadId,
        metadata: &AgentMetadata,
    ) -> String {
        let candidates = self.fallback_subagent_name_candidates(config, metadata);
        let resolved = if candidates.is_empty() {
            fallback_subagent_name(thread_id, None).to_string()
        } else {
            let hash = fallback_subagent_name_hash(thread_id);
            candidates[hash % candidates.len()].clone()
        };
        self.resolved_fallbacks.insert(thread_id, resolved.clone());
        resolved
    }

    fn lease_fallback(
        &mut self,
        config: &Config,
        thread_id: ThreadId,
        metadata: &AgentMetadata,
    ) -> String {
        let candidates = self.fallback_subagent_name_candidates(config, metadata);
        let leased = if candidates.is_empty() {
            fallback_subagent_name(thread_id, None).to_string()
        } else {
            choose_lease_candidate(thread_id, &candidates, &self.active_leases)
        };
        self.resolved_fallbacks.insert(thread_id, leased.clone());
        self.active_leases.insert(thread_id, leased.clone());
        leased
    }

    fn fallback_subagent_name_candidates(
        &self,
        config: &Config,
        metadata: &AgentMetadata,
    ) -> Vec<String> {
        fallback_subagent_name_candidates(config, metadata, self.release_channel_metadata.as_ref())
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct ActiveSubagentPanelState {
    rows: Vec<ActiveSubagentRow>,
    pending_v2_runtime: HashMap<ThreadId, V2RuntimeMetadata>,
    pending_v2_running: HashMap<ThreadId, bool>,
}

#[derive(Clone, Debug)]
struct V2RuntimeMetadata {
    model: String,
    reasoning_effort: Option<ReasoningEffortConfig>,
    task: Option<String>,
}

impl ActiveSubagentPanelState {
    pub(super) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    fn active_rows(&self) -> &[ActiveSubagentRow] {
        &self.rows
    }

    fn upsert_from_tool_call(
        &mut self,
        item: &ThreadItem,
        cached_spawn_request: Option<&multi_agents::SpawnRequestSummary>,
        mut agent_metadata: impl FnMut(ThreadId) -> ActiveSubagentRowFallback,
        mut release_agent_name: impl FnMut(ThreadId),
    ) {
        let ThreadItem::CollabAgentToolCall {
            tool,
            status,
            receiver_thread_ids,
            prompt,
            agents_states,
            ..
        } = item
        else {
            return;
        };

        let fallback_spawn_request = collab_request_summary(item);
        let spawn_request = cached_spawn_request.or(fallback_spawn_request.as_ref());
        for receiver_thread_id in receiver_thread_ids {
            let Ok(thread_id) = ThreadId::from_string(receiver_thread_id) else {
                continue;
            };
            let Some(agent_state) = agents_states.get(receiver_thread_id) else {
                if matches!(tool, CollabAgentTool::SpawnAgent)
                    && matches!(status, CollabAgentToolCallStatus::InProgress)
                {
                    let row_fallback = agent_metadata(thread_id);
                    self.upsert_row(
                        thread_id,
                        ActiveSubagentRowUpdate {
                            metadata: row_fallback.metadata,
                            spawn_request: spawn_request.cloned(),
                            status: CollabAgentStatus::PendingInit,
                            message: None,
                            prompt: prompt.clone(),
                            tool: tool.clone(),
                        },
                    );
                } else if is_terminal_tool_call_status(status) {
                    self.remove(thread_id);
                    release_agent_name(thread_id);
                }
                continue;
            };

            if is_active_agent_status(&agent_state.status) {
                let row_fallback = agent_metadata(thread_id);
                self.upsert_row(
                    thread_id,
                    ActiveSubagentRowUpdate {
                        metadata: row_fallback.metadata,
                        spawn_request: spawn_request.cloned(),
                        status: agent_state.status.clone(),
                        message: agent_state.message.clone(),
                        prompt: prompt.clone(),
                        tool: tool.clone(),
                    },
                );
            } else {
                self.remove(thread_id);
                release_agent_name(thread_id);
            }
        }
    }

    fn upsert_row(&mut self, thread_id: ThreadId, update: ActiveSubagentRowUpdate) {
        let existing = self
            .rows
            .iter_mut()
            .find(|candidate| candidate.thread_id == thread_id);
        match existing {
            Some(row) => {
                row.nickname = update.metadata.agent_nickname;
                row.fallback_nickname = None;
                row.role = update.metadata.agent_role;
                if let Some(spawn_request) = update.spawn_request {
                    row.model = Some(spawn_request.model);
                    row.reasoning_effort = Some(spawn_request.reasoning_effort);
                }
                row.status = update.status;
                row.message = update.message;
                if let Some(prompt) = update.prompt.filter(|prompt| !prompt.trim().is_empty()) {
                    row.last_prompt = Some(prompt);
                }
                row.last_tool = update.tool;
            }
            None => {
                self.rows.push(ActiveSubagentRow {
                    thread_id,
                    agent_path: None,
                    nickname: update.metadata.agent_nickname,
                    fallback_nickname: None,
                    role: update.metadata.agent_role,
                    model: update
                        .spawn_request
                        .as_ref()
                        .map(|request| request.model.clone()),
                    reasoning_effort: update
                        .spawn_request
                        .as_ref()
                        .map(|request| request.reasoning_effort.clone()),
                    status: update.status,
                    message: update.message,
                    started_at: Instant::now(),
                    last_prompt: update.prompt.filter(|prompt| !prompt.trim().is_empty()),
                    last_tool: update.tool,
                });
            }
        }
    }

    fn remove(&mut self, thread_id: ThreadId) {
        self.rows.retain(|row| row.thread_id != thread_id);
    }

    fn upsert_from_v2_activity(
        &mut self,
        thread_id: ThreadId,
        agent_path: String,
        kind: SubAgentActivityKind,
        metadata: AgentMetadata,
    ) {
        let mut status = match kind {
            SubAgentActivityKind::Started | SubAgentActivityKind::Interacted => {
                CollabAgentStatus::Running
            }
            SubAgentActivityKind::Interrupted => CollabAgentStatus::Interrupted,
        };
        if !matches!(kind, SubAgentActivityKind::Interrupted)
            && let Some(running) = self.pending_v2_running.remove(&thread_id)
        {
            status = if running {
                CollabAgentStatus::Running
            } else {
                CollabAgentStatus::Completed
            };
        }
        let pending_runtime = self.pending_v2_runtime.remove(&thread_id);
        let tool = match kind {
            SubAgentActivityKind::Started => CollabAgentTool::SpawnAgent,
            SubAgentActivityKind::Interacted => CollabAgentTool::SendInput,
            SubAgentActivityKind::Interrupted => CollabAgentTool::CloseAgent,
        };
        if let Some(row) = self
            .rows
            .iter_mut()
            .find(|candidate| candidate.thread_id == thread_id)
        {
            row.agent_path = Some(agent_path);
            row.nickname = metadata.agent_nickname;
            row.role = metadata.agent_role;
            row.status = status;
            row.last_tool = tool;
            if let Some(runtime) = pending_runtime {
                row.apply_v2_runtime(runtime);
            }
            return;
        }
        let mut row = ActiveSubagentRow {
            thread_id,
            agent_path: Some(agent_path),
            nickname: metadata.agent_nickname,
            fallback_nickname: None,
            role: metadata.agent_role,
            model: None,
            reasoning_effort: None,
            status,
            message: None,
            started_at: Instant::now(),
            last_prompt: None,
            last_tool: tool,
        };
        if let Some(runtime) = pending_runtime {
            row.apply_v2_runtime(runtime);
        }
        self.rows.push(row);
    }

    fn update_v2_runtime(
        &mut self,
        thread_id: ThreadId,
        model: String,
        reasoning_effort: Option<ReasoningEffortConfig>,
        task: Option<String>,
    ) -> bool {
        let runtime = V2RuntimeMetadata {
            model,
            reasoning_effort,
            task,
        };
        let Some(row) = self
            .rows
            .iter_mut()
            .find(|row| row.thread_id == thread_id && row.agent_path.is_some())
        else {
            self.pending_v2_runtime.insert(thread_id, runtime);
            return false;
        };
        row.apply_v2_runtime(runtime);
        true
    }

    fn set_v2_running(&mut self, thread_id: ThreadId, running: bool) -> bool {
        let Some(row) = self
            .rows
            .iter_mut()
            .find(|row| row.thread_id == thread_id && row.agent_path.is_some())
        else {
            self.pending_v2_running.insert(thread_id, running);
            return false;
        };
        if !matches!(row.status, CollabAgentStatus::Interrupted) {
            row.status = if running {
                CollabAgentStatus::Running
            } else {
                CollabAgentStatus::Completed
            };
        }
        true
    }

    pub(super) fn update_metadata(&mut self, thread_id: ThreadId, metadata: AgentMetadata) -> bool {
        let Some(row) = self
            .rows
            .iter_mut()
            .find(|candidate| candidate.thread_id == thread_id)
        else {
            return false;
        };
        row.nickname = metadata.agent_nickname;
        row.fallback_nickname = None;
        row.role = metadata.agent_role;
        true
    }

    #[cfg(test)]
    fn force_elapsed_zero_for_test(&mut self) {
        let now = Instant::now() + Duration::from_secs(3600);
        for row in &mut self.rows {
            row.started_at = now;
        }
    }
}

impl ChatWidget {
    pub(super) fn open_active_subagents_detail_popup(&mut self) -> bool {
        if self.active_subagents.is_empty() {
            return false;
        }

        let nickname_candidates = self.config.agent_nickname_candidates.as_deref();
        let active_rows = ordered_active_subagent_rows(&self.active_subagents);
        let has_v2_rows = active_rows.iter().any(|row| row.agent_path.is_some());
        let items = active_rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                active_subagent_detail_item(row, &active_rows, index, nickname_candidates)
            })
            .collect();
        let viewer_entries = active_rows
            .iter()
            .map(|row| ActiveSubagentPromptReportViewerEntry {
                display_name: row.panel_name(nickname_candidates),
                prompt: translate_viewer_field(&self.hypo_translator, row.last_prompt.as_deref()),
                report: translate_viewer_field(&self.hypo_translator, row.message.as_deref()),
            })
            .collect::<Vec<_>>();
        let selected_viewer_index = Arc::new(AtomicUsize::new(0));
        let prompt_report_viewer = ActiveSubagentPromptReportViewerRenderable::new(
            viewer_entries.clone(),
            Arc::clone(&selected_viewer_index),
        );
        let stacked_prompt_report_viewer = ActiveSubagentPromptReportViewerRenderable::new(
            viewer_entries,
            Arc::clone(&selected_viewer_index),
        );
        let on_selection_changed: crate::bottom_pane::OnSelectionChangedCallback = Some(Box::new(
            move |idx: usize, _tx: &crate::app_event_sender::AppEventSender| {
                selected_viewer_index.store(idx, Ordering::Relaxed);
            },
        ));
        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: Some(if has_v2_rows {
                "Agents".to_string()
            } else {
                "Active Subagents".to_string()
            }),
            subtitle: Some(if has_v2_rows {
                "Inspect an agent or press Enter to open its thread.".to_string()
            } else {
                "Inspect currently running or waiting subagents.".to_string()
            }),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            row_display: SelectionRowDisplay::Wrapped,
            side_content: Box::new(prompt_report_viewer),
            side_content_width: SideContentWidth::Half,
            side_content_min_width: ACTIVE_SUBAGENT_VIEWER_SIDE_MIN_WIDTH,
            stacked_side_content: Some(Box::new(stacked_prompt_report_viewer)),
            on_selection_changed,
            ..Default::default()
        });
        self.request_redraw();
        true
    }

    pub(super) fn sync_active_subagents_from_tool_call(
        &mut self,
        item: &ThreadItem,
        cached_spawn_request: Option<&multi_agents::SpawnRequestSummary>,
    ) {
        let metadata_by_thread = self.collab_agent_metadata.clone();
        let config = self.config.clone();
        let mut resolver = std::mem::take(&mut self.subagent_display_names);
        for released_thread_id in terminal_subagent_thread_ids(item) {
            resolver.release(released_thread_id);
        }
        for active_thread_id in active_subagent_thread_ids(item) {
            if let Some(metadata) = metadata_by_thread.get(&active_thread_id) {
                resolver.reserve_active_explicit_metadata(active_thread_id, metadata);
            }
        }
        let mut released_during_update = Vec::new();
        self.active_subagents.upsert_from_tool_call(
            item,
            cached_spawn_request,
            |thread_id| {
                let metadata = metadata_by_thread
                    .get(&thread_id)
                    .cloned()
                    .unwrap_or_default();
                ActiveSubagentRowFallback {
                    metadata: resolver.resolve_active_metadata(&config, thread_id, metadata),
                }
            },
            |thread_id| released_during_update.push(thread_id),
        );
        for released_thread_id in released_during_update {
            resolver.release(released_thread_id);
        }
        self.subagent_display_names = resolver;
    }

    pub(super) fn sync_v2_subagent_activity(&mut self, item: &ThreadItem) {
        let ThreadItem::SubAgentActivity {
            agent_thread_id,
            agent_path,
            kind,
            ..
        } = item
        else {
            return;
        };
        let Ok(thread_id) = ThreadId::from_string(agent_thread_id) else {
            return;
        };
        let metadata = self.resolve_collab_agent_metadata(thread_id);
        self.active_subagents.upsert_from_v2_activity(
            thread_id,
            agent_path.clone(),
            *kind,
            metadata,
        );
        self.request_redraw();
    }

    pub(crate) fn set_v2_subagent_runtime(
        &mut self,
        thread_id: ThreadId,
        model: String,
        reasoning_effort: Option<ReasoningEffortConfig>,
        task: Option<String>,
    ) {
        if self
            .active_subagents
            .update_v2_runtime(thread_id, model, reasoning_effort, task)
        {
            self.request_redraw();
        }
    }

    pub(crate) fn set_v2_subagent_running(&mut self, thread_id: ThreadId, running: bool) {
        if self.active_subagents.set_v2_running(thread_id, running) {
            self.request_redraw();
        }
    }

    #[cfg(test)]
    pub(crate) fn active_subagent_panel_lines_for_test(&self, width: u16) -> Vec<String> {
        let now = self
            .active_subagents
            .active_rows()
            .first()
            .map(|row| row.started_at)
            .unwrap_or_else(Instant::now);
        active_subagent_panel_lines(
            &self.active_subagents,
            width,
            now,
            self.config.agent_nickname_candidates.as_deref(),
        )
        .into_iter()
        .map(|line| line.to_string())
        .collect()
    }

    #[cfg(test)]
    pub(crate) fn force_active_subagent_elapsed_zero_for_test(&mut self) {
        self.active_subagents.force_elapsed_zero_for_test();
    }
}

pub(super) struct ActiveSubagentPanelRenderable<'a> {
    state: &'a ActiveSubagentPanelState,
    nickname_candidates: Option<&'a [String]>,
}

impl<'a> ActiveSubagentPanelRenderable<'a> {
    pub(super) fn new(
        state: &'a ActiveSubagentPanelState,
        nickname_candidates: Option<&'a [String]>,
    ) -> Self {
        Self {
            state,
            nickname_candidates,
        }
    }
}

impl Renderable for ActiveSubagentPanelRenderable<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || self.state.is_empty() {
            return;
        }

        let lines = active_subagent_panel_lines(
            self.state,
            area.width,
            Instant::now(),
            self.nickname_candidates,
        );
        Paragraph::new(Text::from(lines)).render(area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        if self.state.is_empty() || width == 0 {
            return 0;
        }

        active_subagent_panel_lines(self.state, width, Instant::now(), self.nickname_candidates)
            .len() as u16
    }
}

fn active_subagent_panel_lines(
    state: &ActiveSubagentPanelState,
    width: u16,
    now: Instant,
    nickname_candidates: Option<&[String]>,
) -> Vec<Line<'static>> {
    let width = usize::from(width);
    if width == 0 || state.active_rows().is_empty() {
        return Vec::new();
    }

    let rows = ordered_active_subagent_rows(state);
    let has_v2_rows = rows.iter().any(|row| row.agent_path.is_some());
    let active_count = rows
        .iter()
        .filter(|row| {
            matches!(
                row.status,
                CollabAgentStatus::PendingInit | CollabAgentStatus::Running
            )
        })
        .count();
    let idle_count = rows
        .iter()
        .filter(|row| matches!(row.status, CollabAgentStatus::Completed))
        .count();
    let mut lines = if has_v2_rows {
        vec![right_aligned_agent_line(
            vec![
                "●".green().bold(),
                " ".into(),
                "Agents".magenta().bold(),
                "  ".into(),
                format!("{active_count} active").dim(),
                if idle_count > 0 {
                    format!(" · {idle_count} idle").dim()
                } else {
                    "".dim()
                },
            ],
            vec!["Ctrl+O 详情".dim()],
            width,
        )]
    } else {
        vec![Line::from(vec![
            "Subagents active".magenta().bold(),
            " ".into(),
            format!("({})", rows.len()).dim(),
            " · ".into(),
            "Ctrl+O 详情".dim(),
        ])]
    };
    let displayed_rows = rows
        .iter()
        .copied()
        .take(ACTIVE_SUBAGENT_MAX_ROWS)
        .collect::<Vec<_>>();
    let columns = has_v2_rows
        .then(|| active_subagent_columns(&displayed_rows, &rows, now, nickname_candidates));
    for (index, row) in displayed_rows.iter().enumerate() {
        lines.extend(active_subagent_row_lines(
            row,
            &rows,
            index,
            width,
            now,
            nickname_candidates,
            columns.as_ref(),
        ));
    }
    let hidden_count = rows.len().saturating_sub(ACTIVE_SUBAGENT_MAX_ROWS);
    if hidden_count > 0 {
        lines.push(format!("  +{hidden_count} more active").dim().into());
    }
    lines
}

#[derive(Clone, Copy, Debug)]
struct ActiveSubagentColumns {
    identity: usize,
    model: usize,
    status: usize,
    elapsed: usize,
}

fn active_subagent_columns(
    displayed_rows: &[&ActiveSubagentRow],
    all_rows: &[&ActiveSubagentRow],
    now: Instant,
    nickname_candidates: Option<&[String]>,
) -> ActiveSubagentColumns {
    let mut columns = ActiveSubagentColumns {
        identity: 0,
        model: 0,
        status: 0,
        elapsed: 0,
    };
    for (index, row) in displayed_rows.iter().enumerate() {
        let identity = format!(
            "{}{} {}",
            tree_prefix(all_rows, index),
            status_glyph(&row.status),
            row.panel_name(nickname_candidates)
        );
        let model = row
            .panel_model_reasoning_label()
            .unwrap_or_else(|| "unknown".to_string());
        let status = v2_status_label(&row.status);
        let elapsed = format_goal_elapsed_seconds(
            now.saturating_duration_since(row.started_at).as_secs() as i64,
        );
        columns.identity = columns.identity.max(Line::from(identity).width());
        columns.model = columns.model.max(Line::from(model).width());
        columns.status = columns.status.max(Line::from(status).width());
        columns.elapsed = columns.elapsed.max(Line::from(elapsed).width());
    }
    columns
}

fn right_aligned_agent_line(
    left: Vec<Span<'static>>,
    right: Vec<Span<'static>>,
    width: usize,
) -> Line<'static> {
    let left_width = Line::from(left.clone()).width();
    let right_width = Line::from(right.clone()).width();
    if left_width + right_width + 2 > width {
        let mut spans = left;
        spans.push("  ".into());
        spans.extend(right);
        return crate::line_truncation::truncate_line_with_ellipsis_if_overflow(
            Line::from(spans),
            width,
        );
    }

    let mut spans = left;
    spans.push(" ".repeat(width - left_width - right_width).into());
    spans.extend(right);
    Line::from(spans)
}

fn ordered_active_subagent_rows(state: &ActiveSubagentPanelState) -> Vec<&ActiveSubagentRow> {
    let mut rows = state.active_rows().iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| match (&left.agent_path, &right.agent_path) {
        (Some(left), Some(right)) => left.cmp(right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => left.started_at.cmp(&right.started_at),
    });
    rows
}

fn active_subagent_row_lines(
    row: &ActiveSubagentRow,
    rows: &[&ActiveSubagentRow],
    index: usize,
    width: usize,
    now: Instant,
    nickname_candidates: Option<&[String]>,
    columns: Option<&ActiveSubagentColumns>,
) -> Vec<Line<'static>> {
    let tree_prefix = tree_prefix(rows, index);
    if row.agent_path.is_none() {
        return vec![
            crate::line_truncation::truncate_line_with_ellipsis_if_overflow(
                Line::from(vec![
                    tree_prefix.dim(),
                    row.status_summary(now, nickname_candidates).into(),
                ]),
                width,
            ),
        ];
    }

    let elapsed =
        format_goal_elapsed_seconds(now.saturating_duration_since(row.started_at).as_secs() as i64);
    let identity = vec![
        tree_prefix.clone().dim(),
        status_glyph(&row.status).to_string().into(),
        " ".into(),
        row.panel_name(nickname_candidates).bold(),
    ];
    let model = row
        .panel_model_reasoning_label()
        .unwrap_or_else(|| "unknown".to_string());
    let status = v2_status_label(&row.status).to_string();
    let columns = columns.copied().unwrap_or(ActiveSubagentColumns {
        identity: Line::from(identity.clone()).width(),
        model: Line::from(model.as_str()).width(),
        status: Line::from(status.as_str()).width(),
        elapsed: Line::from(elapsed.as_str()).width(),
    });
    let required_width = columns.identity + columns.model + columns.status + columns.elapsed + 9;
    let mut row_spans = identity;
    if required_width <= width {
        let identity_width = Line::from(row_spans.clone()).width();
        row_spans.push(
            " ".repeat(columns.identity.saturating_sub(identity_width))
                .into(),
        );
        row_spans.push(" · ".dim());
        row_spans.push(model.clone().dim());
        row_spans.push(
            " ".repeat(
                columns
                    .model
                    .saturating_sub(Line::from(model.as_str()).width()),
            )
            .into(),
        );
        row_spans.push(" · ".dim());
        row_spans.push(status.clone().into());
        row_spans.push(
            " ".repeat(
                columns
                    .status
                    .saturating_sub(Line::from(status.as_str()).width()),
            )
            .into(),
        );
        row_spans.push(" · ".dim());
        row_spans.push(
            " ".repeat(
                columns
                    .elapsed
                    .saturating_sub(Line::from(elapsed.as_str()).width()),
            )
            .into(),
        );
        row_spans.push(elapsed.dim());
    } else {
        row_spans.extend([
            " · ".dim(),
            model.dim(),
            " · ".dim(),
            status.into(),
            " · ".dim(),
            elapsed.dim(),
        ]);
    }
    let mut lines = vec![
        crate::line_truncation::truncate_line_with_ellipsis_if_overflow(
            Line::from(row_spans),
            width,
        ),
    ];
    if row.agent_path.is_some()
        && let Some(task) = row.task_preview()
    {
        lines.push(
            crate::line_truncation::truncate_line_with_ellipsis_if_overflow(
                Line::from(vec![
                    tree_detail_prefix(&tree_prefix).dim(),
                    task.to_string().into(),
                ]),
                width,
            ),
        );
    }
    lines
}

fn active_subagent_detail_item(
    row: &ActiveSubagentRow,
    rows: &[&ActiveSubagentRow],
    index: usize,
    nickname_candidates: Option<&[String]>,
) -> SelectionItem {
    let mut item = SelectionItem {
        name: row.panel_name(nickname_candidates),
        description: Some(row.detail_description(nickname_candidates)),
        selected_description: None,
        search_value: Some(row.search_value(nickname_candidates)),
        ..Default::default()
    };
    if row.agent_path.is_some() {
        let thread_id = row.thread_id;
        item.name_prefix_spans = vec![tree_prefix(rows, index).dim()];
        item.actions = vec![Box::new(move |tx| {
            tx.send(crate::app_event::AppEvent::SelectAgentThread(thread_id));
        })];
        item.dismiss_on_select = true;
    }
    item
}

fn tree_prefix(rows: &[&ActiveSubagentRow], index: usize) -> String {
    let Some(row) = rows.get(index) else {
        return "  ".to_string();
    };
    let segments = agent_path_segments(row);
    if segments.is_empty() {
        return "  ".to_string();
    }

    let mut prefix = String::new();
    for depth in 0..segments.len().saturating_sub(1) {
        prefix.push_str(if has_later_sibling(rows, index, depth) {
            "│  "
        } else {
            "   "
        });
    }
    prefix.push_str(if has_later_sibling(rows, index, segments.len() - 1) {
        "├─ "
    } else {
        "└─ "
    });
    prefix
}

fn tree_detail_prefix(tree_prefix: &str) -> String {
    if let Some(ancestor_prefix) = tree_prefix.strip_suffix("├─ ") {
        format!("{ancestor_prefix}│  ")
    } else if let Some(ancestor_prefix) = tree_prefix.strip_suffix("└─ ") {
        format!("{ancestor_prefix}   ")
    } else {
        "  ".to_string()
    }
}

fn has_later_sibling(rows: &[&ActiveSubagentRow], index: usize, depth: usize) -> bool {
    let current = agent_path_segments(rows[index]);
    rows.iter().skip(index + 1).any(|candidate| {
        let candidate = agent_path_segments(candidate);
        candidate.len() > depth
            && candidate[..depth] == current[..depth]
            && candidate[depth] != current[depth]
    })
}

fn agent_path_segments(row: &ActiveSubagentRow) -> Vec<&str> {
    row.relative_agent_path()
        .map(|path| {
            path.split('/')
                .filter(|segment| !segment.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Clone, Debug)]
struct ActiveSubagentPromptReportViewerEntry {
    display_name: String,
    prompt: Option<TranslatedText>,
    report: Option<TranslatedText>,
}

struct ActiveSubagentPromptReportViewerRenderable {
    entries: Vec<ActiveSubagentPromptReportViewerEntry>,
    selected_index: Arc<AtomicUsize>,
}

impl ActiveSubagentPromptReportViewerRenderable {
    fn new(
        entries: Vec<ActiveSubagentPromptReportViewerEntry>,
        selected_index: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            entries,
            selected_index,
        }
    }

    fn selected_entry(&self) -> Option<&ActiveSubagentPromptReportViewerEntry> {
        let selected_index = self.selected_index.load(Ordering::Relaxed);
        self.entries
            .get(selected_index)
            .or_else(|| self.entries.first())
    }

    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from("Subagent Prompt/Report Viewer".bold())];
        if let Some(entry) = self.selected_entry() {
            lines.push(Line::from(entry.display_name.clone().bold()));
            append_prompt_report_section(
                &mut lines,
                "Prompt",
                entry.prompt.as_ref(),
                "No prompt is available yet.",
                width,
            );
            append_prompt_report_section(
                &mut lines,
                "Report",
                entry.report.as_ref(),
                "No report/message is available yet.",
                width,
            );
        } else {
            lines.push(Line::from("No active subagent is selected.".dim()));
        }
        lines
    }
}

impl Renderable for ActiveSubagentPromptReportViewerRenderable {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        for (idx, line) in self
            .lines(area.width)
            .into_iter()
            .take(area.height as usize)
            .enumerate()
        {
            let line_area = Rect::new(area.x, area.y + idx as u16, area.width, 1);
            render_prompt_report_viewer_line(line, line_area, buf);
        }
    }

    fn desired_height(&self, width: u16) -> u16 {
        self.lines(width).len().try_into().unwrap_or(u16::MAX)
    }
}

fn translate_viewer_field(
    translator: &RuntimeTranslator,
    content: Option<&str>,
) -> Option<TranslatedText> {
    let content = content
        .map(str::trim)
        .filter(|content| !content.is_empty())?;
    Some(translator.translate(TranslationSource {
        text: content.to_string(),
        scope: TranslationScope::Subagent,
        display: TranslationDisplay::Replace,
    }))
}

fn append_prompt_report_section(
    lines: &mut Vec<Line<'static>>,
    title: &'static str,
    content: Option<&TranslatedText>,
    empty_message: &'static str,
    _width: u16,
) {
    lines.push(Line::from(vec![
        title.magenta().bold(),
        "  ".into(),
        "Original".bold(),
        " / 原文".dim(),
    ]));
    let Some(original) = content
        .map(TranslatedText::original)
        .map(str::trim)
        .filter(|content| !content.is_empty())
    else {
        lines.push(Line::from(empty_message.dim()));
        return;
    };
    crate::markdown::append_markdown_agent_with_cwd(original, None, None, lines);

    match content.and_then(TranslatedText::translated) {
        Some(translated) if !translated.trim().is_empty() => {
            lines.push(Line::from(vec![
                title.magenta().bold(),
                "  ".into(),
                "Translated".bold(),
                " / 翻译".dim(),
            ]));
            crate::markdown::append_markdown_agent_with_cwd(translated.trim(), None, None, lines);
        }
        _ => {
            lines.push(Line::from(vec![
                title.magenta().bold(),
                "  ".into(),
                "Translated".bold(),
                " / 翻译".dim(),
            ]));
            lines.push(Line::from(
                "正在生成机器翻译…（本地翻译服务不可用时保持原文）".dim(),
            ));
        }
    }
}

fn render_prompt_report_viewer_line(line: Line<'static>, area: Rect, buf: &mut Buffer) {
    if line_contains_overflow_token(&line, area.width) {
        let plain = line_plain_text(&line);
        buf[(area.x, area.y)].set_symbol(&plain);
    } else {
        line.render(area, buf);
    }
}

fn line_contains_overflow_token(line: &Line<'_>, width: u16) -> bool {
    if width == 0 {
        return false;
    }
    let width = usize::from(width);
    line_plain_text(line)
        .split_whitespace()
        .any(|token| token.chars().count() > width)
}

fn line_plain_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn collab_request_summary(item: &ThreadItem) -> Option<multi_agents::SpawnRequestSummary> {
    if let Some(summary) = multi_agents::spawn_request_summary(item) {
        return Some(summary);
    }

    match item {
        ThreadItem::CollabAgentToolCall {
            model: Some(model),
            reasoning_effort: Some(reasoning_effort),
            ..
        } => Some(multi_agents::SpawnRequestSummary {
            model: model.clone(),
            reasoning_effort: reasoning_effort.clone(),
        }),
        _ => None,
    }
}

fn is_active_agent_status(status: &CollabAgentStatus) -> bool {
    match status {
        CollabAgentStatus::PendingInit | CollabAgentStatus::Running => true,
        CollabAgentStatus::Interrupted
        | CollabAgentStatus::Completed
        | CollabAgentStatus::Errored
        | CollabAgentStatus::Shutdown
        | CollabAgentStatus::NotFound => false,
    }
}

fn is_terminal_tool_call_status(status: &CollabAgentToolCallStatus) -> bool {
    match status {
        CollabAgentToolCallStatus::InProgress => false,
        CollabAgentToolCallStatus::Completed | CollabAgentToolCallStatus::Failed => true,
    }
}

impl ActiveSubagentRow {
    fn apply_v2_runtime(&mut self, runtime: V2RuntimeMetadata) {
        self.model = (!runtime.model.trim().is_empty()).then_some(runtime.model);
        self.reasoning_effort = runtime.reasoning_effort;
        if let Some(task) = runtime.task.filter(|task| !task.trim().is_empty()) {
            self.last_prompt = Some(task);
        }
    }

    fn relative_agent_path(&self) -> Option<&str> {
        self.agent_path
            .as_deref()
            .and_then(|path| path.strip_prefix("/root"))
            .map(|path| path.trim_start_matches('/'))
            .filter(|path| !path.is_empty())
    }

    fn panel_name(&self, nickname_candidates: Option<&[String]>) -> String {
        self.relative_agent_path()
            .and_then(|path| path.rsplit('/').next())
            .map(str::to_string)
            .unwrap_or_else(|| self.display_name(nickname_candidates))
    }

    fn status_summary(&self, now: Instant, nickname_candidates: Option<&[String]>) -> String {
        let is_v2 = self.agent_path.is_some();
        let mut parts = if is_v2 {
            vec![format!(
                "{} {}",
                status_glyph(&self.status),
                self.panel_name(nickname_candidates)
            )]
        } else {
            vec![self.display_name(nickname_candidates)]
        };
        if let Some(model) = self.model_reasoning_label() {
            parts.push(model);
        }
        parts.push(if is_v2 {
            v2_status_label(&self.status).to_string()
        } else {
            status_label(&self.status).to_string()
        });
        parts.push(format_goal_elapsed_seconds(
            now.saturating_duration_since(self.started_at).as_secs() as i64,
        ));
        parts.join(" · ")
    }

    fn task_preview(&self) -> Option<&str> {
        self.last_prompt
            .as_deref()
            .map(str::trim)
            .filter(|task| !task.is_empty())
    }

    fn display_name(&self, nickname_candidates: Option<&[String]>) -> String {
        let nickname = self
            .nickname
            .as_deref()
            .map(str::trim)
            .filter(|nickname| !nickname.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| self.fallback_nickname.clone())
            .unwrap_or_else(|| {
                fallback_subagent_name(self.thread_id, nickname_candidates).to_string()
            });
        match self
            .role
            .as_deref()
            .map(str::trim)
            .filter(|role| !role.is_empty())
        {
            Some(role) => format!("{nickname} [{role}]"),
            None => nickname,
        }
    }

    fn role_label(&self) -> Option<String> {
        self.role
            .as_deref()
            .map(str::trim)
            .filter(|role| !role.is_empty())
            .map(title_case_ascii)
    }

    fn model_reasoning_label(&self) -> Option<String> {
        let model = self.model.as_deref()?.trim();
        if model.is_empty() {
            return None;
        }
        let model = display_model_label(model);
        Some(match &self.reasoning_effort {
            Some(reasoning_effort) => {
                format!("{model} {}", reasoning_label(reasoning_effort.clone()))
            }
            None => model,
        })
    }

    fn panel_model_reasoning_label(&self) -> Option<String> {
        let model = self.model.as_deref()?.trim();
        if model.is_empty() {
            return None;
        }
        let model = display_panel_model_label(model);
        Some(match &self.reasoning_effort {
            Some(reasoning_effort) => format!(
                "{model} {}",
                title_case_ascii(&reasoning_label(reasoning_effort.clone()))
            ),
            None => model,
        })
    }

    fn detail_description(&self, _nickname_candidates: Option<&[String]>) -> String {
        let mut lines = Vec::new();
        if let Some(agent_path) = &self.agent_path {
            lines.push(format!("path: {agent_path}"));
        }
        let status = if self.agent_path.is_some() {
            v2_status_label(&self.status)
        } else {
            status_label(&self.status)
        };
        lines.extend([
            format!("id: {}", self.thread_id),
            format!("status: {status}"),
        ]);
        if let Some(role) = self.role_label() {
            lines.push(format!("role: {role}"));
        }
        if let Some(model) = self.model_reasoning_label() {
            lines.push(format!("model: {model}"));
        }
        lines.push(format!("last tool: {}", tool_label(&self.last_tool)));
        if let Some(prompt) = self
            .last_prompt
            .as_deref()
            .map(str::trim)
            .filter(|prompt| !prompt.is_empty())
        {
            lines.push(format!(
                "recent input: {}",
                truncate_text(prompt, ACTIVE_SUBAGENT_PREVIEW_GRAPHEMES)
            ));
        }
        if let Some(message) = self
            .message
            .as_deref()
            .map(str::trim)
            .filter(|message| !message.is_empty())
        {
            lines.push(format!(
                "message: {}",
                truncate_text(message, ACTIVE_SUBAGENT_PREVIEW_GRAPHEMES)
            ));
        }
        lines.join("\n")
    }

    fn search_value(&self, nickname_candidates: Option<&[String]>) -> String {
        format!(
            "{} {} {} {} {}",
            self.display_name(nickname_candidates),
            self.agent_path.as_deref().unwrap_or_default(),
            self.thread_id,
            self.role.as_deref().unwrap_or_default(),
            self.model.as_deref().unwrap_or_default()
        )
    }
}

fn fallback_subagent_name_candidates(
    config: &Config,
    metadata: &AgentMetadata,
    release_channel_metadata: Option<&ReleaseChannelMetadata>,
) -> Vec<String> {
    if let Some(nickname_candidates) = metadata
        .agent_role
        .as_deref()
        .map(str::trim)
        .filter(|role| !role.is_empty())
        .and_then(|role| config.agent_roles.get(role))
        .and_then(|role| role.nickname_candidates.as_deref())
        .filter(|candidates| !candidates.is_empty())
    {
        return nickname_candidates.to_vec();
    }
    if let Some(nickname_candidates) = config
        .agent_nickname_candidates
        .as_deref()
        .filter(|candidates| !candidates.is_empty())
    {
        return nickname_candidates.to_vec();
    }
    if let Some(nickname_candidates) = release_channel_metadata
        .map(ReleaseChannelMetadata::fallback_names)
        .filter(|candidates| !candidates.is_empty())
    {
        return nickname_candidates.to_vec();
    }
    if let Some(nickname_candidates) = built_in_role_fallback_names(metadata.agent_role.as_deref())
    {
        return nickname_candidates
            .iter()
            .map(|candidate| (*candidate).to_string())
            .collect();
    }

    Vec::new()
}

fn built_in_role_fallback_names(role: Option<&str>) -> Option<&'static [&'static str]> {
    match role.map(str::trim).filter(|role| !role.is_empty()) {
        Some("supervisor") => Some(BUILT_IN_SUPERVISOR_FALLBACK_NAMES),
        Some("explorer" | "simple") => Some(BUILT_IN_EXPLORER_FALLBACK_NAMES),
        Some("worker") => Some(BUILT_IN_WORKER_FALLBACK_NAMES),
        Some("default") => Some(BUILT_IN_FALLBACK_NAMES),
        Some(_) | None => None,
    }
}

fn fallback_subagent_name(thread_id: ThreadId, nickname_candidates: Option<&[String]>) -> &str {
    let hash = fallback_subagent_name_hash(thread_id);
    if let Some(nickname_candidates) = nickname_candidates
        && !nickname_candidates.is_empty()
    {
        return &nickname_candidates[hash % nickname_candidates.len()];
    }
    BUILT_IN_FALLBACK_NAMES[hash % BUILT_IN_FALLBACK_NAMES.len()]
}

fn fallback_subagent_name_hash(thread_id: ThreadId) -> usize {
    thread_id.to_string().bytes().fold(0usize, |acc, byte| {
        acc.wrapping_mul(31).wrapping_add(byte as usize)
    })
}

fn choose_lease_candidate(
    thread_id: ThreadId,
    candidates: &[String],
    active_leases: &HashMap<ThreadId, String>,
) -> String {
    let hash = fallback_subagent_name_hash(thread_id);
    for offset in 0..candidates.len() {
        let candidate = candidates[(hash + offset) % candidates.len()].trim();
        if candidate.is_empty() {
            continue;
        }
        if !active_leases.values().any(|leased| leased == candidate) {
            return candidate.to_string();
        }
    }
    candidates[hash % candidates.len()].clone()
}

fn explicit_nickname(nickname: Option<&str>) -> Option<&str> {
    nickname
        .map(str::trim)
        .filter(|nickname| !nickname.is_empty())
}

fn terminal_subagent_thread_ids(item: &ThreadItem) -> Vec<ThreadId> {
    let ThreadItem::CollabAgentToolCall {
        tool,
        status,
        receiver_thread_ids,
        agents_states,
        ..
    } = item
    else {
        return Vec::new();
    };

    let mut terminal_thread_ids = Vec::new();
    for receiver_thread_id in receiver_thread_ids {
        let Ok(thread_id) = ThreadId::from_string(receiver_thread_id) else {
            continue;
        };
        if agents_states
            .get(receiver_thread_id)
            .is_some_and(is_terminal_agent_state)
            || matches!(tool, CollabAgentTool::CloseAgent) && is_terminal_tool_call_status(status)
        {
            terminal_thread_ids.push(thread_id);
        }
    }
    terminal_thread_ids
}

fn active_subagent_thread_ids(item: &ThreadItem) -> Vec<ThreadId> {
    let ThreadItem::CollabAgentToolCall {
        tool,
        status,
        receiver_thread_ids,
        agents_states,
        ..
    } = item
    else {
        return Vec::new();
    };

    let mut active_thread_ids = Vec::new();
    for receiver_thread_id in receiver_thread_ids {
        let Ok(thread_id) = ThreadId::from_string(receiver_thread_id) else {
            continue;
        };
        let is_pending_spawn = matches!(tool, CollabAgentTool::SpawnAgent)
            && matches!(status, CollabAgentToolCallStatus::InProgress)
            && !agents_states.contains_key(receiver_thread_id);
        let is_active_state = agents_states
            .get(receiver_thread_id)
            .map(|agent_state| is_active_agent_status(&agent_state.status))
            .unwrap_or(false);
        if is_pending_spawn || is_active_state {
            active_thread_ids.push(thread_id);
        }
    }
    active_thread_ids
}

fn is_terminal_agent_state(agent_state: &CollabAgentState) -> bool {
    !is_active_agent_status(&agent_state.status)
}

fn status_label(status: &CollabAgentStatus) -> &'static str {
    match status {
        CollabAgentStatus::PendingInit => "等待中",
        CollabAgentStatus::Running => "运行中",
        CollabAgentStatus::Interrupted => "已中断",
        CollabAgentStatus::Completed => "已完成",
        CollabAgentStatus::Errored => "错误",
        CollabAgentStatus::Shutdown => "已关闭",
        CollabAgentStatus::NotFound => "未找到",
    }
}

fn v2_status_label(status: &CollabAgentStatus) -> &'static str {
    match status {
        CollabAgentStatus::Completed => "空闲",
        status => status_label(status),
    }
}

fn status_glyph(status: &CollabAgentStatus) -> &'static str {
    match status {
        CollabAgentStatus::PendingInit => "◐",
        CollabAgentStatus::Running => "●",
        CollabAgentStatus::Completed => "○",
        CollabAgentStatus::Interrupted
        | CollabAgentStatus::Errored
        | CollabAgentStatus::Shutdown
        | CollabAgentStatus::NotFound => "×",
    }
}

fn tool_label(tool: &CollabAgentTool) -> &'static str {
    match tool {
        CollabAgentTool::SpawnAgent => "spawn",
        CollabAgentTool::SendInput => "send input",
        CollabAgentTool::ResumeAgent => "resume",
        CollabAgentTool::Wait => "wait",
        CollabAgentTool::CloseAgent => "close",
    }
}

fn reasoning_label(reasoning_effort: ReasoningEffortConfig) -> String {
    match reasoning_effort {
        ReasoningEffortConfig::None => "none".to_string(),
        ReasoningEffortConfig::Minimal => "minimal".to_string(),
        ReasoningEffortConfig::Low => "low".to_string(),
        ReasoningEffortConfig::Medium => "medium".to_string(),
        ReasoningEffortConfig::High => "high".to_string(),
        ReasoningEffortConfig::XHigh => "xhigh".to_string(),
        ReasoningEffortConfig::Max => "max".to_string(),
        ReasoningEffortConfig::Ultra => "ultra".to_string(),
        ReasoningEffortConfig::Custom(value) => value,
    }
}

fn display_model_label(model: &str) -> String {
    let Some(rest) = model.strip_prefix("gpt-") else {
        return model.to_string();
    };
    format!("GPT {}", rest.replace('-', " "))
}

fn display_panel_model_label(model: &str) -> String {
    let Some(rest) = model.strip_prefix("gpt-") else {
        return model.to_string();
    };
    let parts = rest
        .split('-')
        .map(|part| {
            if part.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
                part.to_string()
            } else {
                title_case_ascii(part)
            }
        })
        .collect::<Vec<_>>();
    format!("GPT {}", parts.join(" "))
}

fn title_case_ascii(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!(
        "{}{}",
        first.to_uppercase(),
        chars.as_str().to_ascii_lowercase()
    )
}

#[cfg(test)]
mod c7_m7_tests {
    use super::*;
    use crate::legacy_core::config::ConfigBuilder;
    use pretty_assertions::assert_eq;

    fn thread_id(suffix: &str) -> ThreadId {
        ThreadId::from_string(&format!("019cff70-2599-75e2-af72-{suffix}"))
            .expect("valid thread id")
    }

    #[test]
    fn v2_activity_runtime_and_lifecycle_populate_path_backed_row() {
        let thread_id = thread_id("b90000002000");
        let mut state = ActiveSubagentPanelState::default();

        assert!(!state.update_v2_runtime(
            thread_id,
            "gpt-5.6".to_string(),
            Some(ReasoningEffortConfig::High),
            Some("梳理 V1/V2 tool routing 和状态事件".to_string()),
        ));
        assert!(!state.set_v2_running(thread_id, false));
        state.upsert_from_v2_activity(
            thread_id,
            "/root/research".to_string(),
            SubAgentActivityKind::Started,
            AgentMetadata::default(),
        );

        let row = state.active_rows().first().expect("V2 row");
        assert_eq!(row.agent_path.as_deref(), Some("/root/research"));
        assert_eq!(row.model.as_deref(), Some("gpt-5.6"));
        assert_eq!(row.reasoning_effort, Some(ReasoningEffortConfig::High));
        assert_eq!(row.status, CollabAgentStatus::Completed);
        assert_eq!(
            row.last_prompt.as_deref(),
            Some("梳理 V1/V2 tool routing 和状态事件")
        );
    }

    #[test]
    fn v2_panel_renders_nested_tree_and_truncates_at_narrow_width() {
        let mut state = ActiveSubagentPanelState::default();
        for (suffix, path, task) in [
            ("b90000002001", "/root/ui", "设计 V2 Subagents 状态面板"),
            (
                "b90000002002",
                "/root/research/tests",
                "等待新的回归测试任务",
            ),
            (
                "b90000002003",
                "/root/research",
                "梳理 V1/V2 tool routing 和状态事件",
            ),
        ] {
            let thread_id = thread_id(suffix);
            state.upsert_from_v2_activity(
                thread_id,
                path.to_string(),
                SubAgentActivityKind::Started,
                AgentMetadata::default(),
            );
            state.update_v2_runtime(
                thread_id,
                "gpt-5.6".to_string(),
                Some(ReasoningEffortConfig::High),
                Some(task.to_string()),
            );
        }

        let rendered = active_subagent_panel_lines(&state, 100, Instant::now(), None)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(rendered.contains("● Agents"));
        assert!(rendered.contains("├─ ● research"));
        assert!(rendered.contains("│  └─ ● tests"));
        assert!(rendered.contains("└─ ● ui"));
        assert!(rendered.contains("GPT 5.6 High"));
        assert!(rendered.contains(" · 运行中 · 0s"));
        assert!(rendered.contains("│  梳理 V1/V2 tool routing 和状态事件"));

        let narrow = active_subagent_panel_lines(&state, 32, Instant::now(), None)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert!(narrow.iter().any(|line| line.ends_with('…')));
    }

    #[test]
    fn v2_panel_aligns_name_model_status_and_elapsed_columns() {
        let now = Instant::now();
        let started_at = now - Duration::from_secs(67 * 60);
        let mut state = ActiveSubagentPanelState::default();
        for (suffix, path, model, effort, status) in [
            (
                "b90000002101",
                "/root/artifact_path",
                "gpt-5.6-terra",
                ReasoningEffortConfig::Max,
                CollabAgentStatus::Running,
            ),
            (
                "b90000002102",
                "/root/audit",
                "gpt-5.6-luna",
                ReasoningEffortConfig::XHigh,
                CollabAgentStatus::Completed,
            ),
            (
                "b90000002103",
                "/root/cooling_boundary",
                "gpt-5.6-sol",
                ReasoningEffortConfig::Medium,
                CollabAgentStatus::Completed,
            ),
        ] {
            state.rows.push(ActiveSubagentRow {
                thread_id: thread_id(suffix),
                agent_path: Some(path.to_string()),
                nickname: None,
                fallback_nickname: None,
                role: None,
                model: Some(model.to_string()),
                reasoning_effort: Some(effort),
                status,
                message: None,
                started_at,
                last_prompt: None,
                last_tool: CollabAgentTool::SpawnAgent,
            });
        }

        let lines = active_subagent_panel_lines(&state, 100, now, None)
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert!(lines[0].contains("1 active · 2 idle"));
        assert!(lines[1].contains("GPT 5.6 Terra Max"));
        assert!(lines[2].contains("GPT 5.6 Luna Xhigh"));
        assert!(lines[3].contains("GPT 5.6 Sol Medium"));
        assert!(lines[1].contains("运行中"));
        assert!(lines[2].contains("空闲"));
        assert!(lines[3].contains("空闲"));

        let separators = lines[1..=3]
            .iter()
            .map(|line| {
                line.match_indices(" · ")
                    .map(|(index, _)| Line::from(&line[..index]).width())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(separators[0], separators[1]);
        assert_eq!(separators[1], separators[2]);
    }

    #[tokio::test]
    async fn c7_m7_channel_default_name_pool_priority_preserves_explicit_and_configured_names()
    -> std::io::Result<()> {
        let codex_home = tempfile::tempdir()?;
        let mut config = ConfigBuilder::default()
            .codex_home(codex_home.path().to_path_buf())
            .build()
            .await?;
        config.agent_nickname_candidates = Some(vec!["ConfigName".to_string()]);
        let channel_metadata = ReleaseChannelMetadata::new(
            ReleaseBuildSource::InternalGitLab,
            vec!["LabOne".to_string(), "LabTwo".to_string()],
            vec!["诸葛亮".to_string(), "王羲之".to_string()],
        );
        let mut resolver =
            SubagentDisplayNameResolver::default().with_release_channel_metadata(channel_metadata);

        let explicit = resolver.resolve_metadata(
            &config,
            thread_id("b90000001000"),
            AgentMetadata {
                agent_nickname: Some("ExplicitName".to_string()),
                agent_role: Some("worker".to_string()),
            },
        );
        assert_eq!(explicit.agent_nickname.as_deref(), Some("ExplicitName"));

        let configured = resolver.resolve_metadata(
            &config,
            thread_id("b90000001001"),
            AgentMetadata {
                agent_nickname: None,
                agent_role: Some("worker".to_string()),
            },
        );
        assert_eq!(
            configured.agent_nickname.as_deref(),
            Some("ConfigName"),
            "user configured nickname candidates must outrank release-channel defaults"
        );

        config.agent_nickname_candidates = None;
        let internal = resolver.resolve_metadata(
            &config,
            thread_id("b90000001002"),
            AgentMetadata {
                agent_nickname: None,
                agent_role: Some("worker".to_string()),
            },
        );
        assert!(
            matches!(
                internal.agent_nickname.as_deref(),
                Some("LabOne" | "LabTwo")
            ),
            "internal/local/GitLab builds should use VSP Lab name pool by default, got {:?}",
            internal.agent_nickname
        );

        let external = SubagentDisplayNameResolver::default()
            .with_release_channel_metadata(ReleaseChannelMetadata::new(
                ReleaseBuildSource::ExternalGitHub,
                vec!["LabOne".to_string(), "LabTwo".to_string()],
                vec!["诸葛亮".to_string(), "王羲之".to_string()],
            ))
            .resolve_metadata(
                &config,
                thread_id("b90000001003"),
                AgentMetadata {
                    agent_nickname: None,
                    agent_role: Some("worker".to_string()),
                },
            );
        assert!(
            matches!(
                external.agent_nickname.as_deref(),
                Some("诸葛亮" | "王羲之")
            ),
            "external GitHub builds should use the ancient-person name pool by default, got {:?}",
            external.agent_nickname
        );
        Ok(())
    }
}
