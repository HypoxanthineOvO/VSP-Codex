# 证据索引

## 已读取的核心文件

- `README.md`
- `docs/install.md`
- `docs/getting-started.md`
- `docs/authentication.md`
- `docs/config.md`
- `package.json`
- `justfile`
- `.bazelversion`
- `codex-rs/rust-toolchain.toml`
- `codex-rs/README.md`
- `codex-rs/Cargo.toml`
- `codex-rs/core/README.md`
- `codex-rs/core/src/`
- `codex-rs/cli/Cargo.toml`
- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/src/lib.rs`
- `codex-rs/tui/src/cli.rs`
- `codex-rs/exec/src/cli.rs`
- `codex-rs/app-server/Cargo.toml`
- `codex-rs/app-server/README.md`
- `codex-rs/app-server-daemon/README.md`
- `codex-rs/app-server-test-client/README.md`
- `codex-rs/debug-client/README.md`
- `codex-rs/app-server/src/main.rs`
- `codex-rs/app-server/src/lib.rs`
- `codex-rs/app-server-protocol/src/protocol/v2/`
- `codex-rs/exec-server/Cargo.toml`
- `codex-rs/exec-server/README.md`
- `codex-rs/responses-api-proxy/README.md`
- `codex-rs/exec-server/src/`
- `codex-rs/codex-mcp/src/lib.rs`
- `codex-rs/codex-mcp/src/connection_manager.rs`
- `codex-rs/config/src/lib.rs`
- `codex-rs/config/src/types.rs`
- `codex-rs/file-system/src/lib.rs`
- `codex-rs/file-watcher/src/lib.rs`
- `codex-rs/hooks/src/lib.rs`
- `codex-rs/keyring-store/src/lib.rs`
- `codex-rs/memories/read/src/lib.rs`
- `codex-rs/memories/write/src/lib.rs`
- `codex-rs/memories/mcp/src/server.rs`
- `codex-rs/secrets/src/lib.rs`
- `codex-rs/cloud-tasks/src/cli.rs`
- `codex-rs/cloud-tasks/src/lib.rs`
- `codex-rs/core-plugins/src/`
- `codex-rs/core-skills/src/`
- `codex-rs/plugin/src/`
- `codex-rs/skills/src/`
- `codex-rs/codex-api/src/lib.rs`
- `codex-rs/chatgpt/README.md`
- `codex-rs/chatgpt/src/connectors.rs`
- `codex-rs/model-provider/src/lib.rs`
- `codex-rs/model-provider-info/src/lib.rs`
- `codex-cli/package.json`
- `codex-cli/bin/codex.js`
- `sdk/typescript/README.md`
- `sdk/python/README.md`

## 已运行的验证命令

- `git rev-parse --is-inside-work-tree`
- `git status --short`
- `find . -maxdepth 2 -type d`
- `find codex-rs -mindepth 2 -maxdepth 2 -name Cargo.toml`
- `./codex-rs/target/debug/codex --version`
- `./codex-rs/target/debug/codex --help`
- `./codex-rs/target/debug/codex exec --help`
- `./codex-rs/target/debug/codex app-server --help`
- `./codex-rs/target/debug/codex mcp-server --help`
- `command -v cargo rustc just cargo-nextest pnpm node uv bazel`
- `rustc --version`
- `cargo --version`
- `rustup toolchain install 1.93.0 --component rustfmt --component clippy --component rust-src`
- `cargo install --locked just`
- `cargo install --locked cargo-nextest`
- `just --version`
- `cargo nextest --version`
- `rustup component list --toolchain 1.93.0-x86_64-unknown-linux-gnu --installed`
- `cargo metadata --no-deps --format-version=1`
- `cargo build --bin codex`
- `cargo run --bin codex -- --version`
- `just codex --version`
- `just exec --help`
- `./codex-rs/target/debug/codex doctor --summary`
- `volta run --node 22 node --version`

## 未完成或受限的验证

- 未运行完整测试套件；本次修复仅涉及本地工具链、命令入口和 Codex 状态库，没有修改业务代码。
- 未启动 app-server daemon、MCP server、exec-server、responses proxy 等长驻服务。
- 当前 Codex 会话内普通 `node --version` 仍为 `v20.20.0`；Volta Node 22 已验证，可通过 `volta run --node 22 ...` 使用。
