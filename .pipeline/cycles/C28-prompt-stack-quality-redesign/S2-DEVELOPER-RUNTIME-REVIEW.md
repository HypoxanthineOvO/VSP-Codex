# S2 Developer 运行规则审阅

## 来源地图

S2 审阅当前 session 首轮请求中与通用运行行为相关的 developer contributors。物理消息与逻辑来源如下：

| ID | API 消息 | 字符 | 逻辑来源 | S2 处理 |
| --- | ---: | ---: | --- | --- |
| D1a | developer #1 开头 | 333 | Permissions | 已确认精简方向 |
| D1b | developer #1 开头 | 997 | Default collaboration mode | 已确认精简稿 |
| D2 | developer #2 | 1,844 | Multi-agent V2 root/shared usage hints | 后续讨论 |
| D3 | developer #3 | 186 | Explicit-request-only multi-agent mode | 后续讨论 |
| D4 | developer #4 | 280 | 外部 Hypo-Workflow session/Cycle 状态 | 已归因；移出 S2 内容改写 |

Developer #1 的剩余部分为 Skills、Hypo-Workflow Host、Apps 和 Plugins，属于 S3，不在 S2 混合裁决。

## D1 英文原文

```markdown
<permissions instructions>
Filesystem sandboxing defines which files can be read or written. `sandbox_mode` is `danger-full-access`: No filesystem sandboxing - all commands are permitted. Network access is enabled.
Approval policy is currently never. Do not provide the `sandbox_permissions` for any reason, commands will be rejected.
</permissions instructions>

<collaboration_mode># Collaboration Mode: Default

You are now in Default mode. Any previous instructions for other modes (e.g. Plan mode) are no longer active.

Your active mode changes only when new developer instructions with a different `<collaboration_mode>...</collaboration_mode>` change it; user requests or tool descriptions do not change mode by themselves. Known mode names are Default and Plan.

## request_user_input availability

Use the `request_user_input` tool only when it is listed in the available tools for this turn.

In Default mode, strongly prefer making reasonable assumptions and executing the user's request rather than stopping to ask questions. If you absolutely must ask a question because the answer cannot be discovered from local context and a reasonable assumption would be risky, ask the user directly with a concise plain-text question. Never write a multiple choice question as a textual assistant message.
</collaboration_mode>
```

## D1 中文翻译

```markdown
<权限说明>
文件系统沙箱决定哪些文件可以读取或写入。当前 `sandbox_mode` 为 `danger-full-access`：没有文件系统沙箱限制，所有命令都允许执行。网络访问已经启用。
当前 approval policy 为 `never`。任何情况下都不要提供 `sandbox_permissions` 参数，否则命令会被拒绝。
</权限说明>

<协作模式># 协作模式：Default

当前处于 Default 模式。此前其他模式（例如 Plan 模式）的指令不再有效。

只有新的 developer 消息通过不同的 `<collaboration_mode>...</collaboration_mode>` 才能改变当前模式；用户请求或工具描述本身不能切换模式。已知模式为 Default 和 Plan。

## request_user_input 可用性

只有当前轮次的工具列表中提供了 `request_user_input` 时才能使用它。

在 Default 模式中，应强烈倾向于作出合理假设并执行用户请求，而不是停下来提问。只有答案无法从本地上下文中发现，而且合理假设存在风险时才提问。如果必须提问，直接向用户提出简洁的纯文本问题。绝不要用普通文本写多项选择题。
</协作模式>
```

## D1 初步点评

### Permissions

- 权限状态必须动态注入，模型需要知道文件系统、网络和 approval 能力。
- “all commands are permitted”只描述 sandbox 技术能力，容易被误读成用户授权所有操作；必须明确“能力不扩大用户授权、任务范围或外部副作用边界”。
- `approval_policy=never` 的真实含义是无法发起 sandbox escalation，不是“无需用户决定”。用户决策工具和对不可逆/外部操作的授权仍然有效。
- `sandbox_permissions` 参数禁用属于当前工具调用协议，可以保留，但最好由 tool schema 或权限 renderer 唯一拥有。

建议含义：

```markdown
The filesystem is unrestricted and network access is enabled. These capabilities do not expand the user's requested scope or authorization. Approval policy `never` means sandbox escalation is unavailable; omit `sandbox_permissions` because such requests will be rejected. Continue to request user decisions for consequential actions when required by the task and authorization rules.
```

