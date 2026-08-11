# VSP Codex 用户指南

VSP Codex 是面向 VSP Lab 内部工作流定制的 Codex fork。它默认使用 `vsp-codex` 命令和 `~/.vsp-codex` 配置目录，优先接入实验室内部 Provider。

## 预览版安装

Linux x64 预览版可以通过 GitLab Release downloads 安装：

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh
```

安装器会从稳定 release manifest 选择与当前平台和 `lab` 频道匹配的最新版，校验 SHA-256、包元数据和二进制版本，再解压到 `~/.vsp-codex/packages/standalone/`。需要同时配置 VSCode 时追加 `--yes --vscode --no-launch`。

当前预览包是 Linux x64 GNU/glibc release-profile preview 包，不是正式 MUSL release。它适合内部试用和 VSCode 接入验证，体积较大，不代表最终发布形态。

当前 manifest 只提供 Linux x64 GNU/glibc 包。预览包通过 GitLab Release downloads 提供，更新 manifest 使用 main 分支上的稳定 raw 路径，目标用户仍需要有仓库读取权限。

安装后验证：

```shell
vsp-codex --version
codex --version
vsp-codex vscode status --json
```

回滚 VSCode 设置：

```shell
vsp-codex vscode rollback --yes
```

## 启动

在任意仓库运行：

```shell
vsp-codex
```

本地重建当前 Linux x64 GNU/glibc preview release 包：

```shell
cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex
python3 scripts/build_vsp_preview_release.py \
  --entrypoint-bin codex-rs/target/release/vsp-codex \
  --force
```

更多安装器参数、GitLab raw 下载边界和源码构建说明见 [安装与构建](install.md)。

恢复历史会话：

```shell
vsp-codex resume
vsp-codex resume --all
```

普通 `resume` 会按当前仓库过滤会话；`resume --all` 会浏览共享 sessions 中的全部会话。

## Provider 初始化

首次启动时，如果 `~/.vsp-codex/config.toml` 中缺少完整 VSP Provider，TUI 会要求填写：

- Base URL
- API Key

写入后的配置形态等价于：

```toml
model_provider = "vsp"

