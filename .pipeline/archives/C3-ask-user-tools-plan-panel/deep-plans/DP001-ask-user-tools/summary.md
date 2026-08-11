# DP001：Ask User Tools 与 Plan 常驻窗口深度讨论摘要

创建时间：2026-05-26T17:11:55+08:00

更新时间：2026-05-26T17:47:33+08:00

## 目标

为 C3 设计两条主线：

1. Ask User Tools：让 Codex GPT 可以通过工具向用户发起结构化问题，支持单选、多选、填空、排序等形式，并让 TUI 以及未来 VSCode 插件或 GUI 使用同一套交互协议。
2. Plan 常驻窗口：规范化和常驻化 Plan 显示窗口，让 Plan 窗口支持 SubPlan 接口，由 LLM 负责填写，并常驻在 TUI 输入框上面的右侧。

## 关键发现

### Ask User Tools

仓库已经存在一套 `request_user_input` 能力：

- 协议类型在 `codex-rs/protocol/src/request_user_input.rs`。
- Core 工具处理器在 `codex-rs/core/src/tools/handlers/request_user_input.rs`。
- 模型可见工具 schema 在 `codex-rs/core/src/tools/handlers/request_user_input_spec.rs`。
- App-server v2 实验性 RPC 是 `item/tool/requestUserInput`。
- TUI overlay 在 `codex-rs/tui/src/bottom_pane/request_user_input/`。

因此 C3 不应重复发明一套平行 Ask User Tools，而应把现有 `request_user_input` 产品化、协议化、扩展题型，并补齐跨客户端和隐私边界。

用户补充后的硬需求是：Ask Tools 必须能在默认 bypass 工作流中使用，而不是只存在于 Plan mode。Ask 之后 LLM 可能继续写 `.pipeline`、更新计划或执行实现，因此它必须属于普通协作/执行链路的一部分。

### Plan/SubPlan

仓库已有 Plan 相关能力，但不是用户要求的常驻结构化面板：

- `codex-rs/protocol/src/plan_tool.rs` 只有 flat checklist：`step` + `status`。
- `codex-rs/core/src/tools/handlers/plan.rs` 中 `update_plan` 会发 `EventMsg::PlanUpdate`，并且明确不允许在 Plan mode 使用。
- `codex-rs/core/src/session/turn.rs` 会从 Plan mode assistant markdown stream 抽取 `PlanItem`/`PlanDelta`。
- `codex-rs/tui/src/history_cell/plans.rs` 负责 history cell 渲染 proposed plan 和 updated plan。

因此 Plan 常驻窗口不能只复用 history cell；它需要一个结构化 Plan/SubPlan state，由 LLM 通过明确接口更新，TUI 面板消费这个 state。

## 当前限制

- 工具 schema 目前强制每个问题都有 options，且推荐 2-3 个互斥选项。
- 协议没有显式 question kind，无法清晰表达多选、纯填空、排序。
- 答案目前是 `answers: Vec<String>`，对排序、option id、notes、secret redaction 的表达不足。
- App-server RPC 仍标为 experimental。
- `update_plan` 不能表达 SubPlan，也不能携带 item id、层级、summary、owner、blocked reason 等常驻面板需要的信息。
- Plan mode proposed plan 是 Markdown proposal，不是 canonical state。
- 当前 Plan 展示主要在历史里，不是常驻在输入区上方。

## 初步建议

把 C3 的第一版目标定为“结构化问答工具 v1”，而不是完整表单系统：

- 支持 single_choice、multi_choice、short_answer/free_text、order-only ranking。
- 支持 short_answer/free_text 的多问题表单：例如当前这种 1/2/3/4/5 每题写简答，应作为一类常见 Ask。
- 保留多问题，但继续限制单次问题数量，避免模型滥用。
- 以 app-server v2 experimental payload 为跨客户端契约；C3 不急于稳定化。
- TUI 复用现有 overlay 模块扩展，不把逻辑塞进 `chatwidget.rs`。
- secret 输入必须在 history/transcript/logs 中有明确策略和测试。
- Plan/SubPlan 使用结构化 state，不从 Markdown proposed plan 或 history cell 隐式解析。
- Plan 面板优先在宽屏显示在输入框上方右侧；平时显示，Default idle 时折叠成一行 root item / active SubPlan / completed-total 进度。
- SubPlan 首版建议限制为一层树，避免 C3 变成复杂项目管理系统。
- Pending Ask 必须支持跨设备 resume：新客户端支持 native Ask UI 时重新弹出 Ask，不支持时降级成普通聊天问题，不能卡死。native Ask 支持必须是独立 client capability，不能只依赖 `experimentalApi`。

## 决策草案

### 我建议直接采用的决策

