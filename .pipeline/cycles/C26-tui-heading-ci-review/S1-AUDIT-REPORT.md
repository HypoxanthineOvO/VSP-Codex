---
kind: audit-report
cycle: C26-tui-heading-ci-review
stone: S1
status: waiting-review
updated: 2026-08-09T19:49:27+08:00
---

# S1 全仓测试、Snapshot 与 CI 严格审计

## 新结论

当前 C26 不可接受，也没有授权进入 M4。旧报告把 42 个 TUI 共同失败当作 baseline 排除，并把 focused green 视为质量证据；新审计证明问题横跨全仓测试设计与 CI 执行边界。

全仓 inventory 已闭合，源码语义未检查项为 0。结论不是“全部测试都坏了”，而是：大量高价值测试应保留，但当前 pass 数量无法证明它们真实执行过，关键 C26 路径仍是替代/短路径覆盖，且 required Bazel 与 full Cargo 在执行测试前已有硬阻断。

## Findings First

### P0 - Required Bazel CI 当前无法加载

- `tui/BUILD.bazel`、`core/BUILD.bazel` 各重复 `extra_binaries`。
- `exec-server/BUILD.bazel` 传入 macro 不支持的 `unit_test_args`。
- 三个 `bazel query` 已独立复现 load failure；PR blocking Rust 门禁不能到达 tests。

### P0 - Cargo workspace tests 当前无法完整编译

- `install-context` 测试调用旧 5 参数 API。
- `app-server-client` 测试使用旧字段并缺新 question metadata。
- `thread-manager-sample` test harness 手写 Config 缺新字段。
- 默认 bwrap build 还依赖当前机器未安装的 `libcap.pc`；本轮只用官方 skip 继续 inventory，没有宣称该路径通过。

### P1 - 327 个执行失败，没有 baseline 豁免

CI-like 8 MiB stack、零重试、禁止 snapshot 更新的正式运行执行 12,458 个 non-ignored cases：12,131 pass、327 fail、0 timeout、0 stack overflow。另有 29 ignored 和 39 个被 compile blocker 挡住的 tests。

失败归因用于修复分组，不是免责：core integration 214、TUI 42、app-server integration 24、core unit 15、sandbox/network/MCP/exec 及其他 32。

### P1 - pass 数量被系统性 silent-return 污染

- core 仅 `skip_if_no_network!` 就有 795 调用/86 文件，宏普通 return 并被 nextest 记 PASS。
- 其余 crate 至少 81 条默认路径因 sandbox、binary 或 capability 缺失普通返回。
- 许多测试只用 loopback/local mock，本应在 default 中真实执行。
- 29 个 ignored cases 包含多个已知错误、flaky 与真实 OS 行为；不能计入覆盖。

### P1 - same-source 与 mock-only oracle 可假绿

- Python SDK 安装并测试旧 pinned runtime，完全不验证 current checkout。
- TypeScript abort、Windows symlink race、Linux network denial、request-permissions grant、Ask roundtrip 与 parallelism tests 均有确定假绿路径。
- Cloud Tasks HTTP、keyring、analytics delivery、backend client 等真实 seam 只有 mock 或零测试。

### P1 - C26 核心产品路径仍未被真实覆盖

- 最终回复 App test 没有走 `ImageGeneration`、长回复 overflow、完整 event loop 或真实 scrollback。
- `lastAgentMessage` 没有进程级公开 JSON-RPC 验证。
- 23 个 Markdown collector tests 有 22 个走 test-only 替代 renderer。
- 真实 binary/tmux resize tests 3/3 ignored。

### P1 - 外部 Workflow contract 仍在默认 core full

7 个 tests 默认读取 sibling `../Hypo-Workflow` 或扫描源码文本；GitHub/Bazel 没有对应 checkout/env/runfiles。应拆 repo-owned hermetic consumer tests 与固定 release 的独立 mandatory compatibility job。

### P1 - 全局状态与动态 fixture 破坏可重复性

- TUI 仍有 5 个未隔离 theme writes；数百个 ChatWidget 构造也写 global theme。
- cwd、PATH、CODEX_HOME、stdin、OTel、cache 与其他 process-global 状态未统一 RAII/serial 隔离。
- 通用 TUI fixture 动态读取 production bundled model catalog，25 个 snapshots 随 catalog 无关漂移。
- typing、git mtime、negative observation、port rebind 与无 timeout waits 使用真实时间。

