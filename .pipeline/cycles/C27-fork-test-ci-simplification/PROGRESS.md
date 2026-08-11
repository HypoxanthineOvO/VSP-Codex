---
kind: progress
cycle: C27-fork-test-ci-simplification
plan: PLAN.md
status: completed
updated: 2026-08-09T21:30:08+08:00
current: closed
next: none
---

# C27 Progress

## 当前状态

C27 已创建并聚焦。本 Goal 连续执行，不设置中间人工审阅点；C26 保持暂停，其未验收产品改动作为固定工作区输入保留。

| ID | 阶段 | 状态 | 当前证据 | 下一步 |
| --- | --- | --- | --- | --- |
| M1 | 基线与成本 | completed | GitLab required gate仅1个job；最近success约4m12s；已有binary的完整smoke为1.27s；GitHub/Bazel不进入GitLab | 已完成 |
| M2 | 测试组合精简 | completed | 删除22 orphan/redundant snapshots与3个无价值测试；修正1个abort假通过；4个外部合同改手工、3个本仓合同保留默认 | 已完成 |
| M3 | 快速 CI 实现 | completed | GitLab保持单job build+smoke并设10m timeout；新增`just vsp-ci` | 已完成 |
| M4 | 本地与手工入口 | completed | 14组manifest、runner、Just入口、AGENTS选择/失败纪律已落地；focused Rust使用no-tests=fail | 已完成 |
| M5 | 最终验证与报告 | completed | 13个可执行行为组、4类误用gate、format/lint/CI YAML均通过；RESULT v2完成 | 等待最终验收 |

## 时间线

| Time (Asia/Shanghai) | State | Detail |
| --- | --- | --- |
| 2026-08-09 20:22 | started | 用户确认新建独立 C27，并以一个不中途审清单的 Goal 连续精简测试与 CI；C26 保持暂停。 |
| 2026-08-09 20:35 | implemented | M1完成并进入M2：确认required耗时来自Rust冷编译而非tests；保留零GitLab成本的upstream GitHub/Bazel。完成日常/全量/release入口分层，并清理已确认无价值测试与snapshot。 |
| 2026-08-09 20:51 | waiting-review | C27 Goal完成：单job/10m required gate、focused/full/release入口、22 snapshot清理、3个无价值test删除、SDK假绿/binary修正、external suite分层与完整验证已完成。 |
| 2026-08-09 21:00 | rejected | 用户拒绝v1：本地测试没有按用途分组，也没有给AI明确选择和失败处理规则；全量测试仍可能产生“报一堆错但不修”的无效循环。返回M4修订。 |
| 2026-08-09 21:19 | implemented | M4修订完成：14个用途组、机器可读manifest、统一runner和AGENTS规则落地；focused groups强制filter且0 tests失败，external强制env，全量强制reason。进入M5复核。 |
| 2026-08-09 21:23 | waiting-review | v2验证完成：13个可执行组实际通过；upstream-all只做reason dry-run；误用gate、format/lint/CI lint通过。重新提交Goal最终验收。 |
| 2026-08-09 21:30 | accepted | 用户接受C27 v2最终结果；SUMMARY写入，Cycle关闭。 |
