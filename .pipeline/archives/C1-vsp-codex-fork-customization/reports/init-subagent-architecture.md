# Init Subagent Report: 代码架构

状态：只读探索完成，未修改文件，未运行写入型命令或测试。

## 总体结构

- 仓库是混合 monorepo：Rust 主体在 `codex-rs`，npm/发布包装在 `codex-cli`，TypeScript SDK 在 `sdk`。
- 根部 `justfile` 将常用开发命令默认落到 `codex-rs`；`pnpm-workspace.yaml` 覆盖 npm wrapper、responses proxy npm 包和 TS SDK。
- Rust workspace 在 `codex-rs/Cargo.toml`，采用多 crate 分层，`codex-rs/README.md` 明确将 `core`、`exec`、`tui`、`cli` 作为主要组织边界。

## Rust crate 分层

- 入口/产品层：`codex-rs/cli` 是 `codex` 多工具入口；`codex-rs/tui` 是交互式终端 UI；`codex-rs/exec` 是 headless CLI；`codex-rs/app-server` 是 JSON-RPC 服务端；`codex-rs/mcp-server` 把 Codex 暴露成 MCP server；`codex-rs/cloud-tasks` 是云任务 CLI。
- 协议层：`codex-rs/protocol` 定义 core 内部 `Op/EventMsg/SandboxPolicy` 等会话协议；`codex-rs/app-server-protocol` 定义 v2 JSON-RPC API；`codex-rs/app-server-transport` 负责 stdio/websocket/unix socket 传输。
- 核心运行层：`codex-rs/core` 包含 `ThreadManager`、`CodexThread`、session、turn loop、model client、tools、MCP/plugin/skills/sandbox 适配。
- 执行与隔离层：`codex-rs/exec-server` 抽象 local/remote execution environment；`codex-rs/sandboxing` 统一 bwrap/landlock/seatbelt/windows sandbox 转换；另有 `linux-sandbox`、`windows-sandbox-rs` 平台实现。
- 能力扩展层：`codex-rs/codex-mcp` 是 MCP client/connection manager；`core-skills` 和 `skills` 处理技能发现与系统技能；`plugin` 和 `core-plugins` 处理插件 manifest、marketplace、remote sync、MCP/apps/hooks/skills 聚合。
- 持久化/状态层：`rollout` 负责 JSONL 会话历史；`thread-store` 是线程存储抽象；`state` 是 SQLite 元数据/索引库。

## 核心运行链路

- `codex` 入口在 `codex-rs/cli/src/main.rs`：无子命令进入 TUI；`exec` 进入 `codex_exec::run_main`；`app-server` 启动服务；`mcp-server` 启动 MCP server；`plugin/cloud/sandbox` 等是独立子命令。
- TUI 路径：`cli -> codex_tui::run_main -> AppServerSession -> ClientRequest(thread/start, turn/start, ...) -> app-server -> ThreadManager -> CodexThread -> session submission_loop -> RegularTask/run_turn -> ModelClient -> ToolRouter/ToolRegistry -> EventMsg -> app-server notifications -> TUI render`。
- Headless exec 路径类似，但 `codex-rs/exec/src/lib.rs` 通过 in-process app-server 发 `thread/start` 和 `turn/start`，再用 human/jsonl event processor 输出。
- app-server 是当前 UI/CLI 与 core 的主边界：`codex-rs/app-server/src/message_processor.rs` 统一 dispatch RPC；thread/turn 分别落到 `thread_processor.rs` 和 `turn_processor.rs`。
- in-process 路径没有绕过协议：`codex-rs/app-server/src/in_process.rs` 复用 `MessageProcessor` 和 JSON-RPC envelope；`app-server-client/src/lib.rs` 给 TUI/exec 提供统一 client facade。

## 模块边界

