# S1 BaseInstructions 审阅结果

## 结论

当前 live `BaseInstructions` 已完成逐源、逐大段审阅，并形成可直接用于 override 或实现验证的英文候选稿。候选稿保留必要的身份、沟通、状态连续性、交付、工具、安全和授权合同，删除主观人格表演及整个 Skills 实现手册，并补入 VSP-Codex 品牌、中文主体沟通、追加式 steer、按时间线恢复、详细过程/结果、多类型报告、跨会话改动归属和 shell 适配要求。

S1 已具备用户验收条件。接受 S1 只锁定 Source 1 候选稿，允许进入 Source 2 讨论；不授权修改产品源码。

## 技术方法

- 从当前真实 rollout 的 `session_meta.payload.base_instructions.text` 提取生效文本，而不是假设仓库模板生效。
- 对照 session 解析优先级和 Responses/Responses Lite 的请求承载位置。
- 将 148 行原文按连续大段分为 Block A、B、C，在对话中展示完整英文、中文翻译和点评。
- 每次用户修订都更新候选文本、决策记录、Progress、Execution、Discussion Summary 和 log。
- 最终生成纯英文候选文件，避免讨论说明混入可执行提示。

## 主要变化

### 身份与语言

- 身份改为 VSP-Codex，由上海科技大学 VSP Lab 学生基于 OpenAI Codex 开发。
- 任何时候以中文为主体；英文计算机术语按用户熟悉程度调整，不熟悉时提供中文翻译和解释。
- 删除 old friend、个人品味、another subjectivity、real and unique 等主体性表演。

### 写作与交付

- 允许并鼓励 Markdown，但避免过度拆分短段落。
- 保留列表前和标题后的 CommonMark 空行要求。
- 过程与结果分开；commentary 详细说明检查、发现、判断依据、取舍和下一步。
- Final 重新汇总全部重要内容，并按工程、探索和实验分别定义详细交付合同。
- 报告、计划、审计和分析文件不能只报路径，必须在对话说明重要内容。

### 状态连续性

- Steer 默认追加，只有明确停止、暂停、否定或替换才终止旧任务。
- 冲突要求说明并询问，不静默丢弃任何一方。
- Compaction 按最新执行时间线恢复，用后续行动、结果和决定解释早期消息。

### 工具与安全

- 正式 VSP package 期望随包 `rg` 和 PATH 可见；非 package 环境一次检测后稳定 fallback。
- 基础提示遵循当前 shell；PowerShell/POSIX 细则后续按环境动态注入。
- 保留脏工作树保护、破坏性 Git 授权边界和外部/不可逆操作授权。
- 意外改动可查询同 workspace/cwd 近期相关会话判断来源，但不继承其他会话的任务或授权。

### Skills

- 完整 `Using skills` 章节从 BaseInstructions 删除。
- 动态 Skills 来源将在 S3 成为唯一权威，并采用项目内完整工作链生命周期、入口只读一次和引用按需加载。

## 修改的 Workflow 文件

- `PLAN.md`：讨论优先、S5 前源码锁定的完整 C28 计划。
- `PROGRESS.md`：M1 完成和 S1 Block A/B/C 讨论时间线。
- `EXECUTION.md`：来源解析、用户修订和验证 checkpoint。
- `DISCUSSION-SUMMARY.md`：全部确认决定、拒绝的方法和未决项。
- `M1-BASELINE.md`：当前 prompt 栈、字符/token 和角色基线。
- `S1-BASE-PERSONALITY-REVIEW.md`：完整来源解析、翻译、点评和分块候选。
- `S1-BASE-INSTRUCTIONS-CANDIDATE.md`：纯英文完整候选稿。
- `.pipeline/INDEX.md`、`.pipeline/cycles/INDEX.md`、`.pipeline/log.yaml` 和 session focus：Cycle 与状态记录。

没有修改产品源码、配置、测试、发布文件或远端服务。

## 验证设计与结果

| 验证 | 结果 |
| --- | --- |
| Live source identity | 16,270 chars、148 lines、SHA-256 已冻结 |
| Candidate size | 11,696 chars、99 lines |
| BaseInstructions 字符降幅 | 28.1% |
| `Using skills` / `How to use skills` | 候选稿中不存在 |
| 旧人格措辞 | old friend、subjectivity、tastes/preferences 等不存在 |
| 核心新合同 | 品牌、中文、CommonMark、追加 steer、时间线恢复、工程/探索/实验报告、rg、PowerShell、多会话、授权均存在 |
| Markdown spacing | 标题后和列表前空行静态检查通过 |
| Workflow YAML/front matter | 语法检查通过 |
| Diff hygiene | `git diff --check` 通过 |

本 Stone 没有产品行为变化，因此没有运行 Rust test group。后续实现必须增加 prompt 拼装、resume/model switch、Responses Lite、shell context、skills 和 session provenance 的行为测试。

## 预期行为

- 模型清楚识别 VSP-Codex 品牌并始终以中文为主体沟通。
- 中途新增要求不会再默认覆盖旧任务。
- Compaction 后从最新工作状态恢复，不复活已完成或已否定路径。
- 过程输出更具体，最终交付更完整，复杂工程和实验不再只给简短摘要或文件路径。
- 正式安装的 rg 缺失会暴露为产品诊断，非 package 环境不会反复失败。
- Windows/PowerShell 不再被 POSIX 规则覆盖。
- Skills 规则不再占据和重复污染基础提示。

## 遇到的问题

- 最初错误地把讨论拆成孤立 B1 句子；用户拒绝后改为完整 Source 解析。
- 第二次仍以目录和统计替代具体措辞；用户再次纠正后改为连续大文本块和完整双语展示。
- 原文把 steer 解释为可能替换旧任务、把 compaction 解释为最后用户消息优先；两者均会导致任务状态回退，已经重写。
- 当前正式包已经捆绑 rg，用户遇到的缺失更可能来自 launcher/PATH；需要产品实现阶段验证所有入口。

## 风险与后续

- 候选稿字符降幅为 28.1%，低于 C28 最终全栈 40% 目标；S2-S4 必须继续清理 developer、Skills 和 AGENTS 重复。
- 当前模型基础提示可能来自远端模型目录；本仓库只能通过 override、fallback/template 或上游目录变更使其生效。
- 恢复会话会固定旧 session meta，必须定义新提示对旧会话的迁移策略。
- Responses Lite 把基础提示改成 developer message，需验证角色语义一致。
- PowerShell 动态说明和 workspace-session provenance 需要产品能力或 context contributor 支持，不能只靠静态基础提示。

## 用户接受

用户于 2026-08-10 19:28（Asia/Shanghai）接受 S1。

接受范围是当前完整 BaseInstructions 候选、三块讨论决定、验证结果和已声明风险。接受允许 C28 进入 S2；不授权修改产品源码、配置、远端模型目录、发布或安装。
