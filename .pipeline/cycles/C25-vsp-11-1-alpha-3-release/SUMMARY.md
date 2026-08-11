---
kind: cycle-summary
cycle: C25-vsp-11-1-alpha-3-release
status: closed
updated: 2026-08-08T21:59:50+08:00
---

# C25 VSP 11.1 Alpha.3 Release

## 结果

Alpha.3 已提交、构建、发布、远端校验并部署到本机、Nod、Eden。三机 current binary 和
app-server initialize 均一致；GitLab pipeline success。

## 证据

- [Release report](RELEASE-REPORT.md)
- [Release notes](../../release/v0.145.0-vsp.11.1-alpha.3/release-notes.md)
- GitLab Release：`https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.11.1-alpha.3`

## 后续

Alpha.3 进入 soak。下一 Cycle 负责 CI validation、反馈修复、session migration parity 审计，
通过后发布新的不可变 Stable `0.145.0-vsp.11.1`。
