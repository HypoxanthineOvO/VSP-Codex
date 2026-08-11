---
kind: cycle-summary
cycle: C24-responses-multi-provider-config-model-switching
status: closed
updated: 2026-08-08T21:22:00+08:00
---

# C24 Responses Multi-Provider Config And Model Switching

## 目的与边界

在不污染官方 `config.toml` 的前提下，让新版 VSP/DEX 从独立
`$CODEX_HOME/model-providers.toml` 发现多个 Responses providers，并通过统一 `/model`
面板原子切换 provider、model 和 reasoning effort。范围不包含 Chat Completions 兼容层、
明文密钥管理、供应商特有能力承诺或发布操作。

## 结果

- 严格 versioned addon parser、effective registry、collision protection 和 addon-only
  atomic default persistence 已完成。
- App-server v2 通过 `modelProvider/list`/`write` 拥有远端配置根读写边界；TUI 不直接访问
  remote sibling file。
- Provider-grouped `/model`、credential-disabled states、窄屏布局和失败回滚已完成。
- Per-turn provider catalog、resume、fork 和 manual compact 均保持 provider-local metadata
  与 endpoint。
- 同轮 resize replay 贴底和 CJK streamed whitespace 回归已修复并锁定测试。

## 证据

- [M3 runtime/picker report](M3-REPORT.md)
- [M4 final hardening report](M4-REPORT.md)
- S1：用户接受多 provider、single fallback、missing credential 与 narrow snapshots，并明确
  不要求人工手测。
- Protocol 271 unit + 2 schema fixture、Core addon 13、app-server public API 1、compact 1、
  provider metadata 1、C24 TUI focused tests/snapshots 全通过。
- Rustfmt、diff whitespace 和 pending snapshot hygiene 通过。

## 重要决定

- Runtime identity 始终是 `(provider_id, model_id)`，跨 provider 同名 model 合法但 metadata
  必须隔离。
- Credential 只引用 environment variable name；缺失时可发现但不可请求。
- addon default 与 provider definitions 均不写入官方配置；切回官方 provider 时清除 addon
  default。
- Remote persistence 必须由 app-server 执行，不能假设 TUI 与 server 共享本机文件系统。

## 问题与经验

- 全量 TUI 被 9 个无关 snapshot drifts 和默认 test-thread stack overflow 中止；C24 tests
  已通过，overflow 用例增大栈后通过，未接受无关 snapshots。
- Workspace Clippy 被现有 core/app-server deny lints 阻断；C24 touched modules 未报告新 lint。
- 跨 core、protocol、app-server、TUI 的兼容功能应按独立模块和 focused matrix 管理；generated
  schema 应与人工逻辑分开审阅。

## 后续候选

- 使用真实第三方 credential 做供应商级 Responses/tool/reasoning compatibility certification。
- 如需发布，单独确认版本号、target branch/remote、产物、安装与重启范围，再执行 commit、
  push、package、tag 和 release。
