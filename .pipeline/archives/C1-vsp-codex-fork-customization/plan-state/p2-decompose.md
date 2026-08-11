# P2 Decompose: VSP Codex Fork 定制改造

时间：2026-05-24T20:02:00+08:00
状态：proposed checkpoint，等待用户确认后进入 P3 Generate。

## Feature 摘要

将当前 Codex Fork 改造成实验室内部使用的 `VSP Codex`：

- 外部入口从 `codex` 改为 `vsp-codex`。
- 默认使用 `~/.vsp-codex/`，首次 TUI 启动询问是否从 `~/.codex/` 导入。
- 默认模型接入方式为 OpenAI Responses 协议兼容的 Base URL + API Key。
- 保留 MCP、Skills、Plugins 和通用 URL/API provider 能力。
- 隐藏或禁用官方强绑定功能：Cloud、OAuth/ChatGPT 登录、ChatGPT apps/app-link、feedback upload、update checks、官方计划/文档提示等。
- 本 Cycle 优先建立测试基线和 TUI 用户模拟测试，不做全面汉化，只做品牌和官方提示清理。

## 全局验证契约

- 执行 Worker Separation：`strict`。后续执行提示必须分配 `test`、`implement`、`audit` 三个角色。
- 测试优先：任何行为改动前，先完成 Milestone 1 的测试基线调查并由用户确认后续测试范围。
- 伪测试拒绝规则：只编译、不跑相关测试、不记录失败原因、或只跑与触达模块无关的测试，均不能作为验收证据。
- TUI 用户可见变化必须有 snapshot 或 VT100/tmux 级别证据。
- Rust 代码变更后运行 `just fmt`；相关 crate 使用 `just test -p <crate>`，不要直接跑 `cargo test`。

## Milestone 1: 当前测试基线调查与运行矩阵

### technical_solution

不修改业务行为。建立当前仓库在未改动状态下的测试基线：确认哪些测试可作为后续回归门槛、哪些测试慢、失败、忽略或依赖外部环境。输出测试矩阵、耗时、失败分类和推荐基线。

### technical_route

1. 记录当前工作树和工具版本。
2. 验证构建入口：`cargo build --bin codex`、`./target/debug/codex --version`。
3. 调查相关 crate 测试：`codex-tui`、`codex-cli`、`codex-config`、`codex-model-provider-info`、`codex-model-provider`、`codex-login`、`codex-exec-server`。
4. 调查 TUI snapshot 状态：`cargo insta pending-snapshots -p codex-tui`。
5. 调查被忽略的 TUI smoke，如 `resize_reflow` 的 tmux 测试，能跑则跑，不能跑记录环境 blocker。
6. 输出 `.pipeline/reports/test-baseline.md` 和后续建议的测试 profile。

### research_required

状态：required and first milestone。需要实测命令耗时、失败和 blocker，不能预设最终基线。

### risks_and_alternatives

- 风险：`just test -p codex-tui` 可能耗时较长或暴露既有失败。
- 替代：先跑更窄的测试集合，但必须记录为什么不能作为完整基线。
- 拒绝：直接开始改 UI 后再补测试。

### validation_path

执行命令和通过信号：

- `cargo build --bin codex` 成功。
- 至少尝试 `just test -p codex-tui` 并记录耗时/结果。
- 对每个建议纳入基线的 crate，记录命令、结果、耗时。
- 如果 tmux/ignored smoke 不能运行，记录缺失依赖或失败日志。

验证负责人：`test` worker。`audit` worker 检查报告是否真实覆盖了后续会触达的模块。

### audit_focus

- 测试报告是否覆盖 TUI、CLI、config/provider、login/official-disabled、exec-server 相关风险。
- 是否清楚区分 passed、failed、ignored、blocked、not-run。
- 是否有伪测试或只跑无关测试冒充基线。

## Milestone 2: TUI 用户模拟测试框架补强

### technical_solution

在现有 `codex-rs/tui` 测试体系上补强可复用的 TUI 用户流程测试能力。优先复用 `VT100Backend`、snapshot、`chatwidget` slash command tests 和现有 tmux smoke，不引入新 UI 测试框架。

### technical_route

1. 基于 Milestone 1 报告选择稳定测试层级。
2. 为关键用户流程增加或整理测试 helper：启动/空会话、slash popup、选择视图、footer、禁用提示、provider picker。
3. 新增未改功能前的基线测试，确保当前行为可通过。
4. 为后续 VSP 功能预留测试入口，但不提前写实现断言。

### research_required

