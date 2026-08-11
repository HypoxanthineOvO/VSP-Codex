---
kind: plan
mode: plan
cycle: C24-responses-multi-provider-config-model-switching
status: closed
updated: 2026-08-08T21:22:00+08:00
progress: PROGRESS.md
execution: EXECUTION.md
---

# C24 Responses 多 Provider 配置与模型切换

## 执行目的

让新版 VSP 与 DEX 在不污染官方 `config.toml` 的前提下，同时发现并使用多个原生 Responses provider。第三方 provider、独立凭据引用、模型列表和新版客户端默认选择存放在 `$CODEX_HOME/model-providers.toml`；不支持该功能的旧版或官方客户端忽略这个兄弟文件并继续使用原配置。TUI `/model` 在发现附加 provider 时恢复 provider 分组选择，没有附加 provider 时维持当前单 provider 模型列表，并明确显示当前 provider 名称。

## 我的理解

- 目标不是复活 V10 的 Chat Completions 兼容层，而是复用它的多 provider 身份、隔离配置和分组模型选择经验，面向已原生支持 Responses 的第三方服务做更小实现。
- 一个模型选择必须是 `(provider_id, model_id)`，不能只保存 model 字符串；不同 provider 必须分别绑定 Base URL 和凭据来源。
- 兼容性的关键不只是把 provider 定义移出 `config.toml`：新版客户端的默认 provider/model 选择也不能写回官方配置，否则旧版仍会因找不到该 provider 而启动失败。

## 问题原因

- 当前 `Config` 已支持 `model_providers`，但定义与 `model_provider` 默认选择来自同一官方配置层；单独切换 provider 会修改全局选择，且旧客户端无法理解新版私有字段或缺失的 provider 定义。
- 当前 TUI 把 `/provider` 和 `/model` 分成两个面板；`/model` 只展示当前 catalog。V10 曾有 provider-scoped catalog 和统一选择器，但实现同时承载 Chat profile、远端 discovery、凭据管理和大量会话投影，直接恢复会带回不再需要的复杂度。
- 当前主线保留了 provider 切换基础设施，却移除了 V10 的 `ModelProviderCatalog`、provider-grouped picker 和 `modelProvider/list`，因此不能仅增加一个 TOML 文件就完成端到端混用。

## 推荐方案

### 配置合同

首版使用 `$CODEX_HOME/model-providers.toml`，建议格式如下：

```toml
version = 1
default_provider = "deepseek"
default_model = "deepseek-v4-flash"

[providers.deepseek]
name = "DeepSeek"
base_url = "https://api.deepseek.com/v1"
env_key = "DEEPSEEK_API_KEY"
wire_api = "responses"

[[providers.deepseek.models]]
id = "deepseek-v4-flash"
display_name = "DeepSeek V4 Flash"
context_window = 1048576
supported_reasoning_efforts = ["low", "high", "max"]
default_reasoning_effort = "high"
```

- `[providers.<id>]` 仍是“中括号写 provider 名字”，同时给版本和默认选择保留稳定命名空间。
- 首版只接受 `wire_api = "responses"`；不恢复 Chat 私有状态、Chat profile 或跨协议投影。
- TOML 只保存环境变量名，不保存明文 API key。每个 provider 使用独立 `env_key`。
- `models` 是 provider-scoped 的确定性本地目录；每项显式携带 `context_window`，避免不同 provider 的同名模型复用错误 metadata。若后续确认某 provider 提供兼容且可靠的 `/models`，可在不改变文件兼容边界的情况下增加可选 discovery。
- 文件缺失等同于空附加层；未知版本、重复 provider ID、空模型或与官方配置冲突的 provider ID应给新版客户端明确错误，不能静默覆盖。

### 加载与优先级

```text
official config.toml --------------------> 官方基础 Config / 当前 provider
                                             |
model-providers.toml -> 独立严格解析 -> 附加 Responses registry
                                             |
                                             v
                                effective ProviderRegistry
                                             |
                     +-----------------------+----------------------+
                     |                                              |
              TUI grouped /model                         app-server/DEX surfaces
                     |                                              |
                     +----------- atomic (provider, model) --------+
                                             |
                         thread runtime + addon selection persistence
```

- 附加文件在官方配置完成解析后单独加载，不作为未知字段合并进 `ConfigToml`，因此不会改变官方 schema，也不会让旧客户端看到任何新字段。
- `default_provider/default_model` 只覆盖新版 VSP/DEX 的启动选择；缺失时使用官方配置当前选择。
- `/model` 选择同时更新当前 thread 的 provider 和 model；持久化只原子改写附加文件的两个 default 字段，不调用现有会写 `config.toml` 的 provider/model batchWrite 路径。
- 没有附加 provider 时不显示空 tabs 或额外步骤，只在现有模型面板标题/副标题中显示当前 provider 名称。

## 执行边界

