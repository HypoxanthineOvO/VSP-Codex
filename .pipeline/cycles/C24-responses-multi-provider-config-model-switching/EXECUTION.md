---
kind: execution
cycle: C24-responses-multi-provider-config-model-switching
status: completed
updated: 2026-08-08T21:22:00+08:00
---

# C24 Execution

## Current Step

M1-M4 与 S1 completed；C24 closed。发布保留为独立、需明确授权的后续动作。

## Checkpoints

### M1 - Config And Protocol Contract

- Freeze `model-providers.toml` schema, merge precedence, collision behavior, error policy and selection persistence semantics.
- Verify current Responses support for candidate providers and record dated official evidence.
- Locate the effective VSP and DEX config-loading entrypoints.
- Evidence target: parser RED tests and an architecture/compatibility note linked from this checkpoint.

### M2 - Loader, Registry And Persistence

- Add the standalone config module in a new, bounded Rust module with sibling `*_tests.rs`.
- Build the effective provider registry after official `ConfigToml` loading without inserting private keys into official config layers.
- Persist only addon default selection with an atomic write; preserve comments and unrelated provider tables where practical.
- Evidence target: focused core/config tests proving missing-file no-op, strict parsing, collision rejection, credential isolation and untouched `config.toml`.

### M3 - Runtime And Picker

- Carry `(provider_id, model_id)` through thread/runtime settings as one transaction.
- Expose provider-scoped catalogs to connected clients only where the existing public v2 API is insufficient; regenerate schema if changed.
- Implement the compact provider-grouped `/model` surface and the single-provider current-name fallback.
- Evidence target: core/app-server integration tests, TUI interaction tests and reviewed `insta` snapshots.

### S1 - Human Review

- Present real rendered artifacts for multiple providers, one provider, missing credentials and narrow terminal layouts.
- Run an isolated-home smoke using at least two configured Responses providers or deterministic local mocks when live credentials are unavailable.
- Acceptance: the user can switch without editing official config, understands the active provider, and confirms old-client startup compatibility.

### M4 - Final Hardening

- Apply S1 feedback without deleting review history.
- Run focused package tests, schema generation/tests when applicable, TUI snapshots, formatting, lint and diff-size checks.
- Record final changed modules, test design/results, expected behavior, problems, risks and follow-up.

## Investigation Record

- V10 separated Chat-only configuration into `chat-providers.config.toml`, but required a profile flag and still carried Chat-specific protocol/state complexity.
- V10 provider grouping introduced provider-local catalogs and atomic model selection, then accumulated provider recovery, credential and fork/compact fixes. These are evidence for required invariants, not a patch series to replay.
- Current main retains a simple `/provider` popup and provider update event, while `/model` uses one `ModelsManager` catalog. A standalone file alone would therefore be incomplete.
- The compatibility invariant is two-sided: provider definitions and the persisted third-party selection must both remain outside official `config.toml`.

## M1 - Config And Protocol Contract

- 文件：`$CODEX_HOME/model-providers.toml`；缺失或纯空白等同无 addon layer，任何其他内容必须显式 `version = 1`。
- provider：ID 必须匹配 `^[a-z0-9][a-z0-9_-]{0,63}$`；name/base URL/env key 必填；只接受 `wire_api = "responses"`；Base URL 只允许 HTTP(S)，禁止 userinfo/query/fragment。
- model：使用 `[[providers.<id>.models]]`，要求非空 id/display_name、正数 `context_window`，默认 effective percent 为 95；默认 reasoning effort 必须出现在 supported 列表。
- identity：同一 provider 内 model ID 不得重复；跨 provider 同名 model 合法，运行时身份始终为 `(provider_id, model_id)`。
- default：`default_provider/default_model` 必须同时存在或同时缺失，并必须引用 addon 文件自身目录；选择官方 provider 时清除 addon default，回落官方配置。
- merge：addon provider ID 与官方/用户 `config.toml` effective registry 任一 ID 碰撞即报错，不覆盖。
- credential：TOML 只保存环境变量名。环境变量缺失时 provider 可发现但不可发请求。
- entrypoint：仓库没有独立 DEX binary/crate/loader；TUI/exec/direct app-server 等运行入口均经 `ConfigBuilder`，因此连接式 DEX 语义由 app-server 共用该加载点实现。
- named providers：网络核验 gate 未获得授权，DeepSeek/Kimi/GLM 暂不作为已验证内置承诺；首版合同保持 provider-neutral。
- parser evidence：focused `codex-core` 首次 0/7 RED（stub），实现 strict parser 后 7/7 GREEN。

## M2 - Loader, Registry And Persistence

- `Config::load_config_with_layer_stack` 是所有 runtime/connected paths 的现有汇合点；addon loader 在这里读取有效 `CODEX_HOME`，无需改动各入口签名。
- CLI/thread `ConfigOverrides` 优先于 addon default；addon default 优先于官方 `config.toml` default。
- addon provider 转换为 Responses-only `ModelProviderInfo`，只保存独立 `env_key` 引用；环境值本身不进入配置结构。
- 与 built-in 或 `config.toml` effective provider registry 的任何 ID 碰撞均作为 InvalidData 拒绝。
- default persistence 复用现有原子写 helper，只编辑 `model-providers.toml` 顶层 default pair，保留 provider tables/comments；清除 addon selection 时删除 pair 并回落官方默认。
- focused result：11 passed / 0 failed；覆盖 missing/blank、strict parser、跨 provider model identity、collision、default application、credential reference、comment preservation 和 official config byte-for-byte unchanged。
- size note：实现 413 行、专用测试 343 行、Config 接线约 29 行，总计约 785 行。Agent 曾错误地把规模建议升级为人工 gate；用户明确拒绝该限制。后续仅通过模块化和 focused validation 管理复杂度，不拆 Cycle、不改变确认范围。

## Side Effects

- C24 已重新聚焦并继续 M3。
- M1-M2 已修改 core config 模块；本地 standalone `current` 另按用户要求切到 C23 SHA `cbdf855c…`，当前 TUI 需重开生效。

## M3 - Runtime And Picker

- provider/model/effort 通过单个 thread settings update 应用；addon persistence 或 official config persistence 失败时回滚 runtime。
- per-turn model catalog 随 provider 切换，跨 provider 同名 model 不复用旧 metadata。
- /model 在 addon layer 存在时展示 provider tabs，并始终提供 base provider bundled/catalog fallback。
- local mock Responses turn、resume、fork、persistence rollback 与四类 TUI snapshot 已通过。
- 用户追加的 resize-top-anchor 与中英文空格回归已定位并修复，focused tests 通过。
- M3 report：M3-REPORT.md。

## S1 Acceptance

- 2026-08-08 用户接受多 provider、single addon + OpenAI fallback、missing credential 与 44-column narrow snapshots。
- 用户明确表示不再自行手测，并授权继续完成 M4。
- 接受范围不含发布、推送、安装或服务重启；这些外部副作用不在 C24 原计划边界内。

## M4 - Final Hardening

- App-server v2 `modelProvider/list`/`write` 将 addon catalog 与 persistence ownership 放在
  服务端，remote TUI 不直接写本地 sibling file。
- Manual compact 使用当前 turn provider catalog 与 endpoint；provider identity 不回退为
  base provider。
- Protocol schema fixtures、Core/app-server/TUI focused matrix、rustfmt、diff 和 snapshot
  hygiene 通过。全量 TUI/Clippy 的无关 repo 级例外已隔离记录。
- Final report：M4-REPORT.md；Cycle summary：SUMMARY.md。
