---
kind: plan
mode: execution
cycle: C29-alpha6-vsplab-502-hotfix
status: active
updated: 2026-08-11T20:19:00+08:00
builds_on:
  - C28-prompt-stack-quality-redesign
progress: PROGRESS.md
execution: EXECUTION.md
source_edits: authorized
---

# C29 Alpha 6 VSPLab 502 Hotfix

## 目的

修复 `v0.145.0-vsp.11.1-alpha.6` 将 VSP 私有 Multi-Agent V2 routing schema 泄漏到
OpenAI-compatible `/responses` wire 后稳定返回 502 的发布回归，同时保留客户端内部
worker routing 的执行、日志和 telemetry 行为。

## 边界

- 仅修改 Multi-Agent V2 `spawn_agent` 的 model-facing output schema/result 及直接测试。
- 恢复 Alpha 5/官方 `spawn_agent` wire shape；保留 trusted policy、默认 standard
  candidate resolution、child config 和 telemetry。
- 六档显式 routing intent 暂不暴露给模型；后续必须设计客户端内部转换信道，不能新增
  provider-facing 私有参数。
- 不重开 C28 的 prompt/Skills/Apps/Plugins 范围。
- 用户追加授权在修复验证后发布 `v0.145.0-vsp.11.1-alpha.7` Linux hotfix；macOS/Windows
  不阻塞本轮。仍不执行安装、部署、用户配置修改或服务重启。

## 验证

1. 现有 Multi-Agent V2 schema/handler focused tests 通过。
2. 选择最小相关 behavior test group 并通过。
3. Rust format 与 diff hygiene 通过。
4. 修复后二进制以 `gpt-5.6-sol`、V2 collaboration 和现有 VSPLab 配置真实回复 `ok`。
5. Linux archive、SHA256、manifest、GitLab Release 和公开下载复验一致。
