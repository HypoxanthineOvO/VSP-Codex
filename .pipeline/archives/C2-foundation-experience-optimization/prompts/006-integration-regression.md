# M6 集成回归与实际验收

## Goal
完成 C2 的集成回归、实际 TUI 验收和收尾修复。确认默认 YOLO、Subagent 策略与面板、Hypo-Translate、Context Window、MCP 提示迁移、静态汉化能一起工作，并准备进入人工验收。

## Subworker Assignment Plan
Subworkers are authorized for this milestone. Worker separation is strict.

- `test`: 作为主验收 worker，运行 scoped tests、snapshot pending 检查、实际 TUI smoke、跨仓库启动 smoke，整理失败证据。测试 worker 可提出修复需求，但不直接改产品逻辑。
- `implement`: 只处理集成阶段发现的缺陷和小修复；不能新增超出 C2 范围的新功能。
- `audit`: 做最终代码审核，重点查行为回归、测试空洞、未记录风险、CI 影响和 merge 成本。
- Main agent: 汇总证据、决定是否需要用户批准 full test、生成 acceptance report。

## Technical Solution
将前五个 Milestone 的功能组合起来验证，而不是只验证单点：

- 从任意仓库直接运行 `vsp-codex`，默认进入 TUI，默认 YOLO。
- 底栏同时显示 model/provider/path/git/context window，窄屏不重叠。
- Subagent 无显式授权即可使用，最多 32，中文名字池生效，panel 显示 active agents，详情可键盘查看。
- Hypo-Translate 可用时翻译三类动态内容，不可用时 passthrough。
- MCP 普通启动提示不污染主 Working 区，失败仍可见。
- 静态高频文案已汉化，专业术语保留英文。

## Technical Route
1. 收集 M1-M5 的测试命令和未决风险。
2. 运行所有已改 crate 的 scoped tests；如 core/common/protocol 改动较大，向用户请求 full `just test` 授权。
3. 检查 `cargo insta pending-snapshots -p codex-tui`，确认没有未接受 snapshot。
4. 运行实际 TUI smoke：在当前 repo 与另一个 repo 启动 `vsp-codex`，检查默认路径和 UI。
5. 运行 Subagent smoke，确认 panel、中文名、状态和详情。
6. 运行 Hypo-Translate available/unavailable smoke。
7. 根据测试/audit 结果修小缺陷，重新跑受影响 scoped tests。

## Research Required
不需要新 research。需要引用 M1-M5 报告和本地实际测试结果。

## Risks And Alternatives
- 风险：full workspace tests 太慢或需要用户授权。替代方案是先跑受影响 crate 的 scoped tests，并明确未跑 full test 的残余风险。
- 风险：实际 TUI smoke 难以自动化。替代方案是使用现有 `test-tui` 指南和 snapshot/terminal evidence 组合。
- 风险：多功能同时改动导致 regression 定位困难。替代方案是按 Milestone feature flag/commit 范围逐项 bisect。

## Validation Path
- Required: `just fmt` after final Rust edits.
- Required: scoped `just test -p ...` for each changed crate.
- Required for TUI snapshots: `just test -p codex-tui`, `cargo insta pending-snapshots -p codex-tui`.
- Conditional: ask user before full `just test` if shared/core/protocol changes make full suite appropriate.
- Manual: actual `vsp-codex` launch from another repository and Subagent/Hypo-Translate smoke.

## Audit Focus
- C2 behavior works as an integrated product experience, not just isolated tests.
- No official Cloud/OAuth/Remote cleanup regression from C1.
- CI remains bounded and does not produce noisy failure emails.
- Merge-forward cost is recorded if changes diverge from upstream Codex architecture.

## Deliverables
- Final regression report with command evidence.
- Any small integration fixes.
- Acceptance summary for user review before C2 close/sync/docs/release.