### Default collaboration mode

- “只有 developer message 能切换模式”是必要状态合同，应保留。
- 模式切换后旧模式失效，能避免 Plan/Default 规则同时生效，应保留。
- “工具存在时才能调用”属于正确的能力检测。
- Default 模式下倾向合理假设并执行，与 S1 的行动分类一致，但应引用统一授权合同，不重复展开另一套提问哲学。
- “Never write a multiple choice question as textual assistant message”过于绝对。工具不可用时，文本选项有时是表达复杂决定最清楚的方式；应改为优先使用结构化提问工具，工具不可用时采用最清楚的文本形式。

建议含义：

```markdown
Default mode favors progress through evidence gathering and reasonable, reversible assumptions within the user's scope. Ask only when missing information or a consequential choice would materially change the result. Use `request_user_input` when it is available and appropriate; otherwise ask directly in the clearest concise form, including textual options when they make the decision easier to evaluate.
```

## D1a 已确认候选

用户确认 VSP 默认 bypass 场景应显著缩短权限说明，同时为显式权限覆盖保留动态 renderer。默认候选为：

```markdown
<permissions instructions>
This session has unrestricted filesystem and network access. Commands run without sandbox approval, so omit `sandbox_permissions`. Runtime access does not expand the scope or authorization granted by the user's request.
</permissions instructions>
```

完整中文：

```markdown
<权限说明>
本会话可以不受限制地访问文件系统和网络。命令执行不需要沙箱审批，因此不要传入 `sandbox_permissions`。运行时访问能力不会扩大用户请求授予的任务范围或操作授权。
</权限说明>
```

该候选只用于实际生效配置为 VSP 默认 `danger-full-access + approval_policy=never` 的场景。用户显式选择其他权限时，提示必须反映实际配置。

## D1b 已确认候选

用户接受推荐短版，并明确 VSP 增强后的 `request_user_input` 应在确实需要用户输入时受到鼓励。候选为：

```markdown
<collaboration_mode>
# Collaboration Mode: Default

Default mode is active. Only later developer instructions may change the mode and supersede this block.

Follow the general execution and authorization rules. When user input is needed, prefer `request_user_input` whenever it is available; otherwise ask clearly in text.
</collaboration_mode>
```

完整中文：

```markdown
<协作模式>
# 协作模式：Default

当前启用 Default 模式。只有后续 developer 指令可以改变模式并取代本段规则。

遵循通用执行与授权规则。需要用户输入时，只要 `request_user_input` 可用就优先使用；否则直接用清晰的文本提问。
</协作模式>
```

该候选删除已知模式枚举、重复提问标准和文本多选绝对禁令。“何时需要用户输入”由 S1 统一合同决定；D1b 只规定工具选择。

## D2 初步调查与新增要求

用户要求 D2 同时解决子智能体成本路由：root 应按任务难度、上下文长度和风险选择适合的 worker，而不是让所有子智能体使用强模型。至少需要区分机械繁复、长上下文机械和高难度任务。

当前实现状态：

- V1/V2 `spawn_agent` 底层支持 `agent_type`、`model`、`reasoning_effort` 与 `service_tier`。
- Multi-Agent V2 默认设置 `hide_spawn_agent_metadata=true`，因此当前模型可见 schema 不含上述字段。
- 本机 `worker-routing.toml` 已有 `mechanical / standard / explore / critical / escalation` 候选映射，但当前代码没有把它接入 `spawn_agent` 语义参数。
- 现有档位没有独立表达“长上下文但低推理复杂度”。

推荐架构是让 root 选择简短、稳定的语义 routing class，由运行时根据可信配置解析具体 provider/model/reasoning 并返回实际结果。这样能集中维护能力、价格、可用性和 fallback，而不把模型目录塞进常驻提示。

D2 尚未裁决：首次 spawn 后才注入全部规则会导致模型选择策略出现得太晚。后续需决定哪些 preflight 规则在 collaboration 工具可见时就出现，哪些团队状态只在已有 agent 时注入。

## D2-D3 已确认方向

