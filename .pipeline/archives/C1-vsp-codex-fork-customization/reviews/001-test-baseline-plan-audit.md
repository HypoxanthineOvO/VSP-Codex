# P3 Plan Audit: Milestone 1 Test Baseline Prompt

时间：2026-05-24T20:24:00+08:00
审核 Subagent：Maxwell (`019e59c3-a311-73d2-a0cc-2ca046923aa7`)
结论：通过。

## 审核结论

`.pipeline/prompts/001-test-baseline.md` 符合 P3 Generate 对 Milestone 1 的规划边界要求。

## 核对结果

- 只生成并约束 Milestone 1：明确禁止开始 `vsp-codex` 改名、`~/.vsp-codex` home、`/provider`、官方服务禁用、品牌清理或汉化实现，没有越界到 M2-M6。
- 完整保留 P2 的 M1 核心内容：包含 `technical_solution`、`technical_route`、`research_required`、`risks_and_alternatives`、`validation_path`、`audit_focus`，且语义与 P2 一致。
- strict Worker Separation 合格：明确 `test`、`implement`、`audit` 三角色，写入路径分离，职责基本不重叠，并要求主代理记录 lifecycle 状态。
- 测试优先要求合格：强调先建立真实测试基线，记录命令、耗时、失败、blocked/not-run，拒绝只编译或无关测试冒充验收。
- TUI 用户模拟测试调查合格：覆盖 snapshot、VT100、tmux ignored smoke，并要求判断它们是否可作为真实 TUI 用户模拟基线。
- 没有要求修改源码或启动功能改造：明确禁止修改业务源码、测试断言、snapshots、manifest、lockfile。

## 已采纳建议

- 将 `implement` 角色收紧为仅记录 blocker 或生成 implement review，不创建/修改任何可执行脚本。
- 将 worker lifecycle 证据落点明确为 `.pipeline/reports/test-baseline.md` 的 `Worker Lifecycle` 小节以及各 review 文件。
