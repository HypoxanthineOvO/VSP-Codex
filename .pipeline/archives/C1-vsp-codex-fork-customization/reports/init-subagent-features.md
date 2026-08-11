# Init Subagent Report: 功能列表

状态：只读探索完成，未修改仓库文件。

整体看，这是一个以 Rust workspace 为主体的 Codex 本地 agent 平台，npm 包负责分发原生二进制，SDK 主要围绕 CLI / app-server 封装。

## 底层服务功能

- Rust workspace 聚合核心 crate：`core`、`tui`、`exec`、`app-server`、`codex-mcp`、`cloud-tasks`、`plugin`、`skills` 等。
- 核心业务层 `codex-core`：线程/会话/turn、模型客户端、工具调度、MCP、skills、sandbox、rollout、state DB、review、realtime、web search 等。
- 持久化：JSONL rollout + SQLite state/log/goal 数据库；state crate 负责从 rollout 提取元数据并镜像到 SQLite。线程存储抽象支持本地、内存、分页、归档、搜索。
- 执行服务：`codex exec` 是非交互 agent 入口，内部使用 app-server 协议和事件处理器。`exec-server` 提供 JSON-RPC 进程/文件系统服务、远程环境 relay、PTY 进程控制。
- 沙箱服务：平台选择 macOS Seatbelt、Linux seccomp/bubblewrap、Windows restricted token。Linux helper 负责 `codex-linux-sandbox`、bubblewrap、seccomp、网络隔离/代理。
- 配置/策略：`config.toml`、profiles、requirements、permissions、MCP、hooks、plugins、skills、keymap 等由 config crate 定义。
- Auth/secret：支持 API key、ChatGPT OAuth、外部 ChatGPT tokens、Agent Identity；凭据可存文件/keyring/ephemeral；Secrets 走本地 backend + keyring。
- 文件能力：抽象本地/远程 filesystem，支持读写、目录、metadata、copy/remove 和 sandbox context；另有文件监听和模糊文件搜索。
- 记忆系统：read path 注入/引用 memory，write path 做启动记忆流水线、raw memories、rollout summaries、phase1/phase2 consolidation。

## 接口/API/协议功能

- app-server 是富客户端接口，服务 VS Code/桌面等，使用 JSON-RPC 2.0 风格消息，支持 stdio、websocket、unix socket、off transport。
- app-server 生命周期：`initialize` 后才能调用其他方法；核心对象是 Thread、Turn、Item。
- v2 API 覆盖线程生命周期、turn、review、命令执行、进程、文件系统、模型、权限、features、环境、skills、hooks、plugin、MCP、apps、remote control、feedback、config、Windows sandbox。
- 关键 RPC：`thread/start`、`thread/resume`、`thread/fork`、`turn/start`、`review/start`、`command/exec`、`fs/readFile`、`plugin/list`、`mcpServer/tool/call`、`remoteControl/*`、`feedback/upload`、`config/read`。
- 协议类型按 v2 分域拆分：thread/turn、command_exec、MCP、plugin/skills 等分别在 `app-server-protocol/src/protocol/v2/` 下维护。
- MCP 双向能力：Codex 作为 MCP client 连接外部 MCP servers，由 `McpConnectionManager` 管理工具/资源/认证/调用；Codex 也可作为 MCP server 暴露 `codex`、`codex-reply` 工具。
- SDK：TypeScript SDK 通过 spawn `codex` CLI 并交换 JSONL 事件；Python SDK 直接面向 `codex app-server` JSON-RPC v2 over stdio。

## 面向用户的功能

- CLI 主命令包括 interactive TUI、`exec`、`review`、`login/logout`、`mcp`、`plugin`、`mcp-server`、`app-server`、`sandbox`、`debug`、`apply`、`resume`、`fork`、`cloud`、`exec-server`、`features`。
- npm 安装入口 `@openai/codex` 是平台选择和原生二进制转发器。
- TUI 支持 prompt 启动、审批策略、live web search、禁用 alternate screen、resume/fork picker 等。TUI 可连接 in-process 或 remote app-server。
- 非交互 `codex exec` 可用于脚本/CI，支持 prompt/stdin、JSONL 输出、图片、schema output、review 模式、ephemeral session、OSS provider 等。
- Cloud tasks：`codex cloud` 可浏览环境/任务、创建任务、查看 status/list/diff、apply 云端 diff。
- 插件 CLI：`codex plugin add/list/remove/marketplace`，支持 marketplace 安装、列出、卸载。
- MCP CLI：`codex mcp list/get/add/remove/login/logout`，支持 stdio 和 streamable HTTP MCP server。
- Hooks 用户扩展点：PreToolUse、PermissionRequest、PostToolUse、Pre/PostCompact、SessionStart、UserPromptSubmit、SubagentStart/Stop、Stop。

