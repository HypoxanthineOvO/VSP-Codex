# Correctness Gaps — Done-Wrong 闭环修复归档

> 日期：2026-06-01 | 时区：Asia/Shanghai | 语言：zh-CN
> 来源审计：[gap-analysis.md](./gap-analysis.md)（2026-05-31）
> 指令：用户 "我希望全都修啊，你用 Codex Subagent 分别开修吧" —— 修复 review-20260531 识别出的全部 done-but-wrong / 未接线项。

## 背景

`gap-analysis.md` 的主导失败模式是 **"代码+测试在、milestone 标完成，但生产入口是死代码 / 被禁用常量挡住 / 从没接线，用户实际用不到"**。本文件记录每项的**真实闭环修复状态**（不是 milestone 状态），与原审计矩阵一一对应。

## 修复矩阵

| 审计项 | 原状态 | 本轮动作 | 现状态 | 证据 |
|---|---|---|---|---|
| item 2 — 更新确认 UI 未统一 | 🟡 确认走旧 `UpdatePromptScreen`，`release_update_confirmation_zh_cn` 卡片是死代码 | VSP 模式更新确认改为 chat 内 `request_user_input` Workflow 卡片；合成 turn id `vsp_release_update` 在 App 事件循环本地拦截；update_now→退出+更新，skip_this_version→`dismiss_version`，view_notes→插 URL 后重呈现 | ✅ **闭环** | `workflow_cards.rs::vsp_release_update_card`、`app/event_dispatch.rs::handle_vsp_release_update_decision`、`app.rs` VSP 分支；2 新单测 + 卡片结构测试通过 |
| item 6 — Subagent 查看器译/侧栏被削弱 | 🟡 viewer 在，但 `MIN_WIDTH=u16::MAX` 恒堆叠、翻译只覆盖 Reasoning | 侧栏宽度阈值修正使其可触发并排态；prompt/report 纳入翻译范围 | ✅ **闭环** | `chatwidget/active_subagents.rs`、`hypo_translate.rs` `TranslationScope` |
| item 8 — `/hw:*` 没注入 palette | 🟡 `workflow_slash_routing.rs` 数据齐但零引用 | 将 `/hw:*` 路由数据注入 command palette，Ctrl+Space 可见可用 | ✅ **闭环** | `command_palette.rs` 引用 `workflow_slash_routing` |
| item 9a — resume 闪烁抑制从不启用 | 🟡 守卫接了但 `suppress_session_configured_redraw` 恒 false | 在 resume 路径实际置 true，抑制生效 | ✅ **闭环** | `session_flow.rs` / `constructor.rs` |
| (本轮新增) vscode install 写错 scope | 🟡 remote 下写 Machine settings，application-scoped 被 VSCode 静默忽略，install 谎报成功 | install/status 在 remote `.vscode-server` Machine 作用域下明确警告 application-scoped 限制并给出本地 User settings 的 key/value；保留写入+备份（向后兼容） | ✅ **闭环** | `cli/src/vscode_cmd.rs::remote_machine_scope_warning`；3 新集成测试（共 13 绿） |

## 仍开放 / 需用户决策

| 项 | 状态 | 说明 |
|---|---|---|
| #11 VSCode resume shim 实机验证 | ⏳ **待用户 reload** | shim 已打，需用户在 VSCode 里 Reload Window 后实测 resume 是否生效 |
| item 4 — 官方 Codex 双向互通 | 🟡 **范围收窄，未动** | 现为单向导出+脱敏，无 import/resume。是否要做双向需用户确认范围（可能很大） |
| item 5 — 全部决策入口强制卡片 | 🟡 **部分** | 更新确认已接（item 2），其余决策入口尚未统一路由 |
| item 7 — 外部命名池 | 🟡 **测试桩** | `with_release_channel_metadata` 仅测试调用，未接 release/build 元数据；内部命名已闭环 |
| item 9b — 滚动锚点刷新跳回开头 | ❌ **未实现（设计类）** | 走 `/hw:plan`，需 scroll anchor 机制设计 |
| Update rollback（版本回退） | ❌ **未实现（设计类）** | 与 `vscode rollback`（设置回滚）不同；走 `/hw:plan` |
| #7/#8 更新机制接线 | 🟡 | manifest host (`gitlab.vsplab.cn`) URL 接线、复用官方更新检测作 VSP 信号/兜底（已授权，但**不得**把 fork 用户更新到官方 codex） |

## 校验口径

- 本轮所有 ✅ 项均经 `cargo test` 对应包验证（vscode：13 集成测试绿；TUI 更新卡片：2 单测 + 结构测试绿）。
- 工作区存在一份**先于本轮、非本会话**的大块未提交 diff，全程未触碰。
- 设计类项（item 9b、update rollback）与范围决策项（item 4/5）按用户既有意向走 `/hw:plan`，不在本轮直接实现。
