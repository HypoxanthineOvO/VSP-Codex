# M2 Report: Core Ask 工具语义与模型契约

完成时间：2026-05-26T21:32:13+08:00

## 结果

已完成。`request_user_input` 工具从短单选问题扩展为通用 Ask 协作原语，可在普通 Default/Bypass 工作流中用于澄清、Confirm Gate、验收和结构化用户输入。

## 主要变更

- `codex-rs/core/src/tools/handlers/request_user_input_spec.rs`
  - tool schema 支持 request summary/description、逐题 description 和题型。
  - 按题型校验 single-choice、multi-choice、free-text、ranking。
  - 不再强制所有问题必须有 options。
  - 工具说明强调只在缺少会影响任务推进的信息时询问。
- `codex-rs/core/tests/suite/request_user_input.rs`
  - 补充结构化 Ask、legacy 兼容、root-only/default 可用等覆盖。

## 验证

- `cd codex-rs && just test -p codex-core request_user_input`
- `cd codex-rs && just test -p codex-core update_plan`

结果：通过。

## 风险

Ask 仍为 root session 工具；Subagent 直接弹 Ask 被保留为不支持路径。
