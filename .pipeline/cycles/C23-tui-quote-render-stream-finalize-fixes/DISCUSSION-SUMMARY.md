# C23 Discussion Summary

## Outcome

定位 VSP-Codex TUI 三个用户可见问题的根因，并形成可执行的修复计划。本轮按用户要求只做定位与记录，不进入修复实现。

## 用户提出的问题

1. blockquote 底色渲染：输出中途能看到完整底色渲染框（上下行都有色块），输出完成后刷新回「只有文字覆盖部分有色块」的错误形状。
2. 每次输出完成触发一次刷新，把所有内容刷新到最顶上，需要手动翻下去。
3. 上下文特别长时加载仍有性能问题导致闪烁（短对话已改善，长对话仍存在）。

## 定位结论

- 问题 1：`AgentMessageCell` 对 controller 已渲染的 fill 行重新 wrap，`wrap_ranges_trim` 裁剪行尾空白，blockquote 背景 fill 丢失；tail 与最终 `AgentMarkdownCell` 路径都完整，只有中间提交态破坏。
- 问题 2：consolidation 的 Required 分支无条件清屏并全量重插 transcript（表格 holdback 的 live tail 常见触发），滚动位置重置。
- 问题 3：问题 2 的每次完成全量重排是长对话卡顿/闪烁主因，叠加每 delta 全量重渲染。

## 确认的边界

- 本轮只定位与记录；修复实现待用户确认后进入 M2..M4。
- 只改 `codex-rs/tui`；不动 `.pipeline` 受保护文件；不 push/发布/安装。
- 历史刷新（History Refresh）已按 15.0 规范执行：9 个旧 Cycle（C1..C9）生成语义摘要，旧 archives/manifest 只读保留。

## 未决问题

- M2 修复策略在「wrap 后恢复 fill」与「不再重新 wrap」之间选择，需兼顾 resize reflow 换行行为。
- M3 需要确认 overlay 场景与表格 reshape 下 deferred 行插入的正确性。

## 2026-08-08 补充决定

- 用户通过分层标题 Mock：H1 内容宽紫色色带 + 粗体，H2 紫色粗体 + 下划线，H3 亮紫粗体，H4 紫色斜体；所有 Markdown `#` 标记不显示。
- 独立 `---` 使用 U+2500 `─` 按当前内容容器宽度绘制弱化细横线，不显示固定三个中文破折号。
- 该范围加入 pending `M5`，与 Quote、reflow 和长上下文修复一同在 `S1` 做真实 TUI 审阅。

## 2026-08-08 VSPi 完成边界复核

- 用户认为 VSPi 可能已完成 C23 大部分修复，因此暂不接受执行 gate，要求检查原会话。
- 工具级审计确认 session `019fd785-079a-76db-a186-1082662caf38` 只完成 M1 诊断；唯一生产 edit 是一个未完成的 M2 调用点，随后连同临时诊断模块全部回滚。
- 后续找到 VSP-Codex/DeepSeek 前置 rollout `019fc1ba-5883-7e93-bc2c-f33abfdfbcca`：其 Quote/Resume commit `4aa6f52248` 已在 HEAD，但它在 C23 创建前结束，处理的是前置能力，不包含 C23 后续回归修复。
- 另一个 8 月 7 日 VSPi session 对应独立 TypeScript 项目 C09，不是 Rust C23。
- 当前可靠边界为：前置能力已完成，C23 M1 已完成，M2-M5 未开始；在用户接受该区分前不发执行 gate。

## 2026-08-08 最终接受

- 用户在 Nod SSH 中直接运行新候选并以截图指出 Quote 背景仍未铺满；该反馈重新打开 S1。
- 修复真实 terminal scrollback 的二次 wrap 后重新部署，用户确认 Quote 已正确并接受 C23。
- 用户明确不希望用批量 snapshot 更新消除全库失败；测试与 CI 的大幅删除、分层和门禁收缩延后到 C24 完成后的独立 Cycle。
