# C7 Cycle Summary

Status: completed

Cycle: Experience Optimization And Bugfix Sweep

## Completed Milestones

1. M1 `Resume And Scrollback Stability`
   - 修复长会话 resume 启动时重复 settling/reflow 导致的闪烁代理问题，并降低读历史时刷新跳动风险。

2. M2 `Subagent Identity, Role Policy, And Name Leases`
   - 改善 Subagent lifecycle/active panel 命名展示、fallback 名称租约、显式 nickname 优先级，并允许 role 与显式 reasoning effort 合理共存。

3. M3 `Tool And Collaboration Output Rendering Boundaries`
   - 收紧工具/MCP/Subagent lifecycle 输出的翻译边界，避免结构化工具输出被重复翻译成迷惑性的双层文本，同时保留普通 reasoning/prose 翻译。

4. M4 `Subagent Prompt And Report Viewer`
   - 增加 Subagent Prompt 与 report 的可视化/Markdown viewer 路径，支持查看翻译后 prompt 和返回报告。

5. M5 `Structured Interaction Gates And Authorization Cards`
   - 增加 Workflow Request Input helper cards，覆盖计划确认、授权意图和发布更新确认；明确授权卡片不是权限授予本身。

6. M6 `Workflow Plan/SubPlan Live Surface`
   - Workflow Cycle Plan 支持当前 SubPlan 与总 Plan 分开计数，例如 `当前 SubPlan / 当前 SubPlan 总数 | 当前 Plan / Plan 总数`；未来 Milestone 的 SubPlan 默认折叠。

7. M7 `VSP Release Channels, Naming Metadata, And Updater`
   - 增加 VSP manifest-driven update metadata 和 standalone update action；拒绝官方 OpenAI/GitHub/chatgpt 更新端点 fallback；保留非静默确认。

8. M8 `Appearance Entry Point And Final UX Polish`
   - `/theme` 成为统一 Appearance 入口，`/ui-theme` 保持 UI palette 快捷入口；syntax theme 与 UI palette 配置继续独立。

## Final Validation

Focused validation passed across the final C7 closeout:

- M8: `cargo test -p codex-tui c7_m8 -- --nocapture`，7 passed。
- Appearance regression: `theme_picker` 14 passed，`ui_theme_picker` 1 passed，`command_palette` 8 passed。
- C7 dependency regressions: `workflow_slash_routing` 8 passed，`plan_panel` 5 passed，`request_user_input` 85 passed，`c7_m7` 9 passed。
- Formatting and whitespace: `cargo fmt --check --package codex-tui` passed；`git diff --check` passed。
- M7 final release-channel validation also passed earlier with `codex-tui c7_m7` 9 passed、`codex-cli c7_m7` 1 passed、`cargo check -p codex-cli --release --bin vsp-codex`、fmt 和 diff checks。

Known non-blocking warnings:

- `core/src/session/mod.rs:1753` still emits a deprecated `turn_context.cwd` warning in broad TUI test builds.
- Stable rustfmt still warns that nightly-only `imports_granularity = Item` is ignored.

## Audit Status

- M1, M3, M4, M5, M6, M8 audit verdicts: `pass`.
- M2 audit initially returned `needs_changes`; explicit nickname lease collision and `service_tier=default` handling were fixed and rechecked as `pass`.
- M7 audit initially returned `fail`; official endpoint leakage and VSP history notice fallback were fixed and rechecked as `pass`.

## Known Follow-Ups

- 真实 TUI manual smoke 仍建议覆盖：长会话 resume 闪烁、读历史时刷新、Subagent prompt/report viewer、Appearance picker 和更新确认卡片。
- `turn_context.cwd` deprecated warning 可以后续单独清理到 selected turn environment cwd。
- M7 standalone updater 的真实包 smoke 仍需在实际发布资产上验证：`vsp-codex --version`、`vsp-codex vscode status --json`、launcher/VSCode `cliExecutable` 指向 stable `current`。
- 完整 `/hw:*` command palette 接线仍不是 C7 已交付能力；当前文档只声明 helper-level discoverability。
