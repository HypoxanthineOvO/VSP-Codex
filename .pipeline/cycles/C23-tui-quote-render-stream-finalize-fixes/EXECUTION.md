# C23 Execution

## Current Step

C23 completed。用户已在 Nod 真实 SSH TUI 确认 Quote 正确并接受最终结果。

## Checkpoints

- `M1`：定位 Quote fill、完成时全量 reflow 与长上下文闪烁的根因，已完成。
- `M2`：修复 `AgentMessageCell` wrap 后 blockquote fill 丢失并补单测。
- `M3`：仅在流式期间实际发生 resize 时执行完成阶段全量 reflow。
- `M4`：收窄长上下文的完成/加载重排范围并记录性能证据。
- `M5`：实现已确认的 H1-H4、隐藏井号和宽度感知 U+2500 thematic break，并补多主题快照。
- `M6`：修复最后一个未换行 chunk 或 held/live tail 在 finalize/consolidation 期间未完整显示的问题。
- `S1`：展示真实 TUI 产物与性能证据，等待用户接受或拒绝。

## M2 Checkpoint

- 在 `AgentMessageCell::display_hyperlink_lines` 保留现有 adaptive wrap；wrapped row 含带 blockquote 背景的 U+2502 rail 时，补回被 trailing-space trim 裁掉的背景 fill。
- 新增 `streaming_agent_blockquote_background_fills_the_chat_viewport`，通过 ratatui Buffer 逐格断言 40 列视口中第 2 列到右边缘均保留 Quote 背景。
- 验证：`cargo test -p codex-tui --lib streaming_agent_blockquote_background_fills_the_chat_viewport -- --nocapture`，1/1 通过。

## M3 Checkpoint

- 删除仅由 live table tail 触发的 `ConsolidationScrollbackReflow::Required` 与 deferred cell 特殊合同；final tail 统一先通过 FIFO `InsertHistoryCell` 落盘。
- Agent 与 Proposed Plan consolidation 统一调用 `maybe_finish_stream_reflow`，只有 `take_stream_finish_reflow_needed()` 记录到真实 resize 时才清屏重排。
- 验证：plain/live-table completion 2/2，`TranscriptReflowState` 11/11 通过。

## M4 Checkpoint

- 现有 initial/thread-switch replay 与 resize renderer 已按 row cap 只格式化 transcript 最近后缀，startup 上限为 400 行；M3 又消除了常规 completion 对全部 transcript 的重渲染和重插。
- 保留每个 newline commit 的全源 Markdown parse，因为 blockquote、list、fence 和 table 状态可跨 delta 延续；本轮不引入不安全的字符串 suffix parser。
- 验证：capped resize 1/1、initial/thread-switch replay buffer 4/4 通过。

## M5 Checkpoint

- H1 使用终端明暗感知的紫色背景与粗体，并在有 wrap width 时补满当前容器宽度；H2 为紫色粗体下划线，H3 为亮紫粗体，H4 为紫色斜体普通字重。
- H1-H6 不再渲染 Markdown `#`；thematic break 改用 `subtle_border` 颜色的 `─`，宽度扣除 list/blockquote prefix。
- 测试通过纯 palette 构造验证 Teal/Violet/Amber，不切换全局主题，避免并行测试污染。
- 验证：Markdown renderer 110/110、Markdown stream collector 23/23 通过；三主题 × 40/80/120 快照与复杂 Markdown 快照已审阅接受。

## M6 Checkpoint

- 根因：完成事件携带完整 canonical assistant message，但旧逻辑在 stream controller 存在时将其视为冗余并忽略；最后 delta 缺失时，当前 scrollback 因此只保留截断 source，而恢复会从完整 rollout 重建。
- 修复：finalize 前比较 canonical visible message 与 collector source。canonical 仅多 suffix 时将缺失部分补入现有 controller；中间分歧时 consolidation 改用 canonical source，并只为该异常分支强制一次 source-backed scrollback repair。
- 正常 exact/suffix 路径继续遵守 M3：无 resize 时不全量 reflow。
- 验证：新增 suffix/mismatch 回归 2/2，既有 flush 2/2、TranscriptReflowState 11/11 通过。

## S1 Review Report

### 结论

C23 的六项实现目标均已完成，专项测试全绿，diff 与 snapshot hygiene 通过。当前状态为 `waiting-review`；需要用户在真实 TUI 中确认视觉与交互体验后才能接受并关闭 Cycle。

### 技术方案与用户可见变化

