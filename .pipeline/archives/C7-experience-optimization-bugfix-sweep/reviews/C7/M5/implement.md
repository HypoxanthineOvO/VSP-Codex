# C7/M5 Implement Worker Report

## Scope

- 接手首个 implement worker 留下的 M5 生产代码改动。
- 只检查并使用允许写入范围：`workflow_routing.rs`、`request_user_input/mod.rs`、`workflow_cards.rs` 和本报告。
- 未编辑测试文件、snapshot、`.pipeline/state.yaml`、`.pipeline/log.yaml` 或 `.pipeline/PROGRESS.md`。

## Changed Files

- `codex-rs/core/src/workflow_routing.rs`
  - 确认 Workflow runtime guidance 已使用 mandatory wording：required user decisions must use `request_user_input`。
  - 确认 guidance 明确列出六类卡片语义：plan confirmation、phase transition、scope selection、revision/rejection、authorization-intent、release/update confirmation。
  - 确认 confirmation、authorization-intent、`request_permissions`、approval overlay、not a permission grant 的边界说明齐全。
  - 确认 ordinary non-Workflow chat 不会被强制进入 Workflow cards。
- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
  - 保留并确认 `workflow_cards` 模块公开给同模块测试使用。
- `codex-rs/tui/src/bottom_pane/request_user_input/workflow_cards.rs`
  - 确认提供 `plan_confirmation_zh_cn`、`authorization_intent_zh_cn`、`release_update_confirmation_zh_cn` helper。
  - 确认 authorization-intent 文案说明“这不是权限授予”，真实权限仍由 `request_permissions` 或 approval overlay/审批机制处理。
- `.pipeline/reviews/C7/M5/implement.md`
  - 本报告。

## Validation

- `cd /home/heyx/Codex-VSP/codex-rs && cargo test -p codex-core workflow_interaction -- --nocapture`
  - Result: pass. 3 targeted workflow interaction tests passed.
- `cd /home/heyx/Codex-VSP/codex-rs && cargo test -p codex-tui workflow_ -- --nocapture`
  - Result: fail at snapshot assertion only.
  - The production helpers compile and the zh-CN substring assertions pass before snapshot comparison.
  - Remaining failures are missing/unaccepted snapshots for:
    - `workflow_plan_confirmation_card_zh_cn`
    - `workflow_authorization_intent_card_zh_cn`
    - `workflow_release_update_card_zh_cn_narrow`
  - Accepting or writing snapshot files is outside this worker's allowed write boundary.
- `cd /home/heyx/Codex-VSP/codex-rs && cargo fmt --check`
  - Result: pass. Rustfmt emitted existing unstable `imports_granularity = Item` warnings only.
- `cd /home/heyx/Codex-VSP && git diff --check`
  - Result: pass.

## Notes

- I did not run `cargo insta review` or accept `.snap.new` files because snapshots are test artifacts and the task explicitly forbids editing tests or snapshots.
- No unrelated worktree changes were reverted or overwritten.
