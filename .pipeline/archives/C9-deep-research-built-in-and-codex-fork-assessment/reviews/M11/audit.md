# C9 M11 Audit Review - TUI CLI VSCode And User Surface Convergence

审计时间：2026-06-26T17:32:22+08:00

## Verdict

**PASS with follow-up risks.**

当前 M11 reviewed diff 中没有剩余阻断项。聚焦用户面 baseline 已恢复：Workflow slash routing、Question Tool/request_user_input cards、VSCode Remote install/status/rollback、`vsp-codex` identity/update policy 和 `git diff --check` 均有通过证据。

## 审计依据

- `codex-rs/cli/src/main.rs`
  - plain `vsp-codex update` 在非 manifest update 路径上先调用 VSP official-service-disabled guard，再进入 debug-build updater guard。
  - 显式 `--from-vsp-manifest` 不被该 guard 不合理拦截，仍保留 VSP manifest update path。

- `codex-rs/cli/tests/update.rs`
  - plain debug-build `vsp-codex update` 测试同步为 VSP official-service-disabled policy，并断言不会回退到 `not available in debug builds` 消息。

- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
  - request_user_input remapped interrupt snapshot 使用 test-only trim helper 去除 trailing ratatui buffer padding。
  - 变更只作用于单个 stale snapshot，未批量接受 request_user_input 或 broad TUI snapshots。

- `codex-rs/tui/src/bottom_pane/request_user_input/snapshots/...freeform_remapped_interrupt.snap`
  - 可见语义保持为 `enter to submit answer | f12 to interrupt`。

- `.pipeline/reviews/C9/M11/test.md` 与 `.pipeline/reports/C9/M11-user-surfaces.md`
  - 记录 baseline blocker、窄修、复跑结果和 broad TUI nextest 风险。

## 验证复核

已通过的 focused evidence：

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui workflow_slash_routing -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui request_user_input -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vsp_entrypoint -- --nocapture
cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test update -- --nocapture
git diff --check
```

结果摘要：

- `workflow_slash_routing`: 9 passed。
- `request_user_input`: 89 passed。
- `vscode`: 16 passed。此前 `Text file busy` 复跑消失，归类为瞬时环境问题。
- `vsp_entrypoint`: 8 passed。
- `update`: 1 passed。
- `git diff --check`: passed。

## Remaining Risks

- Broad `cargo nextest run --manifest-path codex-rs/Cargo.toml --no-fail-fast -p codex-tui` 仍有历史失败记录：2936 passed / 76 failed / 4 skipped，并包含 snapshot failures 与 stack overflow aborts。该结果不能作为 release-clean evidence。
- Broad TUI snapshot failures 应在 M12 release-candidate dry run 或后续专门 UI convergence 中按用户面类别审查，不能机械接受。
- Stack overflow failures 仍需 separate harness/root-cause triage，尤其是 TUI app tests。
- M12 应重新检查 release-facing update/install 行为，尤其是 release-mode VSP manifest update path。

## Completion Report

- 改动摘要：完成只读 M11 审计，确认 focused M11 blockers 已解除；未修改源码。
- 技术思路：按 M11 prompt 的 user-facing preservation contract，复核 CLI update policy、request_user_input snapshot 语义、VSCode Remote coverage、Workflow slash routing 和 broad TUI 风险。
- 修改文件/模块：仅新增本审计报告。
- 测试设计：审计复核 test/implement worker 与主编排的 focused validation evidence。
- 验证结果：PASS with follow-up risks。
- 预期结果：M11 可推进 M12，但 broad TUI nextest 不得被表述为 clean。
- 问题：未运行新命令；基于已记录证据和 diff 只读审计。
- 风险/后续：M12 必须把 broad TUI nextest、stack overflow、release-mode update/install 行为作为 release-candidate 风险处理。
