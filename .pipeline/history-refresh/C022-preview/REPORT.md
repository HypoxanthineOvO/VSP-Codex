# History Refresh 预览报告

## 结论

当前 9 个旧 Cycle 可以映射为新的只读语义 Cycle。预览为每个 Cycle 生成 Plan、Progress、Execution、Discussion Summary 和 Summary，但不复制 405 个旧归档文件；详细 prompts、reports 和 reviews 继续引用原路径。旧文件没有被修改，Manifest 也没有切换。

本预览**不具备激活授权**。当前仍有 1 个非终态 live Delivery，必须在 S2 明确处理。

## 源数据

| 类别 | 数量 |
| --- | ---: |
| `.pipeline` 源文件（排除 preview/local） | 1051 |
| 旧 Cycle | 9 |
| 旧 Cycle 文件 | 405 |
| Memory Records | 62 |
| 旧 Knowledge 文件 | 9 |
| 旧 Chat 状态文件 | 20 |
| Patch 文件 | 3 |
| PR 文件 | 30 |
| Live Delivery | 11 |

## 映射方式

- `cycle.yaml` 与 `state.yaml` 生成稳定的历史 Plan 和最终 Progress 表。
- milestone history 与 report 路径生成高层 Execution checkpoint。
- `summary.md` 进入新的 Summary；缺失时使用 `cycle.summary` 并标记不确定。
- `confirm-summary.md` 只作为接受证据引用，不伪造逐字 Discussion。
- 62 个 Memory Records 保持原路径，预览重建人类可读索引。
- 旧 Knowledge、Chats、Patches、PR 和 live deliveries 保留原位，不在没有语义依据时强行归类。

## 覆盖与缺口

- 旧 Cycle 映射：9/9。
- 缺少 `summary.md`：无。
- 缺少 `PROGRESS.md`：C6-compact-workflow-runtime-customization、C7-experience-optimization-bugfix-sweep。
- Live Delivery 状态：accepted=10，cancelled=1。

## 不确定项与风险

- C4-experience-optimization-release 缺少 knowledge-summary.md，预览只能使用其他来源补足。
- C6-compact-workflow-runtime-customization 缺少 PROGRESS.md，预览只能使用其他来源补足。
- C7-experience-optimization-bugfix-sweep 缺少 PROGRESS.md，预览只能使用其他来源补足。
- Live Delivery vsp-10-alpha2-clean-release 状态为 cancelled，激活前必须解决或明确保留。
- 9 个旧 Knowledge 文件可能与 Memory Records 重叠，需要人工去重。
- 20 个旧 Chat 状态文件没有可靠 Cycle 绑定，不自动归入 Discussion。

## S2 需要判断

1. 是否接受“语义摘要层 + 旧详细历史只读引用”，而不是复制全部旧文件。
2. 非终态 live Delivery 应先完成/拒绝，还是在激活时作为单独 legacy work item 保留。
3. 旧 Knowledge 与 Memory 的重叠是否交给后续 Maintain 人工去重。

接受 S2 之前，不写 `.pipeline/cycles/` 中的 20 个历史 Cycle，不修改 `.pipeline/manifest.yaml`，不删除 `.pipeline/archives/`。