- Quote：`AgentMessageCell` wrap 后识别带 Quote 背景的 rail，并把背景补到聊天视口右边缘，流式提交行与最终 Markdown cell 一致。
- 完成刷新：最终 tail 先按 FIFO 插入；仅在流式期间真实 resize 或 canonical source 中途分歧时做 source-backed reflow，普通完成和表格 tail 不再清屏重插全部 transcript。
- 长上下文：保留现有 startup/resume/resize 最近 400 行 row cap；普通 completion 不再全量渲染、清屏和重插历史。
- 标题与横线：H1-H6 隐藏 `#`；H1 为终端明暗感知紫色背景粗体并填满容器，H2 紫色粗体下划线，H3 亮紫粗体，H4 紫色斜体；thematic break 使用 `subtle_border` 的 U+2500 并扣除 list/blockquote prefix。
- 最终消息完整性：completion canonical visible message 与 streamed source 对账；缺尾时只补 suffix，分歧时以 canonical source consolidation 并执行一次异常修复，避免必须退出恢复才能看到完整末尾。

### 修改模块

- 流式与 consolidation：`chatwidget/streaming.rs`、`streaming/controller.rs`、`app/agent_message_consolidation.rs`、`app/event_dispatch.rs`、`app/resize_reflow.rs`、`app_event.rs`、`transcript_reflow.rs`。
- Markdown/视觉：`markdown_render.rs`、`markdown_stream.rs`、`style.rs`、复杂 Markdown snapshot。
- Quote cell：`history_cell/messages.rs`。
- 测试：`history_cell/tests.rs`、`markdown_render_tests.rs`、`chatwidget/tests/status_and_layout.rs` 与 controller/collector 既有测试。

### 测试设计与结果

- Quote 背景逐 Buffer cell 覆盖：4/4 通过。
- Markdown renderer：110/110 通过；collector：23/23 通过；streaming controller：45/45 通过。
- status/layout（含 M3、M6 事件顺序和 canonical suffix/mismatch）：138/138 通过。
- TranscriptReflowState：11/11 通过；resume replay/cap：7/7 通过。
- 三主题 × 40/80/120 标题/Rule 合同快照与复杂 Markdown 快照已审阅接受。
- `cargo fmt --all` 成功；`git diff --check` 成功；pending snapshots 为 0。
- 全库 `just test -p codex-tui`：3125 passed、47 failed、4 skipped。47 项均不在 C23 行为面，集中于既有 status/keymap/plan-panel/update 快照、workflow fixture、resume 参数和旧 table-border 断言；未接受这些无关快照。

### 问题与风险

- `just fmt` 的 Bazel/Starlark 子步骤因本机缺少 `dotslash` 未完成；没有安装系统依赖，Rust 格式化已单独成功。
- 尚未替用户在真实交互式 TUI 中观察终端模拟器 scrollback、长会话主观流畅度与真实 provider 的 delta/completion 缺尾场景；这是 S1 人工审阅内容。
- canonical source 中途分歧会进行一次全量 repair，这是保证当前屏幕与持久 rollout 一致的异常兜底；正常 exact/suffix 路径不触发全量 reflow。

### 人工验收清单

1. 连续输出多段 Quote，确认流式中、完成后和 resize 后绿色背景都延伸到内容区右边缘。
2. 完成普通文本和 Markdown 表格，确认屏幕不清空、不跳回顶部。
3. 在长会话中恢复、resize 并完成新回答，确认只显示最近历史且无整体闪烁。
4. 检查 H1-H4 层级、隐藏井号，以及 40/80/120 列下全宽 `─` 不越过 Quote/list 容器。
5. 观察长 final answer 的最后几行，确认完成后无需退出恢复即可看到完整末尾。

## S1 Acceptance Checkpoint

- 2026-08-08：首个 Nod 候选的 Buffer 测试虽通过，但真实 SSH TUI 截图证明 Quote 背景在 terminal scrollback 写入后仍于文本末尾中断。
- 根因是 `insert_history` 对已适配宽度且带背景 fill 的行再次执行 adaptive wrap，裁掉带样式的尾随空格。
- 新增 120 列中文 Quote VT100 回归，当前实现 RED 于第 67 列；修复后 Quote VT100 3/3、insert_history 17/17、streaming/finalized Quote 2/2 和既有 chatwidget VT100 snapshot 通过。
- Nod `current` 已切到 release `c23-cbdf855cb899`，binary SHA-256 为 `cbdf855cb8995a89139d7f1ee1a38e4ff04deed9904fcd8f6915d1cb67788a90`；app-server initialize、config/read、model/list、thread/read 与 control socket 验证通过。
- Nod 配置已隔离为默认 VSP GPT-only catalog（GPT-5.6 372000）与显式 DeepSeek profile/catalog（DeepSeek V4 1048576）；`基础建模` rollout 的错误有效窗口 996147 已等长修正为 353400并保留备份。
- 用户确认真实 Quote 已正确，接受并关闭 C23。
- 全库 47 个既有失败不在 C23 内批量修快照；用户决定在 C24 完成后单独开 Cycle，大幅删除低价值测试与冗余 CI，只保留必要契约。

