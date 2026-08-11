# C7/M7 VSP Release Channels, Naming Metadata, And Updater Report

## 结果

M7 已完成并通过只读复审。VSP-Codex 现在有独立的 manifest-driven 更新路径：VSP 模式不会回落到官方 OpenAI/GitHub/chatgpt 更新端点，启动更新仍是显式确认，standalone 更新会先校验 SHA-256，再切换 `current`。

## 已完成内容

- `codex-rs/tui/src/update_versions.rs`
  - 新增 VSP manifest/channel/target/release selection API。
  - 支持 `0.134.0-vsp.N` 版本比较。
  - `VSP_CODEX_RELEASE_MANIFEST_URL` 提供 manifest 源，`VSP_CODEX_RELEASE_CHANNEL` 提供 channel override。
  - manifest URL、archive URL、notes URL 都拒绝官方 OpenAI/GitHub/chatgpt 更新端点。
  - checksum mismatch 明确 abort，且不切换 standalone `current`。
- `codex-rs/tui/src/update_action.rs`
  - 新增 `UpdateAction::VspStandaloneManifest`，命令面为 `vsp-codex update --from-vsp-manifest ...`。
  - VSP action 携带 manifest/channel/target/version/archive/sha256/notes metadata。
- `codex-rs/tui/src/updates.rs`、`update_prompt.rs`、`history_cell/notices.rs`、`app.rs`
  - VSP 模式仅在配置 manifest 后走 VSP 更新检查。
  - 启动更新 prompt 保持非静默确认。
  - history notice 在 VSP 模式不 fallback 官方 updater action；没有 manifest action 时也不展示官方 release 链接。
- `codex-rs/cli/src/main.rs`
  - release-only VSP manifest updater 下载 archive、校验 SHA-256、解包 staging、规范化 package/legacy standalone layout、写入 releases 并更新 `current`。
  - `run_update_action`/`run_update_command` 改为 async path，避免在 Tokio runtime 中嵌套 `block_on`。
- `codex-rs/tui/src/chatwidget/active_subagents.rs`
  - release-channel name metadata 接入 Subagent name resolver。
  - 优先级保持为显式 nickname > 用户配置池 > release channel 默认 > built-in fallback。

## 验证

- PASS: `cargo test -p codex-tui c7_m7 -- --nocapture`，9 passed。
- PASS: `cargo test -p codex-cli c7_m7 -- --nocapture`，1 passed。
- PASS: `cargo check -p codex-cli --release --bin vsp-codex`。
- PASS: `cargo fmt --check --package codex-cli --package codex-tui`，仅仓库既有 stable rustfmt 对 nightly-only `imports_granularity` 的 warning。
- PASS: `git diff --check`。

## 审计结论

第一次 M7 audit 返回 `fail`，指出 manifest-selected URL 官方端点封锁不完整，以及 VSP history notice 会 fallback 到官方 updater action。两项均已修复并由 `庄周 the 20th` 只读复审确认 `pass`。审计报告见 `.pipeline/reviews/C7/M7/audit.md`。

## 剩余风险

- Windows `current` 切换使用 directory symlink，可能需要 Developer Mode/admin 权限，失败时不具备 Unix rename symlink 的同等原子性。
- 真实 standalone 包仍需手动 smoke：`vsp-codex --version`、`vsp-codex vscode status --json`、launcher/VSCode `cliExecutable` 是否指向 stable `current`。
- C7 不处理 private GitLab token；manifest 和 asset URL 需要对客户端可达。
