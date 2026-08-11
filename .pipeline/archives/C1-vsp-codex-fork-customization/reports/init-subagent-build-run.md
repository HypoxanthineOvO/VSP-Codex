# Init Subagent Report: 编译与运行

状态：只读探索完成，未修改文件，未启动 app-server/daemon/proxy 等长驻服务，未执行构建命令。

## 推荐环境

- 官方源码构建环境：macOS 12+/Ubuntu 20.04+/Debian 10+，Windows 建议 WSL2；Git 2.23+，至少 4GB RAM，建议 8GB；Rust 用 `rustup` 安装并包含 `rustfmt`、`clippy`。
- Rust 工作区在 `codex-rs/Cargo.toml`，常用工具需要 `just`、`cargo-nextest`，TUI snapshot 相关可装 `cargo-insta`。
- Node/pnpm：仓库根 `package.json` 要求 Node `>=22`、pnpm `>=10.33`；TypeScript SDK 要求 Node `>=18`；npm CLI wrapper 自身要求 Node `>=16`。

## 主要构建路径

Cargo 开发主路径：

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo build --bin codex
cargo run --bin codex -- --help
```

顶层二进制 crate 是 `codex-cli`，输出二进制名是 `codex`。

just 路径：

```bash
cd /home/heyx/Codex-VSP
just codex --help
just exec "summarize this repo"
just test -p codex-tui
```

根目录 `justfile` 已把工作目录设为 `codex-rs`。相关 recipe：`codex`、`exec`、`fmt`、`fix`、`test`、`bazel-codex`、`app-server-test-client`、`mcp-server-run`。
`just codex` recipe 本身已经在 `cargo run --bin codex -- "$@"` 中插入分隔符，因此调用 help/version 时使用 `just codex --help` 或 `just codex --version`，不要额外写 `just codex -- --help`。

Bazel 路径：

```bash
cd /home/heyx/Codex-VSP
just bazel-codex -- --help
bazel run //codex-rs/cli:codex --run_under="cd $PWD &&" -- --help
bazel build //codex-rs/cli:release_binaries --config=remote
```

npm/pnpm 路径：`pnpm-workspace.yaml` 包含 `codex-cli`、responses proxy npm 包、TypeScript SDK。`@openai/codex` 的 `bin/codex.js` 是平台 wrapper，会寻找 vendored/native `codex` 二进制再 spawn；本地开发真实编译仍建议走 Cargo/Bazel。

## 可执行入口

- `codex`：默认无子命令时进入 TUI；常用命令包括 `cargo run --bin codex -- "prompt"`、`cargo run --bin codex -- --model gpt-5 "prompt"`、`cargo run --bin codex -- resume --last`。
- TUI：参数来自 `codex-rs/tui/src/cli.rs`，支持 prompt、resume、fork、approval/sandbox/search、`--no-alternate-screen`。也存在直接二进制 `codex-tui`，但常规入口是 `codex`。
- Headless CLI：`cargo run --bin codex -- exec "fix the failing test"` 或 `cargo run -p codex-exec --bin codex-exec -- "fix the failing test"`。
- app-server：`cargo run --bin codex -- app-server --listen stdio://` 或 `cargo run -p codex-app-server -- --listen ws://127.0.0.1:4222`。
- daemon：`codex app-server daemon start|status|stop|restart|exec|run-app-server`，会管理后台 app-server，探索阶段不应启动。
- app-server test client：`cargo run -p codex-app-server-test-client -- --codex-bin ./target/debug/codex model-list`；`serve` 会后台启动服务。
- debug client：`cargo run -p codex-debug-client -- --codex-bin ./target/debug/codex`，会 spawn `codex app-server` 并提供交互式 JSON-RPC 调试命令。
- exec-server：`cargo run --bin codex -- exec-server --listen ws://127.0.0.1:PORT`，长驻进程，支持 `stdio`/`ws`/remote。
- MCP server：`cargo run --bin codex -- mcp-server`、`cargo run -p codex-mcp-server --`、`just mcp-server-run`。
- responses API proxy：`printenv OPENAI_API_KEY | ./target/debug/codex-responses-api-proxy --port 60001`，长驻本地 HTTP 代理。

## 登录与认证

推荐先用 ChatGPT 登录：

```bash
cargo run --bin codex -- login
cargo run --bin codex -- login --device-auth
cargo run --bin codex -- login status
cargo run --bin codex -- logout
```

API key/access token 可通过 stdin 登录：

```bash
printenv OPENAI_API_KEY | cargo run --bin codex -- login --with-api-key
printenv CODEX_ACCESS_TOKEN | cargo run --bin codex -- login --with-access-token
```

代码识别 `OPENAI_API_KEY`、`CODEX_API_KEY`、`CODEX_ACCESS_TOKEN`，认证文件写到 `$CODEX_HOME/auth.json`。

## 证据文件

- `README.md`
- `docs/install.md`
- `package.json`
- `pnpm-workspace.yaml`
- `justfile`
- `codex-rs/README.md`
- `codex-rs/cli/Cargo.toml`
- `codex-rs/cli/src/main.rs`
- `codex-rs/tui/src/cli.rs`
- `codex-rs/exec/src/cli.rs`
- `codex-rs/app-server/README.md`
- `codex-rs/app-server-daemon/README.md`
- `codex-rs/app-server-test-client/README.md`
- `codex-rs/debug-client/README.md`
- `codex-rs/exec-server/README.md`
- `codex-rs/responses-api-proxy/README.md`
- `codex-cli/bin/codex.js`
