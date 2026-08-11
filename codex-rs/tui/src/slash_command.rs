use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::EnumIter;
use strum_macros::EnumString;
use strum_macros::IntoStaticStr;

/// Commands that can be invoked by starting a message with a leading slash.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumIter, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    // DO NOT ALPHA-SORT! Enum order is presentation order in the popup, so
    // more frequently used commands should be listed first.
    Model,
    Provider,
    Ide,
    Permissions,
    Keymap,
    Commands,
    Vim,
    #[strum(serialize = "setup-default-sandbox")]
    ElevateSandbox,
    #[strum(serialize = "sandbox-add-read-dir")]
    SandboxReadRoot,
    Experimental,
    #[strum(to_string = "approve")]
    AutoReview,
    Memories,
    Skills,
    Import,
    Hooks,
    Review,
    Rename,
    New,
    Archive,
    Delete,
    Resume,
    Fork,
    App,
    Init,
    Compact,
    Plan,
    Goal,
    Agent,
    Side,
    Btw,
    Copy,
    Raw,
    Diff,
    Mention,
    Status,
    Usage,
    DebugConfig,
    Title,
    Statusline,
    Theme,
    #[strum(to_string = "syntax-theme")]
    SyntaxTheme,
    UiTheme,
    #[strum(to_string = "pets", serialize = "pet")]
    Pets,
    Mcp,
    Apps,
    Plugins,
    Logout,
    Quit,
    Exit,
    Feedback,
    Rollout,
    Ps,
    #[strum(to_string = "stop", serialize = "clean")]
    Stop,
    Clear,
    Personality,
    Realtime,
    Settings,
    TestApproval,
    #[strum(serialize = "subagents")]
    MultiAgents,
    // Debugging commands.
    #[strum(serialize = "debug-m-drop")]
    MemoryDrop,
    #[strum(serialize = "debug-m-update")]
    MemoryUpdate,
}