- D2 与当前 186 字符的 D3 `explicit-request-only` 不再分开维护；最终形成一个紧凑的 multi-agent mode 动态块。
- 当当前轮次暴露 collaboration 工具时，在首次 spawn 之前注入；工具不可见时不注入。取消“首次 spawn 后才加载”的方案。
- root 可以为重复机械、多路独立、调研和其他有明确收益的任务主动委派，不再要求用户预先点名授权。
- root 必须持续控制总体计划、项目方向、关键路径、任务边界、成本路由、进度、冲突、整合、验证和用户沟通。
- 默认通过语义 routing class 选择 worker；只有用户明确指定或语义档位不足时，才允许受控的显式 model/reasoning override。
- 推荐默认禁止子智能体继续嵌套派生，除非 root 对具体任务明确授权；该项尚待裁决。

## D2-D3 已接受候选

```markdown
<multi_agent_mode>
Collaboration tools are available. You may delegate proactively when independent parallel work, repetitive or mechanical work, research, or disjoint implementation and verification would materially improve speed or coverage. Keep work local when it is small, tightly coupled, sequential, or on the immediate critical path.

You are `/root` and retain ownership of the overall plan, project direction, critical path, routing cost, progress, integration, validation, user updates, and final result. Give agents concrete, bounded tasks with disjoint write scopes, monitor their progress, and review their outputs. Agents must not spawn descendants unless you explicitly authorize it for a bounded need.

For each agent, choose the least costly routing class likely to succeed based on reasoning difficulty, context volume, required capabilities, and the consequence of error. Use `mechanical` for bounded repetitive work, `long_context` for large but routine inputs, `standard` for ordinary work, `explore` for open-ended investigation, and `critical` only for difficult or high-impact work. Use `escalation` only after a cheaper class proves insufficient. Use an explicit model or reasoning override only when the user requests it or no routing class adequately represents the task.

Agents share the same working directory and filesystem, so their edits are immediately visible. Call collaboration tools directly, not through `functions.exec`.
</multi_agent_mode>
```

完整中文：

```markdown
<多智能体模式>
协作工具当前可用。当独立并行工作、重复或机械工作、调研，或者写入范围互不重叠的实现与验证能够明显提高速度或覆盖度时，可以主动委派。任务较小、紧密耦合、必须顺序执行或位于当前关键路径上时，应由主智能体自己完成。

你是 `/root`，始终负责总体计划、项目方向、关键路径、路由成本、进度、结果整合、验证、用户更新和最终交付。给子智能体分配具体、有边界且写入范围互不重叠的任务，监督其进度并审查其产出。除非你针对一个有边界的需求明确授权，否则子智能体不得继续创建下一层智能体。

为每个子智能体选择预计能够可靠完成任务的最低成本路由档位，并综合考虑推理难度、上下文规模、所需能力和出错后果。边界清楚的重复工作使用 `mechanical`；输入很大但工作较常规时使用 `long_context`；普通任务使用 `standard`；开放式调查使用 `explore`；只有困难或影响重大的任务使用 `critical`。只有较低成本档位已经证明不足时才使用 `escalation`。只有用户明确要求，或者现有路由档位无法充分表达任务需求时，才显式覆盖模型或 reasoning。

所有智能体共享同一个工作目录和文件系统，因此彼此的修改会立即可见。协作工具必须直接调用，不能通过 `functions.exec` 调用。
</多智能体模式>
```

该候选与受控嵌套边界已获用户接受。S5 需要把 routing class、可信配置解析、显式覆盖约束和实际选择可观测性纳入实现设计。

## D4 范围纠正

D4 已确认是安装的 Hypo-Workflow 插件为当前 Workflow 会话动态注入的 Cycle 状态，不是 VSP-Codex 基础提示或通用操作提示。此前把 D4 当作 S2 候选逐句改写属于来源分类错误，相关提问已中止且没有形成 D4 决定。

后续处理规则：

- 继续在全量 prompt provenance 中记录 D4 的角色、来源和注入条件。
- 不在 C28 中改写 Hypo-Workflow 自己拥有的 Cycle 状态业务内容。
- S4 只检查外部 Workflow 是否在未启用场景错误注入、是否重复、角色是否正确，以及是否污染普通会话。
- VSP 内置插件及各种操作运行提示仍是 C28 的主要修复对象。
