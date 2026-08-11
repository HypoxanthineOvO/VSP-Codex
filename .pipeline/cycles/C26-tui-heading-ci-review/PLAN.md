---
kind: plan
mode: plan
cycle: C26-tui-heading-ci-review
status: closed
updated: 2026-08-10T01:17:05+08:00
builds_on:
  - C23-tui-quote-render-stream-finalize-fixes
  - C25-vsp-11-1-alpha-3-release
progress: PROGRESS.md
execution: EXECUTION.md
---

# C26 TUI 标题与回复完整性 CI 审核

## 执行目的

对仓库全部测试与 CI 做独立、可追踪的 test-quality 审核，并以标题改动和 C23 最终回复合同作为高风险实证路径。审核不能只检查测试是否为绿色或失败是否为 baseline；必须逐项证明测试没有硬编码、同源 expected/actual、mock-only、弱断言、未控制环境或无效 snapshot，并映射 CI job 实际覆盖。

## Discover

- 当前 CLI、app-server 与本 Session 的 TUI 均运行 `0.145.0-vsp.11.1-alpha.3`；版本未更新不是主要原因。
- alpha.3 包含 C23 的 canonical suffix reconciliation，但当前 Session rollout 中上一条最终回复完整持久化，用户看到的 TUI 仍疑似缺尾。
- 现有 suffix 回归直接调用内部 finalize 方法，没有经过真实 `AgentMessageDelta -> ItemCompleted -> TurnCompleted` 顺序，也没有断言最终 VT100/terminal scrollback。
- 标题新改动仅存在于源码工作区，尚未 build、package 或 install。
- 全量 TUI 的 47 个失败尚未在同一环境下与 clean alpha.3 做严格集合差分。

## Technical And Architecture

```text
provider/app-server
  -> AgentMessageDelta
  -> ItemCompleted(canonical AgentMessage)
  -> TurnCompleted
  -> ChatWidget reconciliation
  -> InsertHistory + Consolidate
  -> terminal scrollback
```

审核逐层比较协议 canonical 文本、rollout 持久文本和终端最终可见文本。任何一层缺尾都必须定位到明确边界，不得用内部 helper 的绿色结果替代端到端证据。

## Builds On

- `C23-tui-quote-render-stream-finalize-fixes`：继承 Quote、stream finalize、canonical suffix 和 Markdown 行为合同，不继承旧任务或“已修复”的结论。
- `C25-vsp-11-1-alpha-3-release`：继承 alpha.3 不可变发布、三机部署与 GitLab CI success 基线，不继承发布、安装或 Stable 任务。

## 执行边界

- 包含：仓库全部 test targets、test files、snapshot、fixture/helper 和 CI workflow/job/command/shard inventory；每项必须有 audited 状态或明确不适用理由。
- 包含：标题 diff、真实回复尾部截断、协议事件顺序、history consolidation、terminal scrollback、Unicode/OSC 8、主题色及相关实证路径。
- 包含：硬编码、mock-only、弱断言、同源 expected/actual、固定路径、sibling checkout、locale/version/time/network/global-state 依赖、重复或低价值 snapshot 和 test-only production branch 审计。
- 包含：clean alpha.3 与当前工作区在相同命令、环境和工具版本下的失败集合差分。
- 不包含：长上下文增量渲染性能改造、Provider 功能、批量接受无关快照或顺带修复所有历史失败。
- 不执行 commit、push、package、release、install、remote deploy 或 service restart；S2 若需要候选安装/重启，届时单独请求授权。
- 不修改受保护的 `.pipeline/state.yaml`、`.pipeline/cycle.yaml`、`.pipeline/rules.yaml`。
- 不以“clean/current 都失败”或“不是本轮新增”为质量豁免；M5 相关 package/CI 必须零未豁免失败。

## Reviewer 隔离

- Reviewer A：真实 app-server/TUI/rollout/terminal 数据流与截断根因，只读审查。
- Reviewer B：测试可信度、硬编码、同源 oracle、mock-only 与 snapshot 风险，只读审查。
- Reviewer C：clean alpha.3 对照、47 个失败和 CI/Clippy 归因设计，只读审查。
- Reviewer D：标题布局、Unicode/OSC 8、主题语义与快照合同，只读审查。
- Reviewer 互不继承结论，不修改源码、不接受快照；主线程负责交叉核对矛盾与最终判断。

