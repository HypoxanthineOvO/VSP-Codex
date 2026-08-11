# M11 Ask UI 卡片式体验重构与本地化

## 状态

completed，时间：2026-05-27T12:58:37+08:00。

## 完成内容

- 为 zh-CN Ask 流程增加中文 chrome：进度、未答数量、Other、讨论入口、键盘提示、未答确认。
- 将 zh-CN option 渲染改为单列边框卡片，区分选项标题与描述。
- 未答确认改为中文 inline panel，列出未答问题，并让默认选中项 `返回修改` 与 Enter 行为一致。
- 修正中文 snapshot 红测中的 compact 断言，使其和实际 ratatui 宽字符快照匹配。
- 保留英文/旧流程的原有 chrome 和快照路径。

## 变更文件

- `codex-rs/tui/src/bottom_pane/request_user_input/mod.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/render.rs`
- `codex-rs/tui/src/bottom_pane/request_user_input/snapshots/codex_tui__bottom_pane__request_user_input__tests__request_user_input_zh_card_hierarchy_tabs.snap`
- `codex-rs/tui/src/bottom_pane/request_user_input/snapshots/codex_tui__bottom_pane__request_user_input__tests__request_user_input_zh_unanswered_confirmation.snap`

## Worker 证据

- test worker：新增 zh-CN 卡片层级与未答确认 snapshot 红测，初始结果 `78 passed, 2 failed`。
- implement worker：已请求但连续超时，关闭后由主代理完成最小生产修复。
- audit worker：只读审计通过，无阻塞发现；残余风险为 CJK 启发式本地化、Other legacy 内部 label 仍保留英文常量、仍需手动验收窄宽度/键盘/secret/mixed-language。

## 验证

- `cd codex-rs && just test -p codex-tui request_user_input_zh_card_hierarchy_tabs_snapshot request_user_input_zh_unanswered_confirmation_snapshot`：2 passed。
- `cd codex-rs && just test -p codex-tui request_user_input`：80 passed。
- `cd codex-rs && cargo insta pending-snapshots --workspace`：No pending snapshots。
- `cd codex-rs && just fmt`：通过。
- `cd codex-rs && just fix -p codex-tui`：通过。

按仓库规则，`just fix` 后未重复跑测试。

## 残余验收

- 仍需要用户在真实 TUI 中验收 Ask UI 视觉、键盘流、Other、未答确认和讨论入口的体验。
- C3 整体验收仍受 M7 真实跨设备 pending Ask resume 验收阻塞。