## M5 Visual Contract Review

2026-08-08 用户通过分层标题与 thematic break Mock，并最终确认：

- H1-H4 均不显示 Markdown `#` 标记。
- H1 使用内容宽紫色色带与粗体；H2 使用紫色粗体下划线；H3 使用亮紫粗体；H4 使用紫色斜体。
- 独立 `---` 不显示三个破折号；renderer 按当前内容容器可用宽度重复 U+2500 `─`，使用弱化边框色。
- H3/H4 不只靠颜色区分，还使用粗体与斜体/普通字重；40/80 列 Mock 中层级可辨。

当前生产实现仍主动为标题添加 `#`，并将 `Event::Rule` 固定渲染为 `"———"`；M5 将替换这两条行为并增加多主题、多宽度、流式/完成和嵌套容器快照。

## VSPi Session Tool Audit

2026-08-08 对独立 Pi session `019fd785-079a-76db-a186-1082662caf38` 做了工具级复核：

- 14:49 与 14:58 的两次 write 均只写入临时诊断文件 `tui/src/diag_quote_tmp.rs`；14:50 仅在 `tui/src/lib.rs` 挂载该诊断模块。
- 15:19:03 唯一的生产 edit 是在 `AgentMessageCell::display_hyperlink_lines` 中调用尚未定义的 `restore_blockquote_fill()`；没有 helper、测试或 M3/M4 实现。
- 15:19:07 用户明确要求先停止修复、只完成定位和 Workflow 记录。
- 15:19:29 与 15:23:55 会话两次执行生产文件回滚；15:23:57 的最终 `git diff --stat` 只剩 `.pipeline/INDEX.md`。
- 全量 Pi session 搜索只有这一条命中 C23 或 `restore_blockquote_fill`；8 月 6 日后无触及 `codex-rs/tui` 的 commit；现有 stash 是 7 月 10 日旧同步；其他 worktree 不含 C23 完成版。
- 当前 `git status -- codex-rs/tui` 和 `git diff -- codex-rs/tui` 为空，临时诊断文件与 `restore_blockquote_fill` 均不存在。

结论：VSPi 完成了 M1 诊断，并短暂写过一个不可独立编译的 M2 调用点，但按用户要求全部回滚。没有证据支持把 M2-M4 记为已完成。

## Predecessor Session Clarification

后续按用户提供的“VSP Codex + DeepSeek”线索找到前置 rollout `019fc1ba-5883-7e93-bc2c-f33abfdfbcca`：

- 会话时间为 2026-08-02 至 2026-08-03，工作目录为当时的 `/home/heyx/VSP-Codex`。
- 它实现了 Resume replay 限流/合并 redraw、Quote 竖线和绿色背景，以及后续 Subagent UI；对应 commit `4aa6f52248`（`fix(tui): stabilize resume replay and restyle quotes`）已包含在当前 HEAD。
- 该会话是 C23 的前置能力实现，不是 8 月 6 日登记的 C23 后续回归修复：会话结束早于 C23 诊断，且没有 `restore_blockquote_fill`、完成阶段 Required reflow 收窄或标题/Rule 新合同。
- 8 月 7 日结束的另一条 Pi session `019fd7ac-68d0-7fc6-b6c0-7d4a034e5549` 修改的是独立 TypeScript 项目 VSPi 的 `C09-ui-rendering-fixes`，对应 commit `1301b3a`；没有写入 VSP-Codex Rust TUI。
- 当前未提交文件全部位于 `.pipeline`；`codex-rs/tui` 为零差异。C22 worktree 删除前只有一个未跟踪 release 压缩包，没有 C23 源码。

最终完成边界：前置 Quote/Resume 能力已完成并在 HEAD；C23 M1 后续回归诊断已完成；C23 M2-M5 尚未实现。

## 定位过程与证据

