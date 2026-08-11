# 数据流

## 交互式 TUI 流

1. 用户运行 `codex [OPTIONS] [PROMPT]`。
2. `codex-rs/cli/src/main.rs` 解析根参数和子命令；没有子命令时进入 TUI 路径。
3. `codex-rs/tui` 建立终端 UI、读取配置、认证状态、session/rollout 信息，并通过 `AppServerSession`/app-server client facade 发送 `thread/start`、`turn/start` 等请求。
4. app-server 的 in-process `MessageProcessor` 将请求路由到 thread/turn processors，再进入 `ThreadManager`、`CodexThread` 和 session turn loop。
5. `codex-core` 构造上下文：AGENTS.md、用户输入、环境、配置、技能/插件/MCP、权限/沙箱说明、历史消息。
6. 模型 provider/client 发起请求；返回的 streaming event 被映射为内部事件。
7. 工具调用由 core tool router 派发到 shell/patch/MCP/parallel/subagent 等能力；命令执行经过审批和沙箱策略。
8. app-server 将 core 事件转为 notifications，TUI 渲染 assistant 消息、reasoning、命令输出、审批弹窗、diff、状态和通知。
9. rollout/state/thread-store 持久化会话，用于 `resume`、`fork`、app-server thread API 和 SDK 恢复。

## 非交互 exec 流

1. 用户运行 `codex exec [PROMPT]` 或向 stdin pipe 输入。
2. `codex-rs/exec/src/cli.rs` 解析 prompt、sandbox/model/config、`--json`、`--output-schema`、`--ephemeral` 等参数。
3. `codex-rs/exec/src/lib.rs` 使用 in-process app-server 发起 thread/turn 请求，并消费 app-server events。
4. event processor 将 notifications 输出为普通终端文本、JSONL event 或最后消息文件。
5. `exec resume` 可通过 session id 或 `--last` 继续历史。

## app-server 流

1. 客户端启动 `codex app-server` 或独立 `codex-app-server`，选择 `stdio://`、`unix://`、`ws://IP:PORT` 或 `off` transport。
2. 客户端先发送 `initialize`，再发送 `initialized`；未初始化的请求会被拒绝。
3. 客户端通过 `thread/start`、`thread/resume`、`thread/fork` 建立会话，再通过 `turn/start` 驱动 agent。
4. app-server request processors 调用 core、state、thread-store、MCP、plugins、fs/process 等模块。
5. 进度通过 JSON-RPC notifications 推送，如 thread/started、turn/started、item delta、turn/completed、fs changed、process output。
6. 富客户端可使用 app-server 额外 API 管理模型、权限、配置、插件、技能、hooks、MCP、远程控制、反馈上传和 Windows sandbox setup。

## MCP 流

Codex 作为 MCP client：

1. 配置从 `config.toml` 或 app-server config API 进入 `codex-mcp`。
2. `McpConnectionManager` 管理外部 MCP server 连接、工具发现、OAuth/auth elicitation 和状态快照。
3. core 将 MCP tools 暴露给模型；工具调用经审批策略后由 MCP client 发往对应 server。

Codex 作为 MCP server：

1. 用户运行 `codex mcp-server`。
2. `mcp-server` crate 以 stdio MCP server 形式暴露 Codex 工具。
3. 外部 MCP client 调用 Codex tool，server 内部启动/驱动 Codex session 并返回结果。

## exec-server 和远程环境流

1. 用户运行 `codex exec-server --listen ws://IP:PORT` 或 `--listen stdio://`；默认本地 websocket。
2. 客户端进行 initialize/initialized 握手。
3. 客户端调用 process/file-system RPC：start/read/write/terminate/readFile/writeFile/list/remove/copy 等。
4. 远程模式下，`--remote URL --environment-id ID` 注册本地环境，通过 rendezvous websocket 传递 protobuf relay frame，承载 JSON-RPC payload。
5. 远程注册需要 ChatGPT auth；Agent Identity JWT 仅在 `--use-agent-identity-auth` 路径使用。

## 插件和技能流

1. 插件 marketplace 或本地插件由 `core-plugins` 发现、加载、安装、同步和过滤。
2. 技能由 `core-skills`/`skills` 发现并读取 `SKILL.md`，按用户触发条件注入上下文。
3. app-server 提供 plugin/skill read/list/install/uninstall/config write 等 API；TUI/CLI 可通过 `codex plugin` 管理插件。
4. core 在构造模型上下文时加入可用插件/技能说明，并按规则加载具体技能内容。

## 沙箱和审批流

1. 用户通过 CLI flags 或 config 选择 sandbox/approval policy。
2. core 解析为权限/沙箱策略，并在模型尝试命令或 patch 时执行审批逻辑。
3. macOS 通过 Seatbelt，Linux 通过 Landlock 或 bubblewrap，Windows 通过 restricted token/elevated sandbox。
4. 对 shell、apply_patch、MCP tool、network approval 等能力，core 维护事件映射和结果回传。
