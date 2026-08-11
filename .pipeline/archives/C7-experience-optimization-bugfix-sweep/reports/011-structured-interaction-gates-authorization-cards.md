# C7/M5 Structured Interaction Gates And Authorization Cards

完成时间：2026-05-30T05:36:14+08:00

## 结果

M5 已完成。Workflow required decisions 现在被明确要求使用 `request_user_input`，并且把普通确认、authorization-intent 和真实权限授予分清楚。TUI 侧新增了三类 zh-CN Workflow card helper 和对应 snapshots，覆盖计划确认、授权意图确认和发布/更新确认。

## 主要变更

- `.pipeline/` Workflow runtime guidance 明确要求 required user decisions must use `request_user_input`。
- Guidance 明确列出六类 Workflow card semantics：plan confirmation、phase transition、scope selection、revision/rejection、authorization-intent、release/update confirmation。
- Guidance 和 card copy 明确说明 authorization-intent 不是 permission grant；真实权限仍由 `request_permissions` 或 approval overlay 处理。
- 新增 `request_user_input::workflow_cards` helper：
  - `plan_confirmation_zh_cn`
  - `authorization_intent_zh_cn`
  - `release_update_confirmation_zh_cn`
- 新增并接受三张 TUI snapshots，确认 zh-CN 卡片在普通宽度和窄宽度下保留核心选项与权限边界文案。
- 保留普通 non-Workflow chat 的轻量行为；非 `.pipeline/` 项目不会注入 Workflow card guidance。

## 验证

- `cargo test -p codex-core workflow_interaction -- --nocapture`：3 项通过。
- `cargo test -p codex-tui workflow_ -- --nocapture`：TUI workflow unit tests 4 项通过，workflow routing tests 6 项通过。
- `cargo fmt --check`：通过；仅有既有 stable rustfmt 对 nightly-only `imports_granularity = Item` 的 warning。
- `git diff --check`：通过。

## 审计

只读 audit worker 孟轲 the 16th 返回 `pass`，无阻塞发现。审计确认 mandatory request_user_input wording、六类 card semantics、authorization-intent 与真实权限边界、ordinary non-Workflow chat 非回归、zh-CN snapshots 和测试证据均满足 M5 验收要求。

## 非阻塞项

- 当前专用 zh-CN helper 只覆盖 plan confirmation、authorization-intent 和 release/update confirmation。phase transition、scope selection、revision/rejection 依赖 core guidance 与通用 `request_user_input` 能力；后续若需要更细的文案一致性，可继续补专用 helper/snapshot。
