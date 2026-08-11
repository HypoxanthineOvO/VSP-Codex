# Quality Report — 2026-05-31

> 语言：zh-CN | 时区：Asia/Shanghai

## Intake

- **Scope**：C7/C8/C9 改动涉及的 codex-rs 模块（tui、app-server、rollout、core、cli）。
- **Mode**：scorecard（首次评分，可作为后续 compare 的 baseline）。
- **Correctness constraints**：近期通过的单测/快照（journal/state 记录）+ 本轮直接取证。
- **Validation path**：定向 cargo test（见 state.yaml `follow_up.validation` 列表）+ 引用计数/调用图核查。

## Scorecard

| 维度 | 分 | 证据 | 行动 |
|---|---:|---|---|
| **Correctness（核心）** | 4 | provider 规范化(`turn_processor.rs:52-60`)、Plan 计数拆分、freshness 指纹(`session/mod.rs:436`)均有针对性单测/快照；无发现错误核心行为 | 维持 |
| **Maintainability（核心）** | 3 | 命名清晰、模块边界合理；但存在死代码/禁用常量(`workflow_cards.rs` 文件级 `allow(dead_code)`、viewer `MIN_WIDTH=u16::MAX`、`suppress_*` 恒 false)，给读者造成"已接线"的误读成本 | 清死代码、给禁用常量加 why 注释或移除 |
| **Structure/Organization（核心）** | 4 | tui/app-server/rollout/core 分层清楚；`workflow_slash_routing` 数据与消费分离但消费端缺失 | 接线消费端 |
| Test Quality | 3 | 单测/快照充分且非占位；**但 test-only seam 冒充生产覆盖**（workflow_cards、with_release_channel_metadata 只在测试触达），通过的测试无法证明生产可用 | 增加"生产入口可达性"测试 |
| Complexity | 4 | 无明显高复杂度热点 | 维持 |
| Observability/Operability | 4 | `vscode status --json`、probe log、freshness reminder、Workflow state 记录到位 | 维持 |
| Documentation/Onboarding | 3 | docs/README 已更新到 vsp.6；但"官方 Codex 兼容"表述强于实际（单向导出） | 据实修正 |
| Performance | 4 | 更新检查后台 spawn 非阻塞；无明显热路径浪费 | 维持 |

## Overall

- **Score：3.5 / 5**（核心三项 4/3/4）
- **Gate：FAIL**（未达 Overall ≥ 4）
- **Core gate：PASS**（核心三项均 ≥ 3）

> 未过质量门的主因不是正确性，而是 **Maintainability 与 Test Quality 被"完成度名实不符"拖累**：代码在、测试绿，但生产闭环断点 + test-only seam 让"完成"打了折。修掉 Action Queue 里 4 项接线缺口后，Maintainability/Test Quality 可回到 4，Overall 上 4。

## Compare

- **Baseline**：本报告即基线（`.pipeline/review-20260531/quality.md`）。后续修复后用 `/hw:quality compare` 对照。

## Action Queue

| Action | Route | Priority | Validation |
|---|---|---|---|
| 移除/接线 `workflow_cards` 死代码 | /hw:patch | high | 去掉文件级 allow(dead_code) 仍编译通过（=已被生产引用） |
| viewer 侧栏宽度 + prompt/report 翻译 | /hw:patch | medium | 宽终端并排视图出现 |
| `/hw:*` 注入 command palette | /hw:patch | medium | 面板可选到条目 |
| 生产入口可达性测试规范 | /hw:plan | medium | 新增检查防止 test-only seam 再冒充完成 |

## Escalations

- **Audit escalations**：RISK-01（对外文案与单向导出能力不符）已在 `audit.md` 记录，按治理优先处理。
- **Optimize candidates**：4 项接线缺口适合用 `/hw:optimize` 在正确性预算下批量收口。
- **Patch candidates**：workflow_cards 接线、viewer 宽度、palette 注入（均小范围）。
- **Plan candidates**：resume 闪烁实启用 + 烟测、滚动锚点、update rollback（需设计）。
