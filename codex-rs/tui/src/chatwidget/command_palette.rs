use super::*;
use codex_protocol::config_types::CollaborationModeMask;

use crate::bottom_pane::SelectionRowDisplay;
use crate::bottom_pane::SideContentWidth;
use crate::bottom_pane::slash_commands::BuiltinCommandFlags;
use crate::bottom_pane::slash_commands::ServiceTierCommand;
use crate::bottom_pane::slash_commands::SlashCommandItem;
use crate::bottom_pane::slash_commands::commands_for_input;
use crate::workflow_slash_routing::WorkflowCommandPaletteEntry;
use crate::workflow_slash_routing::workflow_command_palette_entries_for_codex_home;

const COMMAND_PALETTE_VIEW_ID: &str = "command-palette";

impl ChatWidget {
    pub(crate) fn open_command_palette(&mut self) {
        let service_tier_commands = self.current_model_service_tier_commands();
        let workflow_entries = workflow_command_palette_entries_for_codex_home(
            self.config.cwd.join(".pipeline").is_dir(),
            self.config.codex_home.as_path(),
        );
        let items = command_palette_items(
            self.builtin_command_flags(),
            &service_tier_commands,
            workflow_entries,
            crate::collaboration_modes::default_mask(self.model_catalog.as_ref()),
        );
        self.bottom_pane.show_selection_view(SelectionViewParams {
            view_id: Some(COMMAND_PALETTE_VIEW_ID),
            title: Some("Command Palette".to_string()),
            subtitle: Some("Search commands and press Enter to run.".to_string()),
            items,
            is_searchable: true,
            search_placeholder: Some("Search commands".to_string()),
            col_width_mode: ColumnWidthMode::AutoAllRows,
            row_display: SelectionRowDisplay::SingleLine,
            side_content_width: SideContentWidth::Fixed(0),
            ..Default::default()
        });
        self.request_redraw();
    }
}

fn command_palette_items(
    flags: BuiltinCommandFlags,
    service_tier_commands: &[ServiceTierCommand],
    workflow_entries: Vec<WorkflowCommandPaletteEntry>,
    default_collaboration_mode: Option<CollaborationModeMask>,
) -> Vec<SelectionItem> {
    let mut items = commands_for_input(flags, service_tier_commands)
        .into_iter()
        .filter_map(|item| match item {
            SlashCommandItem::Builtin(cmd) => command_palette_builtin_item(cmd),
            SlashCommandItem::ServiceTier(command) => {
                let command_for_action = command.clone();
                Some(SelectionItem {
                    name: format!("/{}", command.name),
                    description: Some(command.description.clone()),
                    search_value: Some(format!("{} {}", command.name, command.description)),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::RunCommandPaletteServiceTier(
                            command_for_action.clone(),
                        ));
                    })],
                    dismiss_on_select: true,
                    ..Default::default()
                })
            }
        })
        .collect::<Vec<_>>();

    items.extend(
        workflow_entries
            .into_iter()
            .map(|entry| command_palette_workflow_item(entry, default_collaboration_mode.clone())),
    );

    items.push(SelectionItem {
        name: "Copy latest code block".to_string(),
        description: Some(
            "Copy the most recent fenced code block from the latest assistant response."
                .to_string(),
        ),
        search_value: Some("copy code block latest fenced assistant response".to_string()),
        actions: vec![Box::new(|tx| {
            tx.send(AppEvent::CopyLastCodeBlock);
        })],
        dismiss_on_select: true,
        ..Default::default()
    });

    items
}

fn command_palette_workflow_item(
    entry: WorkflowCommandPaletteEntry,
    default_collaboration_mode: Option<CollaborationModeMask>,
) -> SelectionItem {
    let command = entry.command.to_string();
    let description = entry.description;
    let (actions, disabled_reason) = match default_collaboration_mode {
        Some(mask) => {
            let command_for_action = command.clone();
            (
                vec![Box::new(move |tx: &AppEventSender| {
                    tx.send(AppEvent::SubmitUserMessageWithMode {
                        text: command_for_action.clone(),
                        collaboration_mode: mask.clone(),
                    });
                }) as SelectionAction],
                None,
            )
        }
        None => (Vec::new(), Some("Default mode unavailable".to_string())),
    };

    SelectionItem {
        name: command.clone(),
        description: Some(description.clone()),
        search_value: Some(format!("{command} {description}")),
        actions,
        dismiss_on_select: true,
        disabled_reason,
        ..Default::default()
    }
}

