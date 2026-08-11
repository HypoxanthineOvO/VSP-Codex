# 需求落实检查 — Chat → Prompt → 代码 追溯

> 日期：2026-05-31 | 时区：Asia/Shanghai | 语言：zh-CN
> 范围：回溯起点 **C7**（含 C6 carry-over），主体 C7 / C8 / C9 backfill。
> 方法：以 `.pipeline/chat/journal.yaml`、`plan-state/*-seed/decompose`、`reviews/*/prompt-audit.md` 为"讨论需求来源"，逐条核对是否进入 prompt、是否在当前工作区代码中真正实现闭环。

## 回溯起点说明

- **C1–C5**（archives）：fork 基础定制与早期体验，已归档关闭，本轮不回溯。
- **C6**（Workflow 原生化）：5 个 Milestone 全部进入 prompt 且完成；遗留的 `/hw:*` palette wiring 与 `turn_context.cwd` 警告作为 carry-over 进入 C7-M6。**作为边界纳入，不单独展开。**
- **C7**（体验优化/bugfix sweep）：seed 文件记录 10 条用户真实抱怨，是本轮核查重点。
- **C8**（Remote SSH / 插件兼容）：5 条需求，已完成发布 v0.134.0-vsp.6。
- **C9**（Post-Release Correctness Backfill）：对前述自查发现的部分缺口做补齐，用户确认已做完，未发布。

---

## 核心结论

整体上 **绝大多数 Chat 需求都正确进入了 prompt**——需求→prompt 这一环漏得不多（唯一明确遗漏是 update rollback）。真正的问题集中在 **prompt→代码闭环** 这一环：

> **主导失败模式：代码路径与单元测试存在、milestone 被标记完成，但用户实际可用的产品闭环没有接上。** 一个 milestone 因为"测试通过 + 报告写完"被判 done，但生产入口是死代码或被禁用。

符合这一模式的有 4 项（item 5/6/8/9a），是本轮最该关注的"看起来做完、实际没通"的部分。

---

## A. 追溯矩阵：自查 9 项 × 当前代码

每项状态经本人直接 grep/读码复核（不是仅凭 milestone 报告）。

| # | 需求（来自 Chat 自查） | 进入 Prompt | 当前代码状态 | 关键证据 |
|---|---|---|---|---|
| 1 | VSCode resume 空 `modelProvider` 修复 | ✅ c8-003 + C9 | ✅ **闭环** | `app-server/.../thread_processor.rs:192-197`、`turn_processor.rs:52-60`，含两条单测 |
| 2 | 更新提示 + 一键更新 | ✅ c7-007 | ✅ 闭环，**但确认 UI 没统一** | manifest URL `update_versions.rs:9`、启动检查 `updates.rs:49`、`vsp-codex update` `main.rs:785`；确认走自定义 `UpdatePromptScreen`，非 Request Input 卡片 |
| 3 | Remote SSH 插件真实验收 | ✅ c8-006 | 🟡 **代码能力够，验收证据缺** | `vscode_cmd.rs` status/probe/rollback 齐备；未找到已填写的 `reviews/C8/M6/acceptance-checklist.md` |
| 4 | 官方 Codex 看到 VSP sessions | ✅ c8-004 | 🟡 **单向导出/脱敏，非双向** | `rollout/.../official_codex_discovery.rs:120` 只有 export+sanitize，无 import/resume |
| 5 | Workflow 卡片强制所有决策入口 | ✅ c7-005 | 🟡 **框架+卡片在，但生产没路由** | `workflow_cards.rs:1` 顶部 `#![allow(dead_code)]`；三张卡只在 `request_user_input/mod.rs:4318+` 的测试里被调；更新确认仍走旧 UI |
| 6 | Subagent prompt/report 查看器 + 翻译 | ✅ c7-004 | 🟡 **查看器在，但译/侧栏被削弱** | `active_subagents.rs` 有 `ActiveSubagentPromptReportViewer*`；`hypo_translate.rs`(508行) 存在但 `TranslationScope` 只有 `Reasoning`（不覆盖 prompt/report）；viewer 侧栏 `MIN_WIDTH=u16::MAX`(`active_subagents.rs:19`) 实际永不触发，恒为堆叠态 |
| 7 | Subagent 内/外命名池分离 | ✅ c7-002/007 + C9 | ✅ **内部闭环；外部是测试桩** | 生产默认 VSP Lab（`role.rs:56`、`constructor.rs:147`）；`with_release_channel_metadata` 仅被测试调用，未接 release/build 元数据 |
| 8 | `/hw:*` 完整 command palette | ✅ c7-006（C6 carry） | 🟡 **数据齐，但没注入 palette** | `workflow_slash_routing.rs` 定义完整；但全仓除 `lib.rs:210 pub mod` 外**零引用**，`command_palette.rs` 不调用它 → Ctrl+Space 用不到 |
| 9a | resume 闪烁抑制 | ✅ c7-001 | 🟡 **守卫接了，但从不启用** | `session_flow.rs:150` 用了 `suppress_session_configured_redraw`，但字段恒初始化为 `false`(`constructor.rs:198`)，无处置 true → 抑制不生效 |
| 9b | 滚动翻阅后刷新跳回开头 | ✅ c7-001 | ❌ **未实现** | `chatwidget.rs`/`transcript.rs` 无 scroll anchor / 滚动恢复机制 |
| 9c | Plan/SubPlan 双进度展示 | ✅ c7-006 | ✅ 闭环 | `plan_panel.rs:148-188`，快照测试 `tests/plan_mode.rs:1738` |
| 9d | `/theme` 与 `/ui-theme` 统一 | ✅ c7-008 | ✅ 闭环 | `style.rs:31`、`app_event.rs:986,994` |

