# C8 P2 Decompose Checkpoint

Timestamp: 2026-05-30T13:43:31+08:00
Status: confirmed

## P1 Summary

Cycle 8 聚焦 VSCode Remote SSH 场景下的 VSP-Codex session 兼容和插件交互体验。用户确认所有 VSCode 插件使用都发生在 Remote SSH 窗口内，因此远端主机的 VSP-Codex binary、远端 CODEX_HOME、远端 `.vscode-server` 配置和远端 session store 是主路径。

需要处理的问题：

1. VSCode 插件 resume VSP-Codex session 失败：`Model provider `` not found`。
2. 官方 Codex 无法发现 VSP-Codex sessions，需要调研安全统一策略。
3. `vsp-codex vscode install/status` 应优先 Remote SSH server Machine settings，而不是本机 VSCode user settings。
4. 在不改插件源码的前提下，尽量利用 app-server protocol 和输出形态改善 Request Tool/Input、Markdown Workflow surface、Subagent prompt/report、Plan/SubPlan 等体验。
5. 最终验收必须包含真实 Remote SSH 插件恢复和交互。

## Milestone 1: Remote SSH VSCode Integration Probe

### Goal

让 VSP-Codex 的 VSCode 集成命令正确识别 Remote SSH 主场景，优先配置和诊断远端 `.vscode-server/data/Machine/settings.json`。

### Implementation Scope

- `vsp-codex vscode install/status` 的 settings scope 选择和诊断输出。
- Remote SSH server Machine settings fixture/test。
- CLI 文档或帮助文字中对 Remote SSH 的提示。

Non-goals: 修改 VSCode 插件源码、发布 VSIX、变更 session schema。

### technical_solution

把 Remote SSH server Machine settings 作为 VSP-Codex CLI 的优先写入/读取目标。当检测到远端 `.vscode-server/data/Machine/settings.json` 存在或当前处于 remote server 环境时，优先更新该文件中的 `chatgpt.cliExecutable` 等配置；本机 user settings 只作为显式 fallback 或兼容上下文。

### technical_route

1. 阅读 `codex-rs/cli/src/vscode_cmd.rs` 和 `codex-rs/cli/tests/vscode.rs`。
2. 补 Remote SSH 优先级红测：同时存在 local user settings 与 `.vscode-server` settings 时，VSP 路径选择 server Machine settings。
3. 增加 status 输出中的远端配置路径、binary 路径、CODEX_HOME/session home 提示。
4. 实现最小 scope resolution 调整。
5. 验证本机非 Remote SSH 场景仍保留可用 fallback。

### validation_path

- `cargo test -p codex-cli vscode -- --nocapture`
- `cargo fmt --check --package codex-cli`
- `git diff --check`
- 手动或 dry-run 检查 Remote SSH settings 路径诊断。

## Milestone 2: VSCode Plugin Protocol Capability Map

### Goal

明确官方 VSCode 插件黑盒客户端实际支持哪些 app-server 协议能力，以及哪些能力只能通过服务端输出回退实现。

### Implementation Scope

- VSIX/manifest/log black-box inspection。
- app-server protocol 与 thread item/tool item 能力梳理。
- Request Input、approval、permission、Markdown、Plan/Tool output 的 capability matrix。

Non-goals: 未经确认修改插件源码或 fork 插件。

### technical_solution

把插件当作协议客户端，从 VSIX manifest、extension host 日志、app-server JSON-RPC 请求/响应和现有 protocol tests 中建立能力矩阵。优先发现能被插件原生消费的 message/item types；如果不能消费，则为后续 Milestone 设计 markdown fallback。

### validation_path

- 产出 `.pipeline/reports` 或 review artifact 中的 capability matrix。
- 如有条件，捕获一次 Remote SSH 插件连接 app-server 的协议日志。
- 不做外部发布或插件修改。

## Milestone 3: Provider And Resume Compatibility Contract

### Goal

