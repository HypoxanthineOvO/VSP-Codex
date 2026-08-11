---
kind: plan
mode: plan
cycle: C23-tui-quote-render-stream-finalize-fixes
status: closed
updated: 2026-08-08T19:03:21+08:00
progress: PROGRESS.md
execution: EXECUTION.md
---

# C23 TUI Quote 渲染与流式完成刷新修复

## 执行目的

定位并修复 VSP-Codex TUI 的 Markdown 渲染与流式完成问题：1) blockquote 底色在输出完成时从完整渲染框回退为只有文字部分有色块；2) 每次输出完成后触发全量刷新，把内容滚动到顶部；3) 长对话上下文下加载/刷新仍有性能问题导致闪烁；4) 标题语法标记泄露且 H1-H4 层级不足、thematic break 仅显示固定三个破折号。M1 已完成问题定位；用户于 2026-08-08 明确授权执行 M2-M5。

## 执行边界

- 分析并修改 `codex-rs/tui` 的渲染、流式与 reflow 链路（markdown_render、markdown_stream、streaming/controller、history_cell、insert_history、app/resize_reflow、app/agent_message_consolidation）。
- M1 只做定位、复现与记录；M2-M5 在用户明确开始后执行。
- 不修改 `.pipeline/state.yaml`、`.pipeline/cycle.yaml`、`.pipeline/rules.yaml` 等受保护文件。
- 不 push、不发布、不安装。
- `~/Hypo-Workflow` 按只读规范输入使用。

## 验证目标

- 三个问题各自有可复现的最小场景、根因说明与证据（测试输出/代码路径）。
- H1-H4 隐藏 Markdown `#` 标记并按已确认层级渲染；独立 `---` 使用 U+2500 `─` 填满当前内容区域。
- 定位记录写入本 Cycle 的 EXECUTION.md，结论可被后续修复直接引用。
- Cycle 校验通过（PLAN/PROGRESS ID 镜像、状态合法）。

## 完整计划

| ID | 阶段 | 期望结果 | 验证方式 |
| --- | --- | --- | --- |
| `M1` | 定位三个问题根因 | 每个问题有最小复现、根因代码路径、证据输出 | 诊断测试对比流式/完成渲染；代码路径引用 |
| `M2` | 修复 blockquote 背景在流式/完成切换时丢失 | 流式已提交行与最终 AgentMarkdownCell 渲染一致，fill 保留 | 单元测试断言 covered 列数；现有 blockquote 快照通过 |
| `M3` | 消除完成时非必要全量重排 | 无 resize 时完成不再清屏重插；滚动位置保持 | consolidation 单元/集成测试；`take_stream_finish_reflow_needed` 分支覆盖 |
| `M4` | 收窄长上下文渲染/重排范围 | 长对话完成与加载不再整体闪烁 | 性能对比（行数级渲染）；reflow 调用次数断言 |
| `M5` | 实现 Markdown 标题层级与宽度感知横线 | H1 紫色色带、H2 紫色粗体下划线、H3 亮紫粗体、H4 紫色斜体；不显示井号；`---` 渲染为内容区全宽 `─` | 40/80/120 列、三套主题、流式/完成、blockquote/list 容器测试与 `insta` 快照 |
| `M6` | 修复助手最终输出在当前 TUI 被提前截断 | 最后一个未换行 chunk、held/live tail 与正常文本均在 finish 后完整落入 scrollback/transcript，不需退出恢复 | finalize/consolidation 事件顺序与完整文本回归测试 |
| `S1` | 人工审阅修复后的渲染行为与性能 | 用户确认 Quote、完成刷新、长上下文性能和标题/横线层级在真实会话中符合预期 | 用户实际使用验证 |

## 未决问题

- M2 修复采用「wrap 后恢复 fill」还是「不再重新 wrap」需要实现时权衡 resize 行为，先按恢复 fill 方向评估。
- M3 中 Required reflow 的保留场景（流式期间 resize）需要确认是否覆盖表格 holdback 的 reshape 需求。

## 已确认视觉合同

- Markdown H1-H6 的 `#` 只参与解析，最终输出不显示语法标记。
- H1：主题派生紫色的内容宽背景色带 + 粗体。
- H2：主题派生紫色粗体 + 下划线，无背景色带。
- H3：亮紫色粗体，无背景。
- H4：紫色斜体、普通字重，无背景；H5/H6 延续更弱的现有层级。
- 独立一行的 `---` 解析为 thematic break，以弱化 `subtle_border` 样式重复 U+2500 `─`，填满当前内容容器的剩余宽度；在 list/blockquote 内不得越过容器边界。
- 视觉 Mock 已由用户于 2026-08-08 通过；Mock 位于 `tmp/c23-heading-hierarchy-mock.{html,png}`，不属于生产实现。