### P2 - Snapshot 不是“全坏”，但需要逐项处置

614 个外部 snapshots 中 593 个仍活跃，明确 orphan 21；76 个活跃 snapshot provenance 过期；26 组/53 个 payload 完全重复。两组场景已确认没有观察到声称状态。当前无 `.snap.new`。

旧“38 个 snapshot failure 说明缓存全部失效”的解释不成立；正确做法是删除 21 orphan、刷新 76 metadata、对重复/弱场景补语义 assertion、固定通用 model/time/theme 输入，再保留有独立视觉价值的 golden。

### P2 - CI success 与 current checkout 质量脱节

- GitLab 只 build + smoke 首帧。
- GitHub PR Rust 主门禁是当前无法 load 且漏 115 个 binary tests 的 Bazel；full Cargo 只 postmerge。
- `repo-checks` 固定旧版本和历史远程 artifact。
- nextest retry 与 Bazel flaky retry 会掩盖 first-attempt failure。
- 平台、remote、V8 与 snapshot hygiene 都有明确缺口。

## 技术方法

```text
Cargo metadata / nextest list
  <-> Rust source + helper + fixture inventory
  <-> Bazel macro expansion and query
  <-> GitHub/GitLab job-command-shard mapping
  <-> static anti-pattern upper bounds
  <-> independent per-domain semantic review
  <-> 614 snapshot asset mapping/content duplicate scan
  <-> current-tree zero-retry execution
```

完整数字、逐类证据、snapshot 清单、suite 边界和 CI 映射见 `M2-EXHAUSTIVE-INVENTORY.md`。

## 修改文件与模块

本次 redo 只写入：

- `.pipeline/cycles/C26-tui-heading-ci-review/M2-EXHAUSTIVE-INVENTORY.md`
- `.pipeline/cycles/C26-tui-heading-ci-review/S1-AUDIT-REPORT.md`
- `.pipeline/cycles/C26-tui-heading-ci-review/PROGRESS.md`
- `.pipeline/cycles/C26-tui-heading-ci-review/EXECUTION.md`

未修改 Rust、CI、snapshot、schema、Cargo/Bazel 配置或受保护 state 文件。现有产品/测试改动仍是未验收候选。

## 测试设计与验证结果

- Cargo metadata：131 packages、248 targets、217 test harness targets。
- Nextest list：排除已确认 blocker 后 214 binaries、110 packages、12,487 cases。
- 正式 run：12,458 executed；12,131 pass；327 fail；29 ignored；0 timeout；0 stack overflow。
- 被 blocker 挡住：39 tests + 1 个 0-case sample harness。
- Bazel query：core、tui、exec-server 三处 load blocker 全部复现。
- Snapshots：614 external + 19 inline；21 orphan；76 stale provenance；0 pending `.snap.new`。
- CI：29 workflows、11 actions、94 GitHub jobs、1 GitLab job全部映射。
- 非 Rust：33 files、308 logical cases；95 tracked cases 不在默认 blocking CI。

## 预期修复结果

M4 应分 reviewable stages 完成：

1. 先恢复 truthful execution：修 Cargo/Bazel blockers、binary dependencies 和 target parity。
2. 将 default suite 变为 hermetic 且不 silent-pass；把 remote/OS/cross-repo 能力拆成 mandatory jobs。
3. 重写 same-source、weak、mock-only 与 global-state harness；固定 model/time/theme/env。
4. 为 C26 建立真实 `ImageGeneration -> long canonical completion -> App FIFO -> VT100 overflow/resize` 合同及公开 app-server JSON-RPC 合同。
5. 清理 orphan/duplicate/stale snapshots 并增加显式 pending-snapshot hygiene。
6. 让 Python/TS SDK 与 package checks 测试 current checkout/current artifacts。

M5 仍保持原严格标准：相关 package/CI 零未豁免 failure；baseline 只允许归因，不允许验收。

## 遇到的问题

- 第一次全仓 nextest 执行漏设 CI 规定的 8 MiB stack，产生 840 个 stack abort；已明确作废并用正确环境重跑。
- 默认 workspace list 先后暴露系统 libcap 前置与三个源码 test-build blockers；使用 exclude 仅为完成其余 inventory。
- 一个 reviewer 最初留下 458 个 core tests 未做语义签核；主线程拒绝该“完成”状态，重新拆分，最终源码语义未检查项归零。
- 正式 run 生成 3 个 `codex-rs/core/.tmp*` 目录；安全 hook 要求用户授权删除，当前未清理且未绕过。

