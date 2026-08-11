# C8 M3 Provider And Resume Compatibility Contract Report

完成时间：2026-05-30T15:22:46+08:00

## 结果

M3 已完成，audit 结论为 `pass`。`thread/resume` 现在会在 resume metadata 边界处理 provider：

- 显式空 `modelProvider: ""` 视为 unset，回退到当前配置 provider。
- 省略 provider 的 VSP-style rollout 可正常 resume，并且不会把私有 provider metadata 写回源 rollout。
- 非空未知 persisted provider 会进入 config loading 验证，并返回清晰错误，例如 `Model provider \`missing_provider\` not found`。

## 变更

- `codex-rs/app-server/src/request_processors/thread_processor.rs`
  - 增加 shared persisted resume value merge path。
  - 从 resumed rollout history 中读取非空 `session_meta.model_provider` 并作为 config override 验证。
  - 对空白 provider 做 unset 归一化，保留 configured provider fallback。
- `codex-rs/app-server/tests/suite/v2/thread_resume.rs`
  - 新增 M3 focused tests 覆盖空 provider fallback、省略 provider privacy、未知非空 provider error。

## 验证

- `cargo test -p codex-app-server c8_m3_thread_resume -- --nocapture`：3 项通过。
- `cargo test -p codex-rollout vsp_provider_is_omitted_from_persisted_session_meta -- --nocapture`：通过。
- `cargo fmt --check --package codex-app-server --package codex-rollout`：通过，仅有既有 stable rustfmt `imports_granularity` warning。
- `git diff --check`：通过。

## 后续

按用户要求，本 Cycle 在 M3 完成后暂停。下一次 `/hw:resume` 应从 M4 `Session Discovery Strategy For Official Codex` 的 `write_tests` 开始。

