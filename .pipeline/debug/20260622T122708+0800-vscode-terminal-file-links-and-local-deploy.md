# VSCode Terminal File Links And Local Deploy

## 结论

用户补充反馈的 TUI 文件链接问题已经收束：之前把 `file://` OSC8 链接改写成 `vscode-remote://...` 会触发 VSCode 的通用 URI/Remote 打开路径，表现为 Ctrl+单击后出现 Remote 或外部打开弹窗，而不是直接在当前 VSCode 窗口打开文件。

当前源码已采用正确策略：在 VSCode integrated terminal 中，TUI 对本地文件链接不再输出 OSC8 file URI，也不再生成 `vscode-remote://` destination；可见文本仍渲染为真实路径文本，让 VSCode 终端自己的 file-link provider 接管 Ctrl+单击。这样 VSCode 能按当前 local/remote window 上下文直接打开文件。

本机运行入口也已部署：`~/.vsp-codex/packages/standalone/current/bin/vsp-codex` 已替换为本次 release build，SHA256 为 `33b19d3821023a0e954c24fcc47fcd88474f67631b157c7c4f9fee077e2b29b0`；旧安装二进制 SHA256 `b444971afe4c040351e453b90af71cff92a370ac160115a02483b112f1c51aea` 已备份到 `~/.vsp-codex/packages/standalone/current/bin/vsp-codex.backup-20260622T122524+0800`。

## 技术思路

- 根因：OSC8 的 `vscode-remote://...` 被 VSCode 当成通用 URI 打开，而不是终端文件链接。这个路径会进入 Remote/外部打开确认流程。
- 修复策略：对 VSCode integrated terminal 检测 `TERM_PROGRAM=vscode` 或 `VSCODE_IPC_HOOK_CLI`；当目标 scheme 是 `file` 时，返回 `None`，使 TUI 不发 OSC8 包装，保留普通可见路径文本。
- 可点击能力来源：markdown renderer 对本地文件链接已经把 label 替换为目标路径，例如 `build/outputs/run_report.md` 或 `codex-rs/tui/src/markdown_render.rs:74`。禁用 OSC8 后，VSCode 的终端 file-link provider 仍能识别这些路径，并在当前 VSCode 窗口打开。

## 修改文件/模块

- `codex-rs/tui/src/terminal_hyperlinks.rs`：VSCode integrated terminal 中抑制 `file://` OSC8。
- 本机安装入口：`~/.vsp-codex/packages/standalone/current/bin/vsp-codex` 已部署新 release build。

## 测试设计

- 用 TUI focused test 验证 VSCode integrated terminal 下 `file://` 目标不会生成 OSC8 destination，而普通 web link 仍保留 OSC8。
- 用安装 SHA 与 `--version` 验证本机插件/VSCode shim 后续能加载新二进制。
- 复跑完成报告 guidance 与 CI 文档检查，确认本轮追加目标没有破坏原 goal 的两个主修复面。

## 验证结果

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-tui terminal_hyperlinks::tests::vscode_integrated_terminal_suppresses_file_osc8_for_native_file_links --lib`：1 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_completion_guidance_requires --test all`：2 passed。
- `python3 scripts/check_ci_docs.py`：通过。
- `sha256sum codex-rs/target/release/vsp-codex ~/.vsp-codex/packages/standalone/current/bin/vsp-codex ~/.vsp-codex/packages/standalone/current/bin/vsp-codex.backup-20260622T122524+0800`：release build 与 installed binary 均为 `33b19d3821023a0e954c24fcc47fcd88474f67631b157c7c4f9fee077e2b29b0`，备份为 `b444971afe4c040351e453b90af71cff92a370ac160115a02483b112f1c51aea`。
- `~/.vsp-codex/packages/standalone/current/bin/vsp-codex --version`：`vsp-codex 0.140.0-vsp.0`。

## 预期结果

新开的 VSCode 插件/TUI 会话中，TUI 输出的本地文件路径 Ctrl+单击应由 VSCode terminal file-link provider 处理，直接在当前 VSCode local/Remote 窗口打开文件，不再弹 `vscode-remote://` 相关的 Remote/外部打开确认。

## 遇到的问题

- 当前已有 VSCode 插件或 app-server 进程仍在使用旧二进制：PID `502560` 和 `3420157` 的 `/proc/<pid>/exe` 指向 `/home/heyx/.vsp-codex/packages/standalone/releases/0.140.0-vsp.0-x86_64-unknown-linux-gnu/bin/vsp-codex (deleted)`。需要重新打开/重载 VSCode 插件会话后才能加载新安装入口。
- `.pipeline/.lock` 是 2026-06-14 过期的 C9 旧锁，本轮没有删除它。

## 风险/后续

- 本轮只做本机 standalone 部署，没有推送代码、没有刷新 GitLab Release asset。若要让其他机器获得同一行为，需要后续走 release asset 刷新或安装包发布流程。
