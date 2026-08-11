# C10 M5 alpha.4 最终复审

Verdict: `pass`

Status: `superseded`. This pass applies to `upstream/main@f93c18ed0f`; the final
pre-commit fetch discovered production commit `5c19155cbd`, so a new audit is required.

审计基线：branch `c10-upstream-0.145.0-alpha.4`，code HEAD
`7ffab1c7aec9e1323d5990cfc3d211adfe864ed3`，upstream cutoff
`f93c18ed0f57151b410d25e8e1dff4408440560f`。Ptolemy 执行独立只读复审；没有修改
代码、构建目录、远端资源或本机安装。

## Findings

没有 P0、P1 或 P2 finding。复审发现的两项 P3 evidence/state 一致性问题已在进入
发布门前处理：

1. 测试报告曾把 helper coverage 与代码审查写成直接 listener-loss native/fallback
   E2E。报告现已收窄：直接测试覆盖 connection-removal retarget/dismiss helper 与
   missing-listener resolution cleanup；listener 存活时的 native/fallback replay 由
   websocket 测试覆盖。Forced closed-listener replay 仍明确列为 residual test gap。
2. `.pipeline/state.yaml` 顶层步骤已是 `review_code`，但 M5 `current_step` 曾残留
   `implement`。两处现已统一推进到 `release_gate`，避免 Resume 回到已完成步骤。

## Pascal Findings Closure

### P2 Listener-loss card handoff

状态：`closed`

- `request_user_input_handoff.rs` 将 connection removal 收敛到完整 handoff：迁移回答权、
  向 previous owner 发送 `serverRequest/resolved`，并向恢复 owner replay native/fallback
  Question。
- `thread_lifecycle.rs` 的正常 listener 路径和 `thread_processor.rs` 的 listener-loss
  降级路径调用同一 handoff；无法取得 live conversation 时仍至少 retarget 并 dismiss。
- Handoff 与 answer、Resume、unsubscribe、disconnect 共用同一锁，主要竞态顺序保持
  单一胜者。

### P3 Resolution target cleanup

状态：`closed`

- Ask answer 被接受后，listener 正常路径消费 resolution target 并发送 resolved。
- Listener missing、channel closed 或 completion 丢失时，
  `resolve_request_user_input_after_listener_loss` 原子移除 target，并定向通知实际回答端。
- 正常 listener 已消费 target 时，fallback 观察不到 entry，不会重复发送 resolved。

## Verified Scope

- Ask focused unit 5/5、Ask integration 24/24、app-server lib 282/282。
- Full Access defaults 2/2；top-level/request-config 显式权限优先，request config override 1/1。
- Compaction 内部 summary 不可见；Complete、Interrupted、Error 都有清理路径；真实 final
  reply 保持可见。
- `7ffab1c7ae` 未修改 TUI；VSCode file-link、GitLab updater 和 upstream reminder 的既有
  GREEN 证据未受影响。
- alpha.4 后 9 个提交复核为 7 个 exact patch-id、2 个记录完备的 VSP adaptation；
  `7ffab1c7ae` 不改变该结论。
- M5 prompt、Cycle、release notes、测试/发布/upstream 报告均使用 alpha.4、9 commits
  through `f93c18ed0f`，builder 显式传入 alpha.4 provenance。
- Archive 大小 `352361450` bytes，SHA256
  `dc7715f87e85f75bf26b28faf06cb7092061572eeb47cea8186b3c7a70cba05b`；SHA 文件、
  Git LFS pointer、versioned/stable manifest 一致。
- Archive binary 与 release binary 字节一致并报告 `vsp-codex 0.145.0-vsp.9`；stable
  manifest 只有一个 vsp.9 Linux/lab 条目，并保留 6 个历史条目。
- Release notes 与报告明确声明尚未 push、tag、upload、create release 或 install；本地
  不存在 `v0.145.0-vsp.9` tag。
- 代码修复 399 行增删，低于复杂变更 500 行指导线；没有 app-server API/schema breaking
  change。

## Residual Test Gaps

- 没有直接故障注入 listener missing/closed 后再验证 native 与 fallback 的
  unsubscribe/disconnect replay。
- 没有直接覆盖 listener completion 丢失或 answer/disconnect 并发。
- Compaction Error 有生产清理代码，但不像 Complete/Interrupted 那样有直接清理测试。

这些 gap 不构成当前 P0-P2 阻断 finding，但后续修改 Ask listener 生命周期或 compaction
可见性时应优先补齐。

## Release Risks

- 候选仅包含 Linux x86_64 GNU；Windows/macOS 未打包。
- 没有两台真实电脑的跨进程 UI acceptance；不同 app-server 进程不能互相关闭旧卡片。
- Updater 在启动和 stale-cache refresh 时检查，不是常驻实时推送。
- 远端 asset、checksum、GitLab Release、三入口安装和 pipeline 只能在显式发布门后验证。

当前结论支持停在发布确认门；不授权任何远端或本机发布副作用。
