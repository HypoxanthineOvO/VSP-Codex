---
archived_source: .pipeline/archives/C9-deep-research-built-in-and-codex-fork-assessment
cycle: C9-deep-research-built-in-and-codex-fork-assessment
finished: 2026-06-26T19:12:00+08:00
kind: cycle-summary
started: 2026-06-14T08:44:37+08:00
status: closed
---

# Deep Research Built-In And Codex Fork Assessment 总结

## 目的与边界

Upgrade the VSP-Codex upstream base to `rust-v0.140.0-alpha.18`, preserve VSP runtime and release surfaces, publish a GitLab release after validation and explicit gates, produce a Deep Research Workflow requirement brief, and append a follow-up plan to sync toward `rust-v0.143.0-alpha.25` with SQLite persistent-log runtime validation while preserving the VSP vsp.8 series.

## 最终结果

旧 Cycle 状态：`completed`。本预览不重新判断旧结果，只建立可读导航。

## 验证结果

- 12 个历史 milestone 已映射。
- 56 个旧文件继续保存在 `.pipeline/archives/C9-deep-research-built-in-and-codex-fork-assessment`。

## 重要决定与经验

- 旧 `cycle.yaml` 没有结构化 lessons；请查看原始 Summary。

## 后续候选

- 不自动继承旧任务；新 Cycle 应通过 `builds_on` 选择需要的结果或经验。

## 映射缺口

- 无结构性缺失。

## 旧总结原文

# C9 归档摘要

- Cycle：C9 `Deep Research Built-In And Codex Fork Assessment`
- 类型：feature
- 状态：completed
- 开始时间：2026-06-14T08:44:37+08:00
- 完成时间：2026-06-26T19:12:00+08:00
- 归档整理时间：2026-07-02T16:03:40+08:00
- Preset：tdd

## Milestones

- M1 Checkpoint Current Workflow State And Upgrade Branch：固定升级前基线并创建 `c9-upstream-0.140.0-alpha.18` 分支。
- M2 Upstream Baseline Diff And Merge：合并上游 `rust-v0.140.0-alpha.18`，冲突全部解决，audit pass_with_risks。
- M3 Preserve VSP Runtime Surfaces：保留 VSP provider、CLI 命名、VSCode Remote、Workflow/Subagent 等表面，voice/realtime 与 remote-control 不保留。
- M4 Version Metadata And Release Surface Update：产出 `0.140.0-vsp.0` 版本元数据与发布包。
- M5 Broad Regression And GitLab Release：发布 `v0.140.0-vsp.0` 到 GitLab（含 Generic Package Registry 资产）。
- M6 Deep Research Workflow Requirement Brief：产出 design-only 需求简报与机器可读 handoff。
- M7 Conflict Inventory And Upstream Fix Baseline：alpha.25 冲突盘点与等价矩阵（replace 2 / preserve 8 / defer 7）。
- M8 SQLite Logging Convergence And Runtime IO Smoke：SQLite 持久日志收敛，运行时 smoke 0 噪声行，保留 VSP in-layer `target=="log"` guard。
- M9 Protocol Contract And App Server Bridge Merge：协议/app-server 契约安全对账，schema 无 diff。
- M10 Core Workflow Runtime And Adapter Preservation：goals runtime 接入 state-db，plugin disabled-remote 行为修复。
- M11 TUI CLI VSCode And User Surface Convergence：`update` 遵循 VSP official-service-disabled 策略，VSCode/TUI focused tests 通过。
- M12 Release Candidate Dry Run And GitLab Release Preparation：发布 `v0.140.0-vsp.8.1` 并完成本地分发。

## 关键数据

- 最终发布：`v0.140.0-vsp.8.1`（tag `v0.140.0-vsp.8.1`，commit `26ca6333792c`），GitLab release https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.140.0-vsp.8.1
- 归档后热修：v0.140.0-vsp.8.2（见根 `.pipeline/log.yaml` 与 debug 记录）
- 报告归档：`.pipeline/archives/C9-deep-research-built-in-and-codex-fork-assessment/reports/`
- Prompt 归档：`.pipeline/archives/C9-deep-research-built-in-and-codex-fork-assessment/prompts/`
- Review 归档：`.pipeline/archives/C9-deep-research-built-in-and-codex-fork-assessment/reviews/`
- Plan-state 快照：`.pipeline/archives/C9-deep-research-built-in-and-codex-fork-assessment/plan-state/`
- 知识摘要：`.pipeline/archives/C9-deep-research-built-in-and-codex-fork-assessment/knowledge-summary.md`
- 延后事项：0（12/12 Milestone 全部 completed）

## 完成说明

改动摘要：C9 将 VSP-Codex 上游基线升级到 `rust-v0.140.0-alpha.18`，随后通过 c9-007..c9-012 续作把 SQLite 持久日志、协议契约、core runtime、用户表面收敛到 alpha.25 等价行为（非完整合并），最终发布 `v0.140.0-vsp.8.1` 并完成本地分发；另产出 Deep Research Workflow design-only 需求简报。

技术思路：以等价矩阵（replace/preserve/defer)作为 preserve-by-default 边界；仅在等价性验证加运行时 smoke 通过后替换 SQLite 日志层；协议以 Rust contract 为 source of truth；发布走 GitLab Generic Package Registry 资产路径。

修改文件/模块：`codex-rs/state`（log_db 过滤）、`codex-rs/app-server`、`codex-rs/core`（goals runtime、plugins）、`codex-rs/cli`（update 策略、vscode）、`codex-rs/tui`、发布脚本与 manifest。

测试设计：focused cargo tests（workflow_routing、request_user_input、workflow_markdown_surface、vscode、vsp_entrypoint、sqlite filter）+ 运行时 SQLite IO smoke + release dry-run + 只读 audit 复核。

验证结果：M12 收尾时 release build、打包、sha256、远端下载、VSCode 17 tests、SQLite filter 8 tests、runtime smoke（0 rows）、本地三入口版本 `0.140.0-vsp.8.1`、GitLab main pipeline #216 全部通过。

预期结果：`.pipeline` 运行区清空供 C10 使用；`v0.140.0-vsp.8.1` 为已发布基线（后续热修至 vsp.8.2）。

遇到的问题：全量 workspace nextest 在本机因 libcap.pc 缺失与 app-server 集成套件超时保持历史性红色；上游 alpha.25 完整合并因大范围冲突主动放弃，改走等价收敛路线。

风险/后续：C9 不是完整 alpha.25 合并；VSCode Remote extension 自动更新会重新引入官方 bundled `codex`（已由 6-27 debug 热修加 repair timer 缓解，但属于 Cycle 外热修）；广域 TUI nextest 仍是 release-candidate 风险；Deep Research 仅为需求简报，未实现生产行为。