依赖 Milestone 1 的耗时/失败报告。若 M1 显示现有 TUI 测试严重不稳定，本 Milestone 需要先降级范围并让用户确认。

### risks_and_alternatives

- 风险：TUI 大文件和 snapshot 很多，更新不当会造成大面积 snapshot churn。
- 替代：先只增加 focused VT100/helper 测试，再逐步扩大 snapshot。
- 拒绝：为了测试方便重构 TUI 核心大模块。

### validation_path

- `just test -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui` 无意外 pending。
- 新测试能在未改功能状态下通过。

验证负责人：`test` worker；`implement` 仅可改测试辅助代码；`audit` 检查没有把实现需求伪装成测试基线。

### audit_focus

- 是否遵守 TUI snapshot 要求。
- 是否模拟真实用户可观察行为，而不是只测私有函数。
- 是否避免改动业务行为。

## Milestone 3: `vsp-codex` 外部入口与 VSP home

### technical_solution

将外部用户入口改为 `vsp-codex`，同时将默认 home 从 `~/.codex` 改为 `~/.vsp-codex`。为降低 merge 成本，内部 crate 名称可暂时保持 `codex-*`，但外部二进制、just 入口和用户可见名称改为 VSP Codex。

### technical_route

1. Cargo：将 `codex-rs/cli/Cargo.toml` 的 bin 目标从 `codex` 改为 `vsp-codex`，评估是否保留测试期 alias。
2. just：新增/改用 `vsp-codex` recipe，后续测试用 `cargo build --bin vsp-codex`。
3. Bazel：更新 `codex-rs/cli/BUILD.bazel` 的 binary target 和依赖引用，或记录 Bazel 暂缓范围。
4. npm wrapper：将 `codex-cli/package.json` bin 和 JS wrapper 查找的原生 binary 名称改为 `vsp-codex`；发布包名是否改名留到 release 阶段。
5. Home：修改 `codex-rs/utils/home-dir/src/lib.rs` 默认目录为 `~/.vsp-codex`。第一阶段可保留 `CODEX_HOME` override 以降低测试和脚本 churn，后续再决定是否新增 `VSP_CODEX_HOME`。
6. TUI 首次启动检测：若 `~/.vsp-codex` 缺配置且 `~/.codex` 存在，显示现有风格的选择界面，询问是否导入。

### research_required

已完成初步代码定位；仍需在实现前通过 Milestone 1/2 确认改二进制名称会影响哪些测试和 Bazel target。

### risks_and_alternatives

- 风险：完全移除 `codex` bin 会影响现有测试、Bazel runfiles、npm wrapper 和内部 arg0 dispatch。
- 替代：短期同时产出 `vsp-codex` 和隐藏兼容 `codex`。用户偏好是完全 `vsp-codex`，若测试成本过高需要重新确认。
- 拒绝：只改 TUI 标题而保留外部 `codex` 命令。

### validation_path

- `cargo build --bin vsp-codex`
- `./target/debug/vsp-codex --version`
- `just vsp-codex --version`
- 新 home 目录测试：未设置 env 时解析为 `~/.vsp-codex`；设置 env override 时仍可测试隔离。
- 首次导入选择界面 snapshot/VT100 测试。

验证负责人：`test` worker 拥有新增/调整测试；`implement` 改入口/home；`audit` 检查是否误破坏内部工具链。

### audit_focus

- 用户可见入口是否不再需要 `codex`。
- 是否避免无谓 crate rename 造成巨大 merge 冲突。
- 是否有旧 `~/.codex` 写入遗漏。

## Milestone 4: Provider 配置与 `/provider`

### technical_solution

基于已有 `model_providers` 和 `ModelProviderInfo`，提供 VSP 风格 provider 管理。第一版支持多 provider 展示和选择，不做 TUI 内编辑 API Key。Provider 是 OpenAI Responses 协议兼容的 Base URL + API Key 组合。

### technical_route

1. 定义 VSP 推荐 config 形态：`model_providers.<id>.base_url`、API key 来源、active provider。
2. `/provider` 加入 `SlashCommand`、popup 过滤、dispatch。
3. TUI provider picker 显示 provider 名称、Base URL、当前全局/会话状态。
4. 选择结果支持：
   - 全局默认持久化到 `~/.vsp-codex/config.toml`。
   - 当前 thread/session 临时覆盖，复用现有 thread settings/update 能力或局部 runtime state。
5. `/model` 与 `/provider` 的关系：provider 决定端点和凭据，model 仍负责模型选择。

### research_required

