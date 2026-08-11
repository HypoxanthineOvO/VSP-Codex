# Question Tool Native Context Message

## 结论

用户反馈的 `Question Context / Review this visible context before answering the Question Tool prompt / Answer impact` 不是模型自然回复，也不是 VSCode 插件里真正要交互的 Question Tool 卡片；它是 app-server 在调用原生 Question Tool 之前合成的一条额外 `AgentMessage`。

本轮已移除这条合成消息：支持原生 Question Tool 的 VSCode/插件路径现在会直接发送 `item/tool/requestUserInput` 请求。模型如果在调用工具前自己写了中文解释，那段自然说明仍会保留；app-server 不再自动整理并插入英文 `Question Context` 块。

Fallback 客户端仍保留 Markdown 文本回答机制：没有原生 Question Tool UI 的客户端会继续看到可编号回复的 `Workflow Surface / Request Input` fallback，并在回复编号后解析为工具答案。

本机运行入口已重新构建并部署：`~/.vsp-codex/packages/standalone/current/bin/vsp-codex` 现在与本次 release build 哈希一致，SHA256 为 `2e4ca430aac7dd5c258845ec0b0daa7486d74532a50c70554870f92ab91a3cfc`。旧安装二进制 SHA256 `33b19d3821023a0e954c24fcc47fcd88474f67631b157c7c4f9fee077e2b29b0` 已备份到 `~/.vsp-codex/packages/standalone/backups/vsp-codex-question-context-20260622T162034+0800`。

## 技术思路

- 根因：`EventMsg::RequestUserInput` 的原生 Question Tool 分支此前会通过 app-server 合成可见上下文消息，把工具参数里的 summary、description、question options 整理成 `Question Context` 和 `Answer impact` 后插入聊天流。
- 修复策略：原生 Question Tool 路径只发送协议层 `ToolRequestUserInput` 请求，不再合成前置 `AgentMessage`。这样 VSCode 插件看到的是普通模型说明后接 Question Tool 卡片，而不是额外英文上下文块。
- 兼容策略：无原生 UI 客户端继续走 `workflow_surface_fallback_prompt()`，保留 Markdown fallback 和编号回复解析；这保证旧客户端不会卡住。

## 修改文件/模块

- `codex-rs/app-server/src/bespoke_event_handling.rs`：保留原生 `ToolRequestUserInput` 发送路径，fallback 路径继续发送 Markdown fallback 和 unlock turn-completed。
- `codex-rs/app-server/src/request_user_input_fallback.rs`：fallback 文案仍只用于不支持原生 Question Tool 的客户端，不再承担原生卡片前置上下文职责。
- `codex-rs/app-server/tests/suite/v2/request_user_input.rs`：新增/收紧 native Question Tool 回归断言，确保原生路径不会出现 `*-ask-context`、`## Question Context`、`Review this visible context...` 或 `**Answer impact**` 合成消息。
- 本机安装入口：`~/.vsp-codex/packages/standalone/current/bin/vsp-codex` 已部署新 release build。

## 测试设计

- 用 app-server focused integration test 触发真实 `request_user_input` tool call，声明客户端支持原生 Question Tool，然后断言第一条工具请求前没有合成 agent message。
- 保留已完成的 request_user_input 全过滤回归、core tool description/guidance 测试、TUI file-link 测试、rustfmt、diff check 和 CI docs 检查作为周边验证。
- 用 SHA256 和 `--version` 验证 VSCode 插件后续加载的 managed standalone 二进制已经替换到本机新版本。

## 验证结果

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input_native_ask_ui_does_not_emit_synthetic_question_context_message -- --nocapture`：1 passed。测试输出中直接收到 `item/tool/requestUserInput` 请求，没有前置 `Question Context` agent message。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input_ -- --nocapture`：12 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core request_user_input_tool_description --lib`：2 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_plan_guidance_requires_visible_artifacts_before_question_cards --test all`：1 passed。
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check`：通过，仅有既有 stable rustfmt `imports_granularity=Item` warning。
- `git diff --check`：通过。
- `python3 scripts/check_ci_docs.py`：通过。
- `cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex`：通过。
- `~/.vsp-codex/packages/standalone/current/bin/vsp-codex --version`：`vsp-codex 0.140.0-vsp.0`。
- `sha256sum codex-rs/target/release/vsp-codex ~/.vsp-codex/packages/standalone/current/bin/vsp-codex`：两者均为 `2e4ca430aac7dd5c258845ec0b0daa7486d74532a50c70554870f92ab91a3cfc`。

## 预期结果

重载 VSCode/插件会话后，再触发 Workflow 的 Question Tool 决策时，聊天里不应再出现额外的英文 `Question Context / Answer impact` 块。正确顺序应是：

1. 模型自然输出必要的中文说明或报告摘要。
2. VSCode 插件显示真正的 Question Tool 卡片/弹窗。
3. 用户在卡片中选择答案。

没有原生 Question Tool UI 的客户端仍会看到 Markdown fallback 文本，并可按编号回复。

## 遇到的问题

- 当前仍有已运行 app-server 进程指向旧 release 二进制的 `(deleted)` inode，包括 PID `502560`、`2696644`、`2706753`。这些进程不会自动加载新二进制。
- 本轮只做本机 managed standalone 部署，没有推送代码、没有刷新 GitLab Release asset。

## 风险/后续

- 必须重载 VSCode 窗口或重启插件会话，才能让 app-server 从新安装入口启动。
- 其他机器仍需后续 release asset 刷新或重新安装，才能获得同样行为。
