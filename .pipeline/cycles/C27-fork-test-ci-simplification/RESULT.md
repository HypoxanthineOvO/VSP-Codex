---
kind: goal-result
cycle: C27-fork-test-ci-simplification
version: 2
status: accepted
updated: 2026-08-09T21:30:08+08:00
---

# C27 测试与 CI 精简结果 v2

## 结论

VSP 现在有两层明确的测试合同：GitLab required CI只证明current `vsp-codex`能构建并完成最小CLI/TUI/app-server交互；本地测试则按14个行为用途分组，由AI选择与改动相关的最小组并对结果负责。

全量上游suite不再是AI的默认兜底。它必须有明确reason才能启动；AI主动运行任何组后，必须修复本次改动造成的失败，或用最小复现证明真实阻塞。原始失败清单不能作为完成结果交给用户。

## 本地测试分组

| Group | 保护的合同 | 执行策略 |
| --- | --- | --- |
| `smoke` | current binary build、CLI、TUI首帧、app-server握手 | required CI；交付Rust产品/CI/package改动前运行 |
| `cli` | CLI parsing、doctor、update、top-level routing | 必须提供test-name filter |
| `core` | request、tool、config、goal、conversation、host integration | 必须提供test-name filter |
| `app-server` | public JSON-RPC implementation | 必须提供test-name filter |
| `exec` | headless events、final output、JSONL、resume | 必须提供test-name filter |
| `protocol` | Rust/TypeScript/JSON wire shape和schema fixtures | 小而完整的contract crate |
| `tui` | rendering、input、streaming、replay、routing | 必须提供filter；pending snapshot阻止通过 |
| `persistence` | rollout durability、session compatibility、resume | rollout全套+VSP compatibility |
| `release` | package、installer、compatibility、preview assembly | 不发布、不安装的release check |
| `sdk-typescript` | SDK process、stream、abort、options | 必须提供Jest test-file filter |
| `security` | apply-patch parser与filesystem mutation boundary | 完整小型security-sensitive suite |
| `v8-runtime` | 真实V8 init/eval、CRDTP、sandbox linkage | 真实runtime行为，不测静态字符串 |
| `hypo-external` | cross-repository Workflow manifest/fixture compatibility | 必须设置`HYPO_WORKFLOW_ROOT` |
| `upstream-all` | inherited workspace-wide diagnosis | 只允许明确用户要求、shared harness或全仓机械改动；必须提供reason |

机器可读定义位于`scripts/vsp_test_groups.json`，每组包含purpose、value、useWhen、prerequisites、ciBlocking和argv commands。

## AI 执行合同

- 根`AGENTS.md`要求AI测试前先读取manifest或运行`just test-groups`，选择能保护本次行为的最小组。
- `just test-group explain <group>`先显示用途、价值、前置条件和失败策略。
- `just test-group run <group>`执行统一runner；runner不使用shell字符串，先打印合同再启动命令。
- `cli/core/app-server/exec/tui/sdk-typescript`没有filter会拒绝。
- focused Rust groups使用nextest `--no-tests=fail`；不存在的test-name不是0-test假绿，而是exit 4失败。
- `hypo-external`缺env拒绝；`upstream-all`缺reason拒绝；`just test`无selection拒绝。
- AI必须修复与当前改动有关的失败并重跑；无关失败必须用最小复现证明，仅在阻止当前结果时作为blocker。不得汇报一份未处理的全量失败清单并结束任务。

## Blocking CI

- GitLab保持唯一`vsp-smoke` job：locked build current `vsp-codex`，然后运行CLI/TUI/app-server smoke。
- 增加10分钟硬timeout；最近成功pipeline约4分12秒。
- 新增`just vsp-ci`复现同一合同。
- GitHub 29 workflows/94 jobs和Bazel保留为upstream inherited/manual；它们在GitLab耗时为0，不作为VSP CI状态报告。

CI绿色只表示“能构建、能启动、最小交互可用”，不表示产品没有功能bug。

## 测试组合精简

