# C23 Summary

## 结论

C23 已完成并由用户在 Nod 真实 SSH TUI 中接受。Quote 背景现在从 rail 铺到内容区右边缘；普通完成不再因 live tail 无条件全量 reflow；canonical final message 可补齐缺失 suffix；标题层级与 thematic break 合同已实现。

## 主要改动

- Markdown/视觉：H1-H6 隐藏井号，H1-H4 分层，Rule 按容器宽度绘制 U+2500。
- 流式完成：仅真实 resize 或 canonical source 分歧触发 repair；正常完成不清屏重插全部 transcript。
- 最终消息：streamed source 与 canonical visible message 对账并补齐缺尾。
- Quote：修复 AgentMessageCell fill，并进一步修复 terminal scrollback 二次 wrap 裁掉带背景尾随空格。

## 验证与实测

- insert_history 17/17、Quote streaming/finalized 2/2、中文 120 列 VT100 RED/GREEN、既有 chatwidget VT100 snapshot 通过。
- Markdown renderer、stream collector、controller、status/layout 与 reflow 专项均通过。
- Nod 部署 SHA-256：`cbdf855cb8995a89139d7f1ee1a38e4ff04deed9904fcd8f6915d1cb67788a90`。
- 用户在 Nod SSH TUI 确认 Quote 正确。

## 运维修复

- 默认 VSP catalog 只含 GPT-5.6（372000）；DeepSeek profile/catalog 独立保留 V4 1048576。
- `基础建模` 的错误持久化有效窗口从 996147 修正为 353400，原 rollout 已备份。

## 剩余风险与后续

- C23 未发布正式 release；当前为 Nod 候选部署。
- 全库仍有 47 个既有低信噪比测试失败。用户决定不批量接受旧 snapshot，待 C24 完成后新建独立 Cycle，大幅删除冗余测试与 CI 并保留必要契约。
