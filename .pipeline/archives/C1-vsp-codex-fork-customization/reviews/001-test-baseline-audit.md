# M1 Audit Review: Test Baseline

verdict: pass

## reviewed_refs

- `.pipeline/prompts/001-test-baseline.md`
- `.pipeline/reports/test-baseline.md`
- `.pipeline/reviews/001-test-baseline-test.md`
- `.pipeline/reviews/001-test-baseline-implement.md`
- `.pipeline/reports/test-baseline-artifacts/`
- `.pipeline/PROGRESS.md`
- `.pipeline/state.yaml`
- `.pipeline/log.yaml`
- 当前工作树检查：`git status --short`
- 当前 snapshot 残留检查：`find . -name '*.snap.new' -print`
- 当前源码 diff 抽查：`git diff --stat -- codex-rs codex-cli Cargo.toml Cargo.lock justfile MODULE.bazel.lock`

## 结论

M1 产物满足 `001-test-baseline.md` 的 audit_focus，可以接受为测试基线调查结果。该结论不表示后续 M2 UI/品牌工作已有绿色 TUI 回归门槛；相反，报告清楚记录了 `codex-tui` 当前为红基线，M2 前需要修复或显式豁免。

## 发现

### Pass: 风险覆盖完整

测试报告覆盖了要求的主要风险面：

- TUI：`just test -p codex-tui` 已运行并失败，记录 2603 passed、11 failed、4 skipped；`resize_reflow -- --ignored` tmux smoke 已运行并通过。
- CLI：`just test -p codex-cli` 已运行并通过。
- config/provider：`codex-config`、`codex-model-provider-info`、`codex-model-provider` 已运行并通过。
- login/official-disabled：`codex-login` 已运行并通过，报告将其列为 login、official service disabling、auth storage、workspace 相关 gate。
- exec-server：`just test -p codex-exec-server` 已运行并通过。

### Pass: 状态分类基本清楚

报告没有把失败命令标成通过：

- `just test -p codex-tui` 标为 failed，并保留失败测试名、exit 100、代表性 snapshot/cwd diff。
- `cargo insta pending-snapshots -p codex-tui` 标为 failed，并解释为当前 `cargo-insta 1.47.2` 不支持 `-p`。
- 辅助 `cargo insta pending-snapshots` 单独标为 passed，并只作为 snapshot hygiene 辅助证据。
- tmux ignored smoke 标为 passed，同时记录 `3 passed, 2615 skipped`，没有用它替代完整 TUI gate。

本轮没有发现应标为 blocked 或 not-run 却被标成 passed 的命令。

### Pass: 未发现伪测试冒充基线

报告明确把 `cargo build --bin codex` 和 `./target/debug/codex --version` 定位为 build/version smoke，而非测试 gate。各风险面使用 `just test -p <crate>` 运行相关 crate，没有用无关测试替代相关覆盖。TUI 子集通过也被标为窄 smoke，没有冒充完整 `codex-tui`。

### Pass: Worker Separation 有证据

`.pipeline/PROGRESS.md` 和 `.pipeline/log.yaml` 记录了不同 worker：

- test：Descartes，已 completed/closed，产物为 test baseline report/review。
- implement：Ampere，已 completed/closed，仅写 no-op implement review。
- audit：Dirac，已 started，本文件为 audit 输出。

test review 与 implement review 的写入范围符合 prompt 约束。未发现 test/implement 越权修改源码、测试、snapshots、manifest、lockfile、BUILD 或 justfile 的证据。

### Pass: 无残留 snapshot 或源码改动

当前检查结果：

- `git status --short` 仅显示 `?? .pipeline/` 与 `?? .plan-state/`。
- `find . -name '*.snap.new' -print` 无输出。
- `git diff --stat -- codex-rs codex-cli Cargo.toml Cargo.lock justfile MODULE.bazel.lock` 无输出。

这支持报告中“`.snap.new` 已移除、未接受 snapshot、未改源码/测试/build 文件”的说明。

## 风险

- Warning: `codex-tui` 当前全量测试红基线。M2 若涉及 UI、状态行、terminal title、品牌或 cwd/home 呈现，不应把完整 TUI gate 当作绿色门槛；必须先修复、定位环境原因，或由用户显式豁免。
- Warning: `cargo insta pending-snapshots -p codex-tui` 是 prompt 要求命令，但当前安装的 `cargo-insta 1.47.2` 不支持该参数。后续需要标准化可执行的 snapshot pending 检查命令。
- Info: `.pipeline/reports/test-baseline.md` 的 `Worker Lifecycle` 小节仍写着 implement/audit 未运行，这与后续 `.pipeline/PROGRESS.md`、`.pipeline/log.yaml` 和 implement review 不一致。由于当前 audit 只允许写本文件，未修改报告主体；验收时应以本 audit 和 lifecycle 状态文件补齐该事实。
- Info: `cargo install --locked cargo-insta` 改变了用户级工具链状态，但未改仓库文件；这是 M1 允许安装缺失命令的合理副作用。

## 建议

- 接受 M1 作为“真实测试基线调查”产物。
- M2 开始前做决策：修复或显式豁免当前 11 个 `codex-tui` 失败；如果豁免，应在 M2 gate 中写明哪些失败不计入本轮回归。
- 将 snapshot pending 检查改为当前工具支持的命令，例如报告中已验证的 `cargo insta pending-snapshots`，或另行确认 `cargo-insta` 支持的包过滤写法。
- 后续主流程整合时，可更新生命周期摘要，避免 `test-baseline.md` 中“implement/audit 未运行”的旧状态造成误读。
