---
kind: release-report
cycle: C25-vsp-11-1-alpha-3-release
status: completed
updated: 2026-08-08T21:59:50+08:00
---

# VSP Codex 0.145.0-vsp.11.1-alpha.3 Release Report

## 结论

`0.145.0-vsp.11.1-alpha.3` 已完整发布并部署。`origin/main`、annotated tag、GitLab
Release 和 package registry assets 已创建；本机、Nod、Eden 的 standalone `current` 均为
同一 alpha.3 binary。该版本进入 soak，Stable promotion 等待下一 CI validation Cycle。

## 技术与修改范围

- 发布 commit `cd7921547e89860db41419a08012983a16f513f5` 合并已关闭的 C23 TUI
  quote/stream fixes 和 C24 Responses multi-provider feature。
- 更新 Cargo/npm workspace version、Cargo/Bazel lock、README/install/CHANGELOG、protocol
  schema fixtures、provider snapshots 和 versioned release metadata。
- 标准 release builder 生成 Linux x64 GNU package；archive 未提交 Git，上传到 GitLab
  Generic Package Registry 并通过 Release downloads 暴露。
- 三机通过 VSP standalone updater 下载、验证、写独立 release 目录并原子切换 `current`。

## 产物与远端结果

- Commit：`cd7921547e89860db41419a08012983a16f513f5`
- Tag：`v0.145.0-vsp.11.1-alpha.3`（annotated，peeled commit 与 main 一致）
- Release：`https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.11.1-alpha.3`
- Archive：`vsp-codex-preview-linux-x64-gnu.tar.gz`
- Archive SHA256：`682d8004c098420d78efd6879f58265e0ec856991cf49f6a11489696c95beb6b`
- Binary SHA256：`7881eaebf1efa9eb8cf6e2dda3908653d6452840a0a91b28484c4d5fb73251b5`
- GitLab pipeline：`#346` / IID `#127`，`main`，success。

## 测试与验证

- C24 pre-release matrix：protocol 271 + 2 schema fixtures、Core addon 13、metadata 1、
  compact 1、app-server public API 1、C24 TUI focused tests/snapshots 全通过。
- Release builder unit tests：7/7。
- `cargo check -p codex-cli --bin vsp-codex`、release build、Bazel lock refresh、rustfmt、
  CI docs、JSON、diff 和 pending snapshot hygiene 通过。
- Package extraction smoke：binary version/metadata/target/channel 正确；package binary 与 release
  build byte-identical。
- Remote download smoke：Release SHA file 与本地一致，完整 tarball 流式 SHA 一致。
- 三机：`current --version` 均为 alpha.3，binary SHA 均相同；独立 app-server stdio
  `initialize` 均返回 alpha.3 user agent。
- Nod/Eden control socket owner 均为 alpha.3 server；本机空闲 daemon 无 client 时退出，stdio
  initialize 证明按需启动正常。

## 预期行为

- Alpha channel 用户可通过 `VSP_CODEX_RELEASE_CHANNEL=alpha vsp-codex update` 获取本版。
- C23/C24 UI 与 provider 功能在三台 soak 主机上可实际使用。
- 旧 release 目录保留，`current` 切换是原子的，可在必要时回滚。
- Alpha tag 不移动；Stable 使用 CI/soak 后的新正式 version 和 rebuilt binary。

## 问题、风险与后续

- 三台 updater 均报告 official/VSP state DB migration sets 不一致，自动 session migration
  被跳过。未运行会修改共享 DB 的 `session-compat apply`；下一 CI Cycle 应先审计 migration
  parity 和备份/回滚方案。
- Eden 当前会话的两个 thin proxy 由本轮开始前的本机 TUI 缓存旧绝对路径，暂时仍是 9.5；
  它们连接的 control-socket server 已是 alpha.3。退出并重开当前 TUI 后，repaired VSCode
  shim/current 会让 proxy 也使用 alpha.3。
- 当前这条本机 TUI 会话本身不能在保留最终回复的同时自杀重启；其 app-server 已停止并由
  alpha.3 smoke 验证，新开会话使用 alpha.3。
- Stable gate：在本机/Nod/Eden soak，收集反馈，完成下一 CI validation Cycle，再从最终验证
  commit 发布 `0.145.0-vsp.11.1`；不复用或移动 alpha.3 tag。
