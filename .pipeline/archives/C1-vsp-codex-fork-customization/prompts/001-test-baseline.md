# Milestone 1: 当前测试基线调查与运行矩阵

## Execution Boundary

本提示只执行 Milestone 1。不要修改业务行为，不要开始 `vsp-codex` 改名、`~/.vsp-codex` home、`/provider`、官方服务禁用、品牌清理或汉化实现。

目标是在当前源码基本不变的状态下建立真实测试基线：哪些命令可作为后续回归门槛，哪些失败、慢、忽略、依赖外部环境或暂时不能作为门槛。输出报告给用户确认后，后续 Milestone 2-6 才能通过 `plan-review` 修订并继续。

## Subworker Assignment Plan

Worker Separation: `strict`。用户已授权 Codex 在 `/hw:start` 和 `/hw:resume` 中使用执行 Subagent，角色必须分离。

### test

- Role status: authorized.
- Ownership: 测试调查、命令运行、失败复现、测试矩阵、TUI 用户模拟测试可行性调查。
- Allowed write paths:
  - `.pipeline/reports/test-baseline.md`
  - `.pipeline/reviews/001-test-baseline-test.md`
  - 临时命令输出摘录可以写入 `.pipeline/reports/test-baseline-artifacts/`
- Must not edit source code, snapshots, fixtures, Cargo manifests, lockfiles, or production files.
- Expected evidence:
  - 每个命令的 cwd、完整命令、开始/结束时间或耗时、exit status、passed/failed/ignored/blocked/not-run 分类。
  - 对失败命令保留关键错误摘要和下一步判断。
  - 对 ignored/tmux/VT100/snapshot 测试说明是否可作为真实 TUI 用户模拟基线。
- Pseudo-test rejection:
  - 只编译、不运行相关测试，不能算测试基线。
  - 只运行与后续触达模块无关的测试，不能算覆盖。
  - 失败命令未记录失败原因，不能算有效证据。

### implement

- Role status: authorized.
- Ownership: 本 Milestone 默认无业务实现。仅记录 blocker 或生成 implement review；不要创建或修改任何可执行脚本。
- Allowed write paths:
  - `.pipeline/reviews/001-test-baseline-implement.md`
  - 如确需创建报告辅助目录，仅限 `.pipeline/reports/test-baseline-artifacts/`
- Must not edit `codex-rs/`, `codex-cli/`, source snapshots, tests, manifests, or build logic during this Milestone.
- Expected evidence: 确认没有业务行为修改；若发现必须先修测试基础设施才能调查，记录 blocker，不自行修复。

### audit

- Role status: authorized.
- Ownership: 审查最终报告、命令证据、角色分离和验收风险。
- Allowed write paths:
  - `.pipeline/reviews/001-test-baseline-audit.md`
- Must not edit source or测试报告主体。
- Expected evidence:
  - 检查报告是否覆盖后续会触达的 TUI、CLI、config/provider、login/official-disabled、exec-server 风险。
  - 检查 passed/failed/ignored/blocked/not-run 是否区分清楚。
  - 检查是否存在伪测试或用无关测试冒充基线。

### main agent

- 编排 test、implement、audit 生命周期，在 `.pipeline/reports/test-baseline.md` 的 `Worker Lifecycle` 小节以及各 review 文件中记录 requested、started、completed/failed/blocked、closed/close_failed。
- 集成报告和用户可见总结。
- 不得本地承担 test、implement、audit 任一角色；不得在此 Milestone 中修改业务源码或测试断言。

## P2 Technical Solution

不修改业务行为。建立当前仓库在未改动状态下的测试基线：确认哪些测试可作为后续回归门槛、哪些测试慢、失败、忽略或依赖外部环境。输出测试矩阵、耗时、失败分类和推荐基线。

## P2 Technical Route

