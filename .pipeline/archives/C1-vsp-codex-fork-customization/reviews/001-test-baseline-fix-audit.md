# M1 Post-Fix Audit: TUI Test Baseline

时间：2026-05-24T20:29:21+08:00
审核 Subagent：Hypatia (`019e59f2-c4a1-76b0-a0ce-ec148b9daab2`)
结论：通过，无阻塞问题。

## Findings

无阻塞发现。

## Notes

- `codex-rs/tui/src/chatwidget/tests/helpers.rs:191` 只在 chatwidget 测试 helper 内预置 `status_line_project_root_name_cache = Some(..., root_name: None)`，避免默认 `/tmp/project` fixture 向上遍历到宿主机 `/tmp/.git`。这会改变 `make_chatwidget_manual` 的 fixture 语义：需要真实 project-root discovery 的测试应清空该 cache 或显式更改 cwd；不影响生产行为。
- `codex-rs/tui/src/ide_context/ipc.rs:902` 是 Unix-only 测试。创建 `socket-parent` 并设为 `0700` 后再 bind socket，符合生产 socket-parent 权限校验；不改变生产 IPC 行为。
- IPC 测试可选替代方案：用 `std::fs::DirBuilder` + `std::os::unix::fs::DirBuilderExt::mode(0o700)` 原子创建安全目录。当前 create-then-chmod 只用于测试 tempdir，非阻塞。
- Chatwidget 更完整但成本更高的方案是弃用 `/tmp/project` 默认 cwd，改为隔离 synthetic cwd 并补 snapshot normalization。当前 cache seed 是更小的实用修复。

## Verdict

这是 test-hermeticity-only patch，可以继续。验证充分：3 个定向测试、`just test -p codex-tui`、`just fix -p codex-tui`、Rust `cargo fmt` 已执行。`just fmt` 在 Python SDK ruff 阶段因 glibc Linux 平台缺少 `openai-codex-cli-bin==0.131.0a4` wheel 失败，属于环境/工具链问题，不阻塞这次 Rust-only 测试修复。
