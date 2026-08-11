---
kind: cycle-summary
cycle: C27-fork-test-ci-simplification
status: closed
accepted: 2026-08-09T21:30:08+08:00
result: RESULT.md
---

# C27 Summary

## 接受结果

用户接受C27 v2。VSP required CI保持一条current `vsp-codex` build+smoke门禁并增加10分钟上限；本地验证按14个行为合同分组，AI必须选择最小相关组并对主动运行产生的失败负责。

## 交付内容

- `scripts/vsp_test_groups.json`：14个机器可读测试组，包含用途、价值、适用改动、前置条件、CI属性和命令。
- `scripts/vsp_test_groups.py`：list/explain/validate/run runner；强制filter/env/reason，focused Rust使用`--no-tests=fail`，TUI阻止pending snapshots。
- `AGENTS.md`：AI测试选择、禁止全量兜底、失败修复/阻塞纪律和最终报告规则。
- `justfile`：`vsp-ci`、`vsp-release-check`、`test-groups`、`test-group`与受reason保护的`test-all`。
- `.gitlab-ci.yml`与`docs/ci.md`：单required job、10分钟budget、VSP/上游边界和分组使用说明。
- 测试组合：删除22个orphan/redundant snapshots和3个无价值测试；修正TypeScript abort假绿与错误fork binary path；4个external合同manual化，3个hermetic合同保留default。

## 接受证据

- current build与CLI/TUI/app-server smoke通过。
- CLI 16、core 3、app-server 1、exec 4、protocol 273、TUI 1、persistence 114、release 46、TypeScript SDK 5、security 86、V8 5、Hypo external 4全部通过。
- `upstream-all`只验证reason dry-run，没有启动全仓。
- 空filter、缺env、缺reason和0-test filter都非零失败。
- 临时GNU package metadata/entrypoint正确；package/source binary SHA256一致；包内smoke通过。
- Manifest/Python/Ruff/Prettier/Rustfmt/Justfile/GitLab CI/YAML/diff/snapshot hygiene通过；pending `.snap.new`为0。

## 接受后的行为

AI先识别改动合同，再从manifest选择最小组；相关失败必须修复并重跑。全量上游suite只有在用户明确要求、shared harness变更或全仓机械改动时使用，必须记录reason，且不能把未处理失败清单作为交付结果。

CI绿色仅表示current binary可构建、可启动、最小交互可用，不表示所有功能全面正确。平台、remote/provider、完整TUI、性能和上游全量测试继续按相关任务手工选择。

## 范围与后续

C27未修改C26产品实现，未commit、push、发布、安装、部署或重启。14组是当前高价值目录；未来新增重要行为时应维护manifest，不应退回无目的full suite。
