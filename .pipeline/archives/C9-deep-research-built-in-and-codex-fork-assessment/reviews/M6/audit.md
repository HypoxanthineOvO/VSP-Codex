# C9 M6 Audit - Deep Research Workflow Requirement Brief

审计时间：2026-06-26T15:00:00+08:00
审计角色：audit worker
审计范围：只读审计 Deep Research requirement brief 与其架构来源；未编辑源码、测试、prompt、state、cycle、log 或 PROGRESS。

## Verdict

通过，带后续跟进项。

当前 brief 对 C9 的 scope boundary 是诚实的：architecture brief 和 M6 prompt 都明确说明 C9 只产出可迁移到 Hypo-Workflow 的需求简报，不在本 Cycle 做 Codex-VSP 原型或生产实现。它也覆盖了 Question Tool、Subagent fan-out、递归探索、预算/剪枝、进度控制、follow-up、artifact layout、真实联网验证场景和 non-goal 等核心要求，足够作为后续 Hypo-Workflow 设计/实现的起点。

主要风险不是“误称已实现”，而是“actionability 仍停留在产品简报层”：它列出能力和建议产物，但尚未把命令入口、状态机、预算字段、Subagent 调度 contract、人工控制语义、失败/恢复规则、验收测试矩阵定义到可直接实现的程度。因此后续实现前应补一版 implementation-ready spec 或 machine-readable transfer。

## Reviewed Refs

- `.pipeline/architecture/c9-upstream-base-upgrade-and-deep-research-brief.md`
- `.pipeline/prompts/c9-006-deep-research-workflow-requirement-brief.md`

## Strengths

- Scope boundary 明确：architecture brief 写明 Deep Research 是 C9 次交付，目标是生成可迁移到 Hypo-Workflow 的需求简报，并明确“不在本 Cycle 做 Codex-VSP 原型或生产实现”；M6 prompt 也重复了“Do not implement Codex-VSP Deep Research in C9”。
- 非目标风险被正面处理：architecture brief 的验证策略明确“验证产物是简报本身”和“不把本 Cycle 伪装成已实现 Deep Research”；M6 prompt 的风险段也指出 brief 可能被误认为 implementation。
- 需求覆盖面较完整：Question Tool 澄清、多个 Subagent 并行调研、递归探索、预算/剪枝、进度面板手动控制、follow-up、默认与自定义输出路径、原始报告保存、最终报告、真实联网验证场景均已被列入。
- 平台职责边界合理：brief 建议 Hypo-Workflow 负责 portable command semantics 与 artifact contract，Codex-VSP 或其他平台负责具体 Subagent 调度、UI 面板和执行能力，避免把平台特定实现过早绑死。
- artifact layout 有初步 contract：`.pipeline/research/<slug>/` 下的 `brief.yaml`、`progress.yaml`、`graph.yaml`、`raw/`、`sources.yaml`、`report.md`、`followups/` 结构为后续实现提供了可迁移的文件组织基础。

## Issues / Risks

### Warning: Brief 尚未达到 implementation-ready 的 actionability

证据：architecture brief 只列出 Deep Research 可移植需求和建议产物布局；M6 prompt 的 Technical Solution 要求覆盖 “problem statement, command semantics, Question Tool flow, Subagent fan-out, recursion/budget controls, artifact contract, UI expectations, validation scenario, and non-goals”，但当前被审计的 architecture brief 尚未展开命令语法、状态机、字段 schema、错误处理、恢复路径或验收矩阵。

影响：后续 implement worker 可能需要自行解释“预算上限”“重要性阈值/剪枝比例”“停止条件”“暂停/停止”“选择分支深入”等语义，导致实现差异、验收歧义或平台间不可迁移。

建议：在进入真正实现前补充 implementation-ready spec，至少定义：

- command surface，例如 start/status/pause/resume/stop/follow-up/deepen 的输入输出。
- Question Tool gate 的必填问题、跳过条件和确认记录。
- `brief.yaml`、`progress.yaml`、`graph.yaml`、`sources.yaml` 的字段 schema 与兼容策略。
- Subagent 生命周期、原始报告命名规则、失败重试、部分成功和取消语义。
- 递归预算计算、默认值、硬上限、剪枝算法输入输出和终止条件。
- UI/CLI 进度面板必须暴露的操作与状态。
- 验收测试矩阵，覆盖无网络、联网、预算耗尽、用户暂停、follow-up 和自定义输出根目录。

### Warning: “Question Tool” 与本地工具命名可能引发平台解释歧义

证据：architecture brief 使用 “Question Tool” 表述；当前 workspace 的适配说明提到 OpenCode 使用 `question`，Codex runtime 中对应用户决策工具是 `request_user_input`。

影响：后续跨平台实现时，worker 可能把 Question Tool 理解为某个单一平台工具，而不是“必须先进行结构化用户澄清/确认”的 portable semantic。

建议：后续 brief 应把 Question Tool 改写为 portable semantic，并列出平台映射：OpenCode `question`、Codex `request_user_input`、其他平台的等价交互能力。

### Info: 自定义输出根目录语义已指出，但仍需路径安全与归属边界

证据：architecture brief 说明“允许调用者指定其他输出根目录，例如 Writer 项目目录”，并强调自定义输出路径应替代完整 `<research-root>`。

影响：实现时如果未定义路径校验，可能产生跨项目写入、覆盖既有研究目录、或把敏感来源/原始报告写到非预期目录的问题。

建议：后续 spec 定义输出根目录的解析、创建、覆盖策略、相对/绝对路径规则、目录存在时的冲突处理和敏感信息脱敏要求。

### Info: 不易被误解为已实现功能，但最终报告/README 引用时仍需保留 non-goal 标签

证据：两个被审计文件都明确 C9 不实现 Deep Research；风险段也直接点名“brief may be mistaken for implementation”。

影响：如果后续 C9 summary 只写“Deep Research brief completed”而不标注 design-only，外部读者可能仍误判为 runtime 已可用。

建议：C9 report/summary、release notes 或 handoff 文档中统一使用 “Deep Research requirement brief/design-only” 表述，避免使用 “Deep Research workflow completed/available/enabled” 这类容易被理解为功能已上线的措辞。

## Required Follow-Up

1. M6 完成记录必须明确标注：交付物是 requirement brief，不是 Codex-VSP 或 Hypo-Workflow 的 Deep Research production implementation。
2. 若进入后续实现 Milestone，先补一份 implementation-ready spec 或 `.pipeline/plan-state/deep-research-brief.yaml`，把命令语义、artifact schema、状态机、预算/剪枝规则、Subagent lifecycle、失败/恢复行为和验收矩阵结构化。
3. 在后续 summary/release surface 中避免把 brief completion 写成 feature availability；建议固定用词为“Deep Research requirement brief prepared for later Hypo-Workflow implementation”。
4. 后续实现前增加路径安全和数据保留规则，特别是自定义输出根目录、raw Subagent reports、sources evidence、联网调研记录和敏感信息脱敏。

## Validation

- 审计方法：只读审阅指定 architecture brief 和 M6 prompt，对照用户要求检查 actionability、scope boundary 和误解风险。
- 代码/测试修改：无。
- Workflow 状态修改：无。
- 结论：未发现阻断 M6 brief 验收的问题；存在需要后续实现前补齐的 Warning 级 actionability 风险。