### 复现方法

在 `codex-rs/tui` 添加临时诊断测试，同一宽度（终端 40 列、markdown wrap 38 列）分别走两条渲染路径：

1. **流式路径**：`StreamController` 逐 delta push + commit tick 收集稳定行（`AgentMessageCell`）与 tail（`StreamingAgentTailCell`）。
2. **完成路径**：`AgentMarkdownCell` 从同一 markdown 源重新渲染。

对每行统计 blockquote 背景覆盖列数（`covered/total`）。

### 问题 1：blockquote 底色在输出完成时丢失

- **现象**：输出中途（tail 区）blockquote 上下空行都有完整底色；输出完成或行提交进 scrollback 后只剩文字覆盖部分有色块。
- **复现输出**（`"> xxxxx\n"`，宽度 40）：
  - 流式已提交行：`covered=1/3 text="  │"`、`covered=7/9 text="  │ xxxxx"`（fill 丢失）
  - 最终 AgentMarkdownCell：`covered=38/40`（完整）
- **根因**：`history_cell/messages.rs` 的 `AgentMessageCell::display_hyperlink_lines` 对 controller 已渲染（fill 到 `width-2`）的行，用 `RtOptions::new(width)`（完整宽度）**重新 wrap**。`wrapping.rs` 的 `word_wrap_line` 使用 `wrap_ranges_trim`，textwrap 裁剪行尾 trailing whitespace → blockquote 的 fill 空格被删除 → 背景只在文字 spans 上。
- **对照**：`StreamingAgentTailCell` 直接 `prefix_hyperlink_lines`（不重新 wrap），所以 tail 显示完整；`AgentMarkdownCell` 从源重新渲染并 fill，也完整。只有中间态的 `AgentMessageCell` 破坏 fill。
- **相关测试**：`history_cell/tests.rs::agent_markdown_blockquote_background_fills_the_chat_viewport` 只覆盖最终 cell，未覆盖流式中间态，因此未拦住回归。

### 问题 2：输出完成时全量刷新滚动到顶部

- **根因**：`chatwidget/streaming.rs::flush_answer_stream_with_separator` 在 `controller.has_live_tail()` 为真（表格 holdback 场景常见）时标记 `ConsolidationScrollbackReflow::Required`；`app/agent_message_consolidation.rs::finish_agent_message_consolidation` 的 Required 分支无条件走 `finish_required_stream_reflow` → `clear_terminal_for_resize_replay`（清 scrollback + 可见屏）+ `render_transcript_lines_for_reflow` 从源重插全部 transcript_cells → 滚动位置重置、内容回到顶部。
- **设计意图**：Required 是为了修复「流式期间发生过 resize」导致的旧宽度行；但当前实现不检查 `transcript_reflow.take_stream_finish_reflow_needed()`，导致每次有 live tail 的完成都全量重排。
- **修复方向**：Required 分支先检查 `take_stream_finish_reflow_needed()`；为 false（无 resize）时把 deferred cell（`rendered[emitted_stable_len..]`，未写入 scrollback 的行）直接 `insert_history_cell_lines` 写入，不重插稳定区；为 true 时才走全量 reflow。

### 问题 3：长上下文加载/完成闪烁

- **根因**：问题 2 的每次完成全量重排是主要卡顿源（长对话时 `render_transcript_lines_for_reflow` 渲染并重插大量行）；此外流式期间 `StreamCore::push_delta` 对每个换行 delta 全量重渲染 `raw_source`（`recompute_streaming_render`），长输出时每 delta 开销随累计源线性增长。
- **修复方向**：M3 消除常规完成全量重排；M4 评估每 delta 全量渲染收窄（复用稳定前缀缓存或增量渲染）与 resume replay 范围。

## 验证日志

- RED（定位阶段）：诊断测试显示流式已提交行 blockquote fill 丢失（`covered=1/3`）。
- 基线：现有 `agent_markdown_blockquote_background_fills_the_chat_viewport` 测试通过（仅覆盖最终渲染）。
- 诊断临时文件与半成品修改已回滚，工作区恢复干净。

## 风险与待确认

- M2 若改为「不再重新 wrap」会改变 `AgentMessageCell` 在 resize reflow 中的换行行为（现有测试 `streamed_agent_list_paragraph_preserves_item_indent_when_wrapped` 依赖 wrap）；优先评估「wrap 后恢复 fill」。
- M3 需确认 overlay（Ctrl+T）场景下 deferred 行的落盘路径仍正确。
