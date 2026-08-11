# Question Tool Plugin Fallback And TUI Links

## 结论

本轮修复两个用户可见问题：

- 插件/客户端没有原生 Ask UI 时，`request_user_input` fallback Markdown 已经显示，但原 turn 没有释放输入框，用户看到像是一直“正在思考”。现在 fallback 消息发出后会发送一个合成 `turn/completed` 解锁输入，同时隐藏 Ask 请求保持可恢复，用户用编号回复后会先 resolve 原 Ask，再完成真实 turn。
- TUI 里的文件链接不再把 VSCode Remote 场景改写成 `vscode-remote://...` OSC8 链接。VSCode 集成终端现在对 file link 不输出 OSC8，让 VSCode 自己的 terminal file-link provider 接管 Ctrl+单击，从而在当前 VSCode/Remote 窗口直接打开文件，而不是走外部/Remote URI 弹窗。

## 技术思路

- 强化 `request_user_input` 工具和 Workflow routing 说明：在 Ask/Question Tool 前必须先输出普通 assistant 消息，展示完整决策上下文、报告摘要、验证结果和风险；Ask 卡片只承担选择控件，不替代说明。
- app-server 在 native Ask UI 路径下，如果模型跳过了可见上下文，会从 Ask summary/description 合成一条普通 agent message 作为兜底。
- app-server 在非 native Ask UI 路径下，把 `request_user_input` 保存为线程生命周期级隐藏请求，向客户端发 readable Markdown fallback，并立即发送合成 `turn/completed`，避免插件输入框被 turn in-progress 状态锁住。
- 对 `"VS Code"` / `"Visual Studio Code"`、官方 VSCode/ChatGPT 扩展和 Desktop/remote 这类已知 popup-capable client name，即使没有声明 `nativeAskUi` 也走 native Ask；普通未知客户端仍必须显式声明 native Ask UI，否则走 fallback 解锁路径。
- TUI 保留 markdown file link 的语义目标和精确可点击文本范围，但在 VSCode integrated terminal 渲染 OSC8 时抑制 file OSC8；Web URL 和非 VSCode file OSC8 保持原行为。

## 修改文件/模块

- `codex-rs/core/src/tools/handlers/request_user_input_spec.rs`
- `codex-rs/core/src/workflow_routing.rs`
- `codex-rs/app-server/src/bespoke_event_handling.rs`
- `codex-rs/app-server/src/outgoing_message.rs`
- `codex-rs/app-server/src/request_processors/thread_lifecycle.rs`
- `codex-rs/app-server/src/request_user_input_fallback.rs`
- `codex-rs/app-server/src/thread_state.rs`
- `codex-rs/app-server/src/native_ask_ui.rs`
- `codex-rs/app-server/tests/suite/v2/request_user_input.rs`
- `codex-rs/tui/src/terminal_hyperlinks.rs`

本轮没有 push、tag、GitLab Release 或 release asset 替换。

## 测试设计

- app-server fallback tests 覆盖未知客户端无 native Ask UI、无能力声明等插件形态；断言 fallback 后先有合成 `turn/completed`，用户回复后先 `serverRequest/resolved` 再完成原 turn。
- native Ask tests 覆盖 `"VS Code"` / `"Visual Studio Code"`、官方 popup-capable client name、Desktop remote client、visible context before native Ask、以及模型跳过文字时 app-server 合成 context prompt。
- core tests 覆盖工具说明和 Workflow runtime guidance 要求 Ask 前完整上下文。
- TUI tests 覆盖 VSCode integrated terminal 下 file OSC8 被抑制、Web OSC8 保留、非 VSCode file OSC8 保留、file link 点击范围不包含前缀和中文前缀。
- release build + 本地安装入口替换验证，确保 VSCode shim/managed standalone 后续启动能用到新代码。

## 验证结果

