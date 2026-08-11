---
kind: progress
cycle: C25-vsp-11-1-alpha-3-release
plan: PLAN.md
status: closed
updated: 2026-08-08T21:59:50+08:00
current: completed
next: alpha.3 soak 后创建 CI validation Cycle，再发布 Stable
---

# C25 Progress

| ID | 阶段 | 状态 | 证据 | 下一步 |
| --- | --- | --- | --- | --- |
| M1 | 审计与准备 | completed | origin/main、Glab auth、Nod/Eden current 与 VSP process paths 已确认 | M2 |
| M2 | 版本与候选 | completed | CLI check、release build、builder 7/7、Bazel lock、format/docs/diff 通过 | 已完成 |
| M3 | 提交与打包 | completed | commit cd7921547e；archive/package/SHA smoke 通过 | 已完成 |
| M4 | 远端发布 | completed | main/tag/Release/download SHA 与 pipeline #346 success | 已完成 |
| M5 | 三机部署 | completed | 三机 current/hash/initialize 一致；Nod/Eden server alpha.3 | 已完成 |
| M6 | 报告与关闭 | completed | RELEASE-REPORT.md、SUMMARY.md | 已完成 |
