# P1 前上下文整理：Codex-VSP Fork 改造相关功能

时间：2026-05-24T19:20:00+08:00

## 当前规划目标

这次规划目标是面向 Codex 代码 Fork 做定制改造：

- 清理或弱化不需要的 OAuth、登录、Base URL、Plan Mode 等提示。
- 对 TUI 界面和输出内容进行汉化、美化。
- 先构建尽量完整的现有功能回归测试，尤其是模拟 TUI 使用者的测试。
- Fork 不要求保留官方原始体验，但需要考虑后续合并官方更新的策略。

P0 Configure 已确认：

- Worker Separation 使用 `strict`。
- 后续 `/hw:start` 和 `/hw:resume` 授权使用 `test`、`implement`、`audit` 三类执行 Subagent。
- 验收以完整本地回归为优先，TUI 改动需要 snapshot/模拟用户测试覆盖。

## 与本次改造最相关的功能区域

### 1. TUI 交互界面

入口和主链路：

- `codex-rs/cli/src/main.rs`：无子命令时进入 TUI。
- `codex-rs/tui/src/lib.rs`：TUI 启动、配置和运行主流程。
- `codex-rs/tui/src/app.rs`：App 构建、bootstrap、运行时状态。
- `codex-rs/tui/src/chatwidget.rs`：核心聊天界面、计划流、审批、弹窗、状态交互。
- `codex-rs/tui/src/bottom_pane/`：输入框、footer、快捷键提示、选择器、弹窗。
- `codex-rs/tui/src/history_cell/`：会话头、消息、计划、MCP、通知等历史渲染。

当前 Plan Mode 相关 UI 主要分布：

- `codex-rs/tui/src/chatwidget.rs`
  - `PLAN_MODE_REASONING_SCOPE_TITLE`
  - `PLAN_MODE_REASONING_SCOPE_PLAN_ONLY`
  - `PLAN_MODE_REASONING_SCOPE_ALL_MODES`
  - Plan Mode nudge 状态和刷新逻辑。
- `codex-rs/tui/src/bottom_pane/chat_composer.rs`
  - `plan_mode_nudge_line()`
  - footer 中 `use Plan mode` 文案。
- `codex-rs/tui/src/bottom_pane/footer.rs`
  - `CollaborationModeIndicator::Plan => "Plan mode..."`
  - 多个 footer snapshot 已覆盖 Plan mode 显示。

### 2. Base URL / Provider 状态提示

相关位置：

- `codex-rs/tui/src/app.rs`
  - `resolve_runtime_model_provider_base_url`
  - `runtime_model_provider_base_url` 传入 chat widget。
- `codex-rs/tui/src/chatwidget.rs`
  - 保存 `runtime_model_provider_base_url`，可能参与状态显示或提示。
- `codex-rs/config/src/`
  - `openai_base_url`、`chatgpt_base_url`、provider 配置契约。
- `codex-rs/model-provider/src/` 和 `codex-rs/model-provider-info/src/`
  - provider base URL、认证模式、runtime URL 解析。

### 3. OAuth / 登录 / ChatGPT app auth

用户可见登录入口：

- `codex-rs/cli/src/main.rs`
  - `codex login`
  - `--with-api-key`
  - `--with-access-token`
  - `--device-auth`
  - `issuer_base_url`、`client_id` 等高级 OAuth 参数。
- `codex-rs/cli/src/lib.rs`
  - 对 login 运行函数的导出。
- `codex-rs/login/src/`
  - OAuth callback server、device code login、token parsing、auth.json 保存。

MCP / ChatGPT app auth：

- `codex-rs/codex-mcp/src/connection_manager.rs`
  - MCP auth 错误、OAuth 不支持、未登录提示。
- `codex-rs/codex-mcp/src/auth_elicitation.rs`
  - “Reconnect ... on ChatGPT”“Sign in ... on ChatGPT”等提示。
- `codex-rs/tui/src/bottom_pane/app_link_view.rs`
  - ChatGPT app install/manage/sign-in/reconnect UI。
- `codex-rs/tui/src/history_cell/mcp.rs`
  - MCP auth 状态显示，包括 `OAuth`。

### 4. 可能被清理或弱化的功能簇

需要在 P1/P2 明确是否删除、隐藏、保留但汉化，或默认禁用：

- ChatGPT 浏览器 OAuth 登录和 device auth。
- API key / access token 登录入口。
- MCP server OAuth 登录和 ChatGPT hosted apps。
- Codex Cloud tasks。
- Plugins marketplace、插件 app auth、ChatGPT app install/manage。
- Plan Mode nudge、Plan mode reasoning override 提示。
- Base URL/provider 状态提示。
- 更新检查、feedback/upload、doctor 中面向官方渠道的提示。
- “OpenAI Codex” 品牌会话头、安装来源、官方文档链接、官方计划类型显示。

### 5. 建议的 Fork 合并策略

为了后续合并官方更新，不建议第一阶段直接大面积删除 core/auth/MCP/cloud 代码。更稳的路线：

- 优先在 UI/CLI 边界做隐藏、降噪、汉化和默认策略调整。
- 把 Fork 定制集中成少数新模块或常量层，例如 TUI 文案、品牌、feature visibility、默认配置策略。
- 保持 app-server 协议和 core 数据结构尽量少改，降低后续 merge 冲突。
- 对确实不需要的功能，先做“不可见/不可触发/默认禁用”，待回归测试稳定后再考虑物理删除。
- 对官方强耦合服务如 Cloud、ChatGPT apps、OAuth callback server、MCP OAuth，优先通过入口隐藏和文案替换降低干扰，而不是立即删除底层 crate。

## 测试现状

TUI 现有测试资源：

- `codex-rs/tui/src/**/snapshots/`：大量 `insta` snapshot，覆盖 footer、composer、approval、history cell、app link view、request_user_input、chatwidget 等。
- `codex-rs/tui/tests/test_backend.rs`：VT100 测试 backend。
- `codex-rs/tui/tests/suite/vt100_history.rs` 和 `vt100_live_commit.rs`：终端渲染/历史测试。
- `codex-rs/tui/tests/suite/resize_reflow.rs`：tmux 驱动的忽略测试，可作为真实 TUI 用户场景参考。
- `codex-rs/tui/src/external_agent_config_migration.rs` 内已有按键处理和 snapshot 测试，可参考其“模拟按键 -> 渲染快照”模式。

可用验证命令：

- `just test -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui`
- `cargo insta show -p codex-tui <snapshot.snap.new>`
- `cargo insta accept -p codex-tui`

仓库要求：修改 TUI 用户可见输出时必须更新或新增 snapshot；测试使用 `just test`，不要直接跑 `cargo test`。

## P1 需要继续确认的问题

- 这次 Fork 的品牌名、语言风格和视觉风格。
- 是否只隐藏提示，还是删除/禁用对应功能入口。
- 默认认证路线是什么：仅 API key、自定义 provider、内置 base URL、还是完全不在 UI 中引导登录。
- 是否保留高级用户可通过 config 手动启用 OAuth/MCP/Cloud/plugin，还是完全从入口移除。
- 真实 TUI 使用者测试的目标深度：组件 snapshot、VT100 模拟按键、tmux 端到端，或三者都要。
