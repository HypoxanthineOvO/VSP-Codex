---
kind: milestone-report
cycle: C24-responses-multi-provider-config-model-switching
milestone: M3
status: completed
updated: 2026-08-08T20:36:05+08:00
---

# M3 Runtime And Picker Report

## 结论

M3 已完成。新版 TUI 在存在 model-providers.toml addon layer 时，以 provider tabs 统一展示官方 base catalog 和各 addon catalog；选择以 (provider_id, model_id, reasoning_effort) 单次提交到 thread/settings/update。addon selection 只写 sibling file；切回官方 provider 时清除 addon default，并沿既有 config RPC 保存官方 model/provider。

## 技术实现

- Config 保留 addon 生效前的 base provider ID，并按当前 provider 生成 provider-scoped ModelsResponse。
- ModelsManager::get_model_info 优先使用 per-turn catalog，避免跨 provider 同名 model 复用旧 context/reasoning metadata。
- per-turn config 在每次 turn 构造时重新选择 effective catalog，因此 runtime provider switch 不再只切 endpoint。
- TUI 新增独立 provider_model_popups.rs，多 provider 时显示 tabs；addon 缺少 env credential 时保留可发现状态但禁用模型项，只显示 env key 名。
- TUI 新增独立 provider_model_selection.rs。顺序为 runtime update -> addon/default persistence -> in-memory commit；持久化失败会回滚 thread settings。切回官方 provider 的 config RPC 失败时同时恢复 addon default 和 thread settings。
- 未增加 app-server v2 wire shape；现有 thread/settings/update 已能在一个请求中携带 model provider、model、effort 与 collaboration mode，因此无需 schema regeneration。

## 修改模块

- Core config/registry：core/src/config/model_providers_file.rs、core/src/config/mod.rs
- Runtime metadata：core/src/thread_manager.rs、core/src/session/turn_context.rs、models-manager/src/manager.rs
- TUI transaction：tui/src/app/provider_model_selection.rs、tui/src/app/event_dispatch.rs、tui/src/app_event.rs
- TUI picker：tui/src/chatwidget/provider_model_popups.rs、tui/src/chatwidget/model_popups.rs
- Dedicated tests/snapshots：对应 *_tests.rs 与四个 provider popup snapshots

## 测试设计与结果

- Core addon config：13/13，通过缺失/空文件、strict schema、collision、default、comment preservation 与 official config 不变。
- Provider metadata：get_model_info_prefers_per_turn_provider_catalog 1/1，通过同 model ID 的 provider-local metadata 覆盖。
- Runtime integration：1/1。真实 local mock Responses turn 命中 addon /v1/responses 且 model 正确；完成 rollout 后 resume/fork 保留 provider/model/effort；删除 addon file 后再次选择触发 runtime rollback。
- TUI picker：3/3 tests、4 个 accepted snapshots，覆盖多 provider、single addon + OpenAI 回退、missing credential 和 44-column narrow layout；Enter action 保留 provider identity。
- Build/hygiene：codex-core check、codex-tui check、rustfmt check、git diff --check 和 pending snapshot check 均通过。只有仓库既有 dead-code warnings。

## 同轮 TUI 回归修复

- Resize replay：清屏后 viewport 不再归零，而是按当前 screen size 重新贴底；VT100/viewport regression 1/1。
- 中英文空格：静态 Markdown renderer 已证明原句空格不丢；问题来自新 completed-source reconciliation 用有损 canonical payload 覆盖精确 streamed source。现在仅 whitespace 不同时保留 streamed source，非空白内容不同仍执行 canonical repair。静态 renderer 与 split-delta consolidation 回归均通过。

## 预期行为

- addon 默认启动仍能从 /model 看到并切回官方 base provider。
- provider/model/effort 同时生效，provider-local Base URL、credential env key 和 model metadata 不串线。
- missing credential 不泄漏 value，也不会发请求。
- 失败不会留下 runtime 已切换但默认 selection 未保存的半状态。

## 问题与风险

- 未使用真实第三方 credential；协议路径由 deterministic local Responses mock 验证。DeepSeek/Kimi/GLM 仍保持 provider-neutral、未认证承诺。
- Direct local addon-file persistence 假定 TUI 与 embedded app-server 共享有效 CODEX_HOME；remote app-server 的跨主机文件所有权将在 M4 作为明确兼容边界复核。
- Compact 未发真实模型请求回归；resume/fork 已覆盖。Compact provider inheritance 留在 S1 后 M4 hardening matrix。