### C9 backfill 落实确认（用户称已做完）

| C9 项 | 状态 | 证据 |
|---|---|---|
| Plan 计数 root/subplan 拆分 | ✅ | `plan_panel.rs:246-265` `is_split` + 快照 |
| `update_plan` freshness（.pipeline 文件变更触发 stale 提醒） | ✅ | `session/mod.rs:436-441` 指纹四文件；`plan_freshness_reminder.rs` 注入提醒 |
| Subagent 默认 VSP Lab 命名 | ✅ | `role.rs:56`、`role_tests.rs:64` |

---

## B. 讨论里提过、但**没进任何 prompt** 的需求

| 项 | 出处 | 说明 | 建议 |
|---|---|---|---|
| ❌ **Update rollback**（回退到上一个 release 目录） | C7 seed Issue 8 | seed 明确讨论"保留上一个 release dir + `update rollback` 命令"，但 `c7-007` 只做 forward update。注意：`vsp-codex vscode rollback` 是"插件设置回滚"，不是"版本回滚"，两者不同 | 若仍需要，作为新 Patch/Milestone |
| ⚠️ **VSP Lab 完整内部人名池** | C7 Issue 3/6 | 只在 prompt 里定为 configurable，实际名单靠 config/代码常量，未在 prompt 层固化 | 已在代码中以常量落地（`role.rs:56`），可视为已解决，仅 prompt 未记 |
| ⚠️ **Plan stale marker 面向用户的视觉提示** | C7 Issue 7 | 落地的是"提醒模型 update_plan"，没有给用户看的"state 已领先"视觉标记 | 体验增强项，非必须 |

---

## C. prompt-audit 提过、但未被固化为 prompt 修订的执行风险

这些是 reviewer 在 `prompt-audit.md` 写过的非阻塞建议，停留在 review 文档，没回写进 prompt 的 validation_path，导致执行期可能漂移：

- **C8-M4**：要求"先确定 discovery 策略再实现"，未设硬门控 → 实际就停在了单向导出（见 item 4）。
- **C8-M6**：要求验收记录 host/settings 路径/binary 路径/session id，未列入 pass 必要字段 → 实际无验收证据（见 item 3）。
- **C8-M2**：VSIX 不可用时的 early-exit 策略未写入 prompt。
- **C6-M1~M5**：安全门禁措辞在 C7 全局补齐，但 C6 历史 prompt 未回溯（影响小）。

---

## D. 给用户的判断

1. **需求→prompt 这一环基本可信**：你在 Chat 里提的体验问题，几乎都被正确翻译成了 prompt，唯一真漏的是 update rollback。
2. **真正的债在 prompt→产品闭环**：item 5/6/8/9a 四项是典型的"测试绿了就算完成"，但生产入口要么是 `allow(dead_code)`、要么被极端常量禁用、要么从没 wire 进 palette。这正是你担心的"提了但没正确落实"，只是断点不在 prompt 而在最后一公里。
3. **item 4（官方 Codex 双向互通）与 item 3（真实验收）属于范围被悄悄收窄**：milestone 把"双向发现"做成了"单向脱敏导出"、把"真实 Remote SSH 验收"留成了待填 checklist。

详细工程缺陷分级见 `audit.md`，质量评分见 `quality.md`。
