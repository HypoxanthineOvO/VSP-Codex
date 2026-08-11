---
kind: discussion-ledger
cycle: C27-fork-test-ci-simplification
status: completed
updated: 2026-08-09T21:30:08+08:00
---

# C27 Discussion Ledger

## 2026-08-09 Final Result Rejection

### User feedback

> 除了 CI 之外，我们本地用的这些测试，有给它分组并给我们的 AI 写清楚这些测试的用途吗？
>
> 不要让它每次都给我报啊，全量测试完了之后又说有一堆错的又不修，这样是不行的。你得让它知道，哪些是有用、有价值的。

### Revision response

- 当前只分开了命令，没有给AI机器可读的测试用途和选择规则，拒绝成立。
- C27恢复M4：建立分组manifest、统一runner和AGENTS纪律。
- AI必须先按改动合同选择最小测试组；不得用全量测试作为默认诊断兜底。
- 主动运行某组后，失败必须在当前范围内修复，或证明为阻塞并停止；不得只把历史失败清单交给用户。
- `upstream-all`只在用户明确要求或共享harness/全仓机械变更时运行，并要求记录reason。

## 2026-08-09 v2 Final Acceptance

### User decision

> 接受并关闭 C27

### Accepted scope

- 14个用途分组、机器可读manifest、runner、Just和AGENTS规则。
- 单job/10分钟required CI与current artifact smoke合同。
- 精确snapshot/test清理、SDK假绿修复和external/manual分层。
- RESULT v2声明的验证结果、预期行为与剩余风险。
