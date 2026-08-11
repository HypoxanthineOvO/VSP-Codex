# M2 Subagent 核心策略审核

## Verdict

pass

## Reviewed Refs

- `codex-rs/core/src/agent/role.rs`
- `codex-rs/core/src/agent/role_tests.rs`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/config/config_tests.rs`
- `codex-rs/core/src/tools/handlers/multi_agents_tests.rs`
- `codex-rs/tui/src/app/session_lifecycle.rs`
- `codex-rs/tui/src/app/tests.rs`
- `codex-rs/tui/src/chatwidget.rs`
- `codex-rs/tui/src/chatwidget/tests/popups_and_settings.rs`

## Findings

无阻塞问题。

审计指出 `simple` 角色已有中文名字池但缺少测试断言；已在 `built_in_roles_use_vsp_chinese_nickname_candidates` 中补充并验证通过。

## Residual Risks

- 测试 fixtures 中仍有 `Robie`、`Ada` 等英文样例名，但它们不是内置角色默认名字池的正常运行路径。
- `just test -p codex-core` 宽版测试在当前本机环境有既有失败，需要后续单独收敛测试环境或 CI 配置；M2 相关 scoped tests 已通过。

## Validation

- `just test -p codex-core built_in_roles_use_vsp_chinese_nickname_candidates`：通过。
- `just test -p codex-tui`：通过。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `git diff --check`：通过。
- `just fix -p codex-core -p codex-tui`：通过。
