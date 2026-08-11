# Review 20260531 — 需求落实检查 + Audit + Quality

> 独立审计记录区，与正在收尾/已完成的 Cycle 状态隔离，不写入 `state.yaml` / `cycle.yaml`。
> 发起人：用户 | 日期：2026-05-31 | 时区：Asia/Shanghai | 语言：zh-CN

## 背景

C8 已发布 v0.134.0-vsp.6；C9 Post-Release Correctness Backfill（用户确认已做完，未发布）。本轮目标：
1. **需求落实检查**：核查 Chat 讨论里提出的需求，有多少没正确落实到 prompt 或代码。
2. **Audit**：当前代码工程审计。
3. **Quality**：代码质量评分。

## 三份报告

| 文件 | 内容 | 结论 |
|---|---|---|
| [gap-analysis.md](./gap-analysis.md) | Chat→Prompt→代码 追溯矩阵（回溯起点 C7） | 需求→prompt 基本可信；债集中在 prompt→产品闭环 |
| [audit.md](./audit.md) | 工程审计（Experience/Engineering/Risk） | PASS_WITH_WARNINGS：0 Critical / 6 Warning / 3 Info |
| [quality.md](./quality.md) | 1–5 质量评分卡 | Overall 3.5（门 FAIL），核心三项 PASS |
| [correctness-gaps.md](./correctness-gaps.md) | Done-wrong 闭环修复归档（2026-06-01） | item 2/6/8/9a + vscode scope 已闭环；item 4/5/7/9b、rollback 走 plan/待决策 |

## 一句话总结

> 你在 Chat 里提的需求几乎都正确进了 prompt（唯一真漏：update rollback）。真正的问题是 **prompt→产品闭环的最后一公里**：有 4 项（Workflow 卡片、Subagent 查看器译/侧栏、`/hw:*` palette、resume 闪烁抑制）代码在、测试绿、milestone 标完成，但生产入口是死代码 / 被禁用常量挡住 / 从没接线，用户实际用不到。另有 2 项（官方 Codex 双向兼容、Remote SSH 真实验收）范围被悄悄收窄。

## 优先处理建议

1. **high**：更新确认路由到 Workflow 卡片；据实修正"官方 Codex 兼容"对外文案。
2. **medium**：`/hw:*` 注入 command palette、viewer 侧栏宽度+翻译、resume 闪烁实启用、Remote SSH 实机验收存证。
3. **设计类**：滚动锚点、update rollback —— 走 `/hw:plan`。

## 校验口径声明

- 本轮为只读审计，未改动任何源码。
- 9 项追溯状态均经本人直接 grep/读码复核，更正了一处 subagent 误判（item 6 实为"存在但被削弱"，非"完全未实现"）。
- 未重跑全量 cargo 测试；正确性以 `state.yaml` 记录的近期通过项 + 本轮代码取证为依据。