[model_providers.vsp]
name = "VSP Provider"
base_url = "https://your-provider.example/v1"
experimental_bearer_token = "your-api-key"
wire_api = "responses"
requires_openai_auth = false
supports_websockets = false
```

## 切换 Provider

在 TUI 输入：

```text
/provider
```

选择器只展示：

- `VSP Provider`
- 用户自定义的 OpenAI Responses 兼容 Provider

官方内置 Provider 不在 VSP 选择器中显示。切换 Provider 会先同步当前 thread 设置；同步失败时不会更新 UI 状态。

## 共享原 Codex 资产

VSP Codex 默认隔离配置和认证，但共享以下可复用资产：

- `~/.codex/sessions`
- `~/.codex/session_index.jsonl`
- 用户 Skills
- Plugins cache/data
- `vendor_imports/skills`

不会共享：

- `~/.codex/config.toml`
- `~/.codex/auth.json`
- `~/.codex/.credentials.json`

这样可以复用历史会话和扩展能力，同时避免官方 OAuth/Cloud 配置回流到 VSP Codex。

## 官方服务入口

以下入口在 VSP Codex 中隐藏或禁用：

- ChatGPT/OAuth 登录
- Codex Cloud
- app-server remote-control
- 官方 update check
- feedback upload

这些入口如果被直接调用，会返回 VSP 禁用提示。

## 默认权限与 Subagent

VSP Codex 默认使用实验室内部的 YOLO 工作方式：

- `approval_policy=never`
- `sandbox_policy=danger-full-access`

在 TUI 和 `vsp-codex exec` 中无需额外输入危险模式参数。显式 CLI 参数或配置文件仍可覆盖这些默认值。

Subagent 默认启用，最大线程数为 32。TUI 会在底部显示活跃 Subagent 卡片，卡片包含昵称、模型、reasoning effort、角色、状态和运行时长；按 `Ctrl+O` 可打开详情。没有完整 metadata 时，界面使用 VSP 中文姓名池兜底，不显示 thread id/hash。

## Thinking / Reasoning

VSP Codex 可以显示服务端真实返回的 reasoning 内容，但不会伪造 Thinking。

- 默认显示 reasoning summary。
- `show_raw_agent_reasoning = true` 时显示 raw reasoning。
- `hide_agent_reasoning = true` 时隐藏 reasoning。
- 如果服务端没有返回 reasoning summary 或 raw content，TUI 不会生成假的 Thinking 块。

可以在 `~/.vsp-codex/config.toml` 中配置：

```toml
show_raw_agent_reasoning = true
```

raw reasoning 可能包含更长、更细的模型推理内容，只建议在调试或内部验证时打开。

## Markdown 渲染

TUI 会渲染常见 Markdown 样式：

- `/theme` 会打开统一 Appearance 入口，可继续选择 UI palette 或 syntax theme。
- `/ui-theme` 可选择 `teal`、`violet`、`amber` 三组主题色，并持久化到 `[tui].ui_theme`。
- syntax theme 仍独立持久化到 `[tui].theme`。
- 标题、strong、emphasis、inline code、link、blockquote、列表 marker、表格边框会跟随当前主题。
- History 中 user、assistant、reasoning、tool、ask 等输出块使用不同 accent。
- 输入框 focused prompt 使用当前主题 primary accent，比普通状态更明显。
- 无序列表会把 Markdown 源里的 `-` 渲染成层级 bullet：第一层 `•`，第二层 `◦`，第三层 `▪`，更深层按这组三个符号循环。

旧历史会话中已经渲染过的内容不一定会重新变样；验证样式时建议发送一条新的 Markdown 消息。

## 命令面板和快捷键

TUI 提供 `/commands` 命令面板，也可以用默认 `Ctrl+Space` 打开。

- 命令面板支持搜索和 Enter 执行常用安全命令。
- 默认隐藏退出、清空、debug、测试审批等危险或调试命令。
- `Copy latest code block` 可以复制最近 assistant 消息里的最后一个 fenced code block。
- `tui.keymap.global.command_palette` 可以重映射或解绑命令面板快捷键。

## Hypo-Workflow 项目辅助

当当前工作目录或祖先目录存在 `.pipeline/` 时，VSP Codex 会把项目识别为 Hypo-Workflow 项目，并启用一组有边界的运行时辅助：

- VSP 校验 installed release descriptor、SemVer、command manifest checksum、plugin identity、命令绑定和 Skill 路径，再把 manifest 声明的 `/hw:*` 命令接入解析器与 command palette；当前合同包含 `/hw:experiment`。
- 新语义 workspace 使用 `.pipeline/INDEX.md` 和 `.pipeline/cycles/`。首轮 guidance 优先引导读取语义索引和当前 Cycle，不要求刷新旧 Host Contract v1。
- 旧 manifest-based workspace 继续只读消费 Host Contract v1；损坏或失效的旧 projection 保持 fail closed。
- Workflow 语义、生命周期、Records、Receipts、恢复和所有 `.pipeline` 写入均由 installed Hypo-Workflow plugin 拥有；VSP Codex 只提供 Host UX。
- 通用 Host 能力继续保留：`request_user_input` / Ask UI、Plan/SubPlan freshness、Workflow Markdown fallback、Codex Hooks runtime 和 installed plugin discovery。
- 讨论/背景/想法/抱怨/问题/方案讨论类输入先按 Mini-contract 响应，顺序为 `我的理解` -> `问题原因` -> `推荐方案`，不会直接改文件或启动执行；明确具体的 imperative 请求仍可自动执行。
- 已展示计划、Mini-contract 或推荐方案后，`可以`、`确认`、`OK`、`go ahead`、`apply it` 等确认只授权已展示范围，不扩大到未说明的文件或行为。

限制：Workflow Host UX 不会放宽 destructive、system dependency、network、remote side effect 等确认边界，也不会在 plugin 之外生成或修复 Workflow 状态。Integration sync 是目标仓兼容流程，不是用户 slash command。

## Resume 历史列表

Resume/Fork 历史列表会先显示 SQLite metadata cache 中的可用名称、摘要、路径、分支和时间，再后台做 canonical verification。

- 首屏不读取完整 rollout JSONL，也不会提前加载 transcript preview。
- 如果 cache 中的名称、摘要、cwd、时间过期，后台校验会替换同一行，不会重复插入 session。
- `Ctrl+E` 展开当前 session 时才加载 recent preview。
- `Ctrl+T` 打开 transcript 时才加载完整 transcript。

## VSCode 插件接入

预览版通过修改官方 VSCode ChatGPT 插件配置来接入 VSP Codex。安装器传入 `--vscode` 时会自动写入：

```json
{
  "chatgpt.cliExecutable": "/path/to/vsp-codex",
  "chatgpt.localeOverride": "zh-CN"
}
```

也可以手动执行：

```shell
vsp-codex vscode install --yes
vsp-codex vscode status --json
vsp-codex vscode rollback --yes
```

限制：

- 设置文件按严格 JSON 处理；JSONC 注释、尾逗号和原格式不会保留。
- 真实用户机器仍需要已安装官方 VSCode ChatGPT 插件。
- Remote SSH 场景下，VSCode 可能忽略 Remote Machine 里的 `chatgpt.cliExecutable`；`vsp-codex vscode install` 会把官方 ChatGPT 插件 bundled `codex` 入口替换为 VSP shim，并保留 `codex.vsp-backup` 供 rollback。
- Question Tool 会通过 app-server 的 `item/tool/requestUserInput` 原生请求送到官方 VSCode ChatGPT 插件、Codex Desktop/remote 客户端的 Ask UI 弹窗；其它客户端仍走 Markdown fallback。
- 如果机器上已有官方 Codex CLI，安装器默认只提示冲突，不会删除旧安装；确认要清理时再传 `--remove-existing`。

## Context 与翻译

底栏 Context Window 统一显示为：

```text
Context 12K/400K 3%
```

含义是已用 token / context window 总量 / 已用百分比。旧配置中的 `context-remaining` 仍可解析，但在 VSP Codex 中也按已用量语义显示。

Hypo-Translate 运行时可把 Agent 输出、reasoning summary 和 Subagent 内容异步翻译成中文，并保留英文技术术语。隐藏或 transcript-only reasoning 不会发送到翻译接口，也不会在 transcript overlay 中泄露。

## 保留功能

- MCP
- Skills
- Plugins
- OpenAI Responses 兼容 Base URL/API Key Provider
- `vsp-codex exec`
- `vsp-codex exec-server --remote`

## 版本策略

当前预览版本：`0.145.0-vsp.9.5`，基于 upstream `rust-v0.145.0-alpha.4`，并包含 VSP 发布版本一致性校验。

格式为 `官方基线-vsp.递增号`。只做 VSP 自有修复时递增 `vsp.N`；合并官方新版本线时更新前面的官方基线版本。
