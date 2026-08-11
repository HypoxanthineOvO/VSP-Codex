# M1 Audit：默认 YOLO 与底栏 Context

更新时间：2026-05-26T02:09:21+08:00

## Verdict

pass

## Reviewed Refs

- `codex-rs/exec/src/lib.rs`
- `codex-rs/cli/tests/vsp_entrypoint.rs`
- `codex-rs/tui/src/chatwidget.rs`
- `codex-rs/tui/src/chatwidget/status_surfaces.rs`
- `codex-rs/tui/src/chatwidget/tests/status_and_layout.rs`
- `codex-rs/tui/src/chatwidget/tests/status_command_tests.rs`
- `codex-rs/tui/src/onboarding/welcome.rs`
- `codex-rs/tui/src/**/snapshots/*.snap`

## Findings

- 第一轮 audit 的两个 P1 问题已修复：默认 TUI status line 现在包含 Context；`approval_policy` 单独配置不再阻止 exec 默认 `danger-full-access`。
- 显式 sandbox 覆盖已由 CLI 参数和 `config.toml` 回归测试保护。
- 快照变更与预期一致：默认底栏新增 Context、版本号显示为 VSP 版本、VSP 禁用 `/feedback` 后删除旧 popup snapshot。

## Evidence

- `just test -p codex-cli --test vsp_entrypoint`
- `just test -p codex-exec`
- `just test -p codex-tui`
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`
- `git diff --check`
- `just fix -p codex-cli -p codex-exec -p codex-tui`
