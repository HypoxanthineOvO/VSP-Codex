# VSP Codex

VSP Lab 内部使用的 Codex fork。它保留 Codex 的本地 Agent、MCP、Skills、Plugins 和 OpenAI Responses 兼容接口，同时把默认入口、配置目录和可见服务入口收敛到 VSP 内部使用方式。

## 快速安装

一行命令安装 `lab` 稳定频道的最新版本（当前为 9.5）：

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh
```

安装脚本自动从 release manifest 解析最新版本，无需手动指定参数。如需跳过确认提示或配置 VSCode 扩展：

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh -s -- --yes --vscode --no-launch
```

11.0 目前作为 alpha 发布，需要显式选择 `alpha` 频道：

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | VSP_CODEX_RELEASE_CHANNEL=alpha sh
```

安装后验证：

```shell
vsp-codex --version
codex --version
vsp-codex
```

两个版本命令应输出相同的最新版。当前 release manifest 提供 Linux x64 GNU/glibc 包；其他平台在没有匹配资产时会明确停止安装。

这条命令会把包安装到 `~/.vsp-codex/packages/standalone/`，并在 `~/.local/bin/vsp-codex` 创建可执行入口。已安装的用户可以直接运行 `vsp-codex update` 来更新到最新版本。

如果传了 `--vscode`，安装脚本还会执行 `vsp-codex vscode install --yes --cli <installed-path>`，让官方 VSCode ChatGPT 插件启动 VSP Codex CLI。

需要回滚 VSCode 设置时运行：

```shell
vsp-codex vscode rollback --yes
```

如果机器上已有 npm/brew 管理的官方 Codex，安装器默认只会提示冲突，不会删除它；确认要清理旧安装时再额外加 `--remove-existing`。

## 本地开发启动

```shell
cargo build --manifest-path codex-rs/Cargo.toml --bin vsp-codex
./codex-rs/target/debug/vsp-codex
```

本地重建当前 Linux x64 GNU/glibc preview release 包：

```shell
cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex
python3 scripts/build_vsp_preview_release.py \
  --entrypoint-bin codex-rs/target/release/vsp-codex \
  --channel alpha \
  --force
