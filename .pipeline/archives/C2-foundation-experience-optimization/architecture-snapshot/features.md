# 功能列表

## 面向用户的功能

- 交互式 Codex CLI：默认运行 `codex [PROMPT]` 进入 TUI，支持模型选择、工作目录、附加图片、沙箱策略、审批策略、实时 web search、恢复/分叉历史会话、内联模式等。证据：`README.md`、`codex-rs/README.md`、`codex-rs/tui/src/cli.rs`、`codex-rs/cli/src/main.rs`。
- 非交互执行：`codex exec [PROMPT]` 从参数或 stdin 接收任务，支持 JSONL 输出、最后消息写入文件、schema 输出、临时会话、恢复历史会话和非交互 review。证据：`codex-rs/README.md`、`codex-rs/exec/src/cli.rs`、`./codex-rs/target/debug/codex exec --help`。
- 代码审查：`codex review` 和 `codex exec review` 运行非交互 review 流程。证据：`codex-rs/cli/src/main.rs`、`codex-rs/exec/src/cli.rs`。
- 登录与账号管理：`codex login`、`codex logout` 支持 ChatGPT 登录、API key、access token、device code 等认证路径。证据：`docs/authentication.md`、`codex-rs/cli/src/main.rs`、`codex-rs/cli/src/lib.rs`、`codex-rs/login/Cargo.toml`。
- MCP 配置管理：`codex mcp` 管理外部 MCP server 配置；`codex mcp-server` 将 Codex 作为 MCP server 暴露给其他 MCP client。证据：`codex-rs/README.md`、`codex-rs/cli/src/main.rs`、`codex-rs/codex-mcp/src/lib.rs`。
- 沙箱诊断与执行：`codex sandbox [COMMAND]...` 使用宿主平台沙箱运行命令，支持 macOS Seatbelt、Linux sandbox/bubblewrap、Windows sandbox。证据：`codex-rs/README.md`、`codex-rs/core/README.md`、`codex-rs/cli/src/lib.rs`。
- 插件和技能：`codex plugin` 管理插件 marketplace、插件安装状态，app-server 也提供 plugin/skills API。证据：`codex-rs/cli/src/main.rs`、`codex-rs/core-plugins/src/`、`codex-rs/core-skills/src/`、`codex-rs/app-server/README.md`。
- Codex Cloud 任务：`codex cloud`/`codex cloud-tasks` 支持浏览任务、提交任务、查询状态、查看 diff、应用结果。证据：`codex-rs/cloud-tasks/src/cli.rs`、`codex-rs/cli/src/main.rs`。
- 更新、诊断、补全、apply：`codex update`、`codex doctor`、`codex completion`、`codex apply` 提供安装健康检查、shell completion 和应用 Codex 生成 diff。证据：`./codex-rs/target/debug/codex --help`、`codex-rs/cli/src/main.rs`。

## 接口/API/协议功能

- app-server JSON-RPC v2：`codex app-server` 为 VS Code 扩展等富客户端提供 JSON-RPC 2.0 风格接口，传输支持 stdio、websocket、unix socket 和 off。证据：`codex-rs/app-server/README.md`、`codex-rs/app-server/src/main.rs`。
- Thread/Turn/Item API：app-server 以 thread、turn、item 为核心抽象，支持 thread start/resume/fork/list/read/archive/rollback、turn start/steer/interrupt、review start、compact start 等。证据：`codex-rs/app-server/README.md`、`codex-rs/app-server-protocol/src/protocol/v2/thread.rs`、`turn.rs`、`item.rs`。
- 命令与进程 API：app-server 暴露 `command/exec`、`process/spawn`、stdin 写入、PTY resize、terminate、output delta 等本地执行接口。证据：`codex-rs/app-server/README.md`、`codex-rs/app-server-protocol/src/protocol/v2/command_exec.rs`、`process.rs`。
- 文件系统 API：app-server 暴露 read/write/create/list/remove/copy/watch/unwatch 等文件系统 RPC。证据：`codex-rs/app-server/README.md`、`codex-rs/app-server-protocol/src/protocol/v2/fs.rs`。
- 模型、权限和功能 API：app-server 提供 model list、model provider capabilities、experimental features、permission profiles、config read/write/batchWrite。证据：`codex-rs/app-server/README.md`、`codex-rs/app-server-protocol/src/protocol/v2/model.rs`、`permissions.rs`、`config.rs`。
- MCP app-server API：提供 mcp server status、resource read、tool call、OAuth login 等接口。证据：`codex-rs/app-server/README.md`、`codex-rs/app-server-protocol/src/protocol/v2/mcp.rs`。
- schema 生成：`codex app-server generate-ts` 与 `generate-json-schema` 导出协议绑定/JSON schema；`just write-app-server-schema` 刷新 fixture。证据：`codex-rs/app-server/README.md`、`justfile`。
- exec-server JSON-RPC：`codex exec-server` 是独立小型 JSON-RPC server，管理进程和文件系统，支持本地 websocket/stdio 和远程 relay。证据：`codex-rs/exec-server/README.md`、`codex-rs/cli/src/main.rs`。
- SDK 接口：TypeScript SDK 包装 `@openai/codex` CLI 并通过 stdin/stdout JSONL 事件交互；Python SDK 面向 app-server JSON-RPC v2。证据：`sdk/typescript/README.md`、`sdk/python/README.md`。

## 底层服务功能

