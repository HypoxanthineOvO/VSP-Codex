---
kind: plan
mode: plan
cycle: C28-prompt-stack-quality-redesign
status: closed
updated: 2026-08-11T19:10:44+08:00
builds_on:
  - C6-compact-workflow-runtime-customization
  - C27-fork-test-ci-simplification
progress: PROGRESS.md
execution: EXECUTION.md
source_edits: authorized_by_S5_acceptance
---

# C28 提示词栈质量重构

## 目的

完整盘点当前模型可见提示的内容、角色、来源和注入条件；对 VSP-Codex 基础提示、内置插件提示及操作运行提示形成中英对照、质量点评和用户裁决。确认归属于外部插件的业务提示只审计注入边界，不进入核心改写。统一目标提示稿获得接受后，实施客户端重构，并用真实 prompt 与 token 数据验收。

## 执行边界

- 来源盘点包含：基础提示与人格、通用 developer 规则、多智能体、Skills、Apps、Plugins、AGENTS、环境和 Workflow/Cycle 状态注入。
- 内容改写包含：VSP-Codex 基础提示、随包 System Skills、内置 Apps/Plugins 通用提示，以及权限、mode、多智能体、工具与其他操作运行提示。
- Repo/User/外部 Plugin Skills 按正常机制注入，不改写其业务正文；只做来源归因和是否仍有用的轻量筛选。
- 外部插件拥有的其他业务提示在确认来源和注入条件后移出内容改写；仍审查是否错误常驻、重复注入、角色错误或污染普通会话。
- 包含：仓库拥有的模型 fallback/template、context fragment、session 拼装、skills 渲染、Workflow routing、提示调试与相关测试。
- 包含：Multi-Agent V2 的成本感知 worker routing，使 root 能按任务难度、上下文长度和风险选择语义档位，并由运行时解析为适合的模型与 reasoning 配置。
- 远端 provider 隐藏系统提示不可观测，不声明修改。
- 远端模型目录拥有的提示只形成上游变更稿；除非另获授权，不修改远端服务。
- S1-S5 为讨论与决策阶段；S5 接受前禁止修改产品源码。
- 用户已授权在最终验证通过后准备并发布 Linux、Windows 与 macOS 构建；install、deploy 与 service restart 仍不执行。

## 量化目标

- 同工具配置下，首轮 input tokens 相对 M1 基线至少降低 25%。
- 非工具指令字符相对 77,567 字符基线至少降低 40%。
- 技能目录重复名称归零。
- 现有 System Skills 全部完成触发描述、正文、资源和验证方式审阅；只在真实能力缺口成立时新增至多两个 System Skills。
- 普通问答不加载无关 TUI、app-server 或 Workflow 执行规则。
- 子智能体不再无差别继承最强模型；机械任务、长上下文机械任务和高难度任务可选择不同的成本/能力档位，并记录实际解析结果。
- root 可在重复工作、多路独立工作和调研等合适场景主动委派，同时持续拥有项目方向、关键路径、成本、整合和验收责任。
- 角色、来源、顺序、字符数、token 估算和缓存状态可调试。

## 完整计划

| ID | 阶段 | 期望结果 | 验证方式 |
| --- | --- | --- | --- |
| M1 | 真实基线 | 固化当前 prompt 栈、来源、角色、字符/token 和缓存证据 | rollout、配置和构造源码交叉核对 |
| S1 | 基础提示与人格讨论 | 逐项完成原文、中文、点评和用户裁决，形成基础提示候选稿 | 用户接受身份、语气、沟通和自主执行边界 |
| S2 | 通用运行规则讨论 | 审阅权限、mode、工具、编辑、更新、final、多智能体规则与 worker routing 契约 | 每项只有一个权威来源，冲突有明确裁决；子智能体档位能表达难度、上下文与成本需求 |
| S3 | System Skills / 内置 Apps / Plugins 讨论 | 深审随包 System Skills 与内置通用提示；Repo/User/外部 Plugin Skills 只做归因和 usefulness 筛选 | 用户接受 System Skills 集合、触发/加载方式、内容边界和 token 预算 |
| S4 | AGENTS / 环境 / 外部注入边界讨论 | 将仓库规则按任务和目录拆域；盘点外部插件/Workflow 注入来源但不改写其业务提示 | 用户接受作用域、权威源、注入隔离和普通问答行为 |
| S5 | 集成设计审阅 | 展示最终角色图、完整候选提示、注入矩阵、冲突表和预计 diff | 用户接受目标稿；接受动作才授权 M2 源码修改 |
| M2 | 实现获批设计 | 修改仓库拥有的提示来源、过滤、去重、调试输出和 Multi-Agent V2 worker routing 接口 | 编译通过，实际 prompt 与 S5 一致；语义路由可解析为有效的子智能体模型配置 |
| M3 | 回归与质量验证 | 覆盖 prompt、人格、skills、Workflow、resume 和缓存行为 | 最小相关 test groups 通过，最终 smoke 通过 |
| S6 | 真实效果验收 | 展示同配置 before/after prompt、token、角色顺序和代表任务输出 | 达成量化目标且关键行为不回退 |
| M4 | 跨平台构建发布与 Cycle 收尾 | 固化结果、证据和风险；构建并发布 Linux、Windows、macOS 产物 | 三平台目标产物、校验和、manifest/Release 与安装入口一致，用户接受最终结果并关闭 Cycle |

## Stone 规则

- 每个 Stone 在对话中展示真实内容，不以文件路径代替解释。
- S1-S4 内部逐项讨论，未完成全部裁决前保持 `in_progress`。
- Stone 完成后进入 `waiting-review`；接受后自动进入下一项，拒绝后保留记录并修订。
- S5 是唯一的产品源码修改授权点；S1-S4 的接受不授权提前实现。

## M2 接受后范围调整

- Stage A 已完成并接受。
- Stage B System Skills 已完成并接受。
- 用户将剩余实现范围收敛为 Stage E Worker Routing；Stage C AGENTS/runtime injection 与 Stage D image-provider capability 延期到后续 Cycle，不再阻塞 C28。
- Worker Routing 后直接执行 omissions audit、真实 prompt/token capture、focused regression 与 smoke；只补充审计确认的真实遗漏，不重开更广泛的 prompt-stack 重构。
- 最终验证通过后准备并发布 Linux、Windows 与 macOS 产物；发布前展示确切版本、目标矩阵、产物和校验结果。install、deploy 与 service restart 仍未授权。
