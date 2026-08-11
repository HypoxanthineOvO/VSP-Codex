# C9 Debug: VSP Session Provider Root Cause

## 改动摘要

用户复测确认问题仍存在：新建 VSP-Codex 会话后，VSCode 插件 resume 报错 `failed to load configuration: Model provider `` not found`。本轮结论是前一次只修了历史数据和部分读取 fallback，没有彻底封住新建 session 的空 provider 写入链路。

本轮已完成根因修复、安装替换、真实新会话落盘验证和现有坏数据修复。

## 技术思路

排序假设与验证：

1. 新 session 仍由旧 VSP runtime 创建。确认成立。只读 worker 新建的 session 位于 `~/.codex/sessions/...019edb21...jsonl`，首行显示 `cli_version: 0.134.0-vsp.7.1` 且 `model_provider: null`。
2. 新版代码仍可能把空 provider 写入持久化。确认成立。`Config::effective_model_provider_id_for_persistence()` 反查 provider map 失败时仍返回空 `model_provider_id`；thread-store 默认 provider 和 rollout recorder 也没有最后一道非空兜底。
3. VSCode 插件路径仍可能指向官方或旧二进制。部分成立。wrapper 已指向 `~/.vsp-codex`，但系统中仍有官方 extension binary、npm `codex` 和旧 `vsp-codex` 进程运行；VSCode 本地 User settings 若未指向 wrapper，仍会绕过 VSP。

## 修改文件/模块

- `codex-rs/core/src/config/mod.rs`：`effective_model_provider_id_for_persistence()` 先验证现有 id，再从 provider map 反查，最后回退非空 provider id；不再返回空字符串。
- `codex-rs/core/src/session/session.rs`：session create/resume 持久化继续使用 effective provider id。
- `codex-rs/core/src/tools/handlers/multi_agents_common.rs`：subagent spawn/resume config 从运行时 provider object 恢复 provider id。
- `codex-rs/thread-store/src/local/{create_thread.rs,live_writer.rs,helpers.rs,mod.rs,read_thread.rs}`：local thread-store 对空白 metadata/default provider 做非空兜底。
- `codex-rs/rollout/src/recorder.rs`：rollout `session_meta.payload.model_provider` 在极端空 provider 情况下仍写非空 provider，而不是缺失或 null。
- `codex-rs/state/src/extract.rs`、`codex-rs/rollout/src/state_db.rs`、`codex-rs/thread-store/src/thread_metadata_sync.rs`、`codex-rs/app-server/src/request_processors/thread_processor.rs`：读取和投影旧 session 时过滤空白 provider 并回退默认 provider。
- 测试补充在 `config_tests.rs`、`multi_agents_tests.rs`、`recorder_tests.rs`、`thread-store/src/local/mod.rs`、`thread_processor_tests.rs`、`thread_resume.rs`、`state/src/runtime/threads.rs`。

## 数据修复

只修 `cli_version` 包含 `-vsp` 且首行 `model_provider` 为空、缺失或 null 的 JSONL。没有改官方 `0.140.0` 或 provider 已为 `OpenAI` 的 session。

- 修复数量：8 个 VSP JSONL。
- 备份目录：`/home/heyx/.codex/session-provider-backups/provider-repair-20260618T144505Z`。
- 复扫结果：`vspBlankProviderSessionMetaCount = 0`。
- SQLite 复核：`~/.codex/state_5.sqlite` 与 `~/.vsp-codex/state_5.sqlite` 的 blank provider count 均为 0。

## 验证结果

通过：

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core effective_model_provider_id_for_persistence -- --nocapture`：2 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-rollout blank_provider_is_not_persisted_as_missing_session_meta_provider -- --nocapture`：1 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-thread-store create_thread_never_persists_blank_provider_when_default_provider_is_blank -- --nocapture`：1 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core build_agent_spawn_config_recovers_runtime_model_provider_id -- --nocapture`：1 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-thread-store create_thread_uses_default_provider_when_metadata_provider_is_blank -- --nocapture`：1 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server c8_m3_thread_resume_null_model_provider_falls_back_to_configured_provider -- --nocapture`：1 passed。
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check`：通过，仅稳定 rustfmt 输出 `imports_granularity = Item` warning。
- `git diff --check`：通过。
- `cargo build --manifest-path codex-rs/Cargo.toml -p codex-cli --release`：通过。
- 安装替换后 `/home/heyx/.vsp-codex/vscode/vsp-codex --version`：`vsp-codex 0.140.0-vsp.0`。
- 真实 wrapper smoke：`/home/heyx/.vsp-codex/vscode/vsp-codex app-server --stdio` 后 `thread/start` + `thread/inject_items`，新文件 `/home/heyx/.vsp-codex/sessions/2026/06/18/rollout-2026-06-18T22-44-00-019edb30-5dc4-7061-92ef-100227f34dbb.jsonl` 首行 `model_provider` 为 `vsp`，`cli_version` 为 `0.140.0-vsp.0`。

## 预期结果

新版 VSP-Codex 创建的新 session 不会再写出空 provider。旧 VSP 空 provider session 已回填，VSCode resume 不应再因为 provider 为空而报 `Model provider `` not found`。

## 遇到的问题

- 本轮确认机器上仍有多个旧/官方运行态：官方 VSCode extension binary、npm `@openai/codex`、旧 `vsp-codex` 进程。已替换 wrapper 目标和 standalone binary，但已经运行的旧进程不会自动升级。
- `vscode install` 仍提示 `chatgpt.cliExecutable` 是 VSCode application-scoped setting；远端 Machine settings 写入不一定被 VSCode UI 采用。本地 VSCode User settings 必须指向 `/home/heyx/.vsp-codex/vscode/vsp-codex`。

## 风险/后续

这次没有重启用户的所有 VSCode/agent 进程。若 VSCode UI 仍连接到已存在的官方/旧 app-server 进程，需要在 VSCode 侧 reload window 或重启插件进程，让它重新读取 wrapper 和新 binary。
