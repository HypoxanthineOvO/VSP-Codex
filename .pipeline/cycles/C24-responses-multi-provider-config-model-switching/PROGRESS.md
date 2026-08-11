---
kind: progress
cycle: C24-responses-multi-provider-config-model-switching
plan: PLAN.md
status: closed
updated: 2026-08-08T21:22:00+08:00
current: completed
next: 如需发布，另行确认版本号、目标和外部操作范围
---

# C24 Responses 多 Provider 配置与模型切换进度

## 当前状态

M1-M4 与 S1 均已完成。独立 addon 配置、provider-local runtime、remote app-server
ownership、统一 `/model`、resume/fork/compact 与最终自动化 hardening 已交付；C24 已关闭但
尚未发布。

## 完整计划状态

| ID | 阶段 | 状态 | 当前结果 / 证据 | 下一步 |
| --- | --- | --- | --- | --- |
| `M1` | 固化 Responses provider 与独立配置合同 | `completed` | 结构化 model metadata、严格 v1/URL/env/default/duplicate 合同；parser RED 0/7 后 GREEN 7/7；仓库无独立 DEX loader，连接式入口统一为 app-server → ConfigBuilder | 已完成 |
| `M2` | 实现独立 loader、registry 与安全持久化 | `completed` | 缺失 no-op、ConfigBuilder merge/default、effective ID collision、env-key provider、addon-only 原子写入与注释保留均通过；focused 11/11 | 已完成 |
| `M3` | 接通 runtime、app-server/DEX 与统一 `/model` 面板 | `completed` | 单次 settings update、per-turn catalog、addon/official persistence、rollback、mock request、resume/fork 和 4 类 snapshots 均通过；见 M3-REPORT.md | 已完成 |
| `S1` | 人工审阅真实多 provider 切换 | `completed` | 用户于 2026-08-08 接受四类 UI 证据，并明确不要求人工手测 | 已完成 |
| `M4` | 收口兼容、回归与维护面 | `completed` | remote server-owned list/write、manual compact endpoint、protocol schema、focused UI/runtime 与 hygiene 通过；见 M4-REPORT.md | 已完成 |

## 已有证据

- `4621105f73 fix(config): isolate chat provider profiles` 证明 V10 已遇到共享 `config.toml` 兼容问题，但当时依赖显式 profile，不是自动附加层。
- `c1b07a6478 feat(tui): add provider-grouped model selection` 提供 provider-scoped identity/catalog 与分组 UI 参考，但变更约 6700 行且混有 Chat 与其他功能，不适合整体恢复。
- 当前主线 `Config` 仍有 `model_providers` 和 `ApplyProviderSelection`，但 `/model` 只读取当前 catalog，`modelProvider/list` 和旧统一 picker 已移除。
- 官方配置加载先合并 Config layers 再反序列化 `ConfigToml`；附加文件必须在这条链之外独立解析，才能保证旧版忽略与新字段隔离。

## 最终验证

- App-server protocol 271 unit + 2 schema fixtures 全通过。
- Core addon 13、provider metadata 1、manual compact 1、app-server public JSON-RPC 1 全通过。
- C24 TUI provider catalog/transaction/popup snapshots 以及 resize/CJK regressions 全通过。
- Rustfmt、diff whitespace 与 pending snapshot hygiene 通过。
- 全量 TUI 与 Clippy 的既有、非 C24 例外已记录在 M4-REPORT.md，未把无关修复混入本轮。

## 下一步

代码已 release-ready，但未执行 commit、push、package、install、restart 或 release。发布前
必须另行明确版本号、目标 branch/remote 和产物范围。
