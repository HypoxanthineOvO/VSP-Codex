# 编译与运行

## 本机环境探测

- `rustc`：已安装，命令输出为 `rustc 1.95.0 (59807616e 2026-04-14)`。
- `cargo`：已安装，命令输出为 `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`。
- 仓库 pin 的 Rust toolchain：`codex-rs/rust-toolchain.toml` 指定 `channel = "1.93.0"`，组件包括 `clippy`、`rustfmt`、`rust-src`；这些组件已补齐并验证。
- `pnpm`、`uv`：已在 PATH 中。
- `node`：当前 Codex 会话中普通 `node` 为 `v20.20.0`，不满足仓库根 `package.json` 的 `node >= 22`；Volta 已安装 `node@22.22.3`，可用 `volta run --node 22 node --version` 验证并运行需要 Node 22 的脚本。
- `just`：已安装并验证为 `just 1.51.0`。
- `cargo-nextest`：已安装并验证为 `cargo-nextest 0.9.136`。
- Bazel 版本文件：`.bazelversion` 为 `9.0.0`，但当前 PATH 未确认有 `bazel`。

`cargo metadata --no-deps --format-version=1` 已在 `codex-rs/` 下成功运行；此前的 30 秒超时来自首次触发 pinned Rust 1.93.0 toolchain 同步。

## 已修复的实际运行问题

- Rust 1.93.0 toolchain 原先处于部分安装/首次同步状态；已运行 `rustup toolchain install 1.93.0 --component rustfmt --component clippy --component rust-src` 修复。
- 推荐命令入口缺少 `just`；已运行 `cargo install --locked just`，当前为 `just 1.51.0`。
- `just test` 依赖的 `cargo-nextest` 缺失；已运行 `cargo install --locked cargo-nextest`，当前为 `cargo-nextest 0.9.136`。
- 本地 Codex 状态库 `~/.codex/state_5.sqlite` 曾在启动时触发 migration 1 的 `table threads already exists` 错误；已按 Codex recovery 语义把旧 `state_5.sqlite`、`state_5.sqlite-wal`、`state_5.sqlite-shm` 备份为 `*.codex-repair-1779620444.0.bak`，随后让 Codex 重建状态库。
- 修复后 `./codex-rs/target/debug/codex doctor --summary` 通过，结果为 `12 ok · 1 idle · 3 notes · 1 warn · 0 fail degraded`。
- 当前 Codex 进程环境包含 `_VOLTA_TOOL_RECURSION=1`，PATH 中 Node 20 image 优先，导致普通 `node --version` 仍为 `v20.20.0`；需要仓库 Node 22 脚本时使用 `volta run --node 22 <command>`，例如 `volta run --node 22 pnpm --version`。

## 推荐依赖

根据 `docs/install.md` 和根目录 `justfile`，开发机至少需要：

```bash
rustup component add rustfmt
rustup component add clippy
cargo install --locked just
cargo install --locked cargo-nextest
```

Node 侧要求见根目录 `package.json`：

```bash
node >= 22
pnpm >= 10.33.0
```

npm 分发包 `codex-cli/package.json` 的 runtime wrapper 自身要求 Node `>=16`，但仓库维护脚本要求 Node `>=22`。
TypeScript SDK 自身要求 Node `>=18`；仓库根维护脚本仍以 Node `>=22` 为准。

## 编译 Rust 主二进制

推荐从 `codex-rs/` workspace 根目录编译：

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo build -p codex-cli
```

也可以编译整个 workspace：

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo build
```

直接 Cargo run：

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo run --bin codex -- --help
cargo run --bin codex -- "explain this codebase to me"
```

根目录 `justfile` 已设置 `working-directory := "codex-rs"`，安装 `just` 后可从仓库根目录运行：

```bash
just install
just codex --help
just codex --version
```

注意：`just codex --help` 会执行 `cargo run --bin codex -- --help`；不要写成 `just codex -- --help`，否则额外的 `--` 会传给 Codex CLI。

## 实际运行二进制

本次已重新构建并验证 debug 二进制：

```bash
/home/heyx/Codex-VSP/codex-rs/target/debug/codex --version
/home/heyx/Codex-VSP/codex-rs/target/debug/codex --help
```

输出确认版本为 `codex-cli 0.0.0`，并列出 `exec`、`review`、`login`、`logout`、`mcp`、`plugin`、`mcp-server`、`app-server`、`remote-control`、`completion`、`update`、`doctor`、`sandbox`、`debug`、`apply`、`resume`、`fork`、`cloud`、`exec-server`、`features` 等命令。

从源码重新构建和运行：

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo build --bin codex
cargo run --bin codex -- --version
```

从仓库根目录通过 just 运行：

```bash
cd /home/heyx/Codex-VSP
just codex --version
just exec --help
```

本地健康检查：

```bash
./codex-rs/target/debug/codex doctor --summary
```

交互式 TUI：

```bash
cd /home/heyx/Codex-VSP
./codex-rs/target/debug/codex
./codex-rs/target/debug/codex "explain this codebase to me"
```

非交互执行：

```bash
cd /home/heyx/Codex-VSP
./codex-rs/target/debug/codex exec "summarize this repository"
echo "hello" | ./codex-rs/target/debug/codex exec "Summarize stdin"
./codex-rs/target/debug/codex exec --json "list files"
```