```

完整安装和 release 构建说明见 [安装与构建](docs/install.md)。

本机开发环境已将 `/home/heyx/.local/bin/vsp-codex` 链接到当前仓库的 debug 二进制，因此可以在任意仓库直接运行 `vsp-codex` 或 `vsp-codex resume --all`。

## 默认行为

- 命令入口：`vsp-codex`。
- 配置目录：`~/.vsp-codex`。
- Provider：默认使用 `VSP Provider`，通过 Base URL + API Key 访问 OpenAI Responses 兼容接口。
- 权限：默认使用 VSP YOLO，本地执行无需额外确认，等价于 `approval_policy=never` 和 `danger-full-access`。
- Sessions：共享 `~/.codex/sessions` 和 `session_index.jsonl`，便于继续查看原 Codex 会话。
- Skills / Plugins：共享可复用资产，但不共享官方 `auth.json`、`.credentials.json` 或完整 `config.toml`。

## Provider

首次启动时，TUI 会进入 VSP Provider 初始化页，填写 Base URL 和 API Key 后写入 `~/.vsp-codex/config.toml`。

TUI 内可使用：

```text
/provider
```

`/provider` 只展示 VSP Provider 和用户自定义的 OpenAI-compatible Provider；内置 OpenAI、Amazon Bedrock、Ollama、LM Studio Provider 不再出现在 VSP 选择器中。

## 保留能力

- TUI 本地 Agent 工作流。
- 默认可用的 Subagent，多 agent 并行工作上限为 32。
- Hypo-Workflow Host UX：发现并校验 installed plugin，按语义索引或 legacy Host Contract v1 注入只读 guidance，并保留 Ask UI、Workflow Markdown fallback 与 Plan/SubPlan freshness；Workflow 语义、生命周期和文件写入完全由 installed plugin 拥有。
- 结构化 Ask User Tools：TUI 支持单选、多选、填空、排序等原生问答卡片；不支持原生 UI 的客户端会降级为常规对话。
- 常驻 Plan/SubPlan 面板：模型可通过 structured plan state 更新右侧计划窗口，TUI 在输入框上方常驻展示当前进度。
- Hypo-Translate 运行时中文翻译。
- `vsp-codex exec` 和 `vsp-codex exec-server --remote`。
- MCP。
- Skills。
- Plugins。
- OpenAI Responses 兼容 Base URL/API Key Provider。
- 兼容读取历史 Sessions。

Hypo-Workflow 的 `/hw:*` 和 `$hypo-workflow:*` 路由来自已校验 installed release 的 command manifest；当前合同包含 `/hw:experiment`。TUI command palette 精确展示 manifest 声明的命令，未声明命令和 `/review`、`/compact`、`/model`、`/quality`、`/optimize` 等非 Workflow 命令不会被误判。Integration sync 是目标仓适配流程，不是用户 slash command。

## 禁用或隐藏的官方入口

- ChatGPT/OAuth 登录入口。
- Codex Cloud。
- App remote-control。
- 官方更新检查。
- 官方 feedback 上传。
- 常规 TUI 中的 OpenAI/Codex 官方提示、NUX 和品牌文案。

这些底层代码大多仍保留在仓库中，但通过 VSP gate 在 CLI/TUI/app-server 边界隐藏或禁用，以降低后续合并官方更新的冲突成本。

## 常用开发命令

```shell
cd codex-rs
just fmt
just test -p codex-tui user_typing_provider_slash_command_opens_provider_picker
cargo build --bin vsp-codex
```

TUI 或用户可见文本变化后在本地检查 snapshot；snapshot 不属于普通提交的强制 CI 门禁：

```shell
cd codex-rs
cargo insta pending-snapshots --manifest-path tui/Cargo.toml
```

## 文档

- [VSP Codex 用户指南](docs/vsp-codex.md)
- [配置参考](docs/config.md)
- [Skills](docs/skills.md)
- [Slash Commands](docs/slash_commands.md)
- [安装与构建](docs/install.md)
- [VSP CI 长期契约](docs/ci.md)
- [Cycle C1 归档摘要](.pipeline/archives/C1-vsp-codex-fork-customization/summary.md)
- [Cycle C2 归档摘要](.pipeline/archives/C2-foundation-experience-optimization/summary.md)
- [Cycle C3 归档摘要](.pipeline/archives/C3-ask-user-tools-plan-panel/summary.md)

## 版本

当前稳定预览版本为 `0.145.0-vsp.9.5`；当前 alpha 版本为 `0.145.0-vsp.11.1-alpha.7`。11.0 从 9.5 重新开发；11.1 引入新的本地压缩状态机、长上下文预算、执行连续性契约、Hypo-Workflow semantic Host UX、独立 Responses multi-provider 配置、统一模型切换，以及经过真实终端验证的 Markdown 标题和最终回复完整性合同。alpha.5 允许 session compatibility migration 在共同 migration 前缀和所需 schema 一致时兼容 Official/VSP 的尾部版本差异；alpha.6 重构提示词与 System Skills，新增受信任的 Subagent Worker Routing，并提供 Linux、macOS、Windows 原生预览产物；alpha.7 恢复官方 `spawn_agent` wire contract，修复 Alpha 6 连接 VSPLab `/responses` 稳定 502 的回归。当前 alpha 继续使用 OpenAI Responses 兼容接口，不继承已废弃 10.x 线的 Chat API 实现。

版本格式为 `官方基线-vsp.递增号`。例如 `0.134.0-vsp.2` 表示基于官方 Codex Rust `0.134.0` 版本线的第 2 个 VSP 内部版本；合并新的官方版本线时更新官方基线前缀，`vsp.N` 继续沿用 VSP 自有功能/修复递增序列。只 fork baseline 不加 VSP 功能时不递增；加入 VSP 自有改动时递增。如果上游基线来自 alpha tag，文档和 release notes 会额外记录具体 upstream tag。

## License

本 fork 保留上游 Apache-2.0 license。详见 [LICENSE](LICENSE)。
