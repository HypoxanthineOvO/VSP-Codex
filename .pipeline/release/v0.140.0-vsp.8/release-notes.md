### Upstream Base

- 合并 upstream Codex Rust `rust-v0.140.0-alpha.18` 作为新的 VSP 基线。
- 保留 VSP provider/config、`vsp-codex` CLI、VSCode Remote SSH、Hypo-Workflow `/hw`/Question Tool/Markdown/plan freshness、Subagent UX、plugin/skills compatibility 和 sandbox/permissions safety defaults。
- `voice/realtime` 与 `remote-control` 记录为非 VSP 必保功能，不作为本版本升级阻塞项。

### Docs and Packaging

- 同步 README、安装指南、用户指南和稳定更新 manifest 到 `v0.140.0-vsp.8` GitLab Release downloads。
- 重建 Linux x64 GNU/glibc preview release 包 `.pipeline/release/v0.140.0-vsp.8/vsp-codex-preview-linux-x64-gnu.tar.gz`。

### Validation

- `cargo metadata --manifest-path codex-rs/Cargo.toml --no-deps --format-version=1`
- `cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex`
- `codex-rs/target/release/vsp-codex --version`
- `python3 scripts/build_vsp_preview_release.py --entrypoint-bin codex-rs/target/release/vsp-codex --bwrap-bin /usr/bin/bwrap --cargo-profile release --force`
- `sha256sum -c .pipeline/release/v0.140.0-vsp.8/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`

### Same-Version HotFixes

- 2026-06-18：吸收 installer staging 修复，保持版本号 `0.140.0-vsp.8`，更新 release asset SHA256 为 `4ae4562b7d7447c6572e8034b444512f1c641628376cee1db81e4db75e9013a9`。
- 2026-06-19：发布 provider resume / VSCode Remote HotFix，封堵空 `modelProvider` / `config.modelProvider` 覆盖配置的问题，更新 release asset SHA256 为 `ce68954d74aed2322f6704e792beb677122853f760075de23ad7ec7b769d3f1b`。
- 2026-06-19：发布 TUI VSCode Remote 文件链接 HotFix，保持非 VSCode 终端 OSC8 精确点击范围；在 VSCode 集成终端内交给原生文件链接处理，避免弹出 Remote/外部打开确认，更新 release asset SHA256 为 `6a2e618cf1462261dcc49ca00462e1b56ecf6862d41779a4ac0c71c0f93c3d16`。
- 2026-06-22：发布 VSCode Plugin HotFix，默认 thread/start 覆盖旧配置中的 `on-request` / `read-only` 为 VSP Full Access，native Ask 客户端不再显示合成 Question Context，自动压缩摘要不再作为可见 assistant 回复输出且压缩后继续显示真实回复；随后补充 request `config` 显式权限覆盖保护和 compaction 终局清理，更新 release asset SHA256 为 `20c29f2b75395aa03f2281551c6b482de819a5c55591b75f6363f727312f6c3f`。