- 删除21个生产者已移除的TUI orphan snapshots。
- 删除1个与完整字符串assertion重复的doctor snapshot及其assertion。
- 删除只测试静态Bazel label的V8函数/测试。
- 删除只在`cfg(test)`存在、生产不使用的旧SemVer helper及测试。
- 4个读取external Hypo-Workflow fixture/manifest的合同改为manual；3个只检查本仓源码的快速合同继续默认执行。
- 修正TypeScript aborted stream测试，要求iterator自身reject；修正SDK默认binary为current `vsp-codex`。

## 修改文件与模块

- 分组与AI：`scripts/vsp_test_groups.json`、`scripts/vsp_test_groups.py`、`AGENTS.md`。
- CI与入口：`.gitlab-ci.yml`、`justfile`、`docs/ci.md`。
- 测试代码：CLI doctor output/updates、core Hypo host contracts、V8 POC、TypeScript abort/helper。
- Snapshot：C27精确删除22个；没有接受或更新活跃snapshot。
- Workflow：C27 plan/progress/execution/discussion/result、Cycle索引、Session focus和根log。

未修改C26产品实现，未commit、push、发布、安装、部署或重启。

## 验证结果

### 分组实际运行

- `smoke`：current build与CLI/TUI/app-server smoke通过。
- `cli`：doctor output 16/16。
- `core`：本仓Hypo host contracts 3/3；4 external明确不进入default。
- `app-server`：request-user-input JSON-RPC round trip 1/1。
- `exec`：turn-completed final-output合同4/4。
- `protocol`：273/273，包括schema fixtures。
- `tui`：heading合同1/1，其余3,194项未带入；pending snapshot为0。
- `persistence`：rollout 107/107，session compatibility 7/7。
- `release`：package/install/session/preview 46/46。
- `sdk-typescript`：abort 5/5。
- `security`：apply-patch 86/86。
- `v8-runtime`：真实V8行为5/5。
- `hypo-external`：使用真实sibling checkout，4/4。
- `upstream-all`：只验证带reason的dry-run，没有启动全仓。

### 防误用与静态检查

- 空focused filter、缺external env、缺full-suite reason、无匹配test-name均非零失败。
- `just test`无selection立即exit 2；`just test-all`无reason立即exit 2。
- Manifest validator、Python compile、Ruff check/format、Prettier、Rustfmt、Justfile format通过。
- GitLab CI lint、YAML/front matter、`git diff --check`通过。
- pending `.snap.new`：0。
- v1已有产物验证继续有效：临时GNU package metadata/entrypoint正确，package binary与source SHA256一致，包内smoke通过。

## 预期行为

AI以后应按以下顺序测试：识别本次改动保护的行为 -> `test-groups`/manifest选最小组 -> 必要时`explain` -> 运行 -> 修复相关失败并重跑 -> 最终只报告所选组及其证明的合同。

普通提交不会自动启动全仓测试；完整上游诊断只有在范围本身要求时才运行。用户不会再收到“全量跑出一堆旧错但没人处理”的普通交付报告。

## 遇到的问题

- v1只提供focused命令，没有AI用途目录和失败责任，用户拒绝；v2新增manifest、runner、Just与AGENTS四层约束。
- `cargo test <filter>`在0匹配时可能返回成功；focused Rust groups改用nextest `--no-tests=fail`并验证exit 4。
- Python格式工具不在system interpreter；使用repo声明的`uv --project scripts`环境完成Ruff验证，没有安装system dependency。
- `AGENTS.md`由Hypo-Workflow管理，但当前`hypo-workflow` CLI不可用且`.pipeline/rules.yaml`受保护；本轮直接写入当前AI适配器，同时把同一核心纪律放进manifest、runner错误消息和Just gates，避免只依赖生成文件。
- TypeScript本地Node 20低于workspace声明的Node 22，但SDK目标为Node 18且选定测试通过。

## 剩余风险

- 14组是当前fork的高价值合同目录，不是对未来所有模块的永久穷举。新增重要行为时必须扩展已有组或增加新组，不能退回无目的full suite。
- `smoke`的TUI probe依赖Unix PTY；Windows/macOS/Wine/remote/provider等平台或外部场景按对应前置条件手工验证。
- Python SDK仍验证固定上游runtime，不验证current VSP checkout，因此没有伪装成VSP分组。
- 完整TUI、平台、remote/provider、性能和upstream workspace suite不阻塞普通提交；相关任务必须选择对应focused/manual验证，而不能把CI绿色解释成全面正确。
