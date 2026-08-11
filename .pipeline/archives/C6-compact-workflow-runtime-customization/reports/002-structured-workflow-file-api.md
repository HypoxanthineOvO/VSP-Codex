# M2 Structured Workflow File API Report

时间：2026-05-29T16:08:40+08:00

## 结论

M2 已完成并通过复审。

新增 `codex_core::workflow_files` 作为小型结构化 Workflow 文件 API，用于项目本地 `.pipeline/` 文件的安全读写：

- `WorkflowFiles::open(project_root)`：检测并绑定项目本地 `.pipeline/`。
- `update_current_step`：解析 `.pipeline/state.yaml` 后进行 typed 当前步骤/heartbeat 更新；malformed YAML 不写入。
- `append_log_event`：向 `.pipeline/log.yaml` 的 `events` 或 `entries` 追加结构化事件。
- `append_progress_entry`：向 `.pipeline/PROGRESS.md` 追加时间线。
- `append_subagent_lifecycle_event`：追加 Subagent lifecycle 事件。
- `write_unprotected_yaml`：仅允许写项目本地 `.pipeline/` 下非 protected YAML，拒绝 `state.yaml`、`cycle.yaml`、`rules.yaml`、路径穿越、绝对路径逃逸和 symlink-parent 逃逸。

## 文件变更

- `codex-rs/core/src/workflow_files.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/Cargo.toml`
- `codex-rs/Cargo.lock`
- `codex-rs/core/tests/suite/workflow_files.rs`
- `codex-rs/core/tests/suite/mod.rs`

## Worker Evidence

- Test worker：`.pipeline/reviews/C6/M2/test.md`
- Implement worker：`.pipeline/reviews/C6/M2/implement.md`
- Audit worker：`.pipeline/reviews/C6/M2/audit.md`

第一轮 audit 发现 `write_unprotected_yaml` 可通过 `.pipeline` 内 symlink parent 逃逸。修复轮补充 Unix-only 回归测试 `pipeline_child_symlink_parent_escape_is_denied`，随后实现路径硬化并通过 audit recheck。

## 验证

- `cargo test -p codex-core workflow_files`：通过，8 passed。
- `cargo fmt --check`：通过；仅输出仓库既有 `imports_granularity = Item` nightly-only 警告。
- `git diff --check`：通过。

## Audit

最终 audit recheck verdict: `pass`。

M2 completion accepted。
