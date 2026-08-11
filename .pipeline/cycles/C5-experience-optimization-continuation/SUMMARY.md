---
archived_source: .pipeline/archives/C5-experience-optimization-continuation
cycle: C5-experience-optimization-continuation
finished: 2026-05-28T22:00:16+08:00
kind: cycle-summary
started: 2026-05-28T19:57:25+08:00
status: closed
---

# 体验优化延续 总结

## 目的与边界

Completed C5 experience follow-up: persisted UI themes, executable command palette/copy improvements, and resume metadata cached first paint with progressive canonical verification.

## 最终结果

旧 Cycle 状态：`completed`。本预览不重新判断旧结果，只建立可读导航。

## 验证结果

- 3 个历史 milestone 已映射。
- 14 个旧文件继续保存在 `.pipeline/archives/C5-experience-optimization-continuation`。

## 重要决定与经验

- 旧 `cycle.yaml` 没有结构化 lessons；请查看原始 Summary。

## 后续候选

- 不自动继承旧任务；新 Cycle 应通过 `builds_on` 选择需要的结果或经验。

## 映射缺口

- 无结构性缺失。

## 旧总结原文

# C5 体验优化延续 Summary

- Cycle：C5 `体验优化延续`
- 类型：feature
- 状态：completed
- 开始时间：2026-05-28T19:57:25+08:00
- 完成时间：2026-05-28T22:00:16+08:00
- 验收时间：2026-05-28T22:57:19+08:00
- Preset：tdd

## 完成内容

C5 完成了 C4 顺延的体验优化：持久化 UI 主题选择器、主题色驱动的 Markdown/输出块/输入框样式、VSCode 终端安全的命令面板和复制最新代码块能力，以及 Resume 历史列表的 SQLite metadata cached first paint + progressive canonical verification。

## Milestones

- M1 Theme Picker + Persisted Palettes：新增 `[tui].ui_theme`、`/ui-theme` 和 `teal/violet/amber` 三组主题；Markdown 标题、strong、emphasis、inline code、link、blockquote、列表、表格，history 输出块 accent 和 composer focused prompt 都接入当前主题。
- M2 Command Palette + Focus Stack + Copy Selection：新增 `/commands` 和默认 `Ctrl+Space` 命令面板，隐藏危险命令，支持搜索执行常用安全命令，并新增 `Copy latest code block`。
- M3 Resume Metadata Cache + Progressive Verification：Resume/Fork 首屏先读 SQLite metadata cache，随后用同 token 做 canonical verification；过期行原位替换，不重复插入，preview/transcript 继续懒加载。

## Release Prep

- Release 版本确定为 `0.134.0-vsp.5`，因为 `v0.134.0-vsp.4` 已用于 VSCode 空 `model_provider` hotfix。
- 预览包：`.pipeline/release/C5-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz`
- SHA256：`5221b3410cf3db67cc7d3618a446b6ac42956bd18a8b7aab5ff024a15d7a8598`
- README、`docs/vsp-codex.md` 和 `docs/install.md` 已切到 C5 curl 安装命令。

## 验证摘要

- `cargo build --manifest-path codex-rs/Cargo.toml --bin vsp-codex`
- 包内 `vsp-codex --version`：`vsp-codex 0.134.0-vsp.5`
- 解压包 smoke：`vsp-codex 0.134.0-vsp.5`
- `sha256sum -c .pipeline/release/C5-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz.sha256`
- `just test -p codex-tui resume_picker`：98/98
- `just test -p codex-rollout state_db_only`：1/1
- `just test -p codex-app-server thread_list_state_db_only_returns_sqlite_without_jsonl_repair`：1/1
- `just test -p codex-thread-store read_thread_applies_sqlite_thread_name read_thread_uses_sqlite_metadata_for_rollout_without_user_preview list_threads_preserves_sqlite_title_search_results`：3/3
- `just test -p codex-tui command_palette`：7/7
- `just test -p codex-tui markdown_render`：127/127
- `git diff --check`

## 完成说明

改动摘要：C5 增加了可持久化主题、命令面板、复制最近代码块和 resume metadata cache，并完成 `0.134.0-vsp.5` 预览发布准备。

技术思路：沿用现有 TUI config、slash command、keymap、history cell、resume picker 和 thread metadata 管线，不引入新的外部 UI 框架；release 继续使用已有 standalone package builder 和 curl installer。

修改文件/模块：主要涉及 `codex-rs/tui`、`codex-rs/config`、`codex-rs/core`、`codex-rs/thread-store`、`codex-rs/rollout`、README、docs、CHANGELOG 和 `.pipeline/release/C5-preview`。

测试设计：按功能边界跑聚焦 nextest；Markdown 覆盖 strong、主题色角色和三层 bullet；命令面板覆盖快捷键、过滤和 copy path；resume 覆盖 cached first paint、stale row replacement 和 lazy transcript。

预期结果：安装 `0.134.0-vsp.5-preview` 后，用户可通过 `/ui-theme` 选择主题，通过 `/commands` 或 `Ctrl+Space` 打开命令面板，复制最新代码块，并看到更快的 resume 历史首屏。

遇到的问题：并行启动多个 Cargo 测试时出现 package/artifact lock 等待，但测试最终全部通过；C5 package staging 目录体积约 1.2G，发布提交只保留 tar.gz 和 sha256。

风险/后续：当前预览包是 Linux x64 GNU/glibc debug 包，体积较大；GitLab raw 链接仍依赖目标用户下载权限，若未登录 raw 被重定向到 `/users/sign_in`，需要改用 GitLab Release 或 Package Registry 资产链接。
