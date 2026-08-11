# C2 基础体验优化 Progress

更新时间：2026-05-26T16:45:32+08:00

## 当前状态

- Pipeline：completed
- 当前 Cycle：C2 基础体验优化
- 当前阶段：release prepared
- 发布版本：`0.134.0-vsp.2`
- 执行策略：M1-M6 已连续完成；本轮执行 `$hypo-workflow:sync`、`$hypo-workflow:docs`、`$hypo-workflow:release` 收尾。

## C2 完成范围

- M1：默认 VSP YOLO 权限；TUI 与 `vsp-codex exec` 默认 `approval_policy=never`、`sandbox_policy=danger-full-access`，显式 CLI/config 覆盖仍保留。
- M2：Subagent 默认可用，最大数量提升到 32，使用 VSP 中文姓名池，并保留 model/reasoning/role metadata。
- M3：TUI 增加 active Subagent 常驻卡片；仅显示运行中/等待中 Subagent；详情快捷键改为 `Ctrl+O`。
- M4：接入 Hypo-Translate，本地翻译接口异步翻译 Agent 输出、reasoning summary 和 Subagent 文本；hidden/transcript-only reasoning 不翻译、不泄露。
- M5：MCP 启动噪声降级；普通 `Booting MCP server: context7` 不再污染主状态标题；Slash Command、设置页、状态页、启动页等完成第一轮 VSP 中文化。
- M6：集成回归与实际 smoke 完成；官方 remote plugin、ChatGPT auth、Cloud 相关路径默认门控；Provider Debug 默认脱敏。
- Release prep：修复 `vsp-codex resume` 大历史进入时的终端闪烁；Context Window 统一显示为 `used/window percent`；Read File 输出补齐语法高亮；Subagent hash 兜底显示修复。

## 发布验证

- `glab ci lint .gitlab-ci.yml`：valid。
- `cargo metadata --no-deps --format-version 1`：workspace/package version 为 `0.134.0-vsp.2`。
- `just fmt`：通过。
- `just test -p codex-tui`：2657 passed，4 skipped。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `just test -p codex-cli --test vsp_entrypoint`：8 passed。
- `just test -p codex-exec`：116 passed。
- `just test -p codex-model-provider-info`：21 passed。
- `just test -p codex-core-plugins startup_remote_plugin_sync remote_plugin`：18 passed，181 skipped。
- `just fix -p codex-cli -p codex-exec -p codex-tui -p codex-core-plugins -p codex-model-provider-info`：通过。
- `cargo build -p codex-cli --bin vsp-codex`：通过。
- `./codex-rs/target/debug/vsp-codex --version` 与全局 `vsp-codex --version`：均为 `vsp-codex 0.134.0-vsp.2`。
- `git diff --check`：通过。

## 已知非阻断项

- full workspace 本机尝试：`CODEX_SKIP_BWRAP_BUILD=1 just test` 曾运行 9946 个测试，9786 passed、155 failed、5 timed out、21 skipped；失败集中在本机环境/负载敏感路径和官方 remote/shell/sandbox/realtime/thread_manager 旧路径。C2 相关目标测试已单独收敛。
- `.pipeline/state.yaml` 和 `.pipeline/cycle.yaml` 是执行权威记录，本次 sync/release 未重写；历史证据中保留 `0.134.0-vsp.1` 的旧验证输出属于 C1/M6 当时状态。

## 交付状态

- 文档：`README.md`、`docs/vsp-codex.md`、`CHANGELOG.md` 已更新到 `0.134.0-vsp.2`。
- CI：`.gitlab-ci.yml` lint 通过。
- 发布：等待提交、tag `v0.134.0-vsp.2`、推送 GitLab。