impl SlashCommand {
    /// User-visible description shown in the popup.
    pub fn description(self) -> &'static str {
        match self {
            SlashCommand::Feedback => "VSP 中已禁用官方反馈入口",
            SlashCommand::New => "在当前对话中开始新聊天",
            SlashCommand::Init => "创建包含 Codex 指令的 AGENTS.md",
            SlashCommand::Compact => "压缩对话，避免达到上下文上限",
            SlashCommand::Review => "审查当前改动并找出问题",
            SlashCommand::Rename => "重命名当前 thread",
            SlashCommand::Resume => "恢复已保存的聊天",
            SlashCommand::Archive => "归档当前 session 并退出",
            SlashCommand::Delete => "永久删除当前 session 并退出",
            SlashCommand::Clear => "清空终端并开始新聊天",
            SlashCommand::Fork => "fork 当前聊天",
            SlashCommand::App => "在 Codex Desktop 中继续当前 session",
            SlashCommand::Quit | SlashCommand::Exit => "退出 Codex",
            SlashCommand::Copy => "以 markdown 复制上一条回复",
            SlashCommand::Raw => "切换 raw scrollback mode，便于终端选择复制",
            SlashCommand::Diff => "显示 git diff（包含未跟踪文件）",
            SlashCommand::Mention => "引用文件",
            SlashCommand::Skills => "使用 skills 改善 Codex 执行特定任务的方式",
            SlashCommand::Import => "从 Claude Code 导入设置、项目和最近聊天",
            SlashCommand::Hooks => "查看和管理 lifecycle hooks",
            SlashCommand::Status => "显示当前 session 配置和 token 用量",
            SlashCommand::Usage => "显示账号 usage activity",
            SlashCommand::DebugConfig => "显示 config 层级和要求来源，便于调试",
            SlashCommand::Title => "配置 terminal title 显示的项目",
            SlashCommand::Statusline => "配置 status line 显示的项目",
            SlashCommand::Theme => "打开 Appearance 外观入口：选择 UI palette 或 Syntax 语法主题",
            SlashCommand::SyntaxTheme => "选择 Syntax 语法高亮 theme",
            SlashCommand::UiTheme => "UI palette 快捷入口：选择消息、Markdown 和输入框的 UI 主题色",
            SlashCommand::Pets => "选择或隐藏 terminal pet",
            SlashCommand::Ps => "列出后台 terminals",
            SlashCommand::Stop => "停止所有后台 terminals",
            SlashCommand::MemoryDrop => "DO NOT USE",
            SlashCommand::MemoryUpdate => "DO NOT USE",
            SlashCommand::Model => "选择使用的 model 和 reasoning effort",
            SlashCommand::Provider => "选择 provider Base URL 和凭据",
            SlashCommand::Ide => "包含 IDE 当前选择、打开文件和其他上下文",
            SlashCommand::Personality => "选择 Codex 的沟通风格",
            SlashCommand::Realtime => "切换 realtime voice mode（实验性）",
            SlashCommand::Settings => "配置 realtime 麦克风/扬声器",
            SlashCommand::Plan => "切换到 Plan mode",
            SlashCommand::Goal => "设置或查看长任务目标",
            SlashCommand::Agent | SlashCommand::MultiAgents => "切换当前 agent thread",
            SlashCommand::Side | SlashCommand::Btw => "在临时 fork 中开始侧边对话",
            SlashCommand::Permissions => "选择 Codex 可执行的操作",
            SlashCommand::Keymap => "重映射 TUI 快捷键",
            SlashCommand::Commands => "打开可搜索命令面板",
            SlashCommand::Vim => "切换 composer 的 Vim mode",
            SlashCommand::ElevateSandbox => "设置 elevated agent sandbox",
            SlashCommand::SandboxReadRoot => {
                "允许 sandbox 读取目录：/sandbox-add-read-dir <absolute_path>"
            }
            SlashCommand::Experimental => "切换实验性功能",
            SlashCommand::AutoReview => "批准最近 auto-review 拒绝后的一次重试",
            SlashCommand::Memories => "配置 memory 使用和生成",
            SlashCommand::Mcp => "列出已配置的 MCP tools；使用 /mcp verbose 查看详情",
            SlashCommand::Apps => "管理 apps",
            SlashCommand::Plugins => "浏览 plugins",
            SlashCommand::Logout => "退出 Codex 登录",
            SlashCommand::Rollout => "打印 rollout 文件路径",
            SlashCommand::TestApproval => "测试 approval request",
        }
    }

    /// Command string without the leading '/'. Provided for compatibility with
    /// existing code that expects a method named `command()`.
    pub fn command(self) -> &'static str {
        self.into()
    }

    /// Whether this command supports inline args (for example `/review ...`).
    pub fn supports_inline_args(self) -> bool {
        matches!(
            self,
            SlashCommand::Review
                | SlashCommand::Rename
                | SlashCommand::Plan
                | SlashCommand::Goal
                | SlashCommand::Ide
                | SlashCommand::Keymap
                | SlashCommand::Mcp
                | SlashCommand::Raw
                | SlashCommand::Usage
                | SlashCommand::Pets
                | SlashCommand::Side
                | SlashCommand::Btw
                | SlashCommand::Resume
                | SlashCommand::SandboxReadRoot
        )
    }

    /// Whether this command remains available inside an active side conversation.
    pub fn available_in_side_conversation(self) -> bool {
        matches!(
            self,
            SlashCommand::Copy
                | SlashCommand::Raw
                | SlashCommand::Diff
                | SlashCommand::Mention
                | SlashCommand::Status
                | SlashCommand::Usage
                | SlashCommand::Ide
        )
    }

    /// Whether this command can be run while a task is in progress.
    pub fn available_during_task(self) -> bool {
        match self {
            SlashCommand::New
            | SlashCommand::Archive
            | SlashCommand::Delete
            | SlashCommand::Fork
            | SlashCommand::Init
            | SlashCommand::Compact
            | SlashCommand::Provider
            | SlashCommand::Realtime
            | SlashCommand::Settings
            | SlashCommand::Keymap
            | SlashCommand::Commands
            | SlashCommand::Vim
            | SlashCommand::ElevateSandbox
            | SlashCommand::SandboxReadRoot
            | SlashCommand::Experimental
            | SlashCommand::Memories
            | SlashCommand::Import
            | SlashCommand::Review
            | SlashCommand::Plan
            | SlashCommand::Clear
            | SlashCommand::Logout
            | SlashCommand::SyntaxTheme
            | SlashCommand::MemoryDrop
            | SlashCommand::MemoryUpdate => false,
            SlashCommand::Diff
            | SlashCommand::Resume
            | SlashCommand::Model
            | SlashCommand::Personality
            | SlashCommand::Permissions
            | SlashCommand::Copy
            | SlashCommand::Raw
            | SlashCommand::Rename
            | SlashCommand::Mention
            | SlashCommand::Skills
            | SlashCommand::Hooks
            | SlashCommand::Status
            | SlashCommand::Usage
            | SlashCommand::DebugConfig
            | SlashCommand::Ps
            | SlashCommand::Stop
            | SlashCommand::App
            | SlashCommand::Goal
            | SlashCommand::Mcp
            | SlashCommand::Apps
            | SlashCommand::Plugins
            | SlashCommand::Title
            | SlashCommand::Statusline
            | SlashCommand::AutoReview
            | SlashCommand::Feedback
            | SlashCommand::Ide
            | SlashCommand::Quit
            | SlashCommand::Exit
            | SlashCommand::Side
            | SlashCommand::Btw
            | SlashCommand::UiTheme => true,
            SlashCommand::Rollout => true,
            SlashCommand::TestApproval => true,
            SlashCommand::Agent | SlashCommand::MultiAgents => true,
            SlashCommand::Theme | SlashCommand::Pets => false,
        }
    }

    fn is_visible(self) -> bool {
        match self {
            SlashCommand::Feedback
            | SlashCommand::Logout
            | SlashCommand::SyntaxTheme
            | SlashCommand::UiTheme => false,
            SlashCommand::SandboxReadRoot => cfg!(target_os = "windows"),
            SlashCommand::Copy => !cfg!(target_os = "android"),
            SlashCommand::App => cfg!(any(target_os = "macos", target_os = "windows")),
            SlashCommand::Rollout | SlashCommand::TestApproval => cfg!(debug_assertions),
            _ => true,
        }
    }
}