通过：

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server native_ask_ui -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server --test all request_user_input_ -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core request_user_input_tool_description_requires_visible_context_before_tool_call -- --nocapture`
- `cargo test -p codex-core request_user_input_tool_description --lib`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_guidance_requires_complete_context_before_any_question_tool -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-app-server-daemon managed_codex_bin -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --test vscode vscode_install_remote_scope_repoints_bundled_extension_codex -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui terminal_hyperlinks -- --nocapture`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui file_link_hyperlink -- --nocapture`
- `cargo test -p codex-tui terminal_hyperlinks --lib`
- `cargo test -p codex-tui file_link_hyperlink --lib`
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all`
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check`
- `git diff --check`
- `cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex`
- `~/.vsp-codex/packages/standalone/current/bin/vsp-codex --version`

本地部署：

- 先部署 Question Tool fallback 修复后，安装入口 SHA256 为 `e9f226aed43b26b99d15b0571e92b10e11353230a78dfa6f2e1b563d88e8c01d`，旧 SHA `0c8ccc30acf83167e7e9069bb4d3c02710ac06247484d853852206c4ea6c06bb` 备份在 `~/.vsp-codex/packages/standalone/backups/vsp-codex-ask-fallback-20260619T161425+0800`。
- 合并 TUI 链接修复后，安装入口 SHA256 曾为 `a2bb546d6391d3e2c6eb7411ef1eb69379ab5e8b2633b6201928ef9dd7b9407e`，上一版 SHA `e9f226aed43b26b99d15b0571e92b10e11353230a78dfa6f2e1b563d88e8c01d` 备份在 `~/.vsp-codex/packages/standalone/backups/vsp-codex-ask-fallback-tui-links-20260619T162713+0800`。
- 最终补强 Question Tool 工具说明并重建后，安装入口 SHA256 为 `b444971afe4c040351e453b90af71cff92a370ac160115a02483b112f1c51aea`，上一版 SHA `a2bb546d6391d3e2c6eb7411ef1eb69379ab5e8b2633b6201928ef9dd7b9407e` 备份在 `~/.vsp-codex/packages/standalone/backups/vsp-codex-question-tool-visible-context-20260619T165511+0800`。
- 安装入口版本返回 `vsp-codex 0.140.0-vsp.0`。

观察到的 warning 均为既有 stable rustfmt `imports_granularity` warning、`codex-core` dead-code warning、TUI dead-code warning。

## 预期结果

- 在没有 native Ask UI 的插件客户端里，Question Tool fallback 出现后输入框应恢复可用；用户按编号回复后继续原任务，不会卡在“正在思考”。
- 在已知 popup-capable 的 VSCode / ChatGPT / Desktop 客户端里，Question Tool 应走 native Ask，并在弹窗前显示可见上下文。
- 在 TUI/VSCode Remote 里，Ctrl+单击文件路径应由当前 VSCode 窗口打开远端文件，不再弹外部/Remote URI 处理提示。
- 在支持 native Ask UI 的官方 popup-capable 客户端里，原生 Ask 仍能收到 `item/tool/requestUserInput`，并且 Ask 前有可见上下文。

## 遇到的问题

- Debug skill 配置要求 strict worker separation；本次没有用户显式授权新 Subagent 委派，且当前工具规则要求只有用户显式请求并行/委派时才能 spawn subagent。因此本轮验证是本地 focused tests + source audit + runtime deployment，不能声称完成独立 test/implement/audit worker 分离。
- 尝试用一次性 Python 脚本对安装入口做真实 app-server fallback smoke 时，第一次误用 Content-Length framing，第二次 JSON-lines 可启动但手写 SSE 事件形状不完整，core retry 后超时。该 smoke 未计入通过验证；以仓库集成测试和 release build/installed version smoke 作为有效证据。

## 风险/后续

- 需要 reload VSCode / 重启 Remote SSH 会话，已运行的旧 app-server/TUI 进程不会自动加载新 `vsp-codex` 二进制。
- TUI 文件打开行为最终依赖 VSCode integrated terminal 的 native file-link provider；测试能证明我们不再注入会触发外部 URI handler 的 file OSC8，但实际 Ctrl+单击仍建议用户在目标 VSCode Remote 窗口人工确认一次。
- 本轮未刷新 GitLab Release asset；如果需要分发给其他机器，还需要走同版本 hotfix asset rebuild/push/release 流程。
