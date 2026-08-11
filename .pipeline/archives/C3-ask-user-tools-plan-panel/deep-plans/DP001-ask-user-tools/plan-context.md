# C3 Plan Context：Ask User Tools 与 Plan 常驻窗口

生成时间：2026-05-26T17:47:33+08:00

来源：`.pipeline/deep-plans/DP001-ask-user-tools/`

Readiness：implementation-ready

## Cycle 目标

C3 有两条一等主线：

1. Ask User Tools：把现有 `request_user_input` 产品化为跨客户端结构化问答能力，支持 single choice、multi choice、short answer/free text、order-only ranking，并覆盖 TUI、未来 VSCode 插件和 GUI。
2. Plan/SubPlan 常驻窗口：改造现有 `update_plan`，提供结构化 root plan + 一层 SubPlan state，由 LLM 工具填写，并在 TUI 输入框上方右侧常驻显示。

## 已有代码事实

- `codex-rs/protocol/src/request_user_input.rs` 已有 `RequestUserInputQuestion`、`RequestUserInputAnswer`、`RequestUserInputEvent` 等协议类型。
- `codex-rs/core/src/tools/handlers/request_user_input.rs` 已有 root-thread-only 的 `request_user_input` 工具处理器，会发起问题并等待回答。
- `codex-rs/core/src/tools/handlers/request_user_input_spec.rs` 当前 schema 较窄：1-3 个短问题、每题 2-3 个互斥选项、自动 Other。
- `codex-rs/app-server-protocol/src/protocol/common.rs` 已有 `item/tool/requestUserInput` RPC。
- `codex-rs/app-server-protocol/src/protocol/v2/item.rs` 已有 `ToolRequestUserInput*` 类型，但仍是实验性语义且没有显式 question kind。
- `codex-rs/tui/src/bottom_pane/request_user_input/` 已有 overlay，支持选项、notes/freeform、多问题、排队、确认和 snapshot。
- `codex-rs/tui/src/app/pending_interactive_replay.rs` 是 pending interactive replay 的现有入口，需要验证并扩展到跨设备 Ask resume。
- `codex-rs/protocol/src/plan_tool.rs` 和 `codex-rs/core/src/tools/handlers/plan.rs` 目前只有 flat checklist `update_plan`。
- `codex-rs/core/src/session/turn.rs` 会从 Plan mode assistant markdown stream 抽取 proposed plan，但这不是 canonical state。
- `codex-rs/tui/src/history_cell/plans.rs` 是 history cell 渲染，不是常驻面板。

## 已确认决策

- Ask User Tools 基于现有 `request_user_input` 演进，不新增平行问答工具。
- Ask Tools 必须能在默认 bypass/ordinary interactive turn 使用，不限制在 Plan mode。
- Subagent 不直接弹用户 Ask；用户问题必须路由到 root session。
- C3 题型范围：single_choice、multi_choice、short_answer/free_text、order-only ranking。confirmation 可以先由 single_choice 表达。
- 多个编号逐题简答问题属于 short_answer/free_text 多问题 Ask。
- Ranking 在 C3 必须实现，但只做排序顺序，不做 per-item notes。
- Secret/private 答案在用户可见 history、transcript、日志和摘要中用圆点掩码显示。
- Pending Ask 必须支持跨设备 resume：支持 native Ask UI 的客户端重新弹框；不支持时降级为普通编号聊天问题，不能卡死。
- native Ask UI 支持必须是独立 client capability，不能只依赖 `experimentalApi`。
- app-server v2 Ask API 在 C3 继续保持 experimental，只扩展实验 payload/schema/recovery 语义；稳定化放到有非 TUI client 消费证据之后。
- Plan/SubPlan 改造现有 `update_plan`，保留 flat checklist 兼容层，不新增割裂的平行计划工具。
- Plan/SubPlan canonical state 必须结构化，不从 proposed-plan Markdown 或 history cell 隐式解析。
- 每个 Milestone/root plan item 下先支持一层 SubPlan。
- Plan 面板不支持用户手动编辑；LLM 通过工具更新，用户可查看、展开/折叠、聚焦详情。
- Plan/SubPlan 面板平时常驻显示；Default idle 时折叠。
- Default-idle 折叠态显示一行：当前 root item、当前 SubPlan 标题（如果有）、completed/total 进度。
- Ask/Plan recovery 使用 durable structured events + thread-level current-state snapshot。

## Fallback 普通聊天模板

当客户端不支持 native Ask UI 时，系统生成普通 assistant 消息：

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

实现必须保留内部映射：`call_id`、`turn_id`、question ids、question kinds、option ids、fallback option letter -> option id。能解析用户回复时，转成结构化 Ask answer；不能解析时，继续普通对话追问澄清，不再等待不可用 native UI。

## 建议 Milestone 方向

普通 `/hw:plan` 应把 C3 拆成可验收的串行 Milestone，建议至少覆盖：

1. 协议模型与 app-server experimental schema：显式 question kind、结构化 answer、native Ask capability、TS/schema fixtures、兼容策略。
2. Core 工具 spec 与 handler：默认 bypass 可用、root-only、参数 normalization、fallback/失败策略、模型说明防滥用。
3. TUI Ask overlay 多题型：multi choice、short/free text、order-only ranking、secret 圆点掩码、snapshot 测试。
4. Pending Ask resume/fallback：durable pending Ask metadata、thread-level snapshot、跨设备 replay、unsupported-client numbered chat fallback。
5. Plan/SubPlan structured state 和 evolved `update_plan`：root plan、item id、一层 SubPlan、flat checklist compatibility、structured events/snapshot。
6. TUI 常驻 Plan panel：输入框上方右侧布局、Default idle 折叠、窄屏 fallback、查看/展开/聚焦、不手动编辑、snapshot 测试。
7. Integration/privacy regression：history/transcript/log masking、app-server tests、TUI snapshots、focused Rust tests 和 schema regeneration。

## 必须覆盖的测试/验证

- `just test -p codex-app-server-protocol`，如果 app-server protocol/schema 变更。
- `just write-app-server-schema` 和必要的 experimental schema fixture 更新。
- `just test -p codex-tui`，TUI overlay 或 Plan panel 变更后必须有 snapshot coverage。
- `cargo insta pending-snapshots -p codex-tui` 并审查/接受预期 snapshot。
- `just test -p codex-core` 或更聚焦 core tests，覆盖 request_user_input handler、tool spec normalization、pending/fallback 逻辑。
- `just fmt`，Rust 修改完成后运行。
- `just fix -p <changed-project>`，大型 Rust 改动 finalize 前运行。

## 普通 Plan 需要继续细化的工程细节

- native Ask capability 的具体字段名和 initialize/connection state 存放位置。
- `ToolRequestUserInputQuestion` / `ToolRequestUserInputAnswer` 的精确兼容字段形状，避免破坏现有 TUI/app-server 路径。
- Plan panel 的具体宽度、高度和 responsive breakpoint。
- `update_plan` 的参数形状：是否使用 action/patch 模式，还是每次提交 full structured state。
- thread-level current-state snapshot 的存储位置，以及 durable event replay 的最小事件集合。
- fallback 回复解析失败时由 core 还是 LLM 继续追问。
