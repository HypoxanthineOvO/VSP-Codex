# VSP8 Plugin Full Access And Compaction Release

## Conclusion

本轮修复收束为 `v0.140.0-vsp.8` 同版本 HotFix。错误的 `v0.140.0-vsp.0` 发布命名已迁移到继续 VSP 自有序列的 `v0.140.0-vsp.8`；VSCode Plugin / app-server 默认权限恢复为 VSP Full Access；native Ask UI 不再显示合成 `Question Context`；自动压缩摘要不会作为可见 assistant 回复输出，压缩结束后真实回复仍会继续显示。

## Technical Approach

- 版本序列：将 workspace/package/release manifest/docs 从 `0.140.0-vsp.0` 迁移到 `0.140.0-vsp.8`，延续历史 `vsp.7.x` 后的自有功能版本号。
- 权限默认值：在 app-server config load 的 typed override 层补默认 `approval_policy=never` 与 `sandbox=danger-full-access`。这会覆盖旧 `config.toml` 中遗留的 `on-request` / `read-only`，但保留显式 request override 的能力。
- Question Tool：native Ask UI 客户端直接接收 `item/tool/requestUserInput`，不再由 app-server 合成可见 `Question Context / Answer impact` agent message；无 native Ask UI 客户端继续走 Markdown fallback，并用 synthetic `turn/completed` 解锁输入。
- 自动压缩：app-server 可见事件测试现在完整观察触发 compaction 的 user turn，断言本地 compaction summary 不出现在可见通知中，同时最终 `FINAL_REPLY` 仍出现。
- 发布资产：重建 Linux x64 GNU/glibc preview tarball，刷新 versioned manifest、stable manifest、README/docs 安装 SHA 和 release notes。

## Modified Surfaces

- App-server permission/config and event flow: `codex-rs/app-server/src/config_manager.rs`, `codex-rs/app-server/src/bespoke_event_handling.rs`, `codex-rs/app-server/src/request_processors/thread_lifecycle.rs`, `codex-rs/app-server/src/request_processors/initialize_processor.rs`, `codex-rs/app-server/src/native_ask_ui.rs`.
- App-server tests: `codex-rs/app-server/tests/suite/v2/thread_start.rs`, `codex-rs/app-server/tests/suite/v2/request_user_input.rs`, `codex-rs/app-server/tests/suite/v2/compaction.rs`.
- TUI links and docs/CI discipline from the same hotfix batch: `codex-rs/tui/src/terminal_hyperlinks.rs`, `docs/ci.md`, `scripts/check_ci_docs.py`, `.gitlab-ci.yml`.
- Release metadata and package: `codex-rs/Cargo.toml`, `codex-cli/package.json`, `codex-rs/Cargo.lock`, `README.md`, `docs/install.md`, `docs/vsp-codex.md`, `CHANGELOG.md`, `.pipeline/release/v0.140.0-vsp.8/*`, `.pipeline/release/vsp-release-manifest.json`.

## Validation

- Passed: `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all auto_compaction_local_emits_started_and_completed_items -- --nocapture`.
- Passed: `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all thread_start_defaults_to_vsp_full_access -- --nocapture`.
- Passed: `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input_native_ask_ui_does_not_emit_synthetic_question_context_message -- --nocapture`.
- Passed with larger Tokio worker stack: `RUST_MIN_STACK=16777216 cargo test --manifest-path codex-rs/Cargo.toml -p codex-core --test all auto_compact_runs_after_token_limit_hit -- --nocapture`.
- Passed with larger Tokio worker stack: `RUST_MIN_STACK=16777216 cargo test --manifest-path codex-rs/Cargo.toml -p codex-core --test all auto_compact_emits_context_compaction_items -- --nocapture`.
- Passed: `python3 scripts/check_ci_docs.py`.
- Passed: `git diff --check`.
- Passed: `sha256sum -c .pipeline/release/v0.140.0-vsp.8/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`.

## Release Asset

- Release tag/name target: `v0.140.0-vsp.8`.
- Archive: `.pipeline/release/v0.140.0-vsp.8/vsp-codex-preview-linux-x64-gnu.tar.gz`.
- SHA256: `973471e85537fa2a1bf8ba568e3df6027673ec71b1039041d9a97b084ffb9704`.

## Risks And Follow-Up

- Core auto-compaction focused tests overflow the default Tokio worker stack in this local test binary, but both pass with `RUST_MIN_STACK=16777216`. This matches earlier app-server stack-size-sensitive validation and should be documented rather than treated as a behavior failure.
- Existing VSCode/plugin app-server processes may still run old deleted binaries until VSCode/Remote/plugin sessions are reloaded.
- Remote GitLab release migration is still required after commit/push: create/update `v0.140.0-vsp.8` release assets and retire or redirect the already-published `v0.140.0-vsp.0` naming.
