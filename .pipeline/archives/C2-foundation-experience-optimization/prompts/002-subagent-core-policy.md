# M2 Subagent 核心策略

## Goal
解除 Subagent 使用上的产品限制：不再要求用户明确授权才可使用，默认鼓励使用；最大数量调整为 32；Subagent 展示名从 VSP 中文名字池中选择。保留 `Subagent` 这个术语，不改成别的概念。

## Subworker Assignment Plan
Subworkers are authorized for this milestone. Worker separation is strict.

- `test`: 补核心策略测试，覆盖默认授权、最大数量 32、中文名字池选择、监督者/通用/简单任务的名字分类、模型和 reasoning effort 元数据保留。测试 worker 不实现策略代码。
- `implement`: 修改 Subagent policy/config/metadata 生成路径，负责默认可用、数量上限和中文名字池。实现 worker 不改测试 worker 的断言目标。
- `audit`: 审核是否仍存在显式授权 gate、硬编码低数量上限、英文 fallback 名称泄漏、或因中文名称导致宽度/序列化问题。
- Main agent: 整合 worker 结果，处理冲突，运行 scoped tests，记录证据。

## Technical Solution
把 Subagent 能力改为 VSP Codex 默认可用能力。底层仍可保留安全开关或内部配置，但用户在正常 TUI 使用中不应再被“必须明确授权”阻断。将默认并发/总量限制调整到 32，并把展示名生成逻辑改为从用户指定中文池中选择。

名字池：

- 监督者/重要实现者：李萌、杨超。
- 通用设计/测试/实现：娄鑫、朱昊骋、李超凡、贺云翔、吴浩宇、丁麒涵、李宇轩、李铭杰、闫启航、张雍治、蒋烨。
- 简单任务：陈泉宇、王珺、沈博宇。

Subagent 状态元数据需要为 M3 UI 暴露：显示名、model、reasoning effort、role、status、elapsed time、task summary、agent id。

## Technical Route
1. 搜索 Subagent 授权 gate、限制常量、spawn policy、agent display name 生成点。
2. 增加单元/集成测试，证明无显式授权时也可以 spawn，数量上限为 32，名字来自中文池。
3. 将默认授权策略放在 VSP 配置或 agent policy 初始化层，避免每个调用点重复判断。
4. 增加/调整名字池选择器，按 role hint 优先选择监督者/简单任务/通用名字；不能选到空名。
5. 确保 spawn_agent 可继续指定 model/reasoning effort，现有 API 能透传或记录这些字段。
6. 为 M3 提供稳定状态结构或事件，不强行在 M2 完成 UI 卡片。

## Research Required
不需要外部 research。需要本地代码探索：

- Subagent 是否是 TUI 内部功能、tool wrapper、还是 agent runtime 事件。
- 当前 model/reasoning effort 在 subagent 创建时的参数流。
- 当前授权限制是系统 prompt、tool schema、runtime policy、还是 TUI 层提示。

## Risks And Alternatives
- 风险：授权限制可能部分来自宿主工具协议，不能完全由 repo 代码移除。替代方案是 VSP runtime 默认注入授权配置，并隐藏用户侧阻断文案。
- 风险：最大 32 可能带来资源压力。替代方案是配置默认 32，但保留内部队列/背压，不让 UI 冻结。
- 风险：中文名字在日志、JSON、snapshot 中引发宽度或编码问题。替代方案是内部 ID 仍使用 UUID，中文只作为 display name。

## Validation Path
- `just fmt` from `codex-rs` after Rust edits.
- Run scoped tests for the crate(s) owning Subagent policy.
- If TUI snapshots are touched by name rendering, run `just test -p codex-tui` and accept intended snapshots.
- Manual smoke: ask TUI to delegate without prior explicit authorization and confirm Subagent starts with a Chinese display name.

## Audit Focus
- No remaining user-visible gate requiring explicit Subagent authorization.
- Limit is 32 in the effective policy, not just UI text.
- Model and reasoning effort can be specified or at least displayed when supplied.
- Chinese names are display metadata; durable internal identity remains stable.

## Deliverables
- Tests for default-enabled Subagent policy and name pool.
- Product changes for policy, limit, and display metadata.
- Milestone report documenting whether model/reasoning effort specification is already fully supported or needs follow-up.
