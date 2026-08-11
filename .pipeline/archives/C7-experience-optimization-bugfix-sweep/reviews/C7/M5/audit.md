# C7/M5 Audit Review

## Verdict

pass

## Findings

无阻塞发现。

## Review Scope

- `codex-rs/core/src/workflow_routing.rs`
- `codex-rs/core/tests/suite/workflow_routing.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/workflow_cards.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/snapshots/*workflow_*`
- `.pipeline/reviews/C7/M5/test.md`
- `.pipeline/reviews/C7/M5/implement.md`

## Review Result

- Workflow runtime guidance now uses mandatory wording: required user decisions must use `request_user_input` when available.
- The six required semantic categories are explicitly named: plan confirmation, phase transition, scope selection, revision/rejection, authorization-intent, and release/update confirmation.
- Confirmation, authorization-intent, and real permission grants remain distinct. The guidance and card copy state that authorization-intent is not a permission grant, and true grants remain routed through `request_permissions` or the approval overlay.
- Ordinary non-Workflow chat is not forced into Workflow cards because runtime guidance is injected only for `.pipeline/` projects and tests cover non-Workflow omission.
- zh-CN card templates and snapshots cover approve/revise/cancel, permission-boundary wording, and release/update choices.
- Test and implementation reports match current validation evidence. The earlier implement-side snapshot failure was resolved by the test worker's GREEN snapshot closure.

## Validation

- `cd /home/heyx/Codex-VSP/codex-rs && cargo test -p codex-core workflow_interaction -- --nocapture`
  - 通过：3 passed / 0 failed。
- `cd /home/heyx/Codex-VSP/codex-rs && cargo test -p codex-tui workflow_ -- --nocapture`
  - 通过：TUI workflow unit tests 4 passed，workflow routing tests 6 passed。
- `cd /home/heyx/Codex-VSP/codex-rs && cargo fmt --check`
  - 通过，退出码 0。
  - 仅观察到既有 stable rustfmt 对 nightly-only `imports_granularity = Item` 的 warning。
- `cd /home/heyx/Codex-VSP && git diff --check`
  - 通过，无输出。

## Residual Risks

- TUI helper currently provides dedicated zh-CN templates for plan confirmation, authorization-intent, and release/update confirmation. Phase transition, scope selection, and revision/rejection rely on core guidance plus generic `request_user_input` card capability rather than dedicated helper/snapshot coverage. This is acceptable for M5 but should be considered if later milestones need specialized card copy.
