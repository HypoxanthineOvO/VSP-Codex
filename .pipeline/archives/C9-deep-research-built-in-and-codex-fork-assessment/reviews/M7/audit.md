# C9 M7 Audit - Conflict Inventory And Equivalence Matrix

审计时间：2026-06-26T15:48:00+08:00

## verdict

有条件通过。

M7 冲突清单和 VSP 等价矩阵整体满足本 milestone 的只读规划目标：没有声称已经完成 `rust-v0.143.0-alpha.25` merge，也没有声称已经发布 release；`replace / preserve / defer` 分类总体保守，高风险 VSP 领域未被错误列为可直接替换；SQLite 官方修复覆盖表述与本地 git 证据一致，尤其正确区分了 `63f8f547c0` 只在 alpha.25 prerelease 覆盖、不在 `rust-v0.142.2` stable 覆盖。

本审计不阻断 M8，但要求 M8-M12 保持矩阵中的验证边界：SQLite 只能在等价性确认后替换，协议、Question Tool、Hypo-Workflow、TUI、VSCode Remote、release tooling、plugins/adapters 不能因 alpha.25 文件存在而直接覆盖 VSP 行为。

## reviewed_refs

- `.pipeline/prompts/c9-007-conflict-inventory-and-upstream-fix-baseline.md`
- `.pipeline/reports/C9/M7-upstream-alpha25-conflict-inventory.md`
- `.pipeline/reports/C9/M7-vsp-equivalence-matrix.yaml`
- `.plan-state/c9-upstream-alpha25-architecture.yaml`
- `.pipeline/debug/20260626T134037+0800-sqlite-log-write-amplification.md`

辅助只读核验：

- `git rev-parse rust-v0.143.0-alpha.25 rust-v0.142.2 HEAD`
- `git merge-base --is-ancestor <commit> rust-v0.143.0-alpha.25`
- `git merge-base --is-ancestor <commit> rust-v0.142.2`
- `git show --stat --oneline d81fcdf8ef e98d43ac37 91e6da943b 63f8f547c0`
- `git diff --name-status rust-v0.143.0-alpha.25...HEAD -- . ':!.pipeline' ':!.plan-state'`
- `rg` over M7 prompt/report/matrix for merge, release, replace, preserve, defer, SQLite, Question Tool, Hypo-Workflow, TUI, VSCode, release tooling, plugins

## findings

### Info - M7 没有错误声称完成 alpha.25 merge 或 release

审计对象反复把 M7 定义为 read-only planning artifact，并明确 non-goals：不 merge product source、不改 package/public version metadata、不刷新 release artifacts、不 push/tag/publish/create release。报告中提到当前 VSP 分支包含本地 release/hotfix commits，是历史基线描述；没有把 M7 输出表述为已经完成 alpha.25 merge 或新的 release。

### Info - SQLite 官方修复覆盖表述准确

本地 git 祖先关系核验结果：

- `d81fcdf8ef`：在 `rust-v0.143.0-alpha.25`，也在 `rust-v0.142.2`
- `e98d43ac37`：在 `rust-v0.143.0-alpha.25`，也在 `rust-v0.142.2`
- `91e6da943b`：在 `rust-v0.143.0-alpha.25`，也在 `rust-v0.142.2`
- `63f8f547c0`：在 `rust-v0.143.0-alpha.25`，不在 `rust-v0.142.2`

这与冲突清单的表格和 debug 报告一致。`63f8f547c0` 对 `target='log'` 的覆盖被正确标为 alpha.25 prerelease 特有能力，且与本地 debug 发现的 dominant retained target 对齐。

### Info - replace 分类保持在 SQLite 范围内，动作表达有条件

矩阵中 `replace` 仅包含 SQLite persistent log filtering 以及 Responses WebSocket/session telemetry noise reduction 两类，并且动作均使用条件式表述：M8 中在行为等价时优先采用官方 alpha.25 实现，必要时保留 VSP in-layer guard。该分类有 upstream commit、文件路径和 debug 证据支撑，未把协议、TUI、CLI、插件或 release tooling 误列入直接替换。

### Info - 高风险 VSP 领域被正确 preserve/defer

以下高风险领域均未被标为可直接替换：

- Question Tool / `request_user_input`：标为 preserve，并要求 M9 先保留语义再协调 schema/request processor。
- Hypo-Workflow routing/state-file contract、goals、plan freshness：标为 preserve，并要求 M10 才能在有 direct equivalent 和测试时调整。
- TUI alpha.25 broad UX churn：标为 defer 到 M11，理由包含 250 个 TUI diff 文件与 snapshot churn 风险。
- VSCode Remote file-link behavior 和 VSP CLI entrypoint/package identity：标为 preserve，要求 M11 验证 alpha.25 code-mode/host IPC 兼容性。
- GitLab release/VSP package tooling：标为 preserve 到 M12 release-candidate dry run。
- plugins/adapters、core-plugins/core-skills、marketplace/connector/skill-loader：标为 preserve/defer，未接受 upstream removals 或 connector policy 作为直接替换依据。

### Info - M8-M12 后续边界清楚

冲突清单的 `Next-Milestone Implications` 和矩阵的 `next_milestone` 字段一致：

- M8：SQLite logging 收敛和 runtime SQLite I/O smoke。
- M9：protocol/app-server contracts 与 regenerated schemas。
- M10：core workflow/runtime、Hypo-Workflow、plugins/adapter 相关基础语义。
- M11：TUI、CLI、VSCode、用户可见 workflow surfaces。
- M12：release-candidate dry run、docs/CI/package/install/version/release artifacts，并需要 explicit release confirmation gate。

## risks

- SQLite 仍有残余等价性风险：alpha.25 官方实现可作为 M8 优先来源，但 VSP 本地 in-layer `target == "log"` guard 只有在 targeted tests 和 runtime smoke 证明可移除时才能删除。
- `replace` 项中的 telemetry noise reduction 依赖 upstream 行为引入；如果 VSP 后续有本地 telemetry 假设，M8 需要确认不会降低诊断能力。
- 协议和 generated schemas 的风险仍高：M9 必须从 Rust protocol 合并后的真实合同重新生成 schema，不能手工覆盖 JSON/TypeScript aggregate 文件。
- TUI 和 snapshots 风险仍高：M11 若在 protocol/core/app-server 未稳定前合并 TUI，会把行为回归隐藏在大量 snapshot churn 里。
- release tooling 风险仍高：M12 前不得把 upstream GitHub/zsh/Bazel/release helper 当作 VSP GitLab release flow 的等价替代。

## required follow-up

- M8 必须把 SQLite 替换限定为 alpha.25 官方实现与 VSP hotfix 的等价收敛，并运行 `codex-state` filter tests 与隔离 `CODEX_HOME`/`CODEX_SQLITE_HOME` 的 SQLite I/O smoke；记录是否保留 VSP in-layer guard。
- M9 必须先合并/确认 Rust protocol 语义，再重新生成 schemas，并覆盖 Question Tool、workflow markdown、app-server client/v2 suite 兼容性测试。
- M10 必须 preserve-by-default 处理 Hypo-Workflow routing、state-file contract、goals、plan freshness、worker separation、plugins/adapters；只有 direct equivalent 和匹配测试同时成立时才可替换。
- M11 必须把 TUI、CLI、VSCode Remote 作为用户可见行为验证，而不是仅 snapshot 更新；重点验证 Ask cards、plan/release authorization cards、slash routing、remote file links、VSP entrypoint。
- M12 必须在 explicit release confirmation gate 后才处理 version/package/release artifact/GitLab release dry run；M7 报告不得被当作 release approval。
