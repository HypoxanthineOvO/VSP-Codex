# C4 P2 Technical Route: 体验优化 + 预览版发布闭环

状态：`proposed`  
来源：`.plan-state/p1-discover.yaml`  

本阶段只提出 Milestone 与技术路线，不生成执行 prompt。进入 P3 Generate 前需要用户确认。

## 总体顺序

1. M1 Release Spine + VSCode Hijack
2. M2 Canonical Rename
3. M3 Reasoning Capability Gate
4. M4 VSCode-Terminal-Safe Keymap + Composer Prominence
5. M5 Markdown Color Refinement + Default Subtle Blocks
6. M6 Performance Evidence + Low-Risk Startup Fixes
7. M7 Preview Release Validation

顺序理由：先打通发布与 VSCode 接入主路径，再修正跨 TUI/app-server/VSCode 可见的 session name；Thinking 先做能力判定，避免 UI 伪造；随后处理用户当前最痛的 VSCode 终端快捷键冲突和输入框；Markdown/颜色基于已有 renderer 做补齐；性能只做 evidence 和低风险优化；最后统一做 preview release closure。

## M1 Release Spine + VSCode Hijack

目标：让预览版二进制能自动接入官方 VSCode 插件，并提供可验证安装、回滚和 smoke。

技术方案：
- 新增 `vsp-codex vscode install/status/rollback` 一组 CLI 子命令。
- 写 VSCode settings 中的 `chatgpt.cliExecutable`，指向 released `vsp-codex`。
- 写前备份，rollback 恢复旧值；`chatgpt.localeOverride=zh-CN` 只在为空时写。
- 如果 VS Code CLI 可用且官方插件缺失，尝试安装 `openai.chatgpt`。
- 用 `app-server --analytics-default-enabled` initialize probe 验证官方插件启动路径。

触达模块：
- `codex-rs/cli/src/main.rs`
- 新增 `codex-rs/cli/src/vscode_cmd.rs`
- `codex-rs/app-server`
- release/quickstart/report artifacts

验证：
- `just test -p codex-cli vscode`
- `just test -p codex-app-server`
- initialize probe

## M2 Canonical Rename

目标：Rename 真改 canonical session/thread name，并在 TUI、app-server、VSCode history/list 可见。

技术方案：
- 复用 `thread/name/set` 和 `thread/name/updated`。
- 写 state DB canonical metadata，同时维护 rollout compatibility thread-name record。
- TUI `/rename` 保持 prefill、trim、空名拒绝、Esc cancel、Enter save。
- `thread/list`、`thread/read`、TUI resume list 都从 canonical metadata 显示新名。

触达模块：
- app-server thread processor/state
- rollout session index
- TUI rename/resume picker/session title
- app-server protocol existing v2 thread name notifications

验证：
- current loaded rename
- stored/unloaded rename
- app-server `thread/list/read`
- restart persistence
- TUI snapshot

## M3 Reasoning Capability Gate

目标：只展示真实 Thinking/Reasoning；没有 raw/summary 就明确降级，不伪造。

技术方案：
- 确认 core emitted reasoning summary/raw/section events。
- app-server v2 继续使用 `item/reasoning/summaryTextDelta`、`summaryPartAdded`、`textDelta`。
- TUI 增加 collapsed Thinking block，默认 collapsed。
- 三态显示：`off / summary / raw`；旧 bool config 映射为 summary/raw。

验证：
- core reasoning fixture
- app-server v2 mapping
- TUI summary/raw/off/unsupported snapshots

## M4 VSCode-Terminal-Safe Keymap + Composer Prominence

目标：必要操作不依赖 VSCode 集成终端冲突键，并让输入框更明显。

技术方案：
- 默认必要操作不依赖 `Ctrl+J/K`、`Ctrl+P`、`Ctrl+Shift+P`、`Alt`、`Ctrl+C`。
- Ask/list/ranking 内部移动用方向键。
- 组件间切换使用 Tab/Shift+Tab，保留 Enter/Esc/slash command 兜底。
- 修正 Ask overlay 当前 `ctrl+j/k to reorder` 和 `ctrl+p/n change question`。
- 输入框增加更清晰边框、主题 accent、淡背景和焦点提示。

验证：
- keymap constraint tests
- Ask overlay snapshots
- composer focused/unfocused snapshots

## M5 Markdown Color Refinement + Default Subtle Blocks

目标：在已有 Markdown renderer 上补齐颜色，让 emphasis/list/table 在暗色终端可见。

技术方案：
- 不重写 Markdown renderer；复用 `pulldown-cmark` 路径。
- 三角色 palette：emphasis text、code/link、structural elements。
- bold、italic、bold+italic 必须有非默认前景色。
- list marker、table header、table border、heading、blockquote marker 上色。
- inline code 改为主题派生 code 色。
- 默认 teal/amber-like 淡色块区分主要输出块，但不做完整 Color Theme picker。

触达模块：
- `codex-rs/tui/src/markdown_render.rs`
- `codex-rs/tui/src/markdown.rs`
- `codex-rs/tui/src/history_cell/*`
- TUI snapshots

验证：
- `just test -p codex-tui markdown_render`
- `just test -p codex-tui history_cell`
- `cargo insta pending-snapshots -p codex-tui`

## M6 Performance Evidence + Low-Risk Startup Fixes

目标：给 preview 提供性能证据，只做低风险启动优化，不做缓存系统。

技术方案：
- 记录 cold start、resume list、open historical session、VSCode initialize timing。
- 延后非首屏必要的 full history scan、provider/account probe、MCP/tool discovery。
- 不做 resume metadata cache、recent preview cache、progressive stale-cache verification。
- 性能数据只进 release evidence，不做 `/perf` 产品入口。

验证：
- no-network/slow-provider startup probe
- VSCode initialize timing evidence
- large-history fixture 或可复现 release script

## M7 Preview Release Validation

目标：把所有 blocker 串成可发布 preview。

技术方案：
- 跑 fmt/fix/focused tests/snapshots/schema。
- 构建 Linux x64 preview artifact；macOS arm64 should-have。
- 跑 VSCode install/status/rollback smoke。
- 跑 initialize、thread start/list/read/name set、reasoning verdict probes。
- 输出 quickstart、rollback、known limitations 和 release evidence。

验证：
- `just fmt`
- scoped `just fix -p ...`
- `just test -p codex-cli`
- `just test -p codex-tui`
- `just test -p codex-app-server`
- `just test -p codex-app-server-protocol`

## 明确顺延

- 完整 Color Theme picker、三组主题持久化和完整 fallback matrix。
- 完整 command palette/focus stack/键盘复制选择系统。
- resume metadata cache、recent preview cache、progressive stale-cache verification。
- 自研 VSCode 插件或 patch 官方插件 UI。
