# C6 归档摘要

- Cycle：C6 `Compact 机制、上下文管理与 Workflow 定制化接入`
- 类型：feature
- 状态：completed
- 开始时间：2026-05-29T00:00:00+08:00
- 完成时间：2026-05-29T18:16:02+08:00
- 归档整理时间：2026-06-14T08:44:37+08:00
- Preset：tdd

## Milestones

- M1 Compact Fallback Context Pack：为空或不可用 compact 输出增加有界 `.pipeline` fallback context。
- M2 Structured Workflow File API：增加安全的 Workflow state/log/progress/subagent event 文件 helper。
- M3 Native Workflow Runtime Routing：增加 `.pipeline` guard 的 runtime guidance 和 `/hw:*` / `$hypo-workflow:*` routing helper。
- M4 Ordinary Conversation Capture：增加 root turn-completion conversation capture 到 Journal/Inbox/Mini-Cycle。
- M5 Workflow Command And UX Polish：增加 Workflow command/status helper surface 与文档更新。

## 关键数据

- 报告归档：`.pipeline/archives/C6-compact-workflow-runtime-customization/reports/`
- Prompt 归档：`.pipeline/archives/C6-compact-workflow-runtime-customization/prompts/`
- Review 归档：`.pipeline/archives/C6-compact-workflow-runtime-customization/reviews/C6/`
- 知识摘要：`.pipeline/archives/C6-compact-workflow-runtime-customization/knowledge-summary.md`
- 延后事项：0

## 完成说明

改动摘要：C6 完成 compact fallback、Workflow 文件 API、native command routing、普通对话捕获和 Workflow command/status polish。

技术思路：沿用 `.pipeline` 作为状态来源，用结构化 helper 限定写入边界，并让 native runtime guidance 只在 Workflow 项目中生效。

修改文件/模块：历史归档整理仅移动和补齐 `.pipeline/archives/C6-*` 下的运行态产物，没有修改业务代码。

测试设计：原 Cycle 聚焦测试覆盖 compact fallback、workflow files、workflow routing、conversation capture 和 TUI workflow slash routing。

验证结果：根据归档报告，focused tests、`cargo fmt --check` 和 `git diff --check` 在 C6 收尾时通过。

预期结果：C6 历史现在可通过 archive 目录追踪 prompt、report、review 和 reconstructed cycle metadata。

遇到的问题：C6 原完成时未按 Cycle close 流程生成 archive；本次依据现存报告和 PROGRESS 重建元数据。

风险/后续：开始时间为重建估计值，完成时间来自现存 PROGRESS/report 证据。
