---
kind: validation-report
cycle: C26-tui-heading-ci-review
milestone: M5
status: passed
updated: 2026-08-09T23:57:25+08:00
supersedes: rejected report from 2026-08-09T18:44:04+08:00
---

# C26 M5 严格复验报告

## 结论

M5 通过，可以发布候选。C26/C27 选择的行为测试组、协议合同、Cargo/Bazel build
contracts、Clippy、格式、snapshot hygiene 和 GitLab CI 等价 smoke 均无未处理代码失败。S2
随后在真实 100x34 PTY 中通过，证明标题层级、长 token、代码块和最终回复末尾在实际 TUI
中完整可见。

Cargo native bwrap group 在本机因缺少系统开发依赖 `libcap.pc` 未能构建；该失败没有被写成
pass。相同工作树的 Bazel bwrap、linux-sandbox unit 和 integration 三个原生 targets 全部通过，
因此结论是 host prerequisite gap，而不是产品回归。远端发布 CI 仍需保持绿色。

## 变更与技术路径

- app-server v2 `turn/completed` 透传 canonical `lastAgentMessage`，TUI 在 item completion
  缺失或 active stream 未完成时恢复完整 final，不重复正常 completion。
- App thread events 使用单一 FIFO；真实 protocol sequence 经 ImageView、partial delta、canonical
  completion、history consolidation 和 VT100 scrollback 整体比较。
- Heading renderer 将 H1/H2/H3 分离为独立模块：H1 居中、H2 对称 rule、H3 左对齐；
  Setext streaming structural rewrite 触发 canonical reflow。
- 长 token、CJK/宽字符、quote/list container 和 OSC 8 链接按显示列 wrap/remap，不越界。
- test portfolio 改为显式命名组；Cargo/Bazel binary-root parity、compile data、SDK current-binary、
  Linux network denial、permission preapproval、literal remote notification 和 parallel barrier 均有
  可执行合同。
- Classic shell 只有真实可并行 backend 才声明 parallel-safe；ZshFork 保持串行。测试显式关闭
  UnifiedExec，使 `shell_command` model-visible，避免 hidden fallback 伪测试。
- 严格验证额外修复了 doctest、await-held mutex guard 以及 Clippy 拒绝的冗余 clone/borrow/closure；
  均为等价收口或更明确的错误路径。

## 修改模块

- `codex-rs/app-server-protocol`、`codex-rs/app-server`、`codex-rs/app-server-client`
- `codex-rs/core` 的 session、shell handler、permission 与 parallel integration tests
- `codex-rs/tui` 的 App routing、streaming、heading renderer、snapshots 与 focused tests
- `codex-rs/linux-sandbox`、Cargo/Bazel build contracts、CI workflow 与 test-group runner
- Python/TypeScript SDK current-checkout contracts 与 release/package scripts
- C26/C27 Workflow reports、evidence、progress 和 execution records

## 测试设计

- 公开事件到真实终端：`delta -> image -> partial final -> turn/completed(canonical)`，整体断言
  canonical 文本、唯一 tail sentinel、partial 消失、图片记录保留和窄屏 reflow。
- Heading semantic + snapshot oracle：层级、rule 对称性、主题语义、40/72/100 列、长 token、
  CJK、quote/list、OSC 8 范围和 Setext streaming。
- 并行 oracle：两个 tool command 各自创建 marker 并等待 peer marker；串行执行必然 timeout，
  Classic 并行执行必须双方完成。
- Permission oracle：显式 outside permission 预批准后若出现第二次 approval，测试立即失败。
- Linux network oracle：连接真实本机 listener，要求明确 `PermissionDenied`，不接受 DNS/缺命令/
  任意非零退出。
- Build contract：Cargo binary roots、doctests、Bazel package loading 与四个 binary unit-test targets。

## 验证结果

- `just fmt-check`、`git diff --check`：通过。
- selected package `just clippy --tests`：退出 0；仅保留既有 dead-code/too-many-arguments warning。
- protocol group：273/273。
- core focused：permission preapproval 1/1；parallel barriers 2/2；backend capability 1/1；
  workflow plan stale 1/1。
- TUI focused：heading snapshot 1/1；App final VT100 1/1；provider rollback、active subagents、
  translated history、OSC 8、IDE IPC 各 1/1；C7 M8 9/9。
- app-server remote literal notification：1/1。
- rollout：107/107。
- build-contracts：Cargo binary tests/doctests、Bazel package query 和 4/4 Bazel binary-root
  targets 全通过。
- native Bazel sandbox：bwrap unit、linux-sandbox unit、linux-sandbox integration 3/3 targets 通过。
- `just vsp-ci`：build 通过；version/help/exec/resume、interactive TUI、app-server initialize 和
  thread/list 7 个 smoke checks 通过。
- `cargo insta pending-snapshots`：无 pending；无 `.snap.new`。
- real PTY S2：100x34、truecolor、trace logging、`/exit` 状态 0；详细记录见
  `evidence/tui-acceptance/REAL_TUI_ACCEPTANCE.md`。

## 预期用户结果

- 最终回复无需 resume 即显示到最后一个字符；ImageView 或 partial delta 不再导致缺尾/重复。
- H1/H2/H3 在实际终端有明确层级；长标题和长 token 不覆盖 composer/status line。
- CI 测试不会因 sibling checkout、hidden tool、same-source expected/actual 或缺失 binary label
  假装通过。

## 遇到的问题

- 共享 Cargo target 曾把 clean checkout 的 snapshot root 链进当前测试；切换到固定隔离 target
  后消除路径污染。
- 初版 shell barrier 使用 UnifiedExec 模型的 hidden `shell_command` fallback，导致正确串行；
  让 Classic shell model-visible 后真实并行通过。
- build-contracts 首次暴露 `feedback_tags!` doctest 未定义示例变量；补 hidden declarations 后通过。
- strict Clippy 暴露多个既有拒绝项；逐项做等价修复并重跑到退出 0。
- `cargo-insta 1.47.2` 不支持旧 `pending-snapshots -p` 参数；用当前 workspace 命令重跑并确认无 pending。

## 风险与后续

- 本机没有 `libcap.pc`，所以 Cargo native bwrap build 仍需在具备 `libcap-dev` 的 runner 上验证；
  Bazel 原生 targets 已提供当前主机的行为证据。
- active event FIFO 无界；持续生产快于消费仍可能产生内存压力，inactive replay store 继续有界。
- 发布必须是新不可变版本，不能覆盖 alpha.3；commit/tag/manifest/archive SHA 和远端下载必须一致。