需要在实现前确认 API key 存储方式。现有 provider 支持 env_key 和 auth 配置，直接把 API key 写 config 便利但有安全风险；可作为实验室内部第一版，但必须避免日志/snapshot 泄漏。

### risks_and_alternatives

- 风险：直接写 API key 到 config 会增加泄漏风险。
- 替代：config 存 env var 名称，API key 放环境变量或 auth 文件。
- 拒绝：把 provider 逻辑直接塞进 `codex-core` 大模块。

### validation_path

- Config/provider 解析测试。
- `/provider` popup snapshot。
- 选择全局 provider 后 config 更新测试。
- 会话临时 provider 切换测试。
- provider 切换后新 turn 参数使用期望 provider 的测试。

验证负责人：`test` worker；`implement` 改 config/TUI/provider；`audit` 检查 secret 泄漏和 session/global 边界。

### audit_focus

- Base URL/API Key 不出现在日志、snapshot 或错误文本中。
- 全局和临时 provider 语义不混淆。
- `ConfigToml` 如有变更必须更新 schema。

## Milestone 5: 官方服务入口隐藏/禁用

### technical_solution

采用低 merge 成本策略 A：隐藏官方入口，显式调用时给统一 VSP 禁用提示。底层 crate 暂留，避免大规模删除导致 merge 冲突。保留 MCP、Skills、Plugins 的非官方能力。

### technical_route

1. CLI：隐藏 `cloud`、ChatGPT/OAuth login、update、feedback 等官方入口；调用时返回统一中文提示。
2. TUI：隐藏 `/apps`、`/feedback`、ChatGPT app-link、OAuth/logout 官方路径、Plan Mode nudge 等普通用户入口。
3. Plugin：保留 plugin 基础功能，但隐藏或禁用 ChatGPT app auth 相关路径。
4. exec-server：保留本地 exec-server；隐藏/禁用 `exec-server --remote` 官方 registry/rendezvous 路径，除非后续接入 VSP 自有服务。
5. 文案：统一使用“此功能依赖 OpenAI 官方服务，VSP Codex 暂不支持。”

### research_required

MCP/Plugins 的官方 app auth 与普通 plugin 能力交叉，需要实现时逐项确认，避免误删可用插件能力。

### risks_and_alternatives

- 风险：过度隐藏导致 MCP/plugin 可用性下降。
- 替代：增加 feature visibility 层，只在 normal UI 隐藏，debug/config 可见。
- 拒绝：第一阶段物理删除 Cloud/OAuth/Remote 底层 crate。

### validation_path

- CLI help 不显示禁用官方命令。
- 直接调用禁用命令返回统一提示。
- TUI slash popup 不显示禁用官方命令。
- MCP、Skills、Plugins 基础入口仍可见并可用。

验证负责人：`test` worker；`audit` 检查保留/禁用边界。

### audit_focus

- 禁用范围是否和用户目标一致。
- 是否仍暴露 ChatGPT/OAuth 引导。
- 是否误伤 MCP/Skills/Plugins。

## Milestone 6: VSP 品牌与官方提示清理

### technical_solution

完成最小品牌清理：显示名 `VSP Codex`，简洁中文提示，专业术语保留英文。不做全面汉化。

### technical_route

1. 会话头、启动卡片、footer、禁用提示、状态/帮助中的官方名称替换。
2. 删除或隐藏 Base URL、Plan Mode、官方计划类型、官方文档链接等不必要提示。
3. 保留专业术语：Provider、MCP、Skill、Plugin、Base URL、API Key、Responses API。
4. 更新 snapshot。

### research_required

依赖前序 Milestone 确定哪些官方入口已经隐藏，避免重复改文案。

### risks_and_alternatives

- 风险：大面积文案改动导致 snapshot churn。
- 替代：先改高频入口和官方提示，完整汉化另开 Cycle。
- 拒绝：一次性逐字翻译整个 TUI。

### validation_path

- `just test -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui`
- 关键 TUI snapshot 显示 `VSP Codex`，不再出现普通用户可见的官方 OAuth/Cloud/ChatGPT app 引导。

验证负责人：`test` worker；`audit` 检查未清理的官方提示和过度翻译。

### audit_focus

- 高频 UI 是否一致显示 VSP Codex。
- 中文是否简洁，不破坏专业术语。
- 是否仍残留明显官方服务引导。

## P2 检查点建议

建议 P3 先只生成 Milestone 1 的执行提示。Milestone 1 完成后，使用 plan-review 根据真实测试基线修订 Milestone 2-6，再继续生成后续执行提示。
