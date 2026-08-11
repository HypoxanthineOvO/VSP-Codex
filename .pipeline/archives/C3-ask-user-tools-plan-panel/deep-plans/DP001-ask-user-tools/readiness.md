# Ask User Tools 与 Plan 常驻窗口 Readiness

状态：ready for plan

目标深度：implementation-ready

## 已满足

- 已明确用户目标：C3 同时包含 Ask User Tools 和 Plan/SubPlan 常驻窗口。
- 已完成本地只读证据扫描，确认现有 `request_user_input` 栈存在。
- 已完成本地只读证据扫描，确认现有 `request_user_input` 栈、`update_plan`、Plan mode stream 和 plan history cell 存在。
- 已形成初步架构方向：Ask Tools 基于现有能力演进；Plan 常驻窗口需要结构化 state，而不是复用 Markdown/history cell 作为权威状态。
- 已识别主要组件：tool spec、core handler、protocol、app-server bridge、TUI overlay、future clients、structured Plan/SubPlan state、persistent TUI Plan panel。
- 已确认 Ask Tools 需要在默认 bypass 工作流中可用，不限于 Plan mode。
- 已确认 Subagent 不直接向用户弹 Ask。
- 已确认逐题简答属于 short_answer/free_text Ask。
- 已确认 Plan/SubPlan 改造现有 `update_plan`，每个 Milestone 下先支持一层 SubPlan，C3 不做手动编辑。
- 已确认 pending Ask 跨设备 resume 和 unsupported-client fallback 是硬需求。
- 已确认 secret/private 答案在用户可见记录中用圆点掩码显示。
- 已确认 ranking 首版只做排序顺序，per-item notes 后续再做。
- 已确认恢复正确性优先，可以采用更复杂的事件重建 + thread snapshot 组合方案。
- 已确认 C3 首版需要实现 order-only ranking 的 TUI 排序交互。
- 已确认 Plan 面板平时显示，Default idle 时折叠。
- 已确认 unsupported-client fallback 需要把 Ask 转成普通编号聊天问题，并保留原 Ask 元数据映射。
- 已确认 Default-idle 折叠面板显示一行最小摘要：当前 root item、当前 SubPlan 标题（如果有）和 completed/total 进度。
- 已确认 C3 继续保持 app-server v2 Ask API experimental，只扩展实验字段/schema/语义，暂不稳定化。
- 已确认 native Ask UI 支持必须是独立 client capability，不能只依赖 `experimentalApi`。
- 已确认 fallback 普通聊天模板采用编号问题、题型标签、稳定选项 id/label 和回答格式示例。
- 已确认 Plan/Ask recovery 使用 durable structured events + thread-level current-state snapshot。

## 阻塞项

无 implementation-ready blocker。剩余内容属于普通 `/hw:plan` 的技术分解细节：

- experimental native Ask capability 的具体字段名。
- 旧 `ToolRequestUserInputQuestion` / `ToolRequestUserInputAnswer` 的精确兼容字段形状。
- Plan 面板在 TUI 中的具体 responsive breakpoint。

## 当前 readiness 判断

- directional：满足。
- architecture-ready：满足。
- implementation-ready：满足，可以转换为普通 `/hw:plan` 输入。

## 下一步建议

生成 `plan-context.md`，然后进入普通 `/hw:plan` 的 Discover/Decompose/Generate/Confirm 流程。普通 Plan 仍需把 C3 拆成可执行 Milestone，并为协议、core、TUI、app-server、recovery 和 snapshot 测试分别安排验收。

## 建议决策草案

- 默认可用模式：C3 必须在默认 bypass 工作流中开放 Ask Tools；普通 root interactive turn 可用；非交互 exec 如果没有支持 UI，应走 fallback 或拒绝等待型 Ask。
- Subagent 路由：C3 维持 root-thread-only。Subagent 可以向 root 报告需要询问用户，由 root 决定是否触发 Ask User Tools。
- Secret 策略：C3 使用圆点掩码显示 secret/private 答案；真实值只在当前工具/model path 需要时使用，不在用户可见历史、transcript、日志、摘要中明文显示。
- 题型范围：C3 首版至少支持 single_choice、multi_choice、short_answer/free_text、order-only ranking。confirmation 可由 single_choice 表达，后续若需要再独立成快捷类型。
- 答案结构：C3 建议新增结构化 answer item，保留旧 `answers: Vec<String>` 兼容层，但内部和 v2 新字段使用 option id/order/freeText/redacted 明确表达。
- Plan 面板显示范围：C3 只要线程有 active structured plan 就显示；Plan mode、执行中、普通工作时展开显示，Default idle 时折叠显示。
- SubPlan 深度：C3 支持每个 Milestone/root item 下的一层 SubPlan；后续如需多层再扩展。
- Plan 工具接口：C3 改造现有 `update_plan` 为结构化 Plan/SubPlan 工具，避免平行工具割裂。
- Plan/Ask 恢复：C3 可以采用结构化事件重建 + thread-level 当前快照组合方案；优先保证换电脑 resume 后能恢复 pending Ask 和 Plan 面板。
- Plan 面板交互：C3 首版只读 + 展开/折叠/聚焦详情，不做用户手动编辑；Plan 修改由 LLM 工具负责。
- Resume/fallback：pending Ask 必须持久化问题元数据；支持 Ask UI 的客户端 resume 后弹框，不支持时生成普通编号问题让用户在聊天中回答，并将回答映射回原 ask id/question id，继续任务而不是卡死。
- app-server 稳定性：C3 继续保持 `item/tool/requestUserInput` experimental，扩展实验 payload 和 TS/schema；稳定化放到有非 TUI client 消费证据之后。
- client capability：新增 native Ask capability，和 `experimentalApi` 分离；`experimentalApi` 只代表愿意接收实验 API。
- Plan 折叠态：Default idle 只显示一行 root item / active SubPlan / completed-total 进度，非 idle 常驻显示完整 panel。

## Fallback 机制草案

当客户端不支持 native Ask UI 时，不让 core 永久等待 `request_user_input` overlay。系统应把 pending Ask 转换成普通 assistant/user 对话：

1. 使用原 Ask metadata 生成一段普通聊天问题，包含标题、编号、题型提示和选项。
2. 要求用户按编号回答，例如 `1. ...`、`2. ...`；排序题要求给出选项顺序。
3. 保存 fallback context：原 `call_id`、`turn_id`、question ids、题型、选项 id。
4. 收到用户自然语言回复后，解析为结构化答案；能解析则恢复原工具响应，不能解析则让 LLM 追问澄清。
5. 如果原 turn 已无法继续，应明确说明“原 Ask 已降级为普通对话”，并把答案作为新 turn 上下文继续执行。
