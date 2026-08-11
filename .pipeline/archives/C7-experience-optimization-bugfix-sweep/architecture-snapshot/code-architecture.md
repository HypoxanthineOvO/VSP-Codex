# 代码架构

## 总体分层

1. 分发/启动层：`codex-cli/` npm wrapper、release scripts、安装脚本和平台包负责把原生 `codex` 二进制交给用户。
2. 多工具 CLI 层：`codex-rs/cli` 生成单一 `codex` 二进制，按子命令分发到 TUI、exec、review、login、MCP、app-server、cloud、sandbox、debug、exec-server 等模块。
3. 交互/服务入口层：`tui`、`exec`、`app-server`、`mcp-server`、`exec-server`、`cloud-tasks` 等 crates 提供不同运行形态。
4. 核心业务层：`core` 管理 agent 会话、上下文、模型请求、工具调用、审批、安全、配置、状态和任务生命周期。
5. 协议/数据层：`protocol`、`app-server-protocol`、`codex-api`、`codex-client`、`state`、`thread-store`、`rollout` 定义内部/外部 wire types、后端类型和持久化结构。
6. 能力扩展层：`codex-mcp`、`core-plugins`、`core-skills`、`plugin`、`skills`、`hooks`、`connectors` 为工具、插件、技能、hook 和外部连接提供装载与注入。
7. 执行/隔离层：`exec-server`、`sandboxing`、`linux-sandbox`、`windows-sandbox-rs`、`bwrap`、`shell-command`、`shell-escalation`、`utils/pty` 处理命令执行、沙箱和进程控制。

## 主要入口

- `codex-rs/cli/src/main.rs`：单二进制入口，`Subcommand` 枚举覆盖 `exec`、`review`、`login`、`mcp`、`plugin`、`mcp-server`、`app-server`、`remote-control`、`completion`、`update`、`doctor`、`sandbox`、`debug`、`apply`、`resume`、`fork`、`cloud`、`exec-server`、`features`。
- `codex-rs/tui/src/lib.rs` 和 `tui/src/cli.rs`：交互式 TUI 运行主体和参数。
- `codex-rs/exec/src/lib.rs` 和 `exec/src/cli.rs`：非交互执行主体和参数。
- `codex-rs/app-server/src/main.rs`：独立 app-server 二进制入口。
- `codex-rs/app-server/src/lib.rs`：app-server runtime、transport、request processor 整合。
- `codex-rs/mcp-server/src/main.rs` 与 `mcp-server/src/lib.rs`：Codex-as-MCP-server 入口和运行逻辑。
- `codex-rs/exec-server/src/server.rs` 与 `cli/src/main.rs` 的 `ExecServerCommand`：exec-server 由主 `codex` 二进制子命令承载，不是独立 Cargo bin。
- `codex-cli/bin/codex.js`：npm 包入口，定位平台原生二进制并透传参数。

## 核心运行链路

- TUI 路径：`cli -> codex_tui::run_main -> AppServerSession -> ClientRequest(thread/start, turn/start, ...) -> app-server -> ThreadManager -> CodexThread -> session submission_loop -> RegularTask/run_turn -> ModelClient -> ToolRouter/ToolRegistry -> EventMsg -> app-server notifications -> TUI render`。
- Headless exec 路径与 TUI 类似，但 `codex-rs/exec/src/lib.rs` 通过 in-process app-server 发 `thread/start` 和 `turn/start`，再由 human/jsonl event processor 输出结果。
- app-server 是当前 UI/CLI 与 core 的主边界：`codex-rs/app-server/src/message_processor.rs` 统一 dispatch RPC；thread/turn 请求分别落到 `request_processors/thread_processor.rs` 和 `request_processors/turn_processor.rs`。
- in-process 路径没有绕过协议：`codex-rs/app-server/src/in_process.rs` 复用 `MessageProcessor` 和 JSON-RPC envelope；`app-server-client/src/lib.rs` 给 TUI/exec 提供统一 client facade。