1. Ask User Tools 基于现有 `request_user_input` 演进，不另起平行工具。
2. app-server v2 是 TUI、未来 VSCode、GUI 的共享协议边界。
3. Ask Tools 必须显式表达 question kind，不能靠 `options` 是否存在来推断。
4. C3 有两条一等主线：Ask User Tools 和 Plan/SubPlan 常驻窗口。
5. Plan/SubPlan 的常驻面板以结构化 state 为准，不从 proposed-plan Markdown 或 history cell 解析。
6. 保留旧 `update_plan` 兼容，但新增或扩展更丰富的结构化 Plan/SubPlan 接口。
7. TUI Plan 面板做成独立模块，优先放在输入框上方右侧，窄屏必须有折叠/退化形态。
8. SubPlan 首版使用 bounded tree，不做任意图依赖。
9. Ask Tools 在默认 bypass 工作流中可用，不限制在 Plan mode。
10. Subagent 不允许直接弹用户 Ask；需要通过 root agent 汇总提问。
11. 多个编号简答问题是 `short_answer/free_text` 多问题 Ask。
12. Plan/SubPlan 改造现有 `update_plan`，而不是新增一个割裂的平行计划工具。
13. 每个 Milestone/root plan item 下最多一层 SubPlan，C3 不做多层嵌套。
14. C3 Plan 面板不支持用户手动编辑；LLM 通过工具更新，用户可查看/展开/聚焦。
15. Pending Ask 必须可跨设备 resume；不支持 native Ask UI 的客户端必须降级为普通对话问答。
16. Secret/private 答案在用户可见记录中使用圆点掩码显示，不使用 `<redacted>` 文案。
17. Ranking 首版只做排序顺序，不做每个排序项的 notes。
18. Ask/Plan 恢复可以采用更复杂的事件重建 + thread snapshot 组合；正确恢复优先。
19. C3 首版实现 order-only ranking 的 TUI 排序交互。
20. Plan/SubPlan 面板平时显示，Default idle 时折叠。
21. 不支持 native Ask UI 的客户端将 pending Ask 降级为普通编号聊天问题，并保留映射元数据。
22. Default idle 折叠态显示当前 root item、当前 SubPlan 标题（如果有）和 completed/total 进度。
23. app-server v2 Ask API 在 C3 保持 experimental，先扩展实验字段和恢复语义，不在本 Cycle 稳定化。
24. native Ask UI 支持是独立 client capability，不等同于 `experimentalApi`。
25. fallback 普通聊天模板使用编号、题型标签、稳定选项 id/label 和回答格式示例。
26. Ask/Plan recovery 使用 durable structured events + thread-level current-state snapshot。
27. `update_plan` 保留 flat checklist 兼容层，同时扩展为支持 id、root item 和一层 SubPlan 的结构化接口。

### 已关闭的原开放点

- Default idle 折叠面板显示一行最小摘要：当前 root item、当前 SubPlan 标题（如果有）和 completed/total 进度。
- fallback 普通聊天问题模板采用“编号 + 题型 + 选项 id/label + 回答格式示例”。
- app-server v2 Ask API 在 C3 继续实验化；稳定化延后到有非 TUI client 消费证据之后。

## 第一性原理讨论

真正的痛点不是“多一个 UI 表单”，而是 Codex 在信息不足时缺少低摩擦、可审计、可复用的澄清通道。普通自然语言追问会混在 transcript 里，用户回答也难以被工具层结构化理解；审批弹窗又只适合安全决策，不适合需求澄清、偏好选择、字段补全或排序权衡。

Ask User Tools 的本质应是“模型可发起、客户端原生渲染、答案结构化返回”的协作原语。TUI 只是第一个客户端；如果协议只按 TUI 交互细节设计，未来 VSCode/GUI 会被迫复制 TUI 行为或解析文案。相反，协议应描述问题语义，客户端自行选择合适控件。

Plan 常驻窗口的本质不是“把历史里的 plan cell 换个位置”，而是把当前工作计划变成持续可见的协作状态。用户不应该在长对话里滚动找计划，也不应该只能靠 LLM 的自然语言保持计划一致。Plan/SubPlan 必须有结构化状态和更新接口，TUI 只是它的第一个可视化表面。

最小可行闭环是：模型发起一个带题型的问题集合，客户端展示，用户回答，core 将结构化答案返回给同一次工具调用，turn 保持等待状态且可被取消/恢复；如果用户换设备 resume，pending Ask 仍能在支持的客户端重新弹出；如果客户端不支持，就退化为普通聊天问题，不让原 turn 无限等待。模型也能发起结构化 Plan/SubPlan 更新，TUI 常驻面板即时同步，用户在不滚动历史的情况下知道当前主计划、正在执行的子计划和完成进度。任何超出这个闭环的能力，如条件逻辑、跨页表单、文件上传、动态远程校验、任意图依赖、多人手动编辑，都应谨慎后置。

可能推翻当前方向的证据包括：现有 `request_user_input` 的 pending/replay/cancel 机制无法可靠支持多题型或跨设备 resume；app-server v2 无法兼容扩展字段；客户端能力无法判断是否支持 Ask UI；或 secret 答案无法在 history/transcript/logs 中做到清晰 redaction。当前扫描未发现这些是硬阻塞，但都需要 C3 测试验证。

实施验收信号应包括：core schema 可表达题型、TUI snapshot 覆盖多选/填空/排序/secret 圆点掩码、app-server TS schema 更新、pending interactive replay 不回放 stale prompt、跨设备 resume 可恢复 pending Ask、客户端不支持 Ask UI 时能降级普通聊天、history cell 不泄露 secret、模型工具说明不会鼓励过度询问；Plan/SubPlan 工具可更新结构化状态，TUI 常驻面板在宽/窄屏都不重叠，Default idle 折叠态可读，resume 后能恢复或合理重建面板，旧 `update_plan` 不被破坏。

## Fallback 模板

```text
我需要你回答几个问题后继续。当前客户端不支持原生 Ask UI，请按编号回复：

1. [short_answer] 你的问题文本
2. [single_choice] 请选择一个：
   A. 选项 label - description
   B. 选项 label - description
3. [ranking] 请按优先级排序：A, B, C

请用类似下面格式回答：
1. ...
2. A
3. B, A, C
```

内部保留 `call_id`、`turn_id`、question ids、question kinds、option ids 和 fallback letter 映射。能解析则恢复结构化 Ask answer；不能解析则继续普通对话追问澄清。