fn command_palette_builtin_item(cmd: SlashCommand) -> Option<SelectionItem> {
    if matches!(
        cmd,
        SlashCommand::Quit
            | SlashCommand::Exit
            | SlashCommand::Clear
            | SlashCommand::MemoryDrop
            | SlashCommand::MemoryUpdate
            | SlashCommand::TestApproval
            | SlashCommand::Rollout
    ) {
        return None;
    }

    let description = command_palette_description(cmd);
    let command_name = cmd.command();
    Some(SelectionItem {
        name: format!("/{command_name}"),
        description: Some(description.to_string()),
        search_value: Some(format!("/{command_name} {command_name} {description}")),
        actions: vec![Box::new(move |tx| {
            tx.send(AppEvent::RunCommandPaletteCommand(cmd));
        })],
        dismiss_on_select: true,
        ..Default::default()
    })
}

fn command_palette_description(cmd: SlashCommand) -> &'static str {
    match cmd {
        SlashCommand::Copy => "Copy the last assistant response.",
        SlashCommand::Raw => "Toggle copy-friendly raw scrollback mode.",
        SlashCommand::UiTheme => {
            "Shortcut to choose the UI palette for messages, Markdown, and composer."
        }
        SlashCommand::Keymap => "Remap keyboard shortcuts.",
        SlashCommand::Rename => "Rename the current thread.",
        SlashCommand::Resume => "Open saved chats.",
        SlashCommand::Status => "Show session configuration and token usage.",
        SlashCommand::Diff => "Show the current git diff.",
        SlashCommand::Commands => "Open this command palette.",
        SlashCommand::Ide => "Add IDE context to the composer.",
        SlashCommand::Model => "Choose model and reasoning.",
        SlashCommand::Provider => "Choose provider and credentials.",
        SlashCommand::Permissions => "Choose what Codex can do.",
        SlashCommand::Plan => "Switch to Plan mode.",
        SlashCommand::Goal => "Set or view the long-running task goal.",
        SlashCommand::Review => "Review current changes.",
        SlashCommand::Mention => "Insert a file mention.",
        SlashCommand::Skills => "Browse skills.",
        SlashCommand::Hooks => "View and manage lifecycle hooks.",
        SlashCommand::Mcp => "List configured MCP tools.",
        SlashCommand::Apps => "Manage apps.",
        SlashCommand::Plugins => "Browse plugins.",
        SlashCommand::Agent | SlashCommand::MultiAgents => "Switch agent threads.",
        SlashCommand::Side | SlashCommand::Btw => "Start a side conversation.",
        SlashCommand::Vim => "Toggle Vim composer mode.",
        SlashCommand::Theme => "Open Appearance to choose UI palette or syntax theme.",
        SlashCommand::SyntaxTheme => "Choose syntax highlighting theme.",
        SlashCommand::Pets => "Choose or hide the terminal pet.",
        SlashCommand::Title => "Configure terminal title content.",
        SlashCommand::Statusline => "Configure status line content.",
        SlashCommand::DebugConfig => "Show config layers and requirement sources.",
        SlashCommand::Ps => "List background terminals.",
        SlashCommand::Stop => "Stop all background terminals.",
        SlashCommand::Experimental => "Toggle experimental features.",
        SlashCommand::AutoReview => "Approve one auto-review retry.",
        SlashCommand::Memories => "Configure memory use and generation.",
        SlashCommand::Personality => "Choose Codex communication style.",
        SlashCommand::Realtime => "Toggle realtime voice mode.",
        SlashCommand::Settings => "Configure realtime audio devices.",
        SlashCommand::New => "Start a new chat in this terminal.",
        SlashCommand::Fork => "Fork the current chat.",
        SlashCommand::Init => "Create AGENTS.md instructions here.",
        SlashCommand::Compact => "Compact this conversation.",
        SlashCommand::ElevateSandbox => "Set elevated agent sandbox.",
        SlashCommand::SandboxReadRoot => "Allow sandbox reads from a directory.",
        SlashCommand::Feedback
        | SlashCommand::Import
        | SlashCommand::Archive
        | SlashCommand::Delete
        | SlashCommand::App
        | SlashCommand::Usage
        | SlashCommand::Logout
        | SlashCommand::Quit
        | SlashCommand::Exit
        | SlashCommand::Clear
        | SlashCommand::Rollout
        | SlashCommand::TestApproval
        | SlashCommand::MemoryDrop
        | SlashCommand::MemoryUpdate => cmd.description(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_collaboration_mode() -> CollaborationModeMask {
        CollaborationModeMask {
            name: "Default".to_string(),
            mode: Some(ModeKind::Default),
            model: None,
            reasoning_effort: None,
            developer_instructions: None,
        }
    }

    fn test_command_palette_items() -> Vec<SelectionItem> {
        command_palette_items(flags(), &[], Vec::new(), Some(default_collaboration_mode()))
    }

    fn flags() -> BuiltinCommandFlags {
        BuiltinCommandFlags {
            collaboration_modes_enabled: true,
            connectors_enabled: true,
            plugins_command_enabled: true,
            service_tier_commands_enabled: true,
            goal_command_enabled: true,
            personality_command_enabled: true,
            token_activity_command_enabled: true,
            allow_elevate_sandbox: true,
            side_conversation_active: false,
        }
    }

    #[test]
    fn command_palette_exposes_copy_theme_rename_and_resume() {
        let items = test_command_palette_items();
        let names = items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();

        for expected in ["/copy", "/theme", "/rename", "/resume", "/keymap"] {
            assert!(
                names.contains(&expected),
                "expected command palette to expose {expected}; got {names:?}"
            );
        }
        assert!(
            names.contains(&"Copy latest code block"),
            "expected command palette to expose copy-code-block action; got {names:?}"
        );
    }

    #[test]
    fn command_palette_filters_dangerous_exit_and_clear_actions() {
        let items = test_command_palette_items();
        let names = items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();

        for hidden in ["/quit", "/exit", "/clear"] {
            assert!(
                !names.contains(&hidden),
                "expected command palette to hide {hidden}; got {names:?}"
            );
        }
    }

    #[test]
    fn c7_m8_command_palette_hides_internal_syntax_theme_route() {
        let items = test_command_palette_items();
        let names = items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();

        assert!(
            !names.contains(&"/syntax-theme"),
            "internal Appearance routing command must not be user-visible in the command palette; got {names:?}"
        );
    }

    #[test]
    fn c7_m8_command_palette_hides_ui_theme_shortcut() {
        let items = test_command_palette_items();
        let names = items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();

        assert!(
            !names.contains(&"/ui-theme"),
            "/ui-theme remains dispatchable as a compatibility shortcut but should not be user-visible in the command palette; got {names:?}"
        );
    }

    #[test]
    fn command_palette_includes_workflow_entries() {
        let workflow_entries = [
            "guide",
            "init",
            "goal",
            "plan",
            "cycle",
            "maintain",
            "experiment",
            "resume",
            "accept",
            "reject",
        ]
        .into_iter()
        .map(
            |command| crate::workflow_slash_routing::WorkflowCommandPaletteEntry {
                command: format!("/hw:{command}"),
                description: format!("Hypo-Workflow {command}"),
            },
        )
        .collect();
        let items = command_palette_items(
            flags(),
            &[],
            workflow_entries,
            Some(default_collaboration_mode()),
        );
        let names = items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();

        for expected in [
            "/hw:guide",
            "/hw:init",
            "/hw:goal",
            "/hw:plan",
            "/hw:cycle",
            "/hw:maintain",
            "/hw:experiment",
            "/hw:resume",
            "/hw:accept",
            "/hw:reject",
        ] {
            assert!(
                names.contains(&expected),
                "missing {expected}; got {names:?}"
            );
        }
        let workflow_names = names
            .iter()
            .filter(|name| name.starts_with("/hw:"))
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!("workflow_command_palette_entries", workflow_names);
        assert!(
            !names.contains(&"/hw:status"),
            "retired /hw:status leaked: {names:?}"
        );
    }
}