/// Return all built-in commands in a Vec paired with their command string.
pub fn built_in_slash_commands() -> Vec<(&'static str, SlashCommand)> {
    SlashCommand::iter()
        .filter(|command| command.is_visible())
        .map(|c| (c.command(), c))
        .collect()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    use super::SlashCommand;
    use super::built_in_slash_commands;

    #[test]
    fn stop_command_is_canonical_name() {
        assert_eq!(SlashCommand::Stop.command(), "stop");
    }

    #[test]
    fn clean_alias_parses_to_stop_command() {
        assert_eq!(SlashCommand::from_str("clean"), Ok(SlashCommand::Stop));
    }

    #[test]
    fn pet_alias_parses_to_pets_command() {
        assert_eq!(SlashCommand::Pets.command(), "pets");
        assert_eq!(SlashCommand::from_str("pet"), Ok(SlashCommand::Pets));
    }

    #[test]
    fn certain_commands_are_available_during_task() {
        assert!(SlashCommand::Model.available_during_task());
        assert!(SlashCommand::Personality.available_during_task());
        assert!(SlashCommand::Permissions.available_during_task());
        assert!(SlashCommand::Goal.available_during_task());
        assert!(SlashCommand::Ide.available_during_task());
        assert!(SlashCommand::Title.available_during_task());
        assert!(SlashCommand::Statusline.available_during_task());
        assert!(SlashCommand::Raw.available_during_task());
        assert!(SlashCommand::Raw.available_in_side_conversation());
        assert!(SlashCommand::Raw.supports_inline_args());
        assert!(SlashCommand::App.available_during_task());
    }

    #[test]
    fn official_service_commands_are_hidden() {
        let commands = built_in_slash_commands()
            .into_iter()
            .map(|(name, _)| name)
            .collect::<Vec<_>>();

        assert!(!commands.contains(&"feedback"));
        assert!(!commands.contains(&"logout"));
    }

    #[test]
    fn c7_m8_internal_syntax_theme_command_is_hidden() {
        let contains_syntax_theme = built_in_slash_commands()
            .into_iter()
            .map(|(name, _)| name)
            .any(|name| name == "syntax-theme");

        assert!(
            !contains_syntax_theme,
            "internal Appearance routing command must not be listed as a built-in command"
        );
    }

    #[test]
    fn c7_m8_ui_theme_shortcut_is_hidden_from_default_discovery() {
        let contains_ui_theme = built_in_slash_commands()
            .into_iter()
            .map(|(name, _)| name)
            .any(|name| name == "ui-theme");

        assert!(
            !contains_ui_theme,
            "/ui-theme remains dispatchable as a compatibility shortcut but should not compete with the unified /theme entry"
        );
    }

    #[test]
    fn disabled_official_service_descriptions_do_not_advertise_upload() {
        assert_eq!(
            SlashCommand::Feedback.description(),
            "VSP 中已禁用官方反馈入口"
        );
    }

    #[test]
    fn auto_review_command_is_approve() {
        assert_eq!(SlashCommand::AutoReview.command(), "approve");
        assert_eq!(
            SlashCommand::from_str("approve"),
            Ok(SlashCommand::AutoReview)
        );
    }

    #[test]
    fn core_command_descriptions_are_localized_to_chinese() {
        assert_eq!(
            [
                (
                    SlashCommand::Provider.command(),
                    SlashCommand::Provider.description()
                ),
                (SlashCommand::Mcp.command(), SlashCommand::Mcp.description()),
                (
                    SlashCommand::Plan.command(),
                    SlashCommand::Plan.description()
                ),
                (
                    SlashCommand::Agent.command(),
                    SlashCommand::Agent.description()
                ),
                (SlashCommand::Raw.command(), SlashCommand::Raw.description()),
                (
                    SlashCommand::Permissions.command(),
                    SlashCommand::Permissions.description(),
                ),
            ],
            [
                ("provider", "选择 provider Base URL 和凭据"),
                ("mcp", "列出已配置的 MCP tools；使用 /mcp verbose 查看详情"),
                ("plan", "切换到 Plan mode"),
                ("agent", "切换当前 agent thread"),
                ("raw", "切换 raw scrollback mode，便于终端选择复制"),
                ("permissions", "选择 Codex 可执行的操作"),
            ]
        );
    }

    #[test]
    fn c7_m8_theme_command_describes_unified_appearance_entry() {
        let description = SlashCommand::Theme.description();

        assert!(
            description.contains("Appearance") || description.contains("外观"),
            "/theme should be presented as the unified Appearance entry, got: {description}"
        );
        assert!(
            description.contains("UI") || description.contains("palette"),
            "/theme should mention the UI palette surface, got: {description}"
        );
        assert!(
            description.contains("Syntax") || description.contains("语法"),
            "/theme should still mention syntax theme access, got: {description}"
        );
    }

    #[test]
    fn c7_m8_ui_theme_command_remains_palette_shortcut() {
        let description = SlashCommand::UiTheme.description();

        assert!(
            description.contains("UI"),
            "/ui-theme should remain explicitly scoped to the UI palette, got: {description}"
        );
        assert!(
            description.contains("快捷") || description.contains("shortcut"),
            "/ui-theme should be described as a shortcut entry, got: {description}"
        );
        assert!(
            !description.contains("语法高亮") && !description.contains("Syntax"),
            "/ui-theme must not be described as a syntax theme selector, got: {description}"
        );
    }
}
