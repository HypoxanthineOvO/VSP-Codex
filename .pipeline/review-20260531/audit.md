# Audit Report — 2026-05-31

> 语言：zh-CN | 时区：Asia/Shanghai
> 触发：用户发起的预防性工程审计（配合需求落实检查）。
> 范围：C7/C8/C9 改动涉及的 codex-rs 模块（tui、app-server、rollout、core、cli）。
> 方法说明：基于源码阅读 + 定向 grep 取证；未重跑全量 cargo 测试，正确性以 `state.yaml`/journal 记录的近期通过记录 + 本轮代码核实为依据。

## Intake

- **Trigger**：refactor/release 前的质量与"完成度真实性"治理。
- **User / workflow**：开发者本人 + Remote SSH 下的 VSCode 插件用户；关注"被标记完成的功能是否真的可用"。
- **Good-state**：每个声称完成的能力都有可达的生产入口，不靠死代码/禁用常量/test-only seam 撑场面。
- **Correctness contract**：现有通过的单测/快照不被破坏；resume、provider fallback、Plan 展示、命名不回归。
- **Scope**：近 3 个 Cycle 的改动面；不含 C1–C5 归档代码。
- **Risk tolerance**：闭环断裂判 Warning（除非影响核心正确性/安全），范围被悄悄收窄判 Warning。

## Applied Methods

- **GQM**：目标=声称完成==实际可用；问题=生产入口是否可达；证据=调用图/引用计数/初始化常量。
- **ISO/IEC 25010**：聚焦 Functional Suitability（功能是否真的存在）、Usability（入口可达性）、Maintainability（死代码/范围漂移）。
- **ATAM-lite**：识别"测试通过≠产品闭环"这一系统性敏感点。
- **SWEBOK**：construction（死代码）、testing（test-only seam 冒充生产覆盖）、process（milestone 验收口径）。

## Summary

- **Verdict：PASS_WITH_WARNINGS**
- **Findings：0 Critical, 6 Warning, 3 Info**
- **Gate：非阻塞**——没有数据丢失/安全/核心正确性 Critical；但存在多处"完成度名实不符"的工程债，建议进入 Action Queue。

## Experience

- **[EXP-01] Warning** — `tui/.../request_user_input/workflow_cards.rs:1`
  - 症状：Workflow 决策卡（plan/authorization/release-update）只在测试中渲染；更新确认实际走 `UpdatePromptScreen`(`lib.rs:1361`)。用户得到的不是"统一卡片体系"承诺的体验。
  - 建议：把更新确认等生产决策入口路由到 `release_update_confirmation_zh_cn` 等卡片，移除文件级 `#![allow(dead_code)]` 后让编译器守住"未接线即告警"。

- **[EXP-02] Warning** — `tui/src/chatwidget/active_subagents.rs:19`
  - 症状：Subagent prompt/report 查看器存在，但侧栏 `ACTIVE_SUBAGENT_VIEWER_SIDE_MIN_WIDTH = u16::MAX`，意味着并排"原文/译文"视图实际永远不触发，恒为堆叠；且 `hypo_translate.rs` 的 `TranslationScope` 只有 `Reasoning`，prompt/report 文本并未接入翻译。c7-004 承诺的"原文/译文面板 + lazy 翻译"未真正达成。
  - 建议：给侧栏设合理 min width；将 prompt/report 纳入 `TranslationScope`。

- **[EXP-03] Warning** — `tui/src/workflow_slash_routing.rs`
  - 症状：`/hw:*` palette 条目数据齐全，但除 `lib.rs:210 pub mod` 外零引用，`command_palette.rs` 不调用 → Ctrl+Space 命令面板里用不到。
  - 建议：在 `command_palette_items()` 注入 `workflow_command_palette_entries()`。

- **[EXP-04] Info** — `tui/src/chatwidget/session_flow.rs:150`
  - 症状：resume 闪烁抑制守卫已接线，但 `suppress_session_configured_redraw` 恒为 `false`，从不启用，闪烁问题实际未被抑制。
  - 建议：确定该在何种 resume 路径置 true，或删除死守卫并改用其他方案；补真实终端烟测。

## Engineering

