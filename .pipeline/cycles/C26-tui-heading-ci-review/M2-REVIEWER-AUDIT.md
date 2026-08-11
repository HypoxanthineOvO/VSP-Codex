---
kind: audit-report
cycle: C26-tui-heading-ci-review
milestone: M2
status: completed
updated: 2026-08-09T14:10:32+08:00
---

# M2 四路独立 Reviewer 审计报告

## 结论

四路审计一致否定“现有绿色专项测试足以证明最终回复完整、标题实现可进入发布”的结论。alpha.3 已运行且 rollout 持久文本完整，问题不是简单的版本未更新；真实 app-server 通知、TUI channel、history consolidation 与 terminal scrollback 仍缺端到端合同。标题 diff 还存在至少一个高置信流式结构错误和多项宽度/容器/测试隔离缺陷。

## Findings First

### P1 - TurnCompleted 丢失最终文本兜底

- Core `TurnCompleteEvent.last_agent_message` 持有完整最终文本，但 app-server 构造 `TurnCompletedNotification` 时不携带它，并发出 `items_view: NotLoaded`、空 items。
- TUI 因此固定调用 `on_task_complete(None, ...)`。若 canonical `ItemCompleted` 缺失、延迟或未作用于当前 widget，partial delta 会被直接固化，rollout 仍可保持完整。
- 当前 Session 最近已完成 final 的 rollout/TurnComplete 长度精确匹配，说明持久化侧完整；没有证据证明 terminal live path 完整。
- 证据：`core/src/tasks/mod.rs:781`、`app-server/src/bespoke_event_handling.rs:1321,1556`、`tui/src/chatwidget/protocol.rs:233`。

### P1 - Thread channel 满载 fallback 可能破坏顺序

- 活跃 thread channel 容量为 32768；满载时每个事件独立 `tokio::spawn(sender.send(event))`。
- spawned task 不提供 delta、`ItemCompleted`、`TurnCompleted` 的协议顺序保证；TUI 又没有按 turn/item ID 对 delta 流做重排。
- 该风险能解释 live 屏幕错误而 store/replay 正确，但尚无日志证明本次默认容量确实耗尽。
- 证据：`tui/src/app.rs:246`、`tui/src/app/thread_routing.rs:924`、`tui/src/chatwidget/protocol.rs:76`。

### P2 - 图像生成边界提前 flush streamed source

- 图像生成开始与完成都会调用 `flush_answer_stream_with_separator()`，使用 `FinalAnswerSource::Streamed`，没有 canonical 修补。
- 在事件延迟/错序下，图像边界可先固化缺尾 prefix，后到 delta 再开启孤立 stream。
- 当前没有“image started/completed + partial deltas + full ItemCompleted + TurnCompleted”的真实 App/VT100 测试。
- 证据：`tui/src/chatwidget/tool_lifecycle.rs:25,30`、`tui/src/chatwidget/streaming.rs:24`。

### P1 - Setext 标题生产流式输出结构错误

- 对 `Setext title\n` 的首个 delta，生产 controller 会把 plain paragraph 立即稳定；后续 `===\n` 将完整源重渲染为三行 H1，但已稳定的 index 0 无法替换。
- 结果为缺少 top surface、重复 title；final canonical render 与 live transcript 不同。
- 现有 modified stream tests 使用 `#[cfg(test)]` collector 直渲染路径，绕过生产 `StreamController`。
- 证据：`tui/src/streaming/controller.rs:147,321,373`、`tui/src/markdown_stream.rs:122`。

### P2 - 标题宽度与容器合同不完整

- URL-only 标题沿用 `break_words = false`，H1/H2 中间行可超过 viewport；现有长标题测试只覆盖可按空格换行文本。
- `10. # Title` 中 initial marker prefix 比 continuation prefix 多一列，top/bottom surface 为 19 列而 title row 为 20 列。
- `1. > # Title` 的 inserted surface 会把 quote background patch 到 rail 前的 list indent。
- 证据：`tui/src/wrapping.rs:508`、`tui/src/markdown_render/headings.rs:104,194,217`、`tui/src/markdown_render.rs:918`。

### P2 - 主题与 snapshot 测试存在弱 oracle/全局竞争

- suffix/mismatch 测试直接调用内部 finalize，只检查 queued `AppEvent`/`contains`，不执行 App consolidation/reflow 或 VT100 scrollback。
- 所谓三主题 x 三宽度快照只重复输出静态 `MarkdownStyles`，没有按各主题/宽度渲染真实 `Text`。
- 多个测试直接修改全局 `ACTIVE_UI_THEME`，未使用现有 guard + serial 模式；并行结果不能直接归为无关 baseline。
- 窄屏 H1 只断言行宽/背景，不断言原标题完整且恰好出现一次。
- 证据：`tui/src/chatwidget/tests/status_and_layout.rs:346,388`、`tui/src/markdown_render_tests.rs:190,226,284,307,317`。

### P2 - CI/Clippy 旧归因不够严格

- clean alpha.3 的本地全量 TUI 已固定为 3181 run、3134 passed、47 failed、4 skipped；旧报告只有总数，没有 exact failure set，不能单靠同为 47 证明集合相同。
- 5 个 Workflow tests 依赖外部 sibling `Hypo-Workflow` 路径，不是 hermetic CI fixture；应明确标为测试隔离缺陷。
- 非 `--no-deps` Clippy 先被 rollout 2 个 deny lint 阻断；`--no-deps` 到达 TUI 后仍有 8 个现存 lint。没有诊断落在新 headings 模块，但 Clippy 结果仍不是 pass。
- `just test` 使用本地 nextest profile，不等价于 CI pinned nextest、`ci-test` profile、archive shards 和 sandbox helper。

## 测试可信度处置

### 保留

- heading OSC 8 精确列范围测试。
- `centered_heading_system_snapshot`，但必须移除全局主题竞争并补真实 theme matrix。
- Quote 逐 Buffer cell 与 120 列 VT100 background regressions。

### 替换或升级

- internal suffix/mismatch event tests：升级为真实 `ServerNotification -> App -> Tui -> VT100` 整体比较。
- static style x width snapshot：替换为各 palette、各 width 的真实 rendered `Text`。
- 手写 consolidation backward-walk 测试：改测生产 helper 与 App consolidation。
- narrow H1：断言去除 padding 后完整标题恰好一次，并加入 URL/CJK/emoji/long token。

## M3 验证目标

1. 使用独立 `CARGO_TARGET_DIR` 重跑 clean/current，比较 exact failure IDs 与失败指纹；共享 target 已证明会污染 `insta` workspace root，不可用于结论。
2. 对图像生成、partial delta、full item completion 和 turn completion 建立真实事件序列，验证 rollout source、transcript source 与 VT100 tail sentinel。
3. 对 Setext H1 使用生产 controller chunk/tick 组合，深比较 streamed/final lines、styles 与 hyperlinks。
4. 对 URL-only、ordered-list 10+、list > quote 容器补最小可复现证据。

## 修改与副作用

- Reviewer 未修改源码、测试、snapshot 或 Workflow。
- 主线程仅创建 clean detached worktree 和测试日志；未接受 snapshot、安装、重启、推送或发布。

## 风险

- channel 满载错序是设计风险而非本次运行已证根因；需要可控容量测试验证。
- `TurnCompleted` fallback 与 thread/read reconciliation 涉及 app-server/TUI 合同，修复范围需在 S1 明确后进入 M4。
- 当前标题变更约 714 行，接近 800 行审阅上限；修复应保持模块化并避免混入无关 CI 清理。
