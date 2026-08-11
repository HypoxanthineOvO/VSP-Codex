# C9 M6 Test Review - Deep Research Requirement Brief Coverage

## verdict

PASS。

`.pipeline/architecture/c9-upstream-base-upgrade-and-deep-research-brief.md` 满足 `.pipeline/prompts/c9-006-deep-research-workflow-requirement-brief.md` 要求的 Deep Research 简报覆盖面。架构简报明确说明 C9 只生成可迁移到 Hypo-Workflow 的需求简报，不在本 Cycle 做 Codex-VSP 原型或生产实现；non-goal 边界清楚，降低了把简报误读为实现交付的风险。

## reviewed_refs

- `.pipeline/prompts/c9-006-deep-research-workflow-requirement-brief.md`
- `.pipeline/architecture/c9-upstream-base-upgrade-and-deep-research-brief.md`

## commands

```bash
rg -n "Question Tool|Subagent|recursive|budget|follow-up|\\.pipeline/research|0\\.140\\.0-alpha\\.18" .pipeline/architecture/c9-upstream-base-upgrade-and-deep-research-brief.md
```

Observed matches:

```text
7:C9 主交付是把 VSP-Codex 从当前 `0.134.0-vsp.7.2` 基线升级到上游 Codex `rust-v0.140.0-alpha.18`，完成冲突处理、VSP 定制保留、验证、版本元数据更新和 GitLab release 发布。
19:- Subagent UX：内置 Subagent 命名、active Subagent panel、Prompt/Report viewer、worker separation 约定。
27:  A[Checkpoint current archive/planning state] --> B[Fetch upstream tags and selected tag rust-v0.140.0-alpha.18]
41:- 必须先使用 Question Tool 澄清用户需求，不能直接进入调研。
42:- 支持多个 Subagent 并行调研，并保留每个 Subagent 的原始报告。
44:- 支持预算与剪枝：候选默认最大深度 10、Subagent 上限 128，但必须有预算上限、重要性阈值/剪枝比例和停止条件。
46:- 支持 follow-up 和继续深入研究。
47:- 默认产物写入 `.pipeline/research/<slug>/`；允许调用者指定其他输出根目录，例如 Writer 项目目录。
51:建议 Hypo-Workflow 负责 portable command semantics 与 artifact contract；Codex-VSP 或其他平台负责具体 Subagent 调度、UI 面板和执行能力。
56:.pipeline/research/<slug>/
82:- 验证产物是简报本身：内容覆盖 Question Tool、Subagent fan-out、递归、预算/剪枝、进度控制、follow-up、产物布局和真实联网验证场景。
87:- `0.134.0` 到 `0.140.0-alpha.18` 是跨多版本 alpha 升级，可能带来协议、crate、release 包、Node/Bazel 和 app-server schema 冲突。
```

## coverage checklist

- [x] Problem statement / scope: 架构简报在“目标”中区分 C9 主交付和 Deep Research 次交付，并说明 Deep Research 只生成需求简报。
- [x] Command semantics: 简报建议由 Hypo-Workflow 负责 portable command semantics 与 artifact contract，平台侧负责 Subagent 调度、UI 面板和执行能力。
- [x] Question Tool flow: 明确要求必须先使用 Question Tool 澄清用户需求，不能直接进入调研。
- [x] Subagent fan-out: 明确支持多个 Subagent 并行调研，并保留每个 Subagent 的原始报告。
- [x] Recursion / follow-up: 明确支持递归探索，以及 follow-up 和继续深入研究。
- [x] Budget / pruning controls: 明确默认最大深度 10、Subagent 上限 128，并要求预算上限、重要性阈值/剪枝比例和停止条件。
- [x] Artifact contract: 明确默认写入 `.pipeline/research/<slug>/`，列出 `brief.yaml`、`progress.yaml`、`graph.yaml`、`raw/`、`sources.yaml`、`report.md`、`followups/` 等产物布局，并说明自定义输出路径应替代完整 research root。
- [x] UI expectations: 明确需要进度面板的手动控制，包括查看、暂停/停止、选择分支深入。
- [x] Validation scenario: 明确允许真实联网，验证场景为调研“当前大模型市场最新模型、性能和定价”，并要求来源证据。
- [x] Non-goal clarity: 明确“不在本 Cycle 做 Codex-VSP 原型或生产实现”，并在验证策略中再次说明“不把本 Cycle 伪装成已实现 Deep Research”。
- [x] Upstream version context: 明确 C9 上游目标 tag 为 `rust-v0.140.0-alpha.18`，并记录跨 alpha 升级风险。

## risks

- Deep Research 简报已经覆盖需求，但实现责任仍被拆分为 Hypo-Workflow portable semantics 与平台执行能力；后续实现阶段需要把这条责任边界转成具体接口和验收用例。
- 预算/剪枝要求已写明，但当前仍是需求层描述；后续实现需要定义预算单位、停止条件触发优先级、递归分支评分和 Subagent 上限冲突处理。
- 自定义输出根目录规则已说明为替代完整产物目录；后续实现需要测试 `.pipeline/research/<slug>/` 和外部目录两种路径，避免只重定向最终报告。
- 真实联网验证场景涉及最新模型、性能和定价，后续测试必须固定来源证据和采集日期，避免结果随时间变化导致不可复现。