- **[ENG-01] Warning** — 完成度口径问题（横切 item 5/6/8/9a）
  - 症状：多个 milestone 以"单测/快照通过 + 报告写完"判定 done，但生产入口为死代码/禁用常量/未 wire。`workflow_cards.rs` 是全仓唯一带文件级 `#![allow(dead_code)]` 的**生产源文件**（其余 3 个均为测试支撑文件），是该问题的标志。
  - 建议：milestone 验收增加一条"生产入口可达性"检查（引用计数/调用图），把 test-only seam 与生产接线区分开。

- **[ENG-02] Info** — `rollout/.../official_codex_discovery.rs:120`
  - 症状：只有单向 export+sanitize，无 import/resume。功能正确但范围比 c8-004 讨论的"双向发现"窄。
  - 建议：明确这是最终范围还是阶段成果；若是后者，补 Patch。

- **[ENG-03] Info** — Remote SSH 验收证据缺失
  - 症状：`vscode_cmd.rs` status/probe/rollback 能力齐备，但无已填写的真实验收记录（host/settings 路径/binary/session id）。
  - 建议：补一次实机验收并落证据文件，否则 item 3 长期停在 🟡。

## Risk

- **[RISK-01] Warning** — 范围漂移未被门控
  - 症状：C8-M4/M6 的 prompt-audit 建议（先定策略、验收字段）只留在 review 文档，未回写 prompt validation_path，直接导致 item 4 收窄、item 3 无证据。
  - 影响：发布说明/对外承诺可能与实际能力不符（"官方 Codex 能看到 VSP sessions"实为单向导出）。
  - 建议：对外文案与 release notes 据实修正；将这类 audit 建议固化为 prompt 硬门控。

- **[RISK-02] Info** — 外部 GitHub 命名通道为 test-only seam
  - 症状：`with_release_channel_metadata` 仅测试调用，未接 release/build 元数据；若未来发外部版本，外部命名池不会自动生效。
  - 建议：发布外部 channel 前接线并加生产路径测试。

## Action Queue

| Action | Source | Route | Priority | Validation |
|---|---|---|---|---|
| 更新确认路由到 Workflow 卡片 | EXP-01/ENG-01 | /hw:patch | high | 更新流程弹出 Request Input 卡片而非 UpdatePromptScreen |
| 修 Subagent viewer 侧栏宽度 + prompt/report 翻译 | EXP-02 | /hw:patch | medium | 宽终端出现并排原文/译文；prompt/report 可译 |
| `/hw:*` 注入 command palette | EXP-03 | /hw:patch | medium | Ctrl+Space 能选到 `/hw:*` 条目 |
| resume 闪烁抑制真正启用 + 烟测 | EXP-04/9a | /hw:plan | medium | 真实终端 resume 不闪烁 |
| 滚动锚点/恢复机制 | gap 9b | /hw:plan | medium | 滚动翻阅后 refresh 不跳回开头 |
| 据实修正"官方 Codex 兼容"对外文案 | RISK-01 | manual | high | README/release notes 不再暗示双向自动发现 |
| 真实 Remote SSH 验收并存证 | ENG-03 | manual | medium | acceptance-checklist 填实 |
| update rollback 是否要做 | gap B | /hw:plan | low | 决策记录 |

## Architecture Delta

- 无架构基线破坏。`official_codex_discovery` 作为新单向导出层，与 `utils/home-dir` 的"官方→VSP"方向一致，未引入反向耦合。

## Completion Narrative

- **Change Summary**：本次为只读审计，0 Critical / 6 Warning / 3 Info，结论 PASS_WITH_WARNINGS。
- **Technical Approach**：GQM/25010/ATAM-lite/SWEBOK 四镜头，重点查"完成度名实"。
- **Modified Files / Modules**：未改动代码；审计 tui/app-server/rollout/core/cli 近期改动面。
- **Test Design**：以引用计数、初始化常量、`#[cfg(test)]` 归属、`allow(dead_code)` 分布为取证手段。
- **Validation Results**：见各 finding 的 file:line。
- **Expected Result**：Action Queue 处理后，声称完成的功能具备可达生产入口。
- **Problems Encountered**：先前一份 subagent 核查误判 item 6 为完全未实现；本审计已用直接取证更正为"存在但被削弱"。
- **Risks / Follow-Up**：对外文案与实际能力的一致性（RISK-01）优先级最高。
