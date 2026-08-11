# M13 Subagent 昵称候选池配置化报告

完成时间：2026-05-27T16:27:38+08:00

## 结果

M13 已完成。

本轮把 Subagent nickname candidates 从硬编码公共私有名单改为可配置契约，并统一 core spawn 与 TUI active fallback 的候选池行为。

## 实现内容

- 新增顶层配置：

```toml
[agents]
nickname_candidates = ["陈阳宾", "尹一帆"]
```

- 保留并优先支持 role-level 配置：

```toml
[agents.worker]
nickname_candidates = ["陈阳宾", "尹一帆"]
```

- core spawn 昵称优先级：
  1. role-level configured candidates
  2. top-level `[agents].nickname_candidates`
  3. built-in role candidates
  4. built-in default candidates
- TUI active Subagent fallback 昵称优先级：
  1. metadata nickname
  2. role-level configured candidates
  3. top-level `[agents].nickname_candidates`
  4. built-in role fallback candidates
  5. non-empty ancient Chinese generic fallback
- Unicode/中文昵称现在允许；仍拒绝空列表、trim 后空名、trim 后重复名和控制字符。
- `codex-rs/core/config.schema.json` 已暴露顶层 `[agents].nickname_candidates`。
- 公共默认昵称池改为中国古代人物名，移除了原公共默认中的私人姓名。
- 用户个人名单可通过配置维护，不再写死进公共默认。

## 主要文件

- `codex-rs/config/src/config_toml.rs`
- `codex-rs/core/config.schema.json`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/config/agent_roles.rs`
- `codex-rs/core/src/agent/role.rs`
- `codex-rs/core/src/agent/control.rs`
- `codex-rs/tui/src/chatwidget/active_subagents.rs`
- `codex-rs/tui/src/chatwidget/rendering.rs`
- `codex-rs/core/src/config/config_tests.rs`
- `codex-rs/core/src/agent/role_tests.rs`
- `codex-rs/core/src/agent/control_tests.rs`
- `codex-rs/tui/src/chatwidget/tests/app_server.rs`

## Worker 证据

- Test worker：`.pipeline/reviews/C3/M13/test.md`
- Implement worker：`.pipeline/reviews/C3/M13/implement.md`
- Implement retry：`.pipeline/reviews/C3/M13/implement-retry.md`
- Audit：`.pipeline/reviews/C3/M13/audit.md`
- Audit warning fix：`.pipeline/reviews/C3/M13/implement-audit-fix.md`
- Audit follow-up：`.pipeline/reviews/C3/M13/audit-followup.md`

## 验证

- `cd codex-rs && just write-config-schema`：通过
- `cd codex-rs && just test -p codex-core load_config_accepts_top_level_agent_nickname_candidates load_config_accepts_chinese_agent_role_nickname_candidates load_config_rejects_invalid_top_level_agent_nickname_candidates spawn_thread_subagent_uses_top_level_nickname_candidates_without_role_pool spawn_thread_subagent_uses_role_specific_nickname_candidates built_in_roles_use_public_ancient_chinese_nickname_candidates`：6 passed
- `cd codex-rs && just test -p codex-tui active_subagents_fallback_uses_configured_nickname_candidates active_subagents_fallback_uses_role_specific_nickname_candidates active_subagents_fallback_uses_built_in_role_nickname_candidates`：3 passed
- `cd codex-rs && cargo insta pending-snapshots --workspace`：No pending snapshots
- `cd codex-rs && just fmt`：通过
- `cd codex-rs && just fix -p codex-core -p codex-config -p codex-tui`：通过
- `cd codex-rs && just fix -p codex-tui`：通过
- `cd codex-rs && cargo build --bin vsp-codex`：通过

按仓库规则，`just fix` 后未重复运行测试。

## 审计

首轮 audit 发现一个非阻塞 warning：TUI fallback 未按 row role 使用 role-level/built-in role pool。已修复，并由复审确认 `verdict: pass`。

## 残余事项

C3 仍不能最终验收，因为 M7 的真实跨设备 pending Ask resume 验收还未完成。当前状态应回到 M7 pending acceptance。
