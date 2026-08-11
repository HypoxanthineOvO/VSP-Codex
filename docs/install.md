# 安装与构建

本文面向 VSP Codex。普通用户优先使用预览包安装；开发者需要修改源码时再从源码构建。

## 系统要求

| 需求 | 说明 |
| --- | --- |
| 操作系统 | 当前预览包支持 Linux x64 GNU/glibc |
| Shell 工具 | `curl` 或 `wget`、`tar` |
| 推荐内存 | 8 GB 或以上 |
| VSCode 插件 | 需要使用 VSCode 接入时，先安装官方 ChatGPT 插件 |

当前预览包是 Linux x64 GNU/glibc release-profile preview 包，不是正式 MUSL release。它用于内部试用、Markdown/Thinking/VSCode 接入验证，体积较大。

## 一条命令安装预览版

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh
```

安装器会从 release manifest 选择当前平台和 `lab` 频道的最新版（当前为 9.5），并校验 SHA-256、包元数据版本和真实二进制版本。需要非交互安装并配置 VSCode 时使用：

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh -s -- --yes --vscode --no-launch
```

11.0 目前只在 `alpha` 频道发布。安装或从 9.5 切换到 11.0 alpha：

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | VSP_CODEX_RELEASE_CHANNEL=alpha sh
```

这条命令会：

- 下载并校验预览包。
- 解压到 `~/.vsp-codex/packages/standalone/releases/`。
- 更新 `~/.vsp-codex/packages/standalone/current`。
- 在 `~/.local/bin/vsp-codex` 创建入口。
- 因为传入了 `--vscode`，同时配置官方 VSCode ChatGPT 插件使用 VSP Codex CLI。
- Desktop/remote-control daemon 启动 app-server 时会优先使用 `~/.vsp-codex/packages/standalone/current/bin/vsp-codex`，从而继续连接本地 VSP Codex，而不是官方 Codex。

如果希望安装完成后直接启动 TUI，把 `--no-launch` 改为 `--launch`。

## GitLab Release 下载权限

预览包本体由稳定 manifest 中的 `archive_url` 指向 GitLab Release downloads。普通安装不应手写版本 URL 或 SHA-256；更新 manifest 使用 main 分支上的稳定 raw 路径 `.pipeline/release/vsp-release-manifest.json`。目标用户仍需要有仓库读取权限，不要把 GitLab token 写入仓库文档。

## 验证安装

```shell
vsp-codex --version
codex --version
vsp-codex
```

VSCode 接入状态：

```shell
vsp-codex vscode status --json
```

回滚 VSCode 设置：

```shell
vsp-codex vscode rollback --yes
```

## 预览版体验更新

`0.145.0-vsp.11.1-alpha.7` 在 11.0 alpha 基线上包含新的本地压缩状态机、长上下文预算、执行连续性契约、Hypo-Workflow semantic Host UX、TUI quote/stream/finalize 与标题层级修复、独立 Responses multi-provider 配置和统一模型切换；本版保留受信任的 Subagent Worker Routing，但恢复官方 `spawn_agent` wire contract，修复 Alpha 6 连接 VSPLab `/responses` 稳定 502 的回归。Alpha 7 紧急发布 Linux x64 预览包，macOS ARM64 和 Windows x64 继续使用 Alpha 6。session compatibility migration 可在共同 migration 前缀和所需 schema 一致时兼容 Official/VSP 的尾部版本差异。它不包含已废弃 10.x 线的 Chat API 支持。默认 `lab` 频道仍停留在 9.5，只有显式选择 `alpha` 的安装和更新会进入 11.x alpha。

`0.145.0-vsp.9.5` 是当前内部预览版，基于 upstream `rust-v0.145.0-alpha.4`，并包含发布版本一致性校验。它包含以下可见体验和兼容性更新：

- 上游 Codex Rust 基线为 `rust-v0.145.0-alpha.4`；对不等价或高风险内容继续保留 VSP 行为。
- C20 consultation-first action boundary：`discussion`、`background`、`idea`、`complaint`、`question`、`solution-discussion` 类型输入先得到 Mini-contract（`我的理解` -> `问题原因` -> `推荐方案`），不会直接进入文件编辑或 mutating command；明确 imperative 且有 concrete target 的请求仍可直接执行。
- SQLite 日志写入收敛：默认过滤 `log`、`codex_otel.log_only`、`codex_otel.trace_safe` 等 noisy target，并保留 VSP in-layer `target=log` guard，降低 persistent log 写入放大。
- Plugin/Remote APP 会话可见性启动路径修复：VSCode Remote bundled `codex` shim 漂移会被 `vscode status` 诊断，Remote APP 的 `codex` 入口可通过兼容 wrapper 回到 VSP Codex。
- VSCode repair/self-heal：`vsp-codex vscode repair --yes` 可单独修复远端 ChatGPT 插件 bundled `codex` shim；已安装 wrapper 启动时会静默检查并自修复一次。
- App-server remote socket remap：Desktop/Remote APP 传入官方 `~/.codex/app-server-control` socket 时会 remap 到 `~/.vsp-codex/app-server-control`，避免远程绑定回落到官方 Codex。
- 旧 session provider fallback：app-server thread API 会把空白或未知历史 `model_provider` 替换为当前配置 provider，避免插件/remote 恢复旧 session 时报错。
- VSCode bundled codex shim 与诊断修复：`vsp-codex vscode install/status/rollback` 持续维护官方 ChatGPT 插件 bundled CLI 指向 VSP wrapper，并暴露漂移状态。
- 安装器和 updater 持续维护 `vsp-codex` 与 `codex` 兼容入口；显式 VSP manifest update path 保留，普通官方 update 入口继续被 VSP gate 禁用。
- VSCode 插件启动 VSP Codex 时强制使用 `~/.vsp-codex` 作为 `CODEX_HOME`，避免继承官方 `~/.codex` 配置后再次触发空 provider 恢复失败。
- Remote SSH VSCode 接入：`vsp-codex vscode status/install/rollback` 优先处理远端 VSCode server Machine settings，并补充 app-server initialize probe；当 Machine settings 被 VSCode 忽略时，会把官方 ChatGPT 插件 bundled `codex` 入口替换为指向 VSP wrapper 的 shim。
- VSCode 插件 resume 兼容：恢复 VSP-authored session 时空 provider/model metadata 不再导致 `Model provider `` not found`。
- Hypo-Workflow Host UX：从已校验 installed release manifest 提供 `/hw:*` 与 `$hypo-workflow:*` 路由（包括 `/hw:experiment`），并保留 Question Tool metadata、Workflow Markdown fallback、Plan/SubPlan freshness、Codex Hooks runtime 和只读 semantic/legacy workspace guidance；Workflow 写入仍完全属于 plugin。
- 官方 Codex discovery export：提供 sanitized session export API，保留可读 resume metadata，同时剥离 provider/auth/Workflow 私有内容。
- Request Input 插件体验：支持显式 Workflow metadata；官方 VSCode ChatGPT 插件、Codex Desktop/remote 客户端会使用原生 Ask UI 弹窗接收 Question Tool；不支持 native Ask UI 的客户端会收到可读 Markdown fallback。
- Workflow Markdown surface：插件 fallback 中的 Request Input 会使用 `Workflow Surface` wrapper，Plan/SubPlan 进度计数保持分离。
- Appearance 入口：`/theme` 打开统一选择器，可进入 UI palette 或 syntax theme；`/ui-theme` 仍是 UI palette 快捷入口。
- Markdown 和输出块配色：`/ui-theme` 可选择三组主题色，标题、strong、emphasis、inline code、link、blockquote、列表和表格跟随主题；不同输出块使用不同 accent。
- VSCode terminal-safe 命令入口：`/commands` 和默认 `Ctrl+Space` 可打开命令面板，支持搜索并执行常用安全命令。
- 复制体验：命令面板提供 `Copy latest code block`，复制最近 assistant 消息里的最后一个 fenced code block。
- Resume 加载：历史列表先显示 SQLite metadata cache，再后台校验 canonical metadata；stale 行会更新，不会重复插入。
- VSCode 接入：`--vscode` 会把官方 ChatGPT 插件的 `chatgpt.cliExecutable` 指向 VSP Codex。

## 安装器参数

| 参数 | 说明 |
| --- | --- |
| `--release VERSION` | 显式安装官方 GitHub release；普通 VSP 安装不要使用 |
| `--archive-url URL` | 直接安装指定预览包 |
| `--sha256 SHA256` | 校验预览包 |
| `--target TARGET` | 安装目标，例如 `x86_64-unknown-linux-gnu` |
| `--version VERSION` | 安装目录使用的版本标签 |
| `--manifest-url URL` | 写入 VSP 更新 manifest，供 `vsp-codex update` 和后续更新检查使用 |
| `--channel CHANNEL` | 写入 VSP 更新频道，默认 `lab` |
| `--vscode` | 安装后配置官方 VSCode ChatGPT 插件 |
| `--launch` | 安装后直接启动 `vsp-codex` |
| `--no-launch` | 安装后不询问启动 |
| `--yes` | 非交互确认安装流程 |
| `--remove-existing` | 显式允许安装器处理已有 npm/brew Codex 冲突 |

默认情况下，安装器只会提示已有 npm/brew Codex 冲突，不会删除旧安装。

## 从源码构建

开发调试用 debug 二进制：

```shell
git clone git@gitlab.vsplab.cn:heyx/Codex.git
cd Codex
cargo build --manifest-path codex-rs/Cargo.toml --bin vsp-codex
./codex-rs/target/debug/vsp-codex
```

本地重建 Linux x64 GNU/glibc preview release 包：

```shell
cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex
python3 scripts/build_vsp_preview_release.py \
  --entrypoint-bin codex-rs/target/release/vsp-codex \
  --channel alpha \
  --force
```

脚本会按当前 workspace version 刷新 `.pipeline/release/v<version>/vsp-codex-preview-linux-x64-gnu.tar.gz`、对应 `.sha256` 和 `vsp-release-manifest.json`。默认会把 manifest URL、channel 和 target 写入包内 `codex-package.json`，让本地解包 smoke 和安装后的 standalone release 都能找到 VSP 更新源。

常用开发命令：

```shell
cd codex-rs
just fmt
just test -p codex-tui markdown_render
cargo build --bin vsp-codex
```

TUI 或用户可见输出变化后在本地检查 snapshot；snapshot 不属于普通提交的强制 CI 门禁：

```shell
cargo insta pending-snapshots --manifest-path tui/Cargo.toml
```

## 日志

TUI 默认使用本地受限诊断存储。需要临时查看明文日志时，可以显式设置 `log_dir`：

```shell
vsp-codex -c log_dir=./.vsp-codex-log
tail -F ./.vsp-codex-log/codex-tui.log
```
