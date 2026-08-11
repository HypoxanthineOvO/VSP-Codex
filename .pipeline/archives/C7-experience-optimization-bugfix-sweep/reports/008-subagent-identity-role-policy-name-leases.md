# C7/M2 Subagent Identity, Role Policy, And Name Leases

完成时间：2026-05-30T03:12:48+08:00

## 结果

M2 已完成。Subagent 主界面和生命周期记录现在优先显示稳定协作者名称，而不是把 UUID 作为主要标签；role 默认值与显式 `model` / `reasoning_effort` 能按预期组合；active 名称租约会避让 active explicit nickname；显式 `service_tier = "default"` 使用 no-tier/default 语义，不再被当成 unsupported catalog tier 拒绝。

## 主要变更

- TUI 侧新增共享的 Subagent display-name resolver，用同一套解析和租约逻辑服务 lifecycle history 与 Active Subagents panel。
- fallback 名称按 `ThreadId` 稳定保存，active lease 避免同时活跃的 fallback 名称碰撞，并在 terminal 生命周期释放 active lease。
- explicit nickname 会在 active 状态占用租约，避免 fallback agent 复用同名。
- core spawn 路径先应用 role defaults，再处理显式 model/reasoning 覆盖；full-history fork 的原有限制保持不变。
- explicit `service_tier = "default"` 在验证前转为 no-tier/default sentinel，不污染子配置的具体 service tier。
- 默认栈 stack overflow 通过拆开 session 初始历史记录和跳过非 root subagent 的持久 goal runtime materialization 修复，没有保留 `stack_size` workaround。

## 验证

- `cargo test -p codex-tui active_subagent_fallback_avoids_active_explicit_nickname_collision -- --nocapture`：通过。
- `cargo test -p codex-core spawn_agent_explicit_default_service_tier_uses_no_tier_semantics -- --nocapture`：通过。
- `cargo test -p codex-core spawn_agent_service_tier_inheritance_preserves_supported_or_configured_tiers -- --nocapture`：通过。
- `cargo test -p codex-core multi_agent_v2_spawn_role_allows_explicit_reasoning_effort_override -- --nocapture`：通过。
- `cargo test -p codex-tui collab_lifecycle_history_uses_same_fallback_names_as_active_panel -- --nocapture`：通过。
- `cargo test -p codex-tui active_subagent_name_leases_avoid_collisions_and_reuse_released_names -- --nocapture`：通过。
- `cargo test -p codex-tui active_subagents -- --nocapture`：通过。
- `cargo fmt --check`：通过；仅有既有 stable rustfmt 对 nightly-only `imports_granularity = Item` 的 warning。
- `git diff --check`：通过。

## 审计

首次 audit 返回 `needs_changes`，发现 active explicit nickname 没有占用 fallback lease，以及 explicit `service_tier = "default"` 可能被错误拒绝。

修复后只读复审 worker 孔丘 the 8th 返回 `pass`。复审确认两个阻塞问题已关闭，M2 原始身份、角色、租约和默认栈语义没有回归。

## 剩余风险

- fallback 候选池耗尽时仍会进入确定性退化命名，可能出现重复 fallback；这符合当前候选池策略，后续如要完全避免重复需要扩展候选池或引入序号显示策略。
- 本轮没有实现完整 Subagent Prompt/Report viewer；该工作属于 C7/M4。