- TUI：负责 ratatui UI、输入、事件消费、会话选择、diff/render 等；通过 `tui/src/app_server_session.rs` 调 app-server，不应直接承载业务逻辑。
- CLI：arg0/subcommand 分发层，聚合 app-server、exec、tui、plugin、cloud、sandbox 等入口。
- app-server：对外暴露 v2 API，通过 request processors 调 core、config、filesystem、process、plugin、MCP 等服务。
- exec：非交互表面，默认 headless 策略，事件处理在 `event_processor*`，业务仍走 app-server/core。
- sandbox/exec：core tool runtime 先做 approval/exec-policy，再由 `SandboxManager` 选择 sandbox，最后经 shell/unified exec runtime 或 exec-server 执行。
- MCP：`codex-mcp/src/connection_manager.rs` 负责启动 MCP clients、汇总 tools/resources/templates、处理 elicitation、路由 `call_tool`；core 的 `McpManager` 把 config/plugin MCP 合成运行时 MCP 配置。
- skills：`codex-rs/skills` 只负责嵌入系统技能缓存到 `CODEX_HOME/skills/.system`；`core-skills` 负责 skill root 发现、`SKILL.md` 解析、配置禁用规则、显式 `$skill` 提及解析和 prompt 注入。
- plugin：`plugin` 是通用数据模型/manifest ID；`core-plugins` 负责 marketplace、安装缓存、remote installed cache、manifest 解析，并把插件能力展开为 skills/MCP servers/apps/hooks。
- cloud：`cloud-tasks` 是 `codex cloud` CLI；`cloud-tasks-client` 定义 `CloudBackend` 和 HTTP 实现；`backend-client` 封装 ChatGPT/Codex backend paths；`cloud-requirements` 在启动配置阶段拉取 workspace-managed policies；`agent-identity` 负责云端 agent identity JWT、注册和 task assertion。

## 主要数据流

- 用户输入流：TUI/exec/app client 发 `turn/start`，app-server 转成 core `Op::UserInput`，`submission_loop` 建 task，`run_turn` 构造 prompt/tools，model stream 触发 tool calls 和 follow-up sampling，最后产出 `TurnComplete`。
- 工具调用流：model `ResponseItem::*ToolCall` 到 `ToolRouter::build_tool_call`，再到 `ToolRegistry::dispatch` 和 runtime handler，经过 approval/sandbox/orchestrator 后执行 shell/MCP/app/fs/patch 等具体能力。
- MCP 数据流：config + plugin MCP servers 到 `Config::to_mcp_config`，再到 `McpManager` 和 `McpConnectionManager`，启动 clients 后把 tools 归一化暴露给 model。
- Skills/plugin 数据流：config layer + marketplace/remote cache 到 `PluginsManager::plugins_for_config`，展开 plugin skill roots/MCP/apps/hooks；skills manager 再按 cwd/config 加载 `SKILL.md`。
- 持久化流：core thread 创建/resume 时通过 `ThreadStore` 打开 live writer；canonical history 写 rollout JSONL；SQLite `state` 保存可查询 thread metadata、goals/logs 等索引。
- 云端流：`codex cloud` 用 ChatGPT auth 初始化 backend client，列任务/取 diff/创建任务/本地 apply；启动配置还可能通过 cloud requirements 拉取强制策略并缓存。

## 依赖方向

推荐理解为：表面层 `cli/tui/exec/app-server/mcp-server/cloud` -> 协议/client facade -> `app-server` -> `core` -> model/tools/MCP/plugin/skills/exec/sandbox -> utils/state/rollout。

`protocol` 和 `app-server-protocol` 是共享契约；UI 不应反向进入 core 内部细节，现有 `app-server-client::legacy_core` 是迁移桥。`core` 是业务中心但已经很大；新功能应优先落到专门 crate 或 app-server processor，而不是继续扩大 `codex-core`。

## 证据文件

- `README.md`
- `codex-rs/README.md`
- `codex-rs/Cargo.toml`
- `justfile`
- `codex-rs/cli/src/main.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/tui/src/app_server_session.rs`
- `codex-rs/app-server/README.md`
- `codex-rs/app-server/src/message_processor.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/src/thread_manager.rs`
- `codex-rs/core/src/session/mod.rs`
- `codex-rs/core/src/session/handlers.rs`
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/core/src/tools/registry.rs`
- `codex-rs/exec/src/lib.rs`
- `codex-rs/sandboxing/src/lib.rs`
- `codex-rs/codex-mcp/src/connection_manager.rs`
- `codex-rs/core-skills/src/manager.rs`
- `codex-rs/core-plugins/src/manager.rs`
- `codex-rs/cloud-tasks/src/lib.rs`
- `codex-rs/thread-store/src/store.rs`
- `codex-rs/rollout/src/recorder.rs`
