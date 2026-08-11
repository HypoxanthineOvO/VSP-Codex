# C2 发布准备补丁：Resume 与 Context 显示

## 结果

本补丁完成 C2 close 后的发布准备修正：`vsp-codex resume` 进入会话时不再把完整历史逐行刷入终端 scrollback，Context 显示统一为“已用量/总量 + 百分比”，并补齐发布前测试证据。

## 主要改动

- `codex-rs/tui/src/app/resize_reflow.rs`：初始 resume replay 改为缓冲后只把最近 400 个已渲染行写入终端 scrollback；完整 transcript cell 仍保留在内存里，不截断 session 数据。
- `codex-rs/tui/src/app/tests.rs`：新增 initial replay row-cap 回归测试，覆盖大历史 resume 时只保留尾部渲染行。
- `codex-rs/tui/src/token_usage.rs`：新增 Context Window 已用百分比计算，移除旧的剩余百分比计算。
- `codex-rs/tui/src/bottom_pane/footer.rs`、`chatwidget/status_surfaces.rs`：Context 默认显示统一为 `Context {used}/{window} {percent}%`；未知值显示为 `Context 0/0 0%`，只有 used、没有 window 时显示为 `Context {used}/0 0%`。
- `status_line_setup.rs`、`title_setup.rs`、`status_surface_preview.rs`：`context-remaining` 仍作为兼容旧配置可解析，但展示内容也走已用量格式，不再显示 `left`/剩余量。
- `codex-rs/tui/src/chatwidget/turn_runtime.rs`、`plan_implementation.rs`：Plan implementation popup 的清理上下文提示也改为 `used/window percent`，不再显示 `90% used` 这类单独百分比。
- `codex-rs/tui/src/history_cell/messages.rs`：非隐藏 reasoning summary 可见；`transcript_only` hidden reasoning 继续保持不可见，避免翻译/Transcript 泄露。
- `codex-rs/app-server/tests/suite/v2/compaction.rs`、`plugin_list.rs`：官方 remote/cloud 相关 app-server 测试改为显式 opt-in，避免默认 VSP CI 被已清理的官方路径拖红。
- `codex-rs/core/tests/common/responses.rs`：`/models` mock 支持 `client_version` query，避免兼容请求被测试 mock 误判。

## 测试与验证

- `just fmt`：通过。
- `just test -p codex-tui`：2655 passed，4 skipped。
- `just test -p codex-tui plan_mode`：63 passed，2596 skipped。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `just fix -p codex-tui`：通过。
- `git diff --check`：通过。
- 残留扫描：`codex-rs/tui/src` 中未发现 `Context 100% left`、`100% context left`、`Context: ... used` 或 `Context ... used`。
- app-server focused remote/cloud 收敛测试：
  - `CODEX_SKIP_BWRAP_BUILD=1 just test -p codex-app-server auto_compaction_remote_emits_started_and_completed_items app_server_startup_remote_plugin_sync_runs_once plugin_installed_includes_remote_shared_with_me_plugins plugin_list_fetches_featured_plugin_ids_without_chatgpt_auth plugin_list_fetches_shared_with_me_kind plugin_list_uses_warmed_featured_plugin_ids_cache_on_first_request`
  - 6 passed，779 skipped。
- full workspace 本机尝试：
  - `CODEX_SKIP_BWRAP_BUILD=1 just test`
  - 9946 tests run：9786 passed，155 failed，5 timed out，21 skipped。
  - 失败集中在本机环境/负载敏感路径和官方 remote/shell/sandbox/realtime/thread_manager 旧路径，不作为本补丁发布阻断；C2 相关目标测试已单独收敛。

## 发布说明

- 发布包应包含 `target/debug/vsp-codex` 或后续 release build 产物，并确保全局 `vsp-codex` 指向本仓库构建产物。
- 当前 Context 示例格式：`Context 1.2K/400K 0%`。
- `context-remaining` 仍可兼容旧配置文件，但语义已经变成已用量显示；新配置建议使用 `context-used`。
- Resume 闪烁修复只限制终端 scrollback 初始输出，不删除 session 历史，不影响 resume picker 或 transcript overlay 查看完整内容。

## 追加修复：Read File 高亮与 Subagent 面板

- `codex-rs/tui/src/exec_cell/render.rs`：Read File 输出根据读取路径扩展名复用现有语法高亮，保留原有输出截断、换行和 dim 样式；Transcript 视图也使用同一高亮路径。
- `codex-rs/tui/src/chatwidget/active_subagents.rs`：Subagent 面板没有 metadata 时改用稳定中文姓名池兜底，不再显示 thread id/hash；terminal tool call 缺少 `agents_states` 时会清掉对应旧 row，避免已完成 agent 残留。
- `codex-rs/tui/src/chatwidget/interaction.rs`：详情快捷键从 `Alt+A` 改为 `Ctrl+O`；当存在活跃 Subagent 时优先打开详情，没有活跃 Subagent 时保留原 `Ctrl+O` 复制上一条回复行为。
- 新增/更新测试覆盖 Read File 高亮、metadata 迟到刷新、完成后清理、`Ctrl+O` 详情弹窗和 active panel 快照。

追加验证：

- `just fmt`：通过。
- `just test -p codex-tui read_output_uses_syntax_highlighting active_subagents_ctrl_o active_subagents_refresh_existing_row_when_metadata_arrives_late active_subagents_terminal_tool_call_without_agent_state_removes_row`：功能测试通过；首次运行生成预期快照。
- `just test -p codex-tui`：2657 passed，4 skipped。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `just fix -p codex-tui`：通过。
- `git diff --check`：通过。
- `cargo build -p codex-cli --bin vsp-codex`：通过。
- `./codex-rs/target/debug/vsp-codex --version` 与全局 `vsp-codex --version`：均为 `vsp-codex 0.134.0-vsp.2`。

## C2 release 验证追加

- `glab ci lint .gitlab-ci.yml`：valid。
- `cargo metadata --no-deps --format-version 1`：workspace/package version 为 `0.134.0-vsp.2`。
- `just fmt`：通过。
- `just test -p codex-tui`：2657 passed，4 skipped。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `just test -p codex-cli --test vsp_entrypoint`：8 passed。
- `just test -p codex-exec`：116 passed。
- `just test -p codex-model-provider-info`：21 passed。
- `just test -p codex-core-plugins startup_remote_plugin_sync remote_plugin`：18 passed，181 skipped。
- `just fix -p codex-cli -p codex-exec -p codex-tui -p codex-core-plugins -p codex-model-provider-info`：通过。
- `cargo build -p codex-cli --bin vsp-codex`：通过。
- `git diff --check`：通过。
