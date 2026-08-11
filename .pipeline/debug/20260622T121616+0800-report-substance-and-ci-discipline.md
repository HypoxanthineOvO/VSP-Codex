# Report Substance And CI Discipline

## 结论

本轮修复两个用户明确指出的体验/工程纪律问题：

- 完成回复不能再只列 `.pipeline/...` 产物路径、worker 关闭情况、YAML 解析和测试数量。模型可见的 Workflow runtime guidance、项目 `AGENTS.md`、Hypo-Workflow completion contract 和 OpenCode AGENTS 模板都补充了硬规则：完成前必须在对话里解释报告实质内容。对于 learning gate、quiz、research report、requirement brief、design report，必须直接讲清关键概念、报告/quiz 结构、用户应该理解什么、下一步该怎么做；路径只能作为解释后的参考。
- CI 纪律从“完成后修 CI”改成“测试变更同时更新 CI”。新增 `docs/ci.md`，把 GitHub Actions/GitLab CI 的当前职责、测试变更 checklist、完成前报告要求写清楚；`.gitlab-ci.yml` 的 `format` job 现在执行 `python3 scripts/check_ci_docs.py`，防止 CI 文档和实际 GitLab job 名称/规则继续漂移。

## 技术思路

- Codex-VSP runtime 层：更新 `codex-rs/core/src/workflow_routing.rs` 的 Hypo-Workflow 注入指导，把“display core report content”扩展为“explain the substance of referenced reports/reviews before listing artifact paths”，并明确禁止只枚举 `.pipeline/...` paths 和 validation bullets。
- 回归测试层：新增 core workflow routing 测试，直接断言 runtime guidance 包含 learning gates、quizzes、teach domain content、artifact paths are supporting references 等关键短语。
- Hypo-Workflow 源契约层：更新 `/home/heyx/Hypo-Workflow/references/completion-report-contract.md` 和 OpenCode AGENTS 模板，避免以后重新生成 adapter 时丢掉这条规则；对应 `core/test/completion-report-contract.test.js` 增加 focused contract tests。
- CI 文档层：新增 `docs/ci.md`，并在 `docs/contributing.md` 中把“测试变化必须同步 CI”提升为贡献步骤；新增 `scripts/check_ci_docs.py` 并接入 GitLab `format` job。

## 修改文件/模块

- `AGENTS.md`
- `.gitlab-ci.yml`
- `docs/ci.md`
- `docs/contributing.md`
- `scripts/check_ci_docs.py`
- `codex-rs/core/src/workflow_routing.rs`
- `codex-rs/core/tests/suite/workflow_routing.rs`
- `/home/heyx/Hypo-Workflow/references/completion-report-contract.md`
- `/home/heyx/Hypo-Workflow/core/test/completion-report-contract.test.js`
- `/home/heyx/Hypo-Workflow/plugins/opencode/templates/AGENTS.md`
- `/home/heyx/Hypo-Workflow/AGENTS.md`

## 测试设计

- 用 Codex core focused tests 证明模型可见 runtime guidance 已包含完成报告实质讲解规则。
- 用 Hypo-Workflow focused Node test 证明 completion contract 本身不允许 path-only final response，并要求 learning/research artifacts 在 chat 中做 domain explanation。
- 用 GitLab format job 的本地镜像命令验证 CI 文档检查能在 CI 里执行：Rust fmt、Python pytest、CI docs check。
- 用 `git diff --check` 和 YAML parse 验证工作树文本和 Workflow 状态文件没有格式错误。

## 验证结果

通过：

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-core workflow_completion_guidance_requires --test all`：2 passed。
- `python3 scripts/check_ci_docs.py`：通过。
- `cd sdk/python && uv run --no-project --with pytest --with pydantic --with tomli pytest tests/test_artifact_workflow_and_binaries.py::test_root_fmt_recipes_use_shared_formatter_driver`：1 passed。
- `cd codex-rs && cargo fmt -- --config imports_granularity=Item --check`：通过；仅输出 stable rustfmt 对 `imports_granularity=Item` 的既有 warning。
- `node --test core/test/completion-report-contract.test.js` in `/home/heyx/Hypo-Workflow`：5 passed。
- `git diff --check` in `/home/heyx/Codex-VSP`：通过。
- `git diff --check` in `/home/heyx/Hypo-Workflow`：通过。
- `.pipeline/state.yaml` / `.pipeline/log.yaml` YAML parse：通过。

额外观察：

- 曾误用 `cargo test ... --test workflow_routing`，失败原因是 `codex-core` 没有该 test target；正确目标是 `--test all`。
- 曾误用 `npm test --prefix /home/heyx/Hypo-Workflow/core -- completion-report-contract`，它实际触发 core 全量测试并额外传参，结果为 676 passed / 1 failed。失败项是既有 `current lifecycle log validates real event families and statuses`，与本次 completion contract focused tests 无关；本次相关 focused test 已单独通过。

## 预期结果

- 之后 Milestone/Cycle/Debug/Audit/Patch 完成时，模型应先在对话中解释报告内容，再给路径。像“M0 已完成，产物如下，验证如下，你自己看 quiz”这种回复不再满足运行时 guidance、AGENTS 和 Hypo-Workflow completion contract。
- 对 learning gate/quiz 场景，完成回复应直接讲清 quiz 检查什么、报告里的关键概念是什么、用户应该如何理解本阶段，而不是只说“主要看两份报告”。
- 新增或变更测试时，完成报告必须说明对应 CI job；如果 CI 没覆盖，应当同步修改 `.gitlab-ci.yml` 或 GitHub Actions，或明确记录 local-only 理由。

## 遇到的问题

- Hypo-Workflow 源仓当前有大量既有未提交变更。本轮只针对 completion contract、OpenCode AGENTS 模板和对应 focused test 做小范围同步，没有整理其他无关 dirty state。
- GitLab CI 全量没有实际推送触发；本地运行了新增 `format` job 的关键镜像命令和 CI docs check。

## 风险/后续

- 2026-06-22T12:25:24+08:00 已 rebuild/deploy 本机 standalone：`~/.vsp-codex/packages/standalone/current/bin/vsp-codex` SHA256 为 `33b19d3821023a0e954c24fcc47fcd88474f67631b157c7c4f9fee077e2b29b0`，旧 SHA `b444971afe4c040351e453b90af71cff92a370ac160115a02483b112f1c51aea` 备份为 `vsp-codex.backup-20260622T122524+0800`，`--version` 返回 `vsp-codex 0.140.0-vsp.0`。已有 VSCode/app-server 进程仍需要重载后才会使用新二进制。
- Hypo-Workflow 源仓的 OpenCode generated artifacts 已有大量既有变更；如果要发布 Hypo-Workflow 侧修复，需要在该仓单独做一次 sync/release 流程，避免把无关 dirty state 混入。
