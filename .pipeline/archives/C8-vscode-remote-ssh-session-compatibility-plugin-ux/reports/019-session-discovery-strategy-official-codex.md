# C8 M4 Session Discovery Strategy For Official Codex Report

完成时间：2026-05-30T17:56:52+08:00

## 结果

M4 已完成，audit 复审结论为 `pass`。本里程碑实现了 VSP session 到 official Codex 可见区域的安全脱敏导出层：

- 新增 `codex_rollout::official_codex_discovery`。
- 导出 manifest 到 `official_codex_home/official_codex_discovery/manifest.json`。
- 导出真实脱敏 JSONL artifact 到 `official_codex_home/official_codex_discovery/sessions/**`。
- 保留 official discovery/listing 需要的安全元数据：session id、created timestamp、cwd、安全 title、source、sanitized artifact path。
- 剥离 provider/auth/config/env/Workflow `.pipeline`/rollout_trace/private marker 等私有或敏感字段。

## 变更

- `codex-rs/rollout/src/lib.rs`
  - 暴露 `official_codex_discovery` 模块。
- `codex-rs/rollout/src/official_codex_discovery.rs`
  - 实现 `export_sessions_for_official_codex`。
  - 实现 `classify_field_for_official_discovery`。
  - 递归清理 JSONL 中的 secret/private 字段和值。
  - 对 `session_index.jsonl.thread_name` 和 rollout title/name 做输出前清理。
  - 为重复 basename 的 source rollout 生成去重 artifact 路径，避免覆盖。
- `codex-rs/rollout/tests/official_codex_discovery.rs`
  - 覆盖脱敏导出、字段分类、禁止 raw/symlink 输出、`session_index` title 清理、重复 basename、防 symlink source traversal 回归、以及 artifact 可被 normal rollout reader 读取。

## 验证

- `cargo test -p codex-rollout --test official_codex_discovery`：8 项通过。
- `cargo fmt --check --package codex-rollout`：通过，仅有既有 stable rustfmt `imports_granularity` warning。
- `git diff --check`：通过。
- M4 audit 复审：`pass`。

## 边界

这次实现的是安全 sanitized export API 与 artifact/manifest，不是 official Codex 的自动发现、导入或 resume wiring。当前产物已经能通过本仓库 normal rollout reader 做 id/cwd/preview/source 级别读取；是否接入 official Codex 的自动列表/导入流程，需要后续里程碑继续实现。

非阻塞残余风险：source traversal 目前有 symlink regression 覆盖并在当前环境通过，但实现仍未显式使用 `symlink_metadata()` 或 canonical containment 做跨平台硬化。后续如需要把 session 目录视为不可信输入，可追加 hardening。

