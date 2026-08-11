# C4 体验优化 + 发布 Summary

- Cycle：C4 `体验优化 + 发布`
- 类型：feature
- 状态：completed
- 开始时间：2026-05-27T17:05:53+08:00
- 完成时间：2026-05-28T19:57:25+08:00
- Preset：tdd

## 完成内容

C4 完成了预览版发布闭环：官方 VSCode 插件劫持安装与回滚、canonical Rename、真实 reasoning 能力门控、VSCode 终端安全默认快捷键、Markdown/消息块颜色补齐、性能证据脚本、Linux x64 预览包、README curl 安装方式，以及 `v0.134.0-vsp.4` 的 VSCode resume 配置热修。

## Milestones

- M1 Release Spine + VSCode Hijack：新增 `vsp-codex vscode install/status/rollback`，支持 settings 备份和回滚。
- M2 Canonical Rename：确认 Rename 写 canonical thread metadata，并补持久化和空 provider 回归覆盖。
- M3 Reasoning Capability Gate：只展示真实 reasoning summary/raw 内容；无内容不伪造 Thinking 块。
- M4 VSCode-Terminal-Safe Keymap + Composer Prominence：默认必要操作避开 VSCode 终端冲突键，输入框更明显。
- M5 Markdown Color Refinement + Default Subtle Blocks：补齐 strong/emphasis/list/table 等 Markdown 颜色和默认块 accent。
- M6 Performance Evidence + Low-Risk Startup Fixes：新增性能证据脚本并记录启动、resume、VSCode initialize probes。
- M7 Preview Release Validation：构建并验证 Linux x64 debug preview 包、checksum、install 脚本和发布说明。

## 延后到 C5

- 完整 Color Theme picker、三组主题持久化和 fallback matrix。
- 完整 command palette、focus stack、键盘复制/选择系统。
- resume metadata cache、recent preview cache、progressive stale-cache verification。

## 验证摘要

已运行的关键验证包括 `just fmt`、多个 scoped `just test -p ...`、`just fix -p ...`、release package build、tarball 解包 `vsp-codex --version` smoke、README curl 安装脚本 help smoke、GitLab release asset checksum 验证，以及 YAML/Markdown/diff hygiene 检查。

