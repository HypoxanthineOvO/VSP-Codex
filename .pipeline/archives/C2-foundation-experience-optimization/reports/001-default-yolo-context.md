# M1 默认 YOLO 与底栏 Context 报告

更新时间：2026-05-26T02:09:21+08:00

## 结果

M1 已完成。`vsp-codex exec` 在没有显式沙箱配置时默认使用 YOLO 权限；用户显式传入 CLI sandbox 或在 `config.toml` 中配置 `sandbox_mode` 时仍然优先生效。TUI 默认底栏现在包含 Context 信息，未知上下文窗口显示为稳定占位，而不是误报 `100% left`。

## 主要改动

- `codex-rs/exec/src/lib.rs`：为 VSP exec 默认注入 `SandboxMode::DangerFullAccess`，但保留显式 `sandbox_mode`、`default_permissions` 和非空 `[permissions]` 的覆盖能力。
- `codex-rs/tui/src/chatwidget.rs`：默认 status line 新增 `context-remaining`，并统一未知 Context footer 文案。
- `codex-rs/tui/src/chatwidget/status_surfaces.rs`：刷新 status surface 时同步未知 Context 文案，并补充 Context 百分比测试辅助。
- `codex-rs/cli/tests/vsp_entrypoint.rs`：覆盖默认 YOLO、approval-only config、config sandbox override、CLI sandbox override。
- `codex-rs/tui/src/chatwidget/tests/status_and_layout.rs`：覆盖默认底栏 Context 和未知 Context 快照。
- `codex-rs/tui/src/chatwidget/tests/status_command_tests.rs`：修正 VSP 已禁用 ChatGPT rate-limit refresh 后的 `/status` 期望。
- `codex-rs/tui/src/onboarding/welcome.rs`：更新 Welcome 测试，匹配 VSP Codex 中文启动页。
- 快照：接受默认底栏新增 Context 和当时版本号 `v0.134.0-vsp.1` 带来的预期快照变化；C2 release prep 已将最终发布版本验证为 `v0.134.0-vsp.2`；删除 VSP 已禁用 `/feedback` 后不再使用的旧 popup 快照。

## 验证

- `just fmt`：通过。
- `just test -p codex-cli --test vsp_entrypoint`：8 passed。
- `just test -p codex-exec`：116 passed。
- `just test -p codex-tui`：2632 passed，4 skipped。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `git diff --check`：通过。
- `just fix -p codex-cli -p codex-exec -p codex-tui`：通过；之后仅按仓库规则重新运行 `just fmt`，未重复运行测试。

## 剩余风险

- M1 只让 `vsp-codex exec` 默认 YOLO；TUI 的默认 YOLO 行为在现有配置/入口层仍需要在后续里程碑结合实际启动链路继续确认。
- Context 目前依赖已有 usage/context 数据；没有 usage 时显示占位，真实百分比展示依赖运行期事件。
