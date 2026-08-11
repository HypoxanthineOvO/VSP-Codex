# C7/M8 Appearance Entry Point And Final UX Polish Report

## 结果

M8 已完成并通过只读审核。`/theme` 现在是统一的 Appearance 入口，用户可从同一个选择器进入 `UI Palette` 或 `Syntax Theme`；`/ui-theme` 仍保持为 UI palette 的直达快捷入口。`[tui].theme` 与 `[tui].ui_theme` 继续独立持久化，没有合并配置语义。

## 已完成内容

- `codex-rs/tui/src/theme_picker.rs`
  - 新增 `build_appearance_picker_params(...)`，提供两项入口：`UI Palette` 与 `Syntax Theme`。
  - Appearance 入口复用既有 focused pickers，避免复制语法主题或 UI palette 的持久化逻辑。
  - 保留既有 syntax theme 窄终端 fallback 覆盖。
- `codex-rs/tui/src/chatwidget/settings_popups.rs`
  - `/theme` 改为打开统一 Appearance picker。
  - 新增 `open_syntax_theme_picker()` 供 Appearance 内部路由复用。
  - `/ui-theme` 继续直接打开 UI palette picker。
- `codex-rs/tui/src/slash_command.rs`
  - 新增隐藏内部命令 `syntax-theme`，仅用于 Appearance picker 跳转。
  - `/theme` 描述改为 Appearance/Theme 入口，明确包含 UI palette 与 syntax theme。
  - `/ui-theme` 描述改为 UI palette 快捷入口。
  - 增加回归测试，确保内部 `syntax-theme` 不出现在 built-in slash commands 中。
- `codex-rs/tui/src/chatwidget/command_palette.rs`
  - 更新 `/theme` 与 `/ui-theme` 命令面板描述。
  - 增加回归测试，确保 `/syntax-theme` 不出现在 command palette 中。
- `codex-rs/tui/src/app/config_persistence.rs`
  - 增加回归测试，锁定 UI palette 与 syntax theme 选择互不覆盖。
- `docs/slash_commands.md`、`docs/vsp-codex.md`、`docs/install.md`、`CHANGELOG.md`
  - 记录 `/theme` 统一 Appearance 入口、`/ui-theme` 快捷入口，以及 `[tui].theme` / `[tui].ui_theme` 的独立性。
  - 保持 Hypo-Workflow helper 文档的限制说明，没有过度承诺完整 command palette 集成或 silent auto-update。

## 验证

- PASS: `cargo test -p codex-tui c7_m8 -- --nocapture`，7 passed。
- PASS: `cargo test -p codex-tui theme_picker -- --nocapture`，14 passed。
- PASS: `cargo test -p codex-tui ui_theme_picker -- --nocapture`，1 passed。
- PASS: `cargo test -p codex-tui command_palette -- --nocapture`，8 passed。
- PASS: `cargo test -p codex-tui workflow_slash_routing -- --nocapture`，8 passed。
- PASS: `cargo test -p codex-tui plan_panel -- --nocapture`，5 passed。
- PASS: `cargo test -p codex-tui request_user_input -- --nocapture`，85 passed。
- PASS: `cargo test -p codex-tui c7_m7 -- --nocapture`，9 passed。
- PASS: `cargo fmt --check --package codex-tui`，仅仓库既有 stable rustfmt 对 nightly-only `imports_granularity` 的 warning。
- PASS: `git diff --check`。

测试运行中仍会出现既有 `core/src/session/mod.rs:1753` deprecated `turn_context.cwd` warning；该 warning 不是 M8 新增问题。

## 审计结论

`.pipeline/reviews/C7/M8/audit.md` verdict 为 `pass`，无阻塞发现。审计确认：

- `/theme` 的 Appearance picker 提供 UI palette 与 syntax theme 两个动作。
- `/ui-theme` 仍直达 UI palette picker。
- `[tui].theme` 和 `[tui].ui_theme` 仍分别通过原持久化路径更新。
- 内部 `syntax-theme` 路由不会出现在 slash popup 或 command palette 中。
- Appearance picker 是简单两项选择器，新增窄终端布局风险低。
- 文档没有过度承诺 silent auto-update 或完整 Hypo-Workflow command palette 集成。

## 剩余风险

- 未执行真实交互 TUI 截图或手动终端 smoke。风险较低，因为新增 Appearance 入口为两项 selection view，复杂窄终端布局仍由既有 syntax theme picker 自动化测试覆盖。
- `.pipeline/reviews/C7/M8/implement.md` 中记录的早期 `c7_m8` 结果为 5 passed；最终主代理与 audit 复跑结果均为 7 passed。