- 核心 Agent 业务逻辑：`codex-core` 管理会话、turn、任务、工具注册、上下文、配置、审批、安全策略、MCP 工具调用、插件/技能注入、rollout、状态桥接、realtime、review。证据：`codex-rs/core/README.md`、`codex-rs/core/src/`。
- 配置系统：读取 `~/.codex/config.toml`、profile、CLI override、managed requirements，并支持 schema 生成。证据：`docs/config.md`、`codex-rs/core/src/config/`、`justfile`。
- 沙箱系统：macOS Seatbelt、Linux Landlock/bubblewrap、Windows restricted token/elevated sandbox，统一由 core/config/sandboxing crates 协调。证据：`codex-rs/core/README.md`、`codex-rs/sandboxing/Cargo.toml`、`codex-rs/linux-sandbox/README.md`。
- 进程执行与 PTY：`codex-exec-server`、`codex-utils-pty`、`unified_exec` 负责本地/远程进程、PTY、stdin/stdout、终止、文件系统操作。证据：`codex-rs/exec-server/README.md`、`codex-rs/core/src/unified_exec/`。
- 状态与历史：`codex-state`、`thread-store`、`rollout`、`rollout-trace` 处理 SQLite 状态、会话文件、trace replay/reduce。证据：`codex-rs/Cargo.toml`、`codex-rs/core/src/rollout.rs`、`codex-rs/rollout-trace/README.md`。
- Auth 与后端客户端：`login`、`chatgpt`、`backend-client`、`codex-client`、`codex-api`、`aws-auth` 处理认证、OpenAI/ChatGPT 后端请求、Agent Identity、API 类型。证据：`codex-rs/Cargo.toml`、`codex-rs/cli/src/main.rs`。
- 本地模型/OSS provider：`lmstudio`、`ollama`、`model-provider`、`model-provider-info` 支持本地或可配置 provider。证据：`codex-rs/Cargo.toml`、`codex-rs/cli/src/main.rs`。
- 可观测性与诊断：`otel`、analytics、feedback、doctor、日志配置和 app-server tracing。证据：`codex-rs/otel/README.md`、`codex-rs/app-server/README.md`、`codex-rs/cli/src/main.rs`。
- Secrets 与 keyring：`secrets`、`keyring-store` 管理本地 secret backend、keyring 存储、ephemeral/file 凭据路径。证据：`codex-rs/secrets/src/lib.rs`、`codex-rs/keyring-store/src/lib.rs`、`codex-rs/config/src/types.rs`。
- 文件能力：`file-system` 抽象本地/远程 filesystem，支持 read/write/list/metadata/copy/remove 与 sandbox context；`file-watcher` 负责变更监听；`file-search` 提供模糊文件搜索。证据：`codex-rs/file-system/src/lib.rs`、`codex-rs/file-watcher/src/lib.rs`、`codex-rs/file-search/src/lib.rs`。
- 记忆系统：`memories/read` 在上下文中引用/注入 memory，`memories/write` 做启动记忆流水线、raw memories、rollout summaries 和阶段性 consolidation；`memories/mcp` 暴露只读 memory tools。证据：`codex-rs/memories/read/src/lib.rs`、`codex-rs/memories/write/src/lib.rs`、`codex-rs/memories/mcp/src/server.rs`。
- Hooks：支持 PreToolUse、PermissionRequest、PostToolUse、Pre/PostCompact、SessionStart、UserPromptSubmit、SubagentStart/Stop、Stop 等用户扩展点。证据：`codex-rs/hooks/src/lib.rs`。

## 外部集成/外部功能

- npm 分发：`@openai/codex` 的 JS 包根据平台加载原生二进制，支持 Linux/macOS/Windows 多 target package。证据：`codex-cli/package.json`、`codex-cli/bin/codex.js`。
- Homebrew、GitHub Release、DotSlash：README 与 install 文档描述多渠道安装和平台 release 包。证据：`README.md`、`docs/install.md`。
- IDE/富客户端：app-server 是 VS Code 扩展等富客户端接口，支持 remote-control daemon、proxy、schema 生成。证据：`codex-rs/app-server/README.md`、`codex-rs/app-server-daemon/README.md`。
- MCP 生态：Codex 可作为 MCP client 启动外部 MCP servers，也可作为 MCP server 被其他 agent 调用。证据：`codex-rs/README.md`、`codex-rs/codex-mcp/src/lib.rs`、`codex-rs/mcp-server/src/lib.rs`。
- Codex Cloud：本地 CLI 可提交、浏览、应用云端任务结果，并与 ChatGPT auth 和环境 id 结合。证据：`codex-rs/cloud-tasks/src/cli.rs`、`codex-rs/cloud-tasks/src/lib.rs`。
- 远程 exec environment：`codex exec-server --remote URL --environment-id ID` 将本地环境注册到远程 rendezvous，支持 Agent Identity JWT 或 ChatGPT auth。证据：`codex-rs/exec-server/README.md`、`codex-rs/cli/src/main.rs`。
- SDK 嵌入：TypeScript SDK 和 Python SDK 让外部程序把 Codex agent 嵌入工作流。证据：`sdk/typescript/README.md`、`sdk/python/README.md`。
- OpenAI/ChatGPT API：`codex-api` 封装 Responses、Responses WebSocket、Realtime、Images、Search、Models、Memories 等 endpoint；`chatgpt` crate 承载 Cloud task、connectors/apps、workspace settings 等 first-party 集成。证据：`codex-rs/codex-api/src/lib.rs`、`codex-rs/chatgpt/README.md`、`codex-rs/chatgpt/src/connectors.rs`。
- 模型 provider：内置 OpenAI、ChatGPT Codex base URL、Amazon Bedrock，并支持 OpenAI-compatible provider、env/header/auth command/AWS SigV4 配置。证据：`codex-rs/model-provider-info/src/lib.rs`、`codex-rs/model-provider/src/lib.rs`。
- Responses API proxy：提供受限本地代理，只允许 `POST /v1/responses`，从 stdin 读取 API key，可 dump 请求/响应并做进程 hardening。证据：`codex-rs/responses-api-proxy/README.md`。
