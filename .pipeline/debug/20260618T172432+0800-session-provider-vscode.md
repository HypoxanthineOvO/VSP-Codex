# C9 Debug: Session Provider And VSCode Extension

## 改动摘要

用户报告两个阻塞问题：VSP-Codex 创建的 Session Provider 再次为空，以及 VSCode Codex 插件无法打开。本轮已应用修复：新 VSP session 不再把 `model_provider = "vsp"` 写成 `null`；state metadata 构建对空白 provider 回退默认 provider；VSCode status 会诊断 wrapper 指向的二进制版本是否过旧；本机 VSCode wrapper 已更新到当前 `0.140.0-vsp.0` release binary；历史 VSP session/provider 空值已回填。

## 技术思路

排查假设按优先级验证：

1. Session provider 为空来自 VSP 持久化时刻，而非 UI 展示层。确认：`codex-rs/rollout/src/recorder.rs` 的 `persisted_session_meta_provider` 明确把 `provider_id == "vsp"` 省略为 `None`，旧测试还固化了该行为。
2. SQLite 和 JSONL 的 provider 状态可能分裂。确认：`~/.vsp-codex/sessions` 链到 `~/.codex/sessions`，但 `~/.vsp-codex/state_5.sqlite` 与 `~/.codex/state_5.sqlite` 独立。修复前 `~/.codex/state_5.sqlite` 有 12 条空 provider，session JSONL 有 150 个旧 VSP 文件的 `session_meta.model_provider` 为空。
3. VSCode 插件打不开来自 wrapper 指向旧二进制。确认：`~/.vsp-codex/vscode/vsp-codex` 原本执行 `~/.local/bin/vsp-codex`，版本是 `0.134.0-vsp.7.1`；当前 release binary 是 `0.140.0-vsp.0`。

## 修改文件/模块

- `codex-rs/rollout/src/recorder.rs`：持久化非空 provider，包括 `vsp`。
- `codex-rs/rollout/src/recorder_tests.rs`：更新 VSP provider 持久化测试。
- `codex-rs/state/src/model/thread_metadata.rs`：空白 provider 回退默认 provider，并新增单测。
- `codex-rs/cli/src/vscode_cmd.rs`：`vscode status` 增加 wrapper 目标二进制版本诊断。
- 本机数据修复备份：`~/.codex/session-provider-backups/provider-repair-20260618-172136`。
- 本机 VSCode wrapper：`~/.vsp-codex/vscode/vsp-codex` 已改为执行 `/home/heyx/Codex-VSP/codex-rs/target/release/vsp-codex`。

## 测试设计

由于当前调试在本地执行，未能满足 strict worker separation 的独立复现/实现/审计 worker 分离；本报告明确记录该降级限制。验证覆盖：

- rollout 新建 session provider 持久化。
- state metadata blank provider fallback。
- VSCode wrapper/status 运行时检查。
- 历史数据空 provider 计数复核。
- 格式检查。

## 验证结果

通过：

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-rollout persisted_session_meta`：2 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-state builder_uses_default_provider_when_provider_is_blank`：1 passed。
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli --lib vscode`：lib 编译通过，0 matched tests。
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check`：通过；仅稳定版 rustfmt 对 `imports_granularity = Item` 输出 warning。
- `~/.vsp-codex/vscode/vsp-codex --version`：`vsp-codex 0.140.0-vsp.0`。
- `codex-rs/target/release/vsp-codex vscode status --json`：configured=true，wrapper 版本 warning 消失，仅保留 remote Machine settings scope warning。
- 数据复核：`~/.codex/sessions` 和 `~/.codex/archived_sessions` 中空 provider JSONL 数量为 0；`~/.codex/state_5.sqlite` 空 provider 数量为 0；`~/.vsp-codex/state_5.sqlite` 空 provider 数量为 0。

未通过/残余：

- 宽泛命令 `cargo test --manifest-path codex-rs/Cargo.toml -p codex-cli vscode` 触发无关测试编译错误：`cli/tests/login.rs` 中 `read_auth_json` 未定义。该错误不在本次 VSCode wrapper 修复路径内，已用 `-p codex-cli --lib vscode` 和运行时 wrapper/status 验证替代。

## 预期结果

新创建的 VSP session 会在 rollout `session_meta.model_provider` 中写入 `"vsp"`，不会再显示空 Provider。现有受影响 VSP session 已回填，列表和 state db 读取时也不会再出现空 provider。VSCode 插件启动 wrapper 现在使用当前 `0.140.0-vsp.0` binary，不再因旧 `0.134.0-vsp.7.1` 协议/版本不匹配而打不开。

## 遇到的问题

- 当前环境未使用独立 Subagent worker 执行复现、实现、审计，strict worker separation 降级为本地主编排调试。
- VSCode status 仍提示 remote `.vscode-server` Machine settings 对 `chatgpt.*` application-scoped settings 不生效；这是远程 VSCode 配置作用域限制。wrapper 本身已经修复。

## 风险/后续

M5 release gate 仍然保持 pending。未执行 release commit、tag、push、GitLab release 或资产上传。建议在继续 release 前，用户从 VSCode 客户端侧确认本地 User settings 中 `chatgpt.cliExecutable` 指向 `/home/heyx/.vsp-codex/vscode/vsp-codex`，并实际打开一次插件验证。
