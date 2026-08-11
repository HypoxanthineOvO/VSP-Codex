# C6 Prompt Audit Report

## 结论

pass

可进入 P4，但建议在 P4 前做一轮小修：把 P3 全局的“test owns red tests, fixtures, assertions, snapshots”要求复制到 M2-M5 的各自 Subworker Assignment Plan 中，避免单独执行某个 Prompt 时丢失该约束。

## 阻塞问题

无阻塞问题。

## 建议改进

1. M2-M5 的 `test` 角色都写了 owns tests，但没有像 M1 那样明确“failing/red tests”或“fixtures/assertions/snapshots”。P3 全局政策已覆盖该要求，所以不阻塞；但建议逐 Prompt 重复写入。
2. M5 的 docs/release validation 写成“current release flow”，不够精确。建议实现前由 test worker 固化为仓库实际命令或明确手工场景。
3. 安全边界在 P3 和 M3 中明确保留，建议 M1-M5 都追加一句：destructive operations and system-level dependency installation require user confirmation。

## 总体检查

- P2 technical_solution、technical_route、research_required、risks_and_alternatives、validation_path、audit_focus：已保留到五个执行 Prompt。
- Subworker Assignment Plan：每个实现 Prompt 均恰好包含 `test`、`implement`、`audit` 三个角色。
- Worker Separation：已标记 strict，且 implement 均禁止创建、编辑或重写 test assets。
- 生命周期证据：每个 Prompt 都要求 requested、started、completed/failed/blocked、closed/close_failed。
- Prompt audit 默认规则：`.pipeline/plan-state/c6-p3-generate.md` 已记录“从 C6 P3 开始，生成 Prompt 后默认 Subagent 审核”，并设置 P4 双门控。
- 安全边界：P3 和 M3 明确保留破坏性操作、系统级依赖安装的用户确认门。
- 顺序：匹配用户优先级，M1 Compact 空恢复优先，其后工具 API、内建化、普通对话捕获、UX polish。

## 逐 Prompt 检查表

| Prompt | P2 字段保留 | 三角色 | 测试/实现分离 | 验证路径 | 安全/风险 | 结论 |
|---|---|---:|---:|---:|---:|---|
| M1 Compact Fallback Context Pack | 是 | 是 | 是 | 强 | 是 | pass |
| M2 Structured Workflow File API | 是 | 是 | 是 | 强 | 是 | pass |
| M3 Native Workflow Runtime Routing | 是 | 是 | 是 | 强 | 强 | pass |
| M4 Ordinary Conversation Capture | 是 | 是 | 是 | 强 | 是 | pass |
| M5 Workflow Command And UX Polish | 是 | 是 | 是 | 中 | 是 | pass |

## 是否可进入 P4

可以进入 P4。

P4 确认时建议把上述小修作为非阻塞改进项列出；如果用户希望 Prompt 更自包含，则先修 M2-M5 的 test 角色措辞后再确认。