## Rust workspace 主要模块边界

- `core`：最大业务 crate。拥有 agent 控制、session/turn/task、tool router、MCP 工具调用、上下文构造、exec policy、sandbox policy、plugin/skill instruction、review、realtime、rollout、state bridge 等。后续新增概念应优先避免继续膨胀 `core`。
- `protocol`：Codex CLI 内部协议类型和部分 app-server 外部类型，要求依赖最小、避免业务逻辑。
- `app-server-protocol`：JSON-RPC v1/v2 协议、TypeScript/JSON schema 导出、实验 API 标记。v2 下按 account/apps/command_exec/config/environment/fs/item/mcp/model/permissions/plugin/process/realtime/review/thread/turn 等拆分。
- `app-server`：JSON-RPC server、transport、message processor、request processors、thread state、fs watch、config manager、dynamic tools、MCP refresh、plugin startup 等。
- `tui`：Ratatui 交互界面，管理 composer、history cell、bottom pane、slash commands、resume picker、notifications、render/wrapping、onboarding 等 UI 逻辑。
- `exec`：headless agent runner，把非交互 prompt 通过 in-process app-server 驱动 core 会话，输出人类文本或 JSONL event。
- `mcp-server`：向外暴露 Codex 工具的 MCP server。
- `codex-mcp`：Codex 作为 MCP client 时的 server 配置、连接管理、OAuth/auth elicitation、工具发现和状态快照。
- `exec-server`：轻量 JSON-RPC process/file-system server，支持 stdio/websocket/remote relay；被 app-server 和远程环境能力使用。
- `login`、`chatgpt`、`backend-client`、`codex-client`、`codex-api`：认证与后端通信。
- `core-plugins`、`plugin`：marketplace、manifest、remote bundle、installed store、startup sync。
- `core-skills`、`skills`：技能 discovery、loader、manager、render/injection、内置技能 bundle。
- `sandboxing`、`linux-sandbox`、`windows-sandbox-rs`、`bwrap`：跨平台隔离后端和辅助二进制。
- `state`、`thread-store`、`rollout`、`rollout-trace`：会话和 trace 持久化。
- `cloud-tasks`、`cloud-tasks-client`、`cloud-tasks-mock-client`：Codex Cloud 任务 UI/CLI 和 API client。
- `sdk/typescript`、`sdk/python`：语言 SDK，不属于 Rust workspace，但依赖 Codex CLI/app-server runtime。

## 依赖方向

- `codex-cli` 依赖多个入口 crates，是命令分发层。
- `tui`、`exec`、`app-server` 均依赖 `core`、`protocol` 和配置/状态/工具相关 crates；TUI/exec 的主要业务边界应保持在 app-server client facade 后面。
- `app-server` 依赖 `app-server-protocol` 定义 wire shape，并依赖 `core` 驱动 agent turn。
- `mcp-server` 依赖 `core`/`protocol` 跑 Codex session，并以 MCP server 形式暴露工具。
- `core` 依赖配置、MCP、插件、技能、沙箱、状态、模型 provider、后端 client 等，但应避免被更高层 UI 逻辑污染。
- SDK 位于仓库外层：TypeScript SDK spawn CLI；Python SDK spawn app-server。

## 代码规模与维护注意

- `codex-rs/core` 是最大 crate，AGENTS 指令明确要求抵制向 `codex-core` 随意加新概念。
- TUI 有多个高触摸大文件，新增功能应优先拆新模块，避免继续膨胀 `app.rs`、`chatwidget.rs`、bottom pane 等集中模块。
- app-server API 新增应只做 v2，保持 Rust/TS wire rename 一致，并在协议变更后运行 schema 生成与协议测试。
- Rust 依赖变更需要刷新 Bazel lock：`just bazel-lock-update` 和 `just bazel-lock-check`。
