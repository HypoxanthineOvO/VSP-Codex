# M2 Command Palette + Focus Stack + Copy Selection

## Objective

补齐 command palette、focus stack、键盘复制/选择系统，改善 VSCode 终端中的键盘体验，并确保必要操作不依赖冲突快捷键。

## Source Of Truth

- C4 顺延项：完整 command palette/focus stack/键盘复制选择系统。
- 用户痛点：VSCode 终端中 `Ctrl+J/K` 等冲突让操作不可用，其他键位不是主要问题。
- C4 M4 已修默认必要操作和提示，C5 要扩展为完整可发现的命令入口和复制/选择体验。

## Subworker Assignment Plan

项目配置为 strict worker separation，但当前 Codex 子代理工具仅允许在用户明确要求 sub-agent/parallel agent 时使用。除非用户补充授权，本 Milestone 由主代理执行并在报告中记录降级证据。

## Technical Solution

复用现有 `RuntimeKeymap`、footer shortcut overlay、bottom pane 和 list/ranking 组件。新增可搜索 command palette 作为功能入口；建立轻量 focus stack 管理 bottom pane/main list/composer 的 Tab/Shift+Tab 流转；为消息/代码块提供键盘复制或选择命令，避免使用 `Ctrl+C` 作为唯一复制路径。

## Technical Route

1. 梳理当前 slash command、keymap action、bottom pane focus 和 copy clipboard helper。
2. 新增 command palette UI：搜索命令、方向键移动、Enter 执行、Esc 关闭。
3. 接入常用命令：rename、theme picker、resume、copy last answer、copy code block、toggle thinking、open shortcuts。
4. 建立 focus stack：composer、message/history area、bottom pane/list/ranking 可预测切换；Tab/Shift+Tab 是默认路径。
5. 实现复制/选择系统：至少支持复制最后一条 assistant、当前选中消息、当前/最近 code block；没有 clipboard 时给出可见错误。
6. 更新 keymap hints 和 tests，保证必要入口不依赖 `Ctrl+J/K`、`Ctrl+P/N`、`Alt` chords、`Ctrl+C`。

## Validation Path

- `just test -p codex-tui command_palette`
- `just test -p codex-tui keymap`
- `just test -p codex-tui footer`
- `just test -p codex-tui chat_composer`
- `just test -p codex-tui history_cell`
- `cargo insta pending-snapshots -p codex-tui`

## Audit Focus

- command palette 是真实可执行命令，不只是帮助文本。
- focus stack 不破坏 Enter/Esc 主路径。
- 复制功能不依赖 `Ctrl+C`，失败路径明确。
- VSCode-terminal-safe 默认约束继续成立。
