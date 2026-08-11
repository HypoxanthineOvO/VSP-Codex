# M2 Subagent 核心策略报告

## 结果

M2 已完成。VSP Codex 现在把 Subagent 作为默认可用能力处理：TUI 不再通过旧的 `Enable subagents?` 授权弹窗阻断 `/agent`；默认 Subagent 上限为 32；内置角色使用 VSP 中文名字池；spawn 时显式传入的 `model` 与 `reasoning_effort` 会保留到事件元数据。

## 主要改动

- `codex-rs/tui/src/app/session_lifecycle.rs`：移除 `/agent` picker 的 Collab feature 显式启用 gate；无可用 Subagent 时只显示普通空状态。
- `codex-rs/tui/src/chatwidget.rs`：删除旧 `open_multi_agent_enable_prompt` 和相关 prompt 常量。
- `codex-rs/core/src/config/mod.rs`：默认 `agents.max_threads` 改为 32；MultiAgentV2 默认 session 并发为 33，因为它包含 root thread，等价于 32 个 Subagent slot。
- `codex-rs/core/src/agent/role.rs`：内置 `supervisor`、`worker`、`default`、`explorer`、`simple` 使用 VSP 中文 nickname candidates；角色配置未指定 model/reasoning 时保留当前显式选择。
- `codex-rs/core/src/tools/handlers/multi_agents_tests.rs`：补充无 full-history fork 时显式 model/reasoning metadata 的事件回归测试。

## 测试与验证

- `just fmt`：通过。
- `just test -p codex-core default_config_enables_collab_with_32_agent_threads built_in_roles_use_vsp_chinese_nickname_candidates multi_agent_v2_default_session_thread_cap_counts_root multi_agent_v2_spawn_none_fork_preserves_explicit_model_metadata_in_events`：通过。
- `just test -p codex-tui open_agent_picker_shows_empty_state_when_collab_is_disabled memories_enable_prompt_updates_feature_without_notice memories_enable_prompt_snapshot`：通过。
- `just test -p codex-tui`：2630 passed, 4 skipped。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `git diff --check`：通过。
- `just fix -p codex-core -p codex-tui`：通过。
- 补充审计后新增 `simple` 名字池断言，并通过 `just test -p codex-core built_in_roles_use_vsp_chinese_nickname_candidates`。

## 已知验证说明

`just test -p codex-core` 宽版运行未全绿，但失败不集中在 M2 改动路径：主要包括当前本机 test binary 缺失（如 `test_stdio_server`）、`/tmp` trust/项目层断言、部分 shell/network/remote compact 超时。M2 相关测试在该运行中均通过，完整失败清单保留在 `target/nextest/default/junit.xml`。

## Worker 证据

- test worker：`019e6057-13cc-7830-a6f9-edc72dc0f0b3`，补充 M2 初始测试，后续由 follow-up test worker 收敛。
- implement worker：`019e6067-a0af-7901-9a17-799e756aa923`，落地核心策略初版；因未及时返回最终状态，主代理关闭并整合已落地补丁。
- implement follow-up：`019e6071-61b1-7893-80d2-7bc1439fac8b`，移除旧授权 prompt，确认 MultiAgentV2 33 total/32 subagents。
- test follow-up：`019e6071-91ba-7bf0-9959-27b9e2f7f414`，更新 no-gate TUI 测试并删除旧 prompt snapshot。
- audit：`019e607a-cd8b-7ff2-bcc3-d5e208ad2d57`，无阻塞问题；指出 `simple` 缺少断言，已修复。
