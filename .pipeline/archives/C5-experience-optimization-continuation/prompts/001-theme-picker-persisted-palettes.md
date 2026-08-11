# M1 Theme Picker + Persisted Palettes

## Objective

实现用户可选择的一小组主题色，并把选择持久化；Markdown、消息块边框/accent 和输入框明显化使用同一套主题派生色。

## Source Of Truth

- C4 顺延项：完整 Color Theme picker、三组主题持久化和 fallback matrix。
- 用户要求：主题色一组两三种；基于主题色生成超级淡配色；不同输出块配色不同；用户可选主题色；输入框更明显。
- C4 M5 已完成默认 Markdown/块 accent，C5 要产品化为可选主题。

## Subworker Assignment Plan

项目配置为 strict worker separation，但当前 Codex 子代理工具仅允许在用户明确要求 sub-agent/parallel agent 时使用。除非用户补充授权，本 Milestone 由主代理执行并在报告中记录降级证据。

## Technical Solution

在 TUI 现有配置和 theme/style 层新增 `ui_theme` 或等价配置项，提供 3 组内置 palette。Markdown renderer、history cell block accent、composer focus style 统一从 palette 取 semantic roles；旧配置缺失时使用 C4 默认 palette。

## Technical Route

1. 梳理 `codex-rs/tui` 的 theme/config 读取路径、Markdown style 初始化和 history cell accent helper。
2. 添加配置类型、默认值、schema 更新路径，确保旧 config 兼容。
3. 新增 theme picker 入口，优先使用已有 slash command 或设置 UI；不引入复杂 settings 页面。
4. 提供至少三组主题：每组包含 primary、secondary、accent，并派生 subtle background/border/foreground roles。
5. 将 Markdown strong/emphasis/list/table/code/link、message block accents、composer focus border 接到 palette。
6. 补测试和快照：默认主题、切换主题、持久化 reload、未知主题 fallback、窄宽度不重叠。

## Validation Path

- `just test -p codex-tui theme`
- `just test -p codex-tui markdown_render`
- `just test -p codex-tui history_cell`
- `just test -p codex-tui chat_composer`
- `cargo insta pending-snapshots -p codex-tui`
- 如改 config schema：`just write-config-schema` 和相关 schema diff 检查。

## Audit Focus

- 主题选择必须持久化，不只是当前 session 状态。
- 不把 UI 做成大规模 settings 重构。
- 淡色块必须不影响终端可读性，窄屏不能重叠。
- 旧配置和未知主题值必须 fallback。