修复插件 resume VSP-Codex session 时因空 provider 或缺失 provider 触发的配置加载失败，并定义 provider metadata 的持久化/兼容契约。

### Implementation Scope

- app-server session list/read/resume/fork provider resolution。
- rollout/state-db persisted session metadata。
- VSP provider omission 与 fallback provider 的边界。
- 对 unknown non-empty provider 的错误保持可诊断。

Non-goals: 在 session meta 中泄露 VSP 私有 provider/token。

### technical_solution

把 provider 分为三类处理：缺失/空 provider 走当前请求或 VSP 默认 provider fallback；已知 VSP provider 可被省略但 resume 时恢复到合法配置；未知非空 provider 保持显式错误。resume API 不应把空字符串当成真实 provider 名称传给 config loader。

### validation_path

- app-server resume focused tests。
- rollout recorder/provider metadata focused tests。
- 插件 resume VSP session 的本地/Remote SSH 验证。

## Milestone 4: Session Discovery Strategy For Official Codex

### Goal

设计并实现或准备实现官方 Codex 发现 VSP-Codex sessions 的安全策略。

### Implementation Scope

- `~/.vsp-codex` 与 `~/.codex` session store 的发现/索引/镜像/导入导出策略。
- Workflow metadata、provider metadata、auth/config metadata 的风险分级。
- 对官方 Codex 可读 session 的脱敏或摘要规则。

Non-goals: 无边界共享所有 VSP private runtime 文件。

### technical_solution

先产出 risk-ranked strategy，再实现最小安全路径。候选方案包括：显式导入导出、只读 sanitized mirror/index、或受配置控制的双向 discovery。默认不得让官方 Codex 直接读取 `.pipeline` runtime、VSP provider 私有字段或凭据。

### validation_path

- 单元测试或 fixture 验证 safe fields 与 stripped fields。
- 说明官方 Codex 可见/不可见的数据边界。
- 不破坏 VSP 原生 Workflow 连续性。

## Milestone 5: Request Tool And Request Input UX For Plugin

### Goal

在不改插件源码的前提下，最大化利用现有协议改善 Workflow required decisions、普通澄清问题、authorization intent、tool/permission approvals 的交互体验。

### Implementation Scope

- `item/tool/requestUserInput` native/fallback 行为。
- Workflow required decision mapping。
- authorization-intent 与真实权限授权边界。
- tool/permission approval 与 ordinary question 的区分。

Non-goals: 绕过 request_permissions/approval overlay。

### technical_solution

建立服务端侧 decision routing：插件支持 native request input 时发送结构化问题；不支持时回退为 Markdown 卡片/线程项。Workflow confirmation、phase transition、scope selection、revision/rejection、authorization-intent、release/update confirmation 使用结构化形态；普通非 Workflow chat 不强制卡片化。

### validation_path

- app-server request_user_input tests。
- TUI/协议 fallback snapshot 或 fixture。
- 手动插件交互验证。

## Milestone 6: Markdown Workflow Surface And Remote SSH Acceptance

### Goal

统一插件侧可读的 Markdown Workflow surface，并完成真实 Remote SSH 端到端验收。

### Implementation Scope

- Subagent prompt/report、tool output、Plan/SubPlan、Workflow reports 的 Markdown surface。
- 插件可见 thread items 的渲染顺序和内容。
- Remote SSH 插件恢复 VSP session、展示 Prompt/报告、完成 Request Input 交互的验收脚本/清单。

Non-goals: 发布版本或外部更新推送。

### technical_solution

为插件不可原生展示的 Workflow 信息提供 Markdown-first fallback，同时保留结构化协议路径。最终验收以 Remote SSH VSCode 窗口为准：能找到/恢复 VSP session，能显示 Workflow plan/progress，能处理 request input，能查看 Subagent prompt/report 或对应 markdown report。

### validation_path

- focused protocol/rendering tests。
- `cargo fmt --check`
- `git diff --check`
- 真实 Remote SSH VSCode 插件端到端 acceptance。

