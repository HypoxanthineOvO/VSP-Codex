# C9 Debug: Provider Resume Empty Override Runtime Fix

## 改动摘要

用户复测指出：新建 VSP-Codex 会话后，VSCode 插件仍报 `Failed to resume chat: failed to load configuration: Model provider `` not found`。这说明前一次处理不够，只补历史 provider 数据和 wrapper 入口，仍未封住 app-server resume 请求中空 provider 覆盖配置的路径。

本轮修复了真正的运行时入口：`thread/start`、`thread/resume`、`thread/fork` 在加载配置前会移除空字符串的 `model_provider` / `modelProvider` config override；resume 判断 override 时只把非空 provider 当作有效覆盖。这样即使 VSCode client 发送 `modelProvider: ""` 或 `config.modelProvider: ""`，也不会覆盖 `~/.vsp-codex/config.toml` 中的 `vsp` provider。

## 技术思路

排序假设与验证：

1. 历史数据仍为空。排除。`~/.vsp-codex/state_5.sqlite` 和 `~/.codex/state_5.sqlite` 的空 provider thread count 均为 0。
2. VSCode 仍运行旧 app-server 进程。确认成立。`PID 639742` 与 `PID 3694191` 的 `/proc/<pid>/exe` 均指向 `/home/heyx/.vscode-server/extensions/openai.chatgpt-26.609.30741-linux-x64/bin/linux-x86_64/codex (deleted)`，环境中没有 `CODEX_HOME`。
3. 新 app-server 仍可能被空 provider 请求覆盖。确认成立并已修复。此前 `request_overrides` 中的空 `modelProvider` 会进入 config load，导致 `Model provider `` not found`；现在在 start/resume/fork 入口统一清理。
4. 派生 OpenCode/HW-Plan 规则仍可能让 Question Tool 前没有解释或最终只给报告路径。确认并加固。派生命令模板和 runtime guidance 均补充了“先展示产物再提问”和“最终聊天必须包含报告核心内容”的约束。

## 修改文件/模块

- `codex-rs/app-server/src/request_processors/thread_processor.rs`：新增空 provider config override 清理；resume metadata provider 仅在当前配置认识该 provider 时才恢复；stored thread summary/list 对空 provider 使用 fallback。
- `codex-rs/app-server/src/request_processors/thread_processor_tests.rs`：补充空 provider config override、persisted metadata fallback、非空 override 阻断 fallback、blank summary fallback 单测。
- `codex-rs/app-server/tests/suite/v2/thread_resume.rs`：补充 `thread/resume` 请求带空 provider 时回退 configured provider 的集成测试。
- `AGENTS.md`、`.opencode/agents/hw-plan.md`、`.opencode/agents/hw-report.md`、`.opencode/commands/hw:plan:architecture.md`、`.opencode/commands/hw:plan:technical-stack.md`、`opencode.json`：补充 Plan gate visibility 和 final report surface 约束。
- `codex-rs/core/src/workflow_routing.rs`、`codex-rs/core/tests/suite/workflow_routing.rs`：在 runtime guidance 和测试中要求 Question Tool / Ask 前先展示产物和解释决策意义，最终回答必须包含报告核心内容。
- `codex-rs/tui/src/markdown_render.rs`、`codex-rs/tui/src/markdown_render_tests.rs`、`codex-rs/tui/src/terminal_hyperlinks.rs`：本轮一起保留 TUI 本地文件链接修复，确保 Ctrl+click hyperlink 只覆盖文件路径文本。
- 运行态：重建并部署 `/home/heyx/.vsp-codex/packages/standalone/current/bin/vsp-codex`，SHA256 与 `codex-rs/target/release/vsp-codex` 一致：`e8eb16e904ad1c855badb699ff41a7bdb188b71cd482824a675fa4f394a61a2f`。

## 测试设计

测试覆盖三层：

1. 单元层：确认空 `model_provider` / `modelProvider` 会被移除，且不会阻断 persisted metadata fallback；非空 override 仍保持显式覆盖语义。
2. app-server 集成层：模拟 `thread/resume` 请求中带 `modelProvider: ""`，验证响应 provider 回退到 configured provider。
3. 真实运行态层：从 VSCode 扩展 bundled shim 路径启动 `app-server --listen stdio://`，对真实 VSP session `019edb30-5dc4-7061-92ef-100227f34dbb` 发送 `thread/resume`，请求同时带 `modelProvider: ""` 和 `config.modelProvider: ""`，验证返回 `modelProvider=vsp`。

