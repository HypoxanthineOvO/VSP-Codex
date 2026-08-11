---
kind: plan
mode: goal
cycle: C27-fork-test-ci-simplification
status: completed
updated: 2026-08-09T21:30:08+08:00
builds_on:
  - C26-tui-heading-ci-review
progress: PROGRESS.md
execution: EXECUTION.md
---

# C27 Fork 测试与 CI 精简

## 目的

把 VSP-Codex fork 的日常质量门禁改成快速、明确、与实际交付一致的流程。普通提交只阻塞于当前源码和正式产物能否构建、打包、启动并进入最小正常交互；慢但有价值的覆盖移到本地或手工入口；重复、过时、假通过或没有独立判断价值的测试与 snapshot 删除。

## 已确认原则

- 用户将逐测试技术取舍委托给执行 Agent，不设置中间删除清单审阅点。
- 不因为测试当前失败就删除；删除必须基于功能已移除、重复、orphan、无独立 oracle、默认空跑或已有更短等价覆盖。
- 不因为测试慢就自动删除；高价值慢测试改为本地或手工运行。
- blocking CI 不追求复刻上游全平台测试矩阵。
- 数据库/会话兼容、更新器、发布包、公开协议、安全边界和最终回复完整性保留最小真实合同。
- 缺少运行条件不能伪装成 PASS；非 blocking suite 可以明确不运行。

## 执行边界

- 包含：Cargo/Bazel/test/snapshot inventory、GitHub/GitLab CI、仓库脚本和开发者测试入口。
- 包含：删除确认无价值的测试、snapshot 和仅为这些测试存在的辅助代码。
- 包含：建立快速 blocking CI 与 local/manual suites，验证当前 artifact 而非旧发布版本。
- 不包含：借精简之名修改无关产品功能，或逐项修复 C26 的 327 个失败。
- 不执行 commit、push、package publication、release、install、deploy 或 service restart。
- 不修改受保护的 `.pipeline/state.yaml`、`.pipeline/cycle.yaml`、`.pipeline/rules.yaml`。

## 完整阶段

| ID | 阶段 | 结果 | 验证 |
| --- | --- | --- | --- |
| M1 | 基线与成本 | 当前 CI 入口、测试规模、耗时、失败方式和交付入口冻结 | workflow/job/command inventory；代表性计时；artifact path 核对 |
| M2 | 测试组合精简 | 每类测试归入 blocking、local/manual 或 delete；低价值内容已删除 | source/target/CI 双向对账；无 orphan target/snapshot/helper |
| M3 | 快速 CI 实现 | 单一清晰 blocking 流程验证 current checkout/current artifact | clean build、package smoke、CLI/app-server/TUI startup |
| M4 | 本地与手工入口 | 测试按用途机器可读分组；AI能选择最小有价值组并对失败负责 | group manifest/runner/AGENTS；缺前置条件显式失败；全量要求reason |
| M5 | 最终验证与报告 | CI 更快且保证范围清楚，删除与残余风险可追踪 | 时间对比、build/package/start/critical contract、diff/schema/snapshot hygiene |

## 最小 Blocking CI 合同

- 当前 workspace/正式 binary 能编译。
- 当前发布包能生成并在临时目录解包。
- 包内 CLI `--version`/`--help` 可运行，入口名称正确。
- app-server 能启动并完成最小 initialize/handshake。
- TUI 能启动并进入正常交互首屏，不立即崩溃。
- 关键生成文件、Cargo/Bazel dependency lock 与 package metadata 不漂移。
- 少量高损失合同仍运行：会话/数据库兼容、更新器、公开协议、安全边界、最终回复完整性。

## AI 本地测试合同

- 测试分组必须说明purpose、适用改动、价值、命令、前置条件、是否blocking和失败处理。
- AI先选择最小相关组，不把workspace full suite当默认诊断。
- `upstream-all`只允许用户明确要求、共享test harness变更或全仓机械变更，并强制记录reason。
- AI主动运行的组出现失败时，必须修复相关失败或停止于真实阻塞；不得只汇报一份未处理失败清单。

## 授权

用户于 2026-08-09 明确确认以单一 `hw:goal` 连续执行，不要求中间查看删除清单。只有出现影响产品范围、用户数据安全或发布方式的重大新决定时中断。
