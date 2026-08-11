---
kind: plan
mode: goal
cycle: C25-vsp-11-1-alpha-3-release
status: closed
updated: 2026-08-08T21:59:50+08:00
progress: PROGRESS.md
execution: EXECUTION.md
---

# VSP 11.1 Alpha.3 Release

## 目的与边界

把已关闭的 C23 TUI fixes 与 C24 Responses multi-provider feature 作为同一个兼容批次发布为
`0.145.0-vsp.11.1-alpha.3`。包含 version metadata、release notes、Linux x64 GNU package、
`origin/main`、Git tag、GitLab Release/asset，以及本机、Nod、Eden standalone 部署和 VSP
app-server/proxy 重启。稳定版 promotion 留到 soak 通过后的新不可变版本。

## 完整阶段

| ID | 阶段 | 结果 | 验证 |
| --- | --- | --- | --- |
| M1 | 审计与准备 | 版本、remote、产物、三机入口和进程边界明确 | Git/Glab/SSH read-only evidence |
| M2 | 版本与候选 | alpha.3 metadata、notes、manifest 与源码验证完成 | focused tests、release tests、format/diff |
| M3 | 提交与打包 | release commit 和可复现 Linux package 完成 | binary/package version、SHA256、archive smoke |
| M4 | 远端发布 | main、tag、GitLab Release 与 downloads 可用 | remote refs、HTTP asset、downloaded SHA |
| M5 | 三机部署 | 本机、Nod、Eden current 指向 alpha.3，旧进程退出 | version、hash、process path、app-server smoke |
| M6 | 报告与关闭 | 发布证据、问题、风险与 stable follow-up 完整记录 | release report 与 Cycle summary |

## 主要行为测试

- 发布 binary、package metadata、manifest、tag 和 release notes 使用同一 alpha.3 version。
- 远端 tarball 与本地 candidate SHA256 一致，解包入口可运行。
- 三机切换前保留旧 release；切换后 current 版本和 binary hash 一致。
- 只终止 VSP app-server/proxy 进程，不影响普通 SSH 或其他服务；重启后入口可初始化。
- C23/C24 focused evidence 不因 version bump 或 release build 回归。

## 授权

用户于 2026-08-08 明确选择完整发布 alpha.3，并授权本机、Nod、Eden 安装与重启。