- 包含：独立配置 parser/loader、provider+model 身份与有效 registry、VSP/DEX 共用读取路径、TUI 分组模型面板、必要的 app-server v2 provider/model 查询与 thread 设置衔接、选择持久化、兼容与 UI 测试。
- 候选内置示例为 DeepSeek、Kimi、GLM；M1 只把有当前官方证据证明支持 Responses 且满足工具调用流的 provider 纳入首版验收。其余使用显式 custom Responses 配置，不做未经验证的兼容承诺。
- 不包含：Chat Completions 兼容层、V10 provider-private reasoning state、明文密钥存储/迁移、自动迁移旧 `chat-providers.config.toml`、发布与安装。
- 不修改 `.pipeline/state.yaml`、`.pipeline/cycle.yaml`、`.pipeline/rules.yaml`。
- 源码修改需在本 Proposal 获得确认后开始；网络核验、服务重启、发布或安装仍遵循各自边界。

## 验证目标

- 旧版/官方客户端面对同一 `CODEX_HOME` 时忽略 `model-providers.toml`，基础 `config.toml` 可独立解析和启动。
- 新版在附加文件缺失、空文件、一个 provider、多个 provider、无凭据、重复 ID、未知版本和损坏 TOML 下都有确定行为。
- DeepSeek/Kimi/GLM 只有在官方 Responses 能力验证通过后才进入首版模板或测试矩阵；每个 provider 的请求使用自己的 Base URL 和 API key 环境变量。
- `/model` 在多 provider 时按 provider 分组显示并原子切换 `(provider, model)`；单 provider 时保持现有操作路径并显示 provider 名称。
- 切换、resume、fork、compact 后 provider/model 身份不串线；旧 provider 的模型缓存不泄漏到新 provider。

## 完整计划

| ID | 阶段 | 期望结果 | 验证方式 |
| --- | --- | --- | --- |
| `M1` | 固化 Responses provider 与独立配置合同 | 产出最终字段、优先级、冲突/错误策略和 DeepSeek/Kimi/GLM 当前官方能力矩阵；确认 VSP 与 DEX 共用入口 | parser RED 测试；官方资料带日期核验；读取入口与调用链说明 |
| `M2` | 实现独立 loader、registry 与安全持久化 | `model-providers.toml` 缺失时零行为变化；存在时生成 provider-scoped 模型目录；选择只写附加文件且原子更新 | dedicated `*_tests.rs` 覆盖解析、合并、冲突、凭据隔离、原子写入和旧配置不变 |
| `M3` | 接通 runtime、app-server/DEX 与统一 `/model` 面板 | provider/model 作为单一选择事务应用；多 provider 显示分组，单 provider 显示当前 provider；失败不留下半切换状态 | core/app-server 公共 API 测试；TUI 交互测试与 `insta` 快照；resume/fork/compact provider 身份回归 |
| `S1` | 人工审阅真实多 provider 切换 | 用户看到实际 TUI：多 provider 分组、单 provider fallback、缺凭据/错误状态，并用隔离 `CODEX_HOME` 完成至少两个 Responses provider 的切换 | 用户检查截图/实际运行结果；接受标准是面板清晰、切换无需手改官方配置、旧客户端仍可启动 |
| `M4` | 收口兼容、回归与维护面 | 修复 S1 反馈；完成跨平台、schema（若 API 变化）、格式和变更规模检查；留下可维护的示例与边界说明 | focused crate tests；`just write-app-server-schema`（如适用）；`just test -p codex-app-server-protocol`；`just test -p codex-tui`；snapshot review；rustfmt、Clippy/diff hygiene |

## 主要逻辑与用户行为测试清单

- 附加文件缺失时，启动、当前 provider、模型列表和官方 `config.toml` 写回行为与现状一致。
- 多个 provider 使用相同 model ID 时仍以 `(provider_id, model_id)` 隔离；切换后请求 Base URL、header/env key 和模型目录匹配目标 provider。
- 选择成功时 provider 与 model 同时生效；provider 无凭据、model 不存在、thread update 失败或持久化失败时不会形成不可恢复的半状态。
- 默认选择和后续切换不向 `config.toml` 写入第三方 provider/model；模拟旧客户端只读官方配置仍可启动。
- `/model` 多 provider、单 provider、窄终端、长 label、当前项、空模型、缺凭据和加载错误均有快照。
- 新会话、resume、fork、compact 与子 Agent 继承正确 provider/model；跨 provider 不复用错误 cache 或 provider-private历史字段。
- VSP 与 DEX 的本地/连接式启动路径读取同一个有效附加层；app-server API 不返回 API key 值或上游敏感错误正文。

## 风险与后续边界

- “支持 OpenAI Responses endpoint”不等于完整支持 Codex 所需工具、流式事件和 reasoning item；M1 必须按真实协议能力筛选，不能只看 endpoint 名称。
- 若 DEX 使用独立进程和不同 `CODEX_HOME`，需要在 M1 明确它的配置根目录传递方式；设计保持按有效 `CODEX_HOME` 查找，不硬编码用户 home。
- V10 统一 picker 的原提交约 6700 行，不能整体 Cherry Pick。C24 只提取 provider-scoped identity、catalog 隔离和分组 UI；变更规模用于推动模块化和分阶段验证，不构成人工 gate，也不改变本 Cycle 已确认范围。
- 明文密钥管理和供应商 `/models` 自动发现可作为后续 Cycle；首版优先保证兼容性、可审计配置与确定切换。

## Proposal Gate

用户于 2026-08-08 确认并开始；接受 S1 后授权 Agent 完成自动化 hardening。M1-M4 与 S1
均完成，C24 于 2026-08-08 关闭。发布不在本 Cycle 范围内。