## 外部集成/外部功能

- OpenAI/ChatGPT API：`codex-api` 封装 Responses、Responses WebSocket、Realtime、Images、Search、Models、Memories 等 endpoint。
- 模型 provider：内置 OpenAI、ChatGPT Codex base URL、Amazon Bedrock，以及自定义 OpenAI-compatible provider、env/header/auth command/AWS SigV4 配置。
- ChatGPT first-party 集成：Cloud task、connectors/apps、workspace settings 等在 `chatgpt` crate。
- Codex Apps/connectors：MCP 中有 host-owned `codex_apps` server，并结合 ChatGPT auth 与插件 provenance。
- Plugins/marketplaces：插件 manifest 可声明 skills、MCP servers、apps、hooks、interface metadata。
- Skills：系统内置 skills 安装到 `CODEX_HOME/skills/.system`，包含 `imagegen`、`openai-docs`、`plugin-creator`、`skill-creator`、`skill-installer`；skills manager 支持 repo/user/system/admin/plugin roots、缓存、禁用规则、隐式调用。
- Memories MCP：提供只读 `list/read/search` memory tools。
- Responses API proxy：受限代理只允许 `POST /v1/responses`，从 stdin 读 API key，可 dump 请求/响应并做进程 hardening。

## 证据文件

- `README.md`
- `package.json`
- `codex-cli/package.json`
- `codex-cli/bin/codex.js`
- `codex-rs/Cargo.toml`
- `codex-rs/README.md`
- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/src/lib.rs`
- `codex-rs/cli/src/mcp_cmd.rs`
- `codex-rs/cli/src/plugin_cmd.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/tui/src/cli.rs`
- `codex-rs/exec/src/lib.rs`
- `codex-rs/exec-server/README.md`
- `codex-rs/exec-server/src/lib.rs`
- `codex-rs/app-server/README.md`
- `codex-rs/app-server/src/lib.rs`
- `codex-rs/app-server/src/request_processors.rs`
- `codex-rs/app-server-protocol/src/protocol/common.rs`
- `codex-rs/app-server-protocol/src/protocol/v2/`
- `codex-rs/codex-mcp/src/`
- `codex-rs/mcp-server/src/`
- `codex-rs/sandboxing/src/`
- `codex-rs/linux-sandbox/README.md`
- `codex-rs/login/src/`
- `codex-rs/keyring-store/src/lib.rs`
- `codex-rs/secrets/src/lib.rs`
- `codex-rs/cloud-tasks/src/`
- `codex-rs/cloud-tasks-client/src/`
- `codex-rs/core-plugins/src/`
- `codex-rs/plugin/src/lib.rs`
- `codex-rs/skills/src/lib.rs`
- `codex-rs/core-skills/src/`
- `codex-rs/state/src/lib.rs`
- `codex-rs/thread-store/src/lib.rs`
- `codex-rs/hooks/src/`
- `codex-rs/memories/`
- `codex-rs/file-search/`
- `codex-rs/file-watcher/src/lib.rs`
- `codex-rs/file-system/src/lib.rs`
- `codex-rs/config/src/`
- `codex-rs/model-provider/src/lib.rs`
- `codex-rs/model-provider-info/src/lib.rs`
- `codex-rs/models-manager/src/lib.rs`
- `codex-rs/analytics/src/lib.rs`
- `codex-rs/otel/src/lib.rs`
- `codex-rs/codex-api/src/`
- `codex-rs/chatgpt/`
- `codex-rs/codex-client/README.md`
- `codex-rs/responses-api-proxy/README.md`
- `sdk/typescript/`
- `sdk/python/README.md`
