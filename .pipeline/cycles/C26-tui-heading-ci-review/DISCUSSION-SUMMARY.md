---
kind: discussion-summary
cycle: C26-tui-heading-ci-review
status: closed
updated: 2026-08-10T01:17:05+08:00
---

# C26 Discussion Summary

## 已确认决定

- 用户确认先进入独立的“CI 审核”Cycle。
- 审核对象是刚完成的 TUI Markdown 标题改动；标题实现本身不自动视为已通过最终审核。
- C26 使用全新任务列表，只选择性继承 C23 的渲染合同与 C25 的发布后 CI 基线。
- 用户确认严格 CI 归因 Proposal 并开始执行，明确要求多个 subagent 独立交叉审核硬编码、不应存在或低价值的测试、mock-only 路径、弱断言和 snapshot 掩盖。
- “版本未更新”已被当前进程与 binary 证据排除；当前 rollout 最终 assistant message 完整而用户可见 TUI 疑似缺尾，真实协议到 terminal scrollback 的链路是阻塞性审计目标。
- 用户接受 S1 审计结论，并选择 expanded M4：除推荐修复外，一并修复 5 个 non-hermetic Workflow tests；其余 42 个 baseline failures 不纳入。
- 用户随后明确拒绝 M5/S2 结论：C26 的核心任务是检查全部测试（包括全量测试与 CI）是否存在硬编码、同源 expected/actual、mock-only、弱断言、环境依赖和无效 snapshot；旧工作只做局部检查与失败集合归因，不合格。
- 用户授权“重新做一遍”。C26 回退 M2，必须建立全部 test target/snapshot/CI job 的可追踪 inventory 并逐项人工复核；旧 M5 报告不可用于验收。
- 用户接受 redo 后的新 S1 exhaustive audit，但要求先暂停。接受的是审计结论和分阶段方向，不是当前产品/测试状态；没有授权本轮进入源码修复。
- 用户随后明确恢复C26并要求“该发布发布、该完成完成”，授权完成M4/M5/S2以及通过验证后的不可变发布和安装。
- alpha.4真实安装发现session migration兼容问题后，保持alpha.4不可变，修复后发布alpha.5；没有移动既有tag。

## 范围边界

- 审核标题 diff、C23 final-message 完整性合同、相关测试、快照、格式、check/lint 和 clean/current 双基线归因。
- S1 前只读审计和构造证据；S1 接受后才修复确认缺陷。
- 原始S1范围不含发布/安装；用户恢复时明确扩大到验证通过后的发布与安装。全程未重启service。
- exhaustive audit 覆盖仓库 CI 实际运行与应运行的全部测试面；真实跨仓库 Workflow compatibility 必须与默认 hermetic suite 分层评估。

## 当前检查点

- `M2-M3`：全部测试、snapshot 与 CI 的 exhaustive audit 与主线程复核均已完成，源码语义未检查项为 0。
- `S1`：新报告已接受。
- `M4/M5/S2`：全部完成并通过。
- `Release`：alpha.5 commit/tag/Release/asset/remote SHA/公开installer/本机entrypoints均验证通过。
- `Cycle`：关闭；完整结论见`SUMMARY.md`。

## 恢复授权

- 用户询问C26为何暂停，并明确要求“该发布发布、该完成完成”。
- C26恢复到M4.1；C27已接受的测试分组与失败责任规则适用于后续验证。
- 发布授权仅在C26产品修复、M5和真实S2通过后使用；不发布未验收候选。

## Redo S1 Acceptance

- 接受的产物：`M2-EXHAUSTIVE-INVENTORY.md` 与新版 `S1-AUDIT-REPORT.md`。
- 接受的结论：当前测试与 CI 不能作为 C26 验收证据；327 个实际失败、Cargo/Bazel blockers、silent-pass、弱 oracle、mock-only 和平台缺口都必须进入后续处置。
- 未接受：当前产品/测试状态、任何 failure 豁免、旧 42-failure baseline 排除。
- 未授权：本轮源码/CI/snapshot修复、临时目录删除、commit、push、package、release、install、deploy 或 restart。

## 历史旧 S1 Acceptance（已失效）

- 接受的产物：`S1-AUDIT-REPORT.md`、四路 reviewer findings、独立 target 双基线和 exact JUnit fingerprint 对照。
- 接受的修复范围：最终回复/图像边界、ordered/canonical recovery、标题 Setext/宽度/容器、弱测试，以及 5 个 hermetic Workflow fixture 修复。
- 未授权：commit、push、package、install、deploy、restart 或其余 baseline failures 批量清理。
