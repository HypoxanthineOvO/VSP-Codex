# M1 Theme Picker + Persisted Palettes Report

## 结论

M1 已完成。TUI 现在支持持久化 UI 主题色组，用户可以通过 `/ui-theme` 切换三组内置主题；Markdown 角色、消息输出块 accent、输入框 focused prompt 都会跟随当前主题。

## 具体产出

- 新增配置项：`[tui].ui_theme`，可选值为 `teal`、`violet`、`amber`，默认 `teal`。
- 新增配置类型：`TuiUiTheme`，并接入 core runtime config、config edit、core-api re-export 和生成的 `core/config.schema.json`。
- 新增 Slash command：`/ui-theme`，打开主题选择器。
- 新增 picker：显示三组可持久化主题色组，选择后写入配置并立即更新当前 TUI palette。
- 新增全局 TUI palette 层：`active_ui_theme()`、`set_ui_theme()`、`active_palette()`、`palette_for()`。
- Markdown 渲染接入 palette：
  - 标题、strong、emphasis、inline code、link、blockquote 使用当前主题色。
  - 有序/无序列表 marker 和表格边框使用结构/淡边框色。
  - 保留 C4 的列表 marker 语义：最外层实心圆点、第二层空心圆点、第三层小方块。
- History cell 输出块接入 palette：
  - user、assistant、reasoning、tool、ask 等块使用不同 accent。
  - 主题切换后不同输出块仍保持可区分。
- 输入框接入 palette：
  - focused composer prompt 使用当前主题 primary accent，比普通状态更明显。
- 修复 sample 编译暴露的问题：
  - `codex-thread-manager-sample` 的 `Config` 字面量补齐 `agent_nickname_candidates: None`。

## 主要文件

- `codex-rs/config/src/types.rs`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/config/edit.rs`
- `codex-rs/core/config.schema.json`
- `codex-rs/core-api/src/lib.rs`
- `codex-rs/thread-manager-sample/src/main.rs`
- `codex-rs/tui/src/style.rs`
- `codex-rs/tui/src/ui_theme_picker.rs`
- `codex-rs/tui/src/slash_command.rs`
- `codex-rs/tui/src/app_event.rs`
- `codex-rs/tui/src/app/event_dispatch.rs`
- `codex-rs/tui/src/app/config_persistence.rs`
- `codex-rs/tui/src/chatwidget/settings*.rs`
- `codex-rs/tui/src/chatwidget/slash_dispatch.rs`
- `codex-rs/tui/src/markdown_render.rs`
- `codex-rs/tui/src/history_cell/mod.rs`
- `codex-rs/tui/src/bottom_pane/chat_composer.rs`

## 验证

- `just write-config-schema`：通过，schema 已更新。
- `just fmt`：通过。
- `just fix -p codex-config`：通过。
- `just fix -p codex-core`：通过。
- `just fix -p codex-tui`：通过。
- `just fix -p codex-core-api`：通过，目标无 test target，为 no-op。
- `just fix -p codex-thread-manager-sample`：先暴露缺失字段，补齐后通过。
- `just test -p codex-core runtime_config_uses_tui_ui_theme`：1/1 通过。
- `just test -p codex-tui markdown_render`：127/127 通过。
- `just test -p codex-tui history_cell`：122/122 通过。
- `just test -p codex-tui chat_composer ui_theme_picker_exposes_three_persistable_theme_groups focused_prompt_uses_selected_ui_theme_accent`：188/188 通过。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：无 pending snapshots。
- `git diff --check`：通过。

## 风险和后续

- 当前 palette 使用终端基础色，未引入 truecolor 自定义 RGB；这更稳，后续如果要更细腻的“超级淡背景”可在检测终端能力后扩展。
- M2 将继续处理 command palette、focus stack、复制/选择体验，避免与 `/ui-theme` 入口冲突。