1. 记录当前工作树和工具版本。
2. 验证构建入口：`cargo build --bin codex`、`./target/debug/codex --version`。
3. 调查相关 crate 测试：`codex-tui`、`codex-cli`、`codex-config`、`codex-model-provider-info`、`codex-model-provider`、`codex-login`、`codex-exec-server`。
4. 调查 TUI snapshot 状态：`cargo insta pending-snapshots -p codex-tui`。
5. 调查被忽略的 TUI smoke，如 `resize_reflow` 的 tmux 测试，能跑则跑，不能跑记录环境 blocker。
6. 输出 `.pipeline/reports/test-baseline.md` 和后续建议的测试 profile。

## Research Required

状态：required and first milestone。需要实测命令耗时、失败和 blocker，不能预设最终基线。

需要调查的具体问题：

- `just test -p codex-tui` 在当前机器上是否可稳定通过，耗时多少。
- TUI snapshot 是否有 pending 或 `.snap.new`。
- ignored tmux smoke 当前机器是否能运行；如果不能，缺少什么依赖或环境。
- 相关 crate 的测试是否能作为后续 VSP 改造回归门槛。
- Node 相关脚本是否需要通过 `volta run --node 22 ...` 才能稳定执行。

## Risks And Alternatives

- 风险：`just test -p codex-tui` 可能耗时较长或暴露既有失败。
- 替代：先跑更窄的测试集合，但必须记录为什么不能作为完整基线。
- 拒绝：直接开始改 UI 后再补测试。

## Validation Path

必须在 `codex-rs` 目录执行 Rust 相关命令，除非命令明确要求 repo root。

必跑或必须尝试的命令：

```bash
git status --short
rustc --version
cargo --version
just --version
cargo nextest --version
cargo insta --version
cargo build --bin codex
./target/debug/codex --version
just test -p codex-tui
cargo insta pending-snapshots -p codex-tui
just test -p codex-cli
just test -p codex-config
just test -p codex-model-provider-info
just test -p codex-model-provider
just test -p codex-login
just test -p codex-exec-server
```

TUI smoke/ignored 调查：

```bash
tmux -V
just test -p codex-tui resize_reflow -- --ignored
```

如果某条命令因为缺少工具、环境、时间或当前仓库状态不能运行，报告中必须标记为 `blocked` 或 `not-run`，并写明原因。不要把未运行命令标为通过。

不要在未再次获得用户确认前运行完整 workspace `just test`。M1 报告可以建议是否需要完整套件，以及建议何时运行。

## Expected Report

写入 `.pipeline/reports/test-baseline.md`，建议结构：

1. Summary
2. Environment
3. Worktree State
4. Command Matrix
   - area
   - command
   - cwd
   - status: passed | failed | ignored | blocked | not-run
   - duration
   - key evidence
   - recommended baseline role
5. TUI User Simulation Coverage
   - snapshot tests
   - VT100 tests
   - tmux ignored smoke
   - gaps before M2
6. Failure And Blocker Details
7. Recommended Regression Profile For M2-M6
8. Upstream Merge Risk Notes
9. Next Decision Gate

## Audit Focus

- 测试报告是否覆盖 TUI、CLI、config/provider、login/official-disabled、exec-server 相关风险。
- 是否清楚区分 passed、failed、ignored、blocked、not-run。
- 是否有伪测试或只跑无关测试冒充基线。
- 是否遵守仓库约束：Rust 测试用 `just test`，不直接跑 `cargo test`；没有用户确认不跑完整 workspace `just test`；未修改 `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` 或 `CODEX_SANDBOX_ENV_VAR` 相关代码。

## Completion Criteria

- `.pipeline/reports/test-baseline.md` 已生成。
- `.pipeline/reviews/001-test-baseline-test.md` 和 `.pipeline/reviews/001-test-baseline-audit.md` 已生成。
- 报告能支持用户判断：继续 M2、缩小测试范围、先修测试基础设施，或要求补跑命令。
- 源码、测试断言、snapshots、manifest 和 lockfile 没有被此 Milestone 修改。