## 风险与后续

- 327 failures 包含环境、fixture、真实 assertion 和现有产品问题，需要 M4 按 root cause 去重；不能机械地写 327 个补丁。
- 平台源码已审，但 macOS/Windows/Wine/remote/provider 行为仍需各自 mandatory dynamic lanes。
- 全部修复明显超过 800 行，应按 CI truth、suite boundary、oracle、C26 E2E、snapshot/SDK 五个以上独立阶段落地。
- 不应把所有 contains/field assertions 或固定 synthetic literals 批量删除；独立协议 fixture 和 whole-object assertions 仍是高价值覆盖。

## S1 推荐决定

建议接受本报告的审计结论与分阶段修复方向，但不接受当前产品/测试状态。接受后进入 M4 第 1 阶段，仅修复 build/load truth 和 target 依赖；每一阶段独立复验并更新 remaining finding inventory。拒绝则继续修订报告，不修改源码。

## 已失效历史报告

以下内容是 2026-08-09 14:20 的旧 bounded audit，已被本报告取代，仅保留审计轨迹，不再构成范围或修复授权。

# 旧 S1 TUI 标题与最终回复完整性严格审计

## 结论

当前结果不能接受为“标题与最终回复完整性已解决”。版本未更新已排除；clean/current 双基线证明当前标题 diff 没有新增现有全套测试失败，但 reviewer 发现的用户可见缺陷位于现有测试没有覆盖的路径。CI 绿色与用户体验正确在此处不是同一件事。

## Findings First

### P1 - 最终回复缺尾仍有确定恢复缺口

- Core 的 TurnComplete 持有完整 final text；app-server 的 `TurnCompletedNotification` 丢弃该字段并发送空 items；TUI 最终固定收到 `last_agent_message = None`。
- 若 canonical `ItemCompleted` 缺失、延迟或未正确作用于 live widget，TUI 会把 partial delta 当最终答案，而 rollout 仍完整。
- 当前运行版本为 alpha.3；最近三次 completed final 的 rollout 与 TurnComplete 长度分别精确匹配 `854/854`、`490/490`、`312/312`。因此版本和持久化不是主要问题。
- 图像生成 started/completed 会按 streamed source 主动 flush，且没有 image + partial delta + full item + turn completed 的组合测试。

### P1 - Setext H1 的生产流式输出结构错误

- `Setext title\n` 先作为一行 plain paragraph 稳定，后到 `===\n` 重渲染成三行 H1 时无法替换已发出的 index 0。
- 结果会缺 top surface 并重复标题；final canonical render 与 live stream 不同。
- 当前 stream tests 通过 test-only collector 直渲染，不经过生产 controller，因此全绿仍会漏掉该错误。

### P2 - 标题仍有三个高置信布局缺陷

- URL-only/不可断词标题可超过 terminal width，H1/H2 都受影响。
- `10. # Title` 的 H1 title row 与 top/bottom surface 相差一列。
- `1. > # Title` 的 inserted surface 会把 Quote background 涂到 rail 前的 list indent。

### P2 - 测试可信度不足

- suffix/mismatch tests 直接调用内部 finalize，只检查 queued event/`contains`，不执行 App consolidation、reflow 或 VT100 scrollback。
- “三主题 x 三宽度”快照实际只重复静态 Style 字符串，没有渲染对应 palette/width。
- 多个 heading tests 修改进程级全局 theme，没有 guard/restore；存在并行竞争。
- 窄屏 H1 不检查原标题是否完整、顺序正确且恰好一次。
- Workflow 5 个失败依赖硬编码的外部 sibling `Hypo-Workflow` 路径，是非 hermetic 测试缺陷；它们是 baseline，但不是合理的 CI 合同。

## 技术路径

```text
M4 RED tests
  -> real ServerNotification sequence
  -> ordered thread routing
  -> ChatWidget reconciliation
  -> App InsertHistory/Consolidate/Reflow
  -> VT100 terminal full-text equality

heading RED tests
  -> production StreamController chunks/ticks
  -> full renderer deep equality
  -> URL/list/quote/theme/OSC8 width matrix
```