也可以直接运行 headless crate：

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo run -p codex-exec --bin codex-exec -- "summarize this repository"
```

认证状态和登录：

```bash
./codex-rs/target/debug/codex login
./codex-rs/target/debug/codex login --device-auth
./codex-rs/target/debug/codex login status
./codex-rs/target/debug/codex logout
```

运行需要有效 ChatGPT 登录或 API key 配置；具体认证文档在 `docs/authentication.md` 指向官方 Codex auth 文档。

stdin 登录路径：

```bash
printenv OPENAI_API_KEY | cargo run --bin codex -- login --with-api-key
printenv CODEX_ACCESS_TOKEN | cargo run --bin codex -- login --with-access-token
```

认证相关代码识别 `OPENAI_API_KEY`、`CODEX_API_KEY`、`CODEX_ACCESS_TOKEN`，认证文件写到 `$CODEX_HOME/auth.json`。

## app-server

作为子命令运行：

```bash
./codex-rs/target/debug/codex app-server --listen stdio://
./codex-rs/target/debug/codex app-server --listen ws://127.0.0.1:0
./codex-rs/target/debug/codex app-server generate-ts --out /tmp/codex-app-server-ts
./codex-rs/target/debug/codex app-server generate-json-schema --out /tmp/codex-app-server-schema
```

也可以单独编译 app-server binary：

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo build -p codex-app-server
./target/debug/codex-app-server --listen stdio://
```

app-server 是长驻/流式服务；启动后客户端必须先发 `initialize` 再发 `initialized`。

daemon 管理命令：

```bash
./codex-rs/target/debug/codex app-server daemon start
./codex-rs/target/debug/codex app-server daemon status
./codex-rs/target/debug/codex app-server daemon stop
./codex-rs/target/debug/codex app-server daemon restart
```

daemon 会管理后台 app-server，探索或只读梳理阶段不应启动。

app-server test client：

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo build --bin codex
cargo run -p codex-app-server-test-client -- --codex-bin ./target/debug/codex model-list
cargo run -p codex-app-server-test-client -- --codex-bin ./target/debug/codex serve --listen ws://127.0.0.1:4222 --kill
```

`serve` 会后台启动服务；其他子命令包括 `watch`、`send-message-v2`、`resume-message-v2`、`thread-list`、`test-login`、`live`。

debug client：

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo build --bin codex
cargo run -p codex-debug-client -- --codex-bin ./target/debug/codex
```

debug client 会 spawn `codex app-server` 并提供交互式 JSON-RPC 调试命令。

## MCP server

```bash
./codex-rs/target/debug/codex mcp-server
cargo run -p codex-mcp-server --
```

用于让其他 MCP client 把 Codex 当成 MCP server 调用。README 示例：

```bash
npx @modelcontextprotocol/inspector codex mcp-server
```

## responses API proxy

```bash
cd /home/heyx/Codex-VSP/codex-rs
cargo build -p codex-responses-api-proxy
printenv OPENAI_API_KEY | ./target/debug/codex-responses-api-proxy --port 60001
```

该程序从 stdin 读取 API key，运行后是本地 HTTP 代理，属于长驻服务。

## exec-server

`exec-server` 由主 `codex` 二进制子命令承载：

```bash
./codex-rs/target/debug/codex exec-server --listen ws://127.0.0.1:0
./codex-rs/target/debug/codex exec-server --listen stdio://
```

远程注册模式：

```bash
./codex-rs/target/debug/codex exec-server --remote <URL> --environment-id <ID> --name <NAME>
```

远程注册需要 ChatGPT auth；`--use-agent-identity-auth` 需要 `CODEX_ACCESS_TOKEN`。

## Codex Cloud tasks

```bash
./codex-rs/target/debug/codex cloud list --limit 20
./codex-rs/target/debug/codex cloud exec "fix failing tests" --env <ENV_ID>
./codex-rs/target/debug/codex cloud status <TASK_ID>
./codex-rs/target/debug/codex cloud diff <TASK_ID>
./codex-rs/target/debug/codex cloud apply <TASK_ID>
```

该功能依赖 ChatGPT/Codex Cloud 后端和环境 id。

## npm wrapper

`codex-cli/bin/codex.js` 是 npm 包 `@openai/codex` 的入口，会按平台查找原生二进制：

```bash
cd /home/heyx/Codex-VSP/codex-cli
node bin/codex.js --help
```

正常发布安装路径：

```bash
npm install -g @openai/codex
codex
```

## Bazel 路径

根目录 `justfile` 提供 Bazel 入口：

```bash
just bazel-codex --help
just bazel-test
just bazel-lock-check
```

直接 Bazel 运行主二进制：

```bash
bazel run //codex-rs/cli:codex --run_under="cd $PWD &&" -- --help
```

Bazel release binary target：

```bash
bazel build //codex-rs/cli:release_binaries --config=remote
```

## 测试与格式化

仓库约定不直接跑 `cargo test`，使用 `just test`：

```bash
just fmt
just fix -p <crate>
just test -p <crate>
```

示例：

```bash
just test -p codex-tui
just test -p codex-app-server-protocol
```

`just test` 使用 `cargo nextest run --no-fail-fast` 并追加 benchmark smoke；本机已安装 `just` 和 `cargo-nextest`。本次只修复运行环境和状态库，未修改业务代码，因此没有运行完整测试套件。
`just test` 使用 `cargo nextest run --no-fail-fast` 并追加 benchmark smoke；本机已安装 `just` 和 `cargo-nextest`。本次只修复运行环境和状态库，未修改业务代码，因此没有运行完整测试套件。

## Subagent 运行报告

- `.pipeline/reports/init-subagent-build-run.md`
- `.pipeline/reports/init-subagent-architecture.md`