## 验证结果

通过：

- `python3 -m json.tool opencode.json`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server empty_config_model_provider_override -- --nocapture`：3 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server c8_m3_thread_resume_empty_model_provider_falls_back_to_configured_provider -- --nocapture`：1 passed；输出显示请求 `modelProvider: ""` 返回 `modelProvider: "mock_provider"`。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_plan_guidance_requires_visible_artifacts_before_question_cards -- --nocapture`：1 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_completion_guidance_requires_core_report_content_in_final_chat -- --nocapture`：1 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui hyperlink -- --nocapture`：13 passed。
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all -- --check`：通过，仅 stable rustfmt 对 `imports_granularity = Item` 输出 warning。
- `git diff --check`：通过。
- `cargo build --manifest-path codex-rs/Cargo.toml -p codex-cli --release`：通过。
- `/home/heyx/.vscode-server/extensions/openai.chatgpt-26.609.30741-linux-x64/bin/linux-x86_64/codex --version`：`vsp-codex 0.140.0-vsp.0`。
- 真实运行态 smoke：从 VSCode bundled shim 启动的新 app-server 返回 `codexHome=/home/heyx/.vsp-codex`，对真实 thread resume 返回 `responseModelProvider=vsp`、`threadModelProvider=vsp`、`model=gpt-5.5`。
- `sqlite3 /home/heyx/.vsp-codex/state_5.sqlite` 与 `sqlite3 /home/heyx/.codex/state_5.sqlite` 空 provider count 均为 0。

已知风险：

- 宽过滤 `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_ -- --nocapture` 中，本轮新增的两个 workflow routing 测试已先后通过，但后续既有测试触发 `tokio-rt-worker has overflowed its stack` 并 abort。已用精确测试证明本轮 guidance 改动通过；宽过滤 stack overflow 需要单独治理。

## 预期结果

新拉起的 VSCode Codex 插件 app-server 应使用 `/home/heyx/.vsp-codex`，即便 resume 请求里带空 provider 字段，也应回退为配置中的 `vsp`，不再报 `Model provider `` not found`。

HW-Plan 后续在 Question Tool / Ask 之前，应先在聊天中展示阶段产物、解释为什么现在需要决策、各选项会改变什么，以及用户正在确认哪份产物或证据。完成或报告类回答不应只给路径，必须在聊天中包含结论、改动、技术路径、文件、测试、验证、问题和风险。

TUI 本地文件 Markdown 链接 Ctrl+click 只应覆盖文件路径文本，不覆盖前缀说明文字。

## 遇到的问题

- 前次修复不充分：修了历史数据、provider 持久化和 VSCode bundled 入口，但没有处理 client request/config 中空 provider override 覆盖配置的路径。
- 当前仍有两个用户的 VSCode app-server 旧进程：`PID 639742`、`PID 3694191`。它们的可执行文件是 `(deleted)`，且没有 `CODEX_HOME`，不会自动使用新部署的 shim 或二进制。
- 未强杀用户 VSCode/app-server 进程；这避免破坏用户当前编辑状态，但也意味着 VSCode 必须 reload 或重启 Remote SSH 才能切换到新入口。
- 本轮未使用独立 Subagent worker；当前工具策略要求用户显式授权子代理工作，故角色分离验证以本地聚焦测试和真实运行态 smoke 为主。

## 风险/后续

必须在 VSCode 执行 `Developer: Reload Window`，或重启 Remote SSH 窗口/插件宿主，让旧 `(deleted)` app-server 进程退出。reload 后如果仍报同样错误，优先检查是否还有 `/home/heyx/.vscode-server/.../codex (deleted)` app-server 进程残留，以及新进程的 `CODEX_HOME` 是否为 `/home/heyx/.vsp-codex`。

后续建议单独处理 `codex-core workflow_` 宽过滤 stack overflow，避免 CI 或本地 broad filter 再次把已通过的 focused tests 淹没在非目标 abort 中。
