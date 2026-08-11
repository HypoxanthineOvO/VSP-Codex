# S2 通用 Developer 运行提示审阅结果

## 结论

S2 已完成 VSP-Codex 通用运行提示的来源拆分、逐段裁决和连续候选。候选覆盖默认 bypass 权限、Default collaboration mode 与 Multi-Agent V2，删除重复工具手册和相互冲突的行为规则，并新增用户要求的主动委派、root 持续控制和成本感知 worker routing。

Hypo-Workflow Cycle 状态 D4 已确认属于外部插件动态注入。它继续保留在全量 provenance 盘点中，但不进入 VSP-Codex 通用提示候选或内容改写。

S2 已具备用户验收条件。接受只锁定当前设计并进入 S3；S5 接受前仍禁止修改产品源码。

## 技术方法

- 从当前真实首轮 developer 消息拆出 D1 Permissions/Default、D2 Multi-Agent V2 root/shared usage、D3 multi-agent mode 和 D4 Workflow session state。
- 对照仓库模板、session 拼装、context update 和工具 schema，确认每段的物理来源与动态更新条件。
- 对照 VSP TUI 默认权限实现，确认无显式配置时使用 `danger-full-access + approval_policy=never`，显式配置仍可覆盖。
- 对照 Multi-Agent V2 工具 schema，确认底层支持 model/reasoning override，但默认 `hide_spawn_agent_metadata=true` 导致当前模型不可配置。
- 检查本机 `worker-routing.toml`，确认已有候选映射但尚未接入当前 `spawn_agent` 可见接口。
- 根据用户裁决生成纯英文连续候选，并对字符、否定短语、必要合同、Markdown、YAML 和 diff hygiene 做静态验证。

## 主要变化

### 权限

- VSP 默认 bypass 场景不再展开通用沙箱教程，只陈述文件系统/网络不受限、无需 sandbox approval 和不得传 `sandbox_permissions`。
- 明确运行时能力不会扩大用户请求授予的任务范围或授权。
- 用户显式覆盖权限时仍由动态 renderer 反映真实生效配置。

### Default mode

- 只保留当前 mode、developer 指令切换权威和对通用执行/授权规则的引用。
- 删除已知 mode 枚举、重复提问标准和禁止文本多选题的绝对规则。
- 确实需要用户输入时优先使用 VSP 增强的 `request_user_input`；工具不可用时清晰文本 fallback。

### Multi-Agent V2

- D2 与 D3 合并；从 `explicit-request-only` 改为 root 可在独立并行、重复机械、调研和互斥实现/验证中主动委派。
- root 始终拥有总体计划、项目方向、关键路径、成本、进度、整合、验证、用户更新和最终结果。
- 子任务必须具体、有边界且并发写范围互斥；root 监督并审查产出。
- 子智能体默认不得继续派生；root 可为具体有边界的需要显式授权嵌套。
- 使用 `mechanical / long_context / standard / explore / critical / escalation` 语义档位，默认选择可靠完成任务的最低成本档位。
- 显式 model/reasoning override 只在用户要求或语义档位不足时使用。

## 产品实现要求

S2 不只产生提示文本，还增加以下 S5/M2 产品要求：

- `spawn_agent` 暴露语义 `routing_class`。
- 可信 worker-routing 配置把语义档位确定性解析为 provider/model/reasoning。
- 支持独立 `long_context` 档位，不把长上下文机械任务混入开放探索。
- 显式 model/reasoning override 受策略约束并与 routing class 冲突检查。
- Spawn 结果、调试输出和 telemetry 记录 requested class、有效模型、reasoning、fallback/escalation 结果。
- collaboration 工具可见时注入合并短提示；工具不可见时不注入。

## 修改的 Workflow 文件

- `PLAN.md`：区分全量注入盘点与内容改写，并加入 worker routing 实现范围。
- `S2-DEVELOPER-RUNTIME-REVIEW.md`：完整原文、翻译、点评、用户决定和 D4 来源纠正。
- `S2-DEVELOPER-RUNTIME-CANDIDATE.md`：纯英文连续候选。
- `PROGRESS.md`、`EXECUTION.md`、`DISCUSSION-SUMMARY.md` 和 `.pipeline/log.yaml`：决定、证据、状态与下一步。

没有修改产品源码、配置、测试、远端服务、发布或安装内容。

## 验证设计与结果

| 验证 | 结果 |
| --- | --- |
| 原 D1+D2+D3 | 3,360 chars |
| S2 candidate | 2,104 chars、21 lines |
| 字符降幅 | 37.4% |
| 已删除冲突/冗余 | `explicit-request-only`、能力相同宣言、消息 envelope、mode 枚举、文本多选绝对禁令、`all commands are permitted` |
| 必要合同 | 权限边界、Ask 优先、mode 覆盖、root ownership、routing class、`long_context`、受控嵌套、共享工作树、direct-call 均存在 |
| Workflow YAML | 解析通过 |
| Diff hygiene | `git diff --check` 通过 |

本 Stone 没有产品行为变化，因此未运行 Rust test group。M2 实现后需要覆盖权限 renderer、mode update、tool exposure、routing resolver、override 冲突、fallback、受控嵌套和真实 spawn metadata。

## 预期行为

- 默认 VSP 会话不再向模型解释无用的沙箱概念或暗示技术能力等于用户授权。
- 需要用户输入时优先使用结构化 Request，同时保留可靠文本 fallback。
- root 主动利用多智能体提高重复、多路和调研任务效率，但始终掌握项目方向和交付质量。
- 机械与长上下文机械工作不再默认消耗最强模型；强模型只用于确有需要的复杂或高影响任务。
- 外部 Workflow 业务提示不会混入 VSP 通用系统提示候选，普通会话只接受与实际启用能力相关的注入。

## 遇到的问题

- 最初把 Permissions 与 Default mode 作为一个 D1 单元，掩盖了两个独立动态来源；已拆为 D1a/D1b。
- 最初建议首次 spawn 后才注入 D2，但模型选择必须发生在第一次 spawn 之前；已改为 collaboration 工具可见时注入短 preflight。
- 当前 V2 隐藏模型配置字段，单靠提示无法完成成本路由；已把语义 routing interface 纳入 M2。
- 最初把外部 Hypo-Workflow D4 当作通用系统提示审阅；用户纠正后撤回，建立“全量盘点、按所有者决定是否改写”的边界。

## 风险与后续

- routing class 的配置 schema、fallback、模型可用性和跨 provider 兼容性仍需在 S5 给出完整集成设计。
- prompt 只能指导 root，成本控制还需要 resolver 与工具参数的机械约束，不能仅依赖模型自律。
- proactive delegation 会增加总调用量；M3/S6 必须同时验证延迟、token 成本、质量和并发写冲突，而不只验证功能可用。
- 外部插件注入隔离将在 S4 审查；外部插件自身业务文案不属于 C28 改写范围。

## 用户接受

用户于 2026-08-10 21:14（Asia/Shanghai）接受 S2。

接受范围为 2,104 字符完整候选、37.4% 字符降幅、外部 Hypo-Workflow D4 来源边界，以及 proactive multi-agent、root ownership、受控嵌套和成本感知 worker routing 产品设计。接受允许 C28 进入 S3；不授权修改产品源码、配置、远端服务、发布或安装。
