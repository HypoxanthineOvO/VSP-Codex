# M2 Command Palette + Focus Stack + Copy Selection

完成时间：2026-05-28T21:36:43+08:00

## 结果

M2 已完成本 Cycle 可交付的键盘体验优化：新增真实可执行的 command palette，提供 VSCode terminal-safe 入口，并把复制能力扩展到最近代码块。

## 具体产出

- 新增 `/commands` slash command：在 TUI 中打开可搜索命令面板，不只是帮助文本。
- 新增默认快捷键 `Ctrl+Space`：可打开 command palette；同时保留 `/commands` 作为终端吞键时的 typed fallback。
- 新增 `tui.keymap.global.command_palette` 配置：进入 RuntimeKeymap、keymap action catalog、`/keymap` picker 和 config schema，可被用户重映射或显式解绑。
- command palette 复用现有 `ListSelectionView`：支持搜索、方向键、Enter 执行、Esc 关闭，选中后发 AppEvent 并调用既有 slash command dispatch。
- command palette 暴露常用安全动作：`/copy`、`/ui-theme`、`/rename`、`/resume`、`/keymap`、模型/权限/状态等；隐藏 `/quit`、`/exit`、`/clear`、debug memory、rollout/test approval 等危险或调试动作。
- 新增 `Copy latest code block` palette 动作：从最近 assistant markdown 中提取最后一个完整 fenced code block，并复用现有 clipboard backend、lease 保存和错误提示路径。
- 保留并强化现有 `/copy` 路径：`/copy`、`Ctrl+O`、command palette 选择 `/copy` 都进入同一套复制最近 assistant markdown 的实现。
- 更新 slash popup 行为预期：新增 `/commands` 后 `/co<Tab>` 现在补全为 `/commands`，测试已调整为新排序。

## Focus Stack 处理

本轮没有重写全局 focus stack。原因是当前 bottom pane 已有 view stack：弹窗、列表、审批、选择器会接管输入并用 Enter/Esc 完成或关闭。M2 的用户痛点主要是 VSCode 终端中的 `Ctrl+J/K` 冲突和必要动作不可发现，因此本轮采用低风险方案：

- 必要动作入口改为 `Ctrl+Space` + `/commands`，不依赖 `Ctrl+J/K`、`Ctrl+P/N`、Alt chords 或 `Ctrl+C`。
- command palette 使用现有列表焦点模型，避免重写 composer/history/bottom pane 焦点路由引入回归。
- 复制当前选中消息这类需要完整 transcript selection model 的能力未在本轮落地；已由 `/copy` 和 `Copy latest code block` 覆盖最高频复制场景。

## 验证

- `just write-config-schema`：通过。
- `just fmt`：通过。
- `just fix -p codex-config`：通过。
- `just fix -p codex-tui`：通过。
- `just test -p codex-tui command_palette`：7/7 通过。
- `just test -p codex-tui keymap`：109/109 通过。
- `just test -p codex-tui slash_commands`：111/111 通过。
- `just test -p codex-tui chat_composer`：187/187 通过。
- `cargo insta pending-snapshots --manifest-path codex-rs/tui/Cargo.toml`：无 pending snapshots。
- `git diff --check`：通过。

## Worker Separation 说明

项目配置为 strict worker separation，且 `/hw:start`、`/hw:resume` 已授权 Codex execution subworkers；但当前可用子代理工具要求用户明确请求 sub-agent/parallel agent work 才能 spawn。本里程碑由主代理执行，保留完整命令级验证证据。
