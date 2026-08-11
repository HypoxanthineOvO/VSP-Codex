---
kind: progress
cycle: C23-tui-quote-render-stream-finalize-fixes
plan: PLAN.md
status: closed
updated: 2026-08-08T19:03:21+08:00
current: completed
next: C24；测试与 CI 大幅精简在 C24 完成后的独立 Cycle 处理
---

# C23 TUI Quote 渲染与流式完成刷新修复进度

## 当前状态

M1-M6 与 S1 均已完成。Nod 已部署 Quote scrollback 补丁，用户在真实 SSH TUI 确认 Quote 正确；C23 已接受并关闭。

## 完整计划状态

| ID | 阶段 | 状态 | 当前结果 / 证据 | 下一步 |
| --- | --- | --- | --- | --- |
| `M1` | 定位三个问题根因 | `completed` | 三个根因已确认；诊断测试复现 blockquote fill 丢失（流式 1/3 vs 完成 38/40）；reflow 无条件清屏重插路径已定位 | 无，等待进入 M2 |
| `M2` | 修复 blockquote 背景在流式/完成切换时丢失 | `completed` | wrap 后检测带背景的 Quote rail 并补齐右侧 fill；定向测试 1/1 通过 | 无 |
| `M3` | 消除完成时非必要全量重排 | `completed` | live tail 先按 FIFO 插入；consolidation 仅在真实 resize 标志时 reflow；相关测试 13/13 通过 | 无 |
| `M4` | 收窄长上下文渲染/重排范围 | `completed` | Resume/resize 继续只渲染最近最多 400 行；completion 不再重插 transcript；cap/replay 测试 5/5 通过 | 无 |
| `M5` | 实现 Markdown 标题层级与宽度感知横线 | `completed` | H1 紫色背景粗体并填满容器；H2-H4 分层；H1-H6 隐藏井号；Rule 使用 subtle-border `─` 并扣除嵌套 prefix | renderer 110/110、stream collector 23/23 通过；三主题 × 40/80/120 快照已接受 |
| `M6` | 修复助手最终输出在当前 TUI 被提前截断 | `completed` | completion canonical source 与 streamed source 对账；prefix 缺尾只追加 suffix，分歧使用 canonical source 并请求一次强制 repair | 两条新增回归测试 2/2，flush 2/2、reflow state 11/11 通过 |
| `S1` | 人工审阅修复后的渲染行为与性能 | `completed` | Nod 真实 SSH TUI 暴露 scrollback 二次 wrap 裁掉 Quote 背景；新增 VT100 RED/GREEN 后部署 SHA `cbdf855c…`，用户确认 Quote 正确 | 已接受 |

## 阻塞

- 无。

## 计划变化

- 2026-08-08：在不重排既有 `M1-M4/S1` 的前提下，于 pending Stone `S1` 前插入 `M5`，记录用户新增并通过的 Markdown 标题/thematic break 视觉合同。
- 2026-08-08：复核 VSP-Codex/DeepSeek 前置会话 `019fc1ba-5883-7e93-bc2c-f33abfdfbcca` 与 Pi 诊断会话 `019fd785-079a-76db-a186-1082662caf38`；前置 Quote/Resume commit 已在 HEAD，但 C23 的后续 M2-M5 没有完成实现。
- 2026-08-08：用户明确授权执行 C23，M2 进入 `in_progress`，之后自动推进 M3-M5 并在 S1 等待人工审阅。

## 下一步

C24 从 M1 开始；测试与 CI 大幅精简在 C24 完成后的独立 Cycle 处理。