## 完整计划

| ID | 阶段 | 预期结果 | 验证方式 |
| --- | --- | --- | --- |
| `M1` | 聚焦 C26、冻结证据与双基线 | 明确 alpha.3 clean baseline、当前 heading diff、真实截断样本和 47 个失败集合 | 隔离 worktree；记录版本、commit、命令、环境、失败 test ID 与 snapshot 文件 |
| `M2` | 全部测试与 CI exhaustive audit | 产出完整 target/file/snapshot/job inventory 和逐项 test-quality findings | 自动枚举 + anti-pattern scan + 人工复核；reviewer 不编辑工作区、不共享预设结论 |
| `M3` | 主线程反向复核与重现 | 验证 inventory 无遗漏，复现硬编码/弱 oracle/环境依赖并给出处置分类 | source-to-target-to-CI 双向映射；代表性 RED；无法自动判断项逐个阅读 |
| `S1` | 人工审核 exhaustive audit 报告 | 展示完整数量、每项状态、findings、未检查项和修复建议 | 用户接受标准：不存在未解释测试面；没有把 baseline 当免责项；snapshot 逐项处置 |
| `M4` | 修复 S1 确认缺陷 | 修复测试设计、CI 分层和产品缺陷；默认 suite 只保留 hermetic tests | dedicated tests、真实事件路径、结构化 oracle、固定 fixture；跨仓库 compatibility 独立 job |
| `M5` | 零失败严格复验 | 相关 package 与 CI 零未豁免失败，Clippy/check/snapshot/diff 均通过 | CI-equivalent commands、全量、Clippy、pending snapshots 为零；豁免需用户逐项接受 |
| `S2` | 最终真实体验验收 | 展示长回复完整末尾、标题三主题与窄屏实际产物 | 用户确认无需 resume 即显示完整末尾；候选安装或重启须另行授权 |

## 主要逻辑与用户行为测试清单

- rollout canonical 完整时，live TUI 在 `ItemCompleted` 和 `TurnCompleted` 后显示完全相同的最后字符、换行与 Markdown 内容。
- delta 缺尾/final 完整、中途分歧、whitespace-only 分歧、final 缺失或为空、event 重复/延迟/乱序均有确定行为。
- 测试从公开 protocol notification 入口驱动，不直接把期望完整文本塞进被测 reconciliation helper 作为唯一证据。
- VT100/terminal scrollback 最终内容做整体比较，不只断言 `contains(suffix)` 或内部 event payload。
- H1 三行全宽 surface、H2 对称规则及窄屏同时隐藏、H3 emphasis 粗体在 Teal/Violet/Amber 下均有语义快照。
- 长标题、CJK、宽字符、长单词和 OSC 8 hyperlink 的显示列、居中与链接范围正确。
- Quote rail 与标题 surface 独立；流式、完成、resize 后标题合同一致。
- clean alpha.3/current 的失败使用 exact test ID/snapshot path 做集合差分；既有失败仍需证明同环境可复现。

## Proposal Gate

用户于 2026-08-09 确认并开始。M1-M3 自动推进；S1 展示完整审计证据后等待人工接受或拒绝。S1 通过后才执行 M4-M5；S2 保留最终真实体验验收。

2026-08-09 S1 用户选择扩大 M4：除推荐的最终回复/标题/弱测试修复外，将审计发现的 5 个 non-hermetic Workflow tests 一并改为 hermetic fixture。其余 42 个 baseline failures 仍不在 C26 批量清理范围。

2026-08-09 用户拒绝旧 M5/S2：原审计没有检查全部测试与 CI，且错误地把 baseline 归因当成质量通过。该拒绝撤销“42 个 baseline failures 可排除”的假设；C26 返回 M2，按 exhaustive inventory 与零失败门槛重新执行。

2026-08-09 用户恢复C26并要求该完成完成、该发布发布。M4-M5遵循已接受的C27测试分组纪律：选择相关行为组、修复相关失败，不以无目的full suite失败总数作为交付。C26产品与真实体验通过后，可进入下一不可变版本的候选、发布与既有三机目标验证；不得发布未通过M5/S2的候选。
