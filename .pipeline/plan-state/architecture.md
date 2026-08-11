# C9 Architecture Checkpoint

状态：confirmed

时间：2026-06-14T09:08:51+08:00

确认时间：2026-06-14T09:10:28+08:00

## 架构结论

C9 分为两个不同性质的交付面：

- 主交付：上游基线升级到 `rust-v0.140.0-alpha.18`，并发布新的 VSP GitLab release。
- 次交付：Deep Research 需求简报，放入 `.pipeline/architecture/c9-upstream-base-upgrade-and-deep-research-brief.md`，供后续 Hypo-Workflow 实现使用。

## 升级架构保护面

必须保护 VSP CLI identity、provider/config、VSCode Remote SSH 接入、Workflow runtime surface、Subagent UX、release channel、installer/updater。

## 技术路线

1. 先提交或检查点当前归档/规划改动。
2. 创建隔离升级分支。
3. fetch upstream 并以 `rust-v0.140.0-alpha.18` 为目标合并。
4. 解决冲突，保留 VSP 定制面。
5. 运行 focused + broad validation。
6. 更新版本、README/docs、release manifest。
7. 构建 release archive 并 smoke。
8. 经用户确认后推送 GitLab tag/release。

## Deep Research 简报落点

简报已写入 `.pipeline/architecture/c9-upstream-base-upgrade-and-deep-research-brief.md`。本 Cycle 不做 Codex-VSP 原型或完整实现。

## 确认结果

用户已接受该 Architecture 检查点，进入 Decompose。