推荐先写公开事件到 VT100 的 executable RED，再选择 TurnCompleted canonical fallback 或 thread/read reconciliation 的最小合同；同时修复 thread channel 的 ordered backpressure。标题侧先修 Setext stable-prefix 策略、不可断词标题和 prefix/surface 宽度，再替换静态/竞争测试。

## 修改文件与模块

### 本次审计写入

- `.pipeline/cycles/C26-tui-heading-ci-review/`：Plan、Progress、Execution、M2 reviewer 报告与本 S1 报告。
- `.pipeline/cycles/INDEX.md`、本地 Session focus。

### 尚未修改

- S1 前没有因审计修改任何 Rust 源码、测试或 snapshot。
- 当前工作区中的 heading implementation/tests/snapshots 是进入 C26 前已完成的待审改动。
- 未接受 snapshot、commit、push、package、install、deploy 或 restart。

## 测试设计与验证结果

### 严格双基线

- clean：detached `v0.145.0-vsp.11.1-alpha.3@cd7921547e`，独立 source/target。
- current：同一 HEAD 加当前 heading diff，独立 source/target。
- 固定：`INSTA_UPDATE=no`；`just test` 实际设置 `RUST_MIN_STACK=8388608`、nextest local profile；同一 Rust/Cargo 1.95.0、cargo-insta 1.47.2、just 1.51.0。
- clean 两次：3181 run、3134 passed、47 failed、4 skipped。
- current 两次：3187 run、3140 passed、47 failed、4 skipped。
- 两侧 exact 47 failure IDs 相同；规范化 JUnit failure fingerprints 相同；当前新增 6 tests 全部通过。
- `cargo insta pending-snapshots`：无 pending snapshots。
- `git diff --check`：通过；没有 `.snap.new`。

### 被废弃的实验

- 首轮复用同一 Cargo target 导致 current test binary 读取 clean snapshot root，制造 3 个伪 heading failures；该结果已废弃，未用于归因。
- 这个问题证明 worktree A/B 必须隔离 target，不能只保证测试串行。

### Clippy

- 非 `--no-deps`：被 clean rollout 模块 2 个现存 deny lint 阻断。
- `--no-deps`：TUI 仍有 8 个现存 lint；没有诊断落在新 headings 模块，但结果不是 pass。

## 预期结果

M4 完成后必须满足：rollout canonical 完整时，live VT100 与 transcript source 都完整且 tail sentinel 恰好一次；image boundary、missing/late/duplicate/out-of-order item、resize/mismatch 均有确定行为。标题的 production streamed/final 输出必须深相等，所有行不超过可用宽度，list/quote prefixes 不泄漏主题 surface。

## 遇到的问题

- 旧 C23/C25 记录只有失败总数，没有 exact set/JUnit，无法直接证明 baseline。
- 共享 target 污染 compile-time workspace root，第一次 A/B 无效。
- 现有 helper-level tests 不能生成真实 terminal RED；按 S1 前只读边界，executable RED 将作为 M4 第一项源码变更。
- `cargo insta pending-snapshots -p codex-tui` 在本机 cargo-insta 1.47.2 不支持 `-p`；改在 crate 目录运行无参数命令，结果为零 pending。

## 风险与后续

- Thread channel 32768 满载后 per-event spawned send 的错序是确定设计风险，但尚未证明本次运行达到满载；不得把它写成唯一根因。
- TurnCompleted fallback 涉及 app-server/TUI v2 通知合同，需保持跨平台和远端 server/client 分离。
- 当前 heading diff 约 714 行，继续混入 Workflow test cleanup 会超过合理审阅范围。
- 两个独立 test targets 各约 17 GiB，clean worktree 约 3.2 GiB；S1 接受前保留以支持复验，尚未删除。

## S1 推荐决定

接受本审计并进入 M4，但将范围限制为：

1. 最终回复真实 event-to-VT100 RED 与修复，包括 image boundary；
2. ordered thread routing / canonical completion recovery 的最小必要合同；
3. Setext、URL overflow、ordered-list/quote prefix 标题修复；
4. 替换本轮范围内的弱 oracle 与全局 theme 竞争测试；
5. 47 项 baseline 和 5 个 Workflow non-hermetic tests 记录为独立后续 Cycle，不在 C26 批量清理。
