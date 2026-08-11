# 初始化决策与注意事项

## 初始化决策

- 项目类型判定为已有大型 monorepo，不是空项目。
- 由于源文件和 crates 数量明显超过小项目阈值，采用 `.pipeline/architecture/` 文件夹模式，而不是单文件 `.pipeline/architecture.md`。
- 非交互初始化默认使用 `automation.level=balanced`，符合 Hypo-Workflow init 规范。
- 项目本地 worker separation 记录为 `recommended`，后续 Cycle 若进入规划，应在 P0 Configure 中确认 subagent 授权、验收模式、PR/MR 远程写入、完整回归、分析边界等。
- Rules 使用推荐预设：`.pipeline/rules.yaml` 写入 `extends: recommended`。
- 本次初始化不导入 Git 历史，因为用户没有请求 `--import-history`。

## 仓库约束

- Rust crate 名称以 `codex-` 为前缀；新增/修改 Rust 代码需遵守 AGENTS.md 中 clippy、format、trait、测试、TUI 和 app-server API 约束。
- 不得添加或修改与 `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` 或 `CODEX_SANDBOX_ENV_VAR` 相关代码。
- Rust 代码变更后自动运行 `just fmt`；较大变更运行 scoped `just fix -p <project>`；测试使用 `just test`，不要直接运行 `cargo test`。
- 修改 `ConfigToml` 或嵌套配置类型后运行 `just write-config-schema`。
- 修改 app-server v2 API 后更新 README/示例，运行 `just write-app-server-schema` 和 `just test -p codex-app-server-protocol`。
- 修改 Rust 依赖后运行 `just bazel-lock-update` 和 `just bazel-lock-check`。
- 新增 build-time file include 时同步更新对应 `BUILD.bazel` 的 compile/build/test data。

## 环境风险

- `just` 和 `cargo-nextest` 原先缺失，已安装并验证；推荐测试命令不再被该问题阻塞。
- `cargo metadata --no-deps` 原先因首次同步 Rust 1.93.0 toolchain 超时；toolchain 补齐后已成功运行。
- 已重新执行 `cargo build --bin codex` 并验证 `codex-rs/target/debug/codex` 可运行。
- 当前 Codex 进程内普通 `node` 仍为 `v20.20.0`，原因是 `_VOLTA_TOOL_RECURSION=1` 与 PATH 中 Node 20 image 优先；需要 Node 22 的仓库命令使用 `volta run --node 22 ...`。
- 本地 Codex 状态库曾出现 migration 冲突；旧 `~/.codex/state_5.sqlite*` 已按 repair 语义备份为 `*.codex-repair-1779620444.0.bak`，新状态库已通过 doctor 检查。
- app-server、MCP server、exec-server 是长驻或协议服务；初始化阶段只验证 help，不启动长驻会话。

## 后续建议

- 若要把初始化拓展为完整开发计划，下一步使用 `/hw:cycle new` 或 `/hw:plan`，并先完成 P0 Configure。
- 若要导入既有 Git 历史，单独运行 `/hw:init --import-history`，必要时加 `--interactive` 先审查拆分方案。
- 若要做代码改动，按 crate 作用域运行格式化、lint 和测试；Rust 代码变更后运行 `just fmt`，测试使用 `just test -p <crate>`。

## Plan Review: Milestone M1 后置测试基线修复

时间：2026-05-24T20:38:01+08:00

### ADDED

- 增加 M1 后置测试修复事实：`codex-tui` 当前本机基线已转绿，`just test -p codex-tui` 通过，2614 passed、4 skipped。
- 增加格式化 gate 修复事实：`just fmt` 已能在当前 Linux glibc 环境通过；Python Ruff 阶段改为 no-project 工具环境运行，不再安装 Python SDK runtime 依赖。

### CHANGED

- M2 的前置假设从“需要先修复或豁免 11 个 TUI 失败”变为“可以直接基于绿色 `codex-tui` 补强 TUI 用户模拟测试”。
- 后续所有涉及 Rust/TUI/Python SDK 文本或测试辅助的变更，都可以把 `just fmt` 重新纳入基础 gate。
- Python SDK 的问题边界被拆分：`just fmt` 已不再受阻；但 `sdk/python` 默认安装仍可能因为缺少 `openai-codex-cli-bin` Linux glibc/manylinux wheel 而失败。

## Cycle C2 P3：基础体验优化执行计划

时间：2026-05-25T23:42:15+08:00

### ADDED

- 生成 C2 六个执行 prompt：默认 YOLO 与底栏 Context、Subagent 核心策略、Subagent 常驻卡片、Hypo-Translate 运行时翻译、MCP 提示迁移与静态汉化、集成回归。
- 增加 C2 架构计划文件：`.pipeline/architecture/c2-foundation-experience.md`。

### CHANGED

- `.pipeline/state.yaml` 进入 `p4_confirm`，等待用户确认后才开始执行。
- `.pipeline/cycle.yaml` 记录 C2 为 `build` workflow，严格 worker separation，`test`、`implement`、`audit` 三类 worker 必须保留。

### REASON

- 用户已确认进入 P3，需要把 P2 讨论结果固化为可执行、可审核、可恢复的 Milestone prompts。

### IMPACT

- 下游执行顺序固定为 M1 到 M6。
- 后续 `$hypo-workflow:start` 或 `$hypo-workflow:resume` 必须按 prompt 中的 Subworker Assignment Plan 执行，先补测试，再实现，再审计。

## Cycle C2 执行 Gate 策略调整

时间：2026-05-25T23:46:00+08:00

### CHANGED

- C2 从 `p4_confirm` 改为 `ready_to_start`。
- 用户后续输入 Start 即视为执行确认。
- Start 后不设置 Milestone 间人工 Gate，M1 到 M6 连续执行。

### STOP CONDITIONS

- 真实技术阻塞。
- 同一验证项重复失败且需要用户决策。
- 缺少无法从本地上下文推断的外部输入。
- M6 结束后的最终人工验收。

### REASON

- 原 `codex-tui` 失败来自测试夹具非 hermetic：`/tmp/project` 被宿主机 `/tmp/.git` 影响；Unix IPC 测试 socket 父目录权限也不符合生产校验。
- 原 `just fmt` 失败来自工具命令安装了整个 Python SDK 项目依赖，而 Ruff 格式化本身不需要安装 `openai-codex-cli-bin`。

### IMPACT

- 下游 prompts 受影响：M2、M3、M4、M5、M6。
- 推荐更新：
  - M2：删除“若 TUI 基线仍红则先降级/豁免”的阻塞假设；改为在绿色基线之上新增用户模拟测试，并把 `just fmt`、`just test -p codex-tui`、supported `cargo insta pending-snapshots` 作为核心验证。
  - M3-M6：保留原顺序，但所有 user-visible TUI 变更都必须基于绿色 `codex-tui` snapshot/VT100 证据；`just fmt` 应作为例行验证。
  - Python SDK default install/runtime packaging 不放入 M2-M6 主线，作为后续独立 Patch 或新 Milestone 决策。
