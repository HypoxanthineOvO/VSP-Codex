---
kind: execution
cycle: C26-tui-heading-ci-review
status: closed
updated: 2026-08-10T01:17:05+08:00
current: complete
---

# C26 Execution

## Current Step

C26已完成：M4修复、M5严格复验、S2真实PTY、alpha.5不可变发布、本机安装和session compatibility恢复均通过。

## M1 Checkpoint

- 验证 CLI、app-server、TUI 运行版本与实际二进制路径。
- 对照当前 Session rollout 的最终 assistant text 与用户可见 TUI 症状。
- 建立 clean alpha.3 隔离 worktree；记录当前 diff 精确文件与 numstat。
- 冻结双基线命令、环境变量、工具版本、test-thread 策略和 snapshot hygiene 规则。

### M1 Result

- clean worktree：`/tmp/vsp-codex-c26-alpha3`，detached `v0.145.0-vsp.11.1-alpha.3@cd7921547e`，无源码改动。
- 当前标题 diff：tracked 6 files、205 insertions、66 deletions；新增 implementation/tests/snapshot 共 443 行，总审核规模约 714 行。
- 工具链：Rust/Cargo 1.95.0、cargo-insta 1.47.2、just 1.51.0、Linux x86_64。
- 双基线串行复用当前 99 GiB Cargo target；不并发运行会写 snapshot 的测试。
- 当前 Session rollout 的上一条 final assistant message 为完整 490 字符记录；不保存或复制正文到审核报告。

## M2 Checkpoint

- 四个 reviewer 分别审查协议/终端路径、测试可信度、CI 归因、标题渲染。
- Reviewer 不改源码、不运行 snapshot accept、不把 C23/C25 结论当作事实前提。
- 每份报告必须列 findings、文件/行号、可复现证据、测试缺口和置信度。

## M3 Checkpoint

- 主线程核对 reviewer 冲突并验证高风险结论。
- 使用真实 protocol notifications 驱动最小重现，比较 rollout canonical 与 VT100/terminal scrollback 完整输出。
- clean/current 相同环境运行失败集合差分，禁止仅凭历史报告标记 baseline。

### M2 Result

- 协议 reviewer：确认 TurnCompleted 丢弃 core final text fallback；识别 channel-full spawned sends 的顺序风险和 image boundary streamed flush。
- 测试 reviewer：确认 suffix/mismatch tests 未执行 App/VT100；M3 completion tests 不证明无清屏；image + final answer 没有组合覆盖。
- CI reviewer：确认 47 旧归因缺 exact set；定位 5 个非 hermetic Workflow tests 与 Clippy 两层失败。
- 标题 reviewer：确认 Setext production streaming divergence、URL overflow、ordered-list width mismatch、quote-prefix pollution、theme race 与静态 snapshot 弱 oracle。
- 共享 target 首轮 current run错误地读取 clean snapshot root，产生 3 个伪新增失败；该实验已废弃，改用独立 `/tmp/c26-current-target`。

## S1 Checkpoint

- 展示 findings-first 报告、技术路径、涉及模块、测试可信度矩阵、验证结果、问题、风险和修复建议。
- 等待用户接受或拒绝；拒绝后保留审计记录并将相关 Milestone 恢复为 `in_progress`。

### M3 Result

- clean 独立 target 两次均为 3181 run、3134 passed、47 failed、4 skipped。
- current 独立 target 两次均为 3187 run、3140 passed、47 failed、4 skipped。
- 47 exact failure IDs 和规范化 JUnit fingerprints 完全相同；本轮新增 6 tests 全部通过。
- pending snapshots 为零，diff whitespace 通过，没有 `.snap.new`。
- final-message 恢复缺口与 Setext production streaming divergence 已由真实 call path 证明；为遵守 S1 前只读边界，executable RED 尚未写入，将作为 M4 第一项。

## M4-M5 Checkpoints

- 仅修 S1 确认问题；测试优先经过公开事件路径和最终 terminal 输出。
- 重跑 focused/full/check/Clippy/snapshot/diff matrix，并用 clean/current 集合差分归因。

### S1 Acceptance

- 2026-08-09 用户接受 `S1-AUDIT-REPORT.md` 的结论：当前实现不能视为已经解决最终回复完整性和标题边界问题。
- 用户选择 expanded scope：M4 额外修复 5 个依赖外部 sibling `Hypo-Workflow` 的 non-hermetic tests。
- 其余 42 个 clean/current 共同 baseline failures、发布、安装、远端部署与重启仍不在本次授权范围。
- M4 必须从 executable RED 开始，不接受无关 snapshot。

### M4 Result

- `turn/completed` v2 notification 新增 nullable `lastAgentMessage`，app-server 从 core `TurnCompleteEvent` 透传 canonical final text；schema 和 README 已同步。
- TUI 仅在 item-level completion 缺失或仍有 active stream 时消费 fallback，并触发 canonical mismatch reflow；正常 `ItemCompleted` 路径不会重复最终回答。
- thread event channel 改为 direct unbounded FIFO；send 后同步可见、非阻塞且不再由多个 spawned send 竞争顺序，inactive replay store 仍按 capacity 截断。
- 真实 App protocol sequence 覆盖 `delta -> image item -> partial final -> turn/completed fallback`，最终 transcript 经 VT100 验证包含图片记录、完整末尾且无 partial 残留/重复。
- Setext 后续行重解释已发出前缀时，production `StreamController` 记录 structural rewrite 并要求 canonical reflow。
- Heading 使用允许长 token 分割的 wrap 路径，同时通过 `remap_wrapped_line` 保留 OSC 8 destination；新增长 URL、长单词、quote/list、3 themes x 3 widths 语义断言。
- Workflow slash tests 全部使用 staged Host Contract fixture；command palette unit test 只测试传入 projection，不再读取 sibling checkout。
- 删除两个已无引用的静态/重复 snapshot；保留 complex UI snapshot 和行为断言。

### M4 Focused Validation

- `cargo test -p codex-tui --test all workflow_slash_routing`: 11 passed。
- heading focused matrix: 10 passed；Setext structural rewrite: 1 passed。
- canonical fallback、ordered channel、App-to-VT100 regression: 各 1 passed。
- app-server canonical producer: 1 passed。
- `just write-app-server-schema`: passed；`cargo insta pending-snapshots`: none；`git diff --check`: passed。

### M5 Result

- 最终 TUI 全量两次均为 3191 run、3149 passed、42 failed、4 skipped；42 exact IDs 与 M3 共同 baseline 相同，恰好减去已修复的 5 个 Workflow failures。
- `cargo check -p codex-tui -p codex-app-server -p codex-app-server-protocol` 通过；protocol 68 unit + doc tests 通过。
- focused final bundle 通过：canonical fallback 2、App-to-VT100 1、ordered FIFO 1、active-side immediate delivery 1、Setext plain/blockquote 1、Workflow routing 11、app-server producer 1。
- app-server 全量为 1008 run、985 passed、23 failed、1 skipped；失败属于缺 binary、MCP/approval timeout、provider/plugin/model/permission branch fixtures，未触及 canonical producer。
- Clippy 本轮新增 lint 已清零；仍保留 M3 已记录的 app-server 2 个与 TUI 8 个既有 lint，结果不是全绿。
- 无 pending snapshots；38 个历史 `.snap.new` 均 reject；fmt check、`git diff --check` 通过。
- 初版 async relay 的 immediate-visibility regression 与初版 Setext 全量 clone 性能问题均在 M5 中发现并修正。

## S2 Rejection Checkpoint

- 用户指出 C26 明确要求检查全部测试和 CI 是否存在硬编码、同源 oracle、mock-only、弱断言、环境依赖和无效 snapshot。
- 旧 M2 reviewer 只做 bounded/变更相关审计；主线程只比较全量失败 ID/fingerprint 并抽查部分 snapshot，没有逐个检查全部通过/失败测试，也没有建立 CI job 到 test target 的完整映射。
- 因此“没有新增失败”不能推出“测试可信”或“M5 通过”；`M5-VALIDATION-REPORT.md` 已标记 rejected。
- 旧假设“其余 42 个共同 baseline failures 可排除在 C26 验收外”被撤销。新的 M5 门槛是相关 package/CI 零未豁免失败；任何豁免必须逐项说明 owner、理由和期限并由用户确认。
- 返回 Milestone：`M2`。S1/M4/M5/S2 重新 pending；既有源码改动保留但不视为已验收。

## M2-M3 Exhaustive Redo Result

- 完成 131 Cargo packages、248 targets、217 test harnesses、1,686 Rust test/support sources、33 非 Rust test files、614 external snapshots、19 inline snapshots、29 GitHub workflows、11 actions、94 GitHub jobs 和 1 GitLab job 的全量 inventory。
- 最初 reviewer 留下 core A-M 458 tests 未逐项语义签核；主线程拒绝该完成状态并重新拆分。最终 core scope reconciliation 为 287 files、3,476 test functions、3,489 logical cases，源码、fixture 与 support implementation 未检查项 0。
- `cargo nextest list --workspace` 首先被系统 `libcap.pc` 前置阻断；使用 `CODEX_SKIP_BWRAP_BUILD=1` 后又暴露 `install-context`、`thread-manager-sample`、`app-server-client` 的 test-build drift。排除已确认 blockers 后列出 12,487 Rust cases。
- 三个 required Bazel load blockers已动态复现：TUI/Core重复 `extra_binaries`，exec-server传入不支持的 `unit_test_args`。
- 正式 current-tree run 对齐 CI 的 `RUST_MIN_STACK=8388608`，设置 `CI=1`、`INSTA_UPDATE=no`、`--retries 0`：12,458 executed、12,131 passed、327 failed、29 ignored、0 timeout、0 stack overflow。
- 一次漏设 8 MiB stack 的先行诊断产生 840 个 stack abort；该结果作废，没有进入 S1 结论。
- Snapshot audit：614 external assets 中 593 active、21 definite orphan、76 stale provenance；26 groups/53 assets exact duplicate；0 `.snap.new`。
- 报告明确区分 default hermetic tests、需要重写 oracle/harness 的 tests、以及 remote/OS/cross-repo mandatory suites。独立 suite 不是豁免；缺 prerequisite 必须 infra failure 或 runner-visible skip。
- 写入 `M2-EXHAUSTIVE-INVENTORY.md`，并在 `S1-AUDIT-REPORT.md` 顶部写入新报告；旧 S1 保留为失效历史附录。
- 本轮未修改 Rust、CI、snapshot、schema、Cargo/Bazel配置或受保护state文件。正式run生成3个 `codex-rs/core/.tmp*` 目录；安全hook要求用户授权删除，未绕过。

## New S1 Checkpoint

- 当前 verdict 是“接受审计结论，但不接受产品/测试状态”。
- 等待用户接受或拒绝新的 exhaustive audit。
- 接受后只进入 M4 第 1 阶段：修复 Cargo/Bazel build/load truth、binary dependencies 和 target parity；其余阶段必须独立 review/validation。
- M5 标准保持零未豁免 failure；baseline 仅用于归因，不能用于验收。

## New S1 Acceptance

- 用户接受新的 exhaustive audit 结论，但不接受当前产品/测试状态。
- 用户选择“接受报告，但先暂停”。本轮没有进入 M4，也没有修改 Rust、CI、snapshot、Cargo/Bazel 或受保护 state。
- 已接受的事实：required Bazel 与 full Cargo 有执行前 blocker；正式 current-tree 零重试运行仍有 327 failures；大量 PASS 被 silent-return、same-source/mock-only oracle 和环境依赖削弱；C26 真实端到端路径仍缺覆盖。
- 恢复后第一步限定为修复 Cargo/Bazel build/load truth、错误 binary 依赖和 target parity。后续 suite boundary、oracle/global state、C26 E2E、snapshot/SDK 仍需分阶段审核。
- 3 个测试生成的 `codex-rs/core/.tmp*` 目录仍保留；用户没有授权删除。

## S2 Checkpoint

- 展示真实长回复末尾和标题主题/窄屏产物，等待用户验收。
- package、install、remote deploy 或 service restart 不由本 Plan 自动授权。

## Resume Checkpoint

- 用户解除暂停并要求C26完成，真实验收后该发布则发布。
- C27已接受并关闭：required CI、14个行为测试组、22个snapshot清理、3个无价值test删除、SDK abort/binary修复和4个external Workflow contracts manual化作为当前工作区事实保留。
- C26不重跑无目的full upstream suite；M4-M5使用C27 manifest选择相关组，运行产生的相关失败必须修复。
- 当前从M4.1继续：Cargo/Bazel build/load truth、binary dependencies与target parity。
- 发布是M5/S2之后的后置动作，不将未验收工作区直接发布。

## 已知证据

- 当前 CLI、app-server 和 Session TUI：`0.145.0-vsp.11.1-alpha.3`。
- 当前 HEAD/tag：`cd7921547e89860db41419a08012983a16f513f5` / `v0.145.0-vsp.11.1-alpha.3`。
- 当前 Session 创建时记录 alpha.2，但已由 alpha.3 binary resume；rollout 上一条最终回复完整持久化。
- C23 suffix 测试直接调用内部 finalize 并断言 `contains`/event source，尚无真实协议顺序到 terminal scrollback 的整体证据。
- 标题源码 diff 未安装；已有 focused 结果不能替代本轮独立复验。

## 执行约束

- 不为追求全绿修改无关模块或批量接受无关快照。
- 不改变长上下文性能行为或 Provider 功能。
- 不执行外部副作用；不修改受保护 Workflow state 文件。

## M4 Implementation Checkpoint

- M4.1 build/load truth：修复`install-context`旧API、`app-server-client`旧fixture字段、`thread-manager-sample`缺失Config字段；Cargo 39个相关tests通过。Bazel重复`extra_binaries`和缺失`unit_test_args`已修，全仓旧`codex`/`codex-rs` test binary引用改为`vsp-codex`。
- Bazel target parity：`codex_rust_crate`现在为每个Cargo binary生成unit-test harness；CLI main、state logs client、app-server main与exec main共115个既有tests不再遗漏。core跨package goal templates已加入`compile_data`。
- M4.2 portfolio truth：C27 runner/manifest补齐自测、direct-argv/shell/secret-env校验和确定性环境；新增`build-contracts`、`linux-sandbox`、`sdk-python`组。外部/平台能力保持显式prerequisite，不把缺失能力报告为默认通过。
- M4.3 oracle：remote notification改为literal wire fixture；permission grant出现二次approval立即失败；parallel使用确定性barrier和backend capability断言；Linux network denial改为本机已监听socket并要求明确权限错误。Ask roundtrip和Windows outside victim已由当前候选中的强断言覆盖。
- M4.4 product contract：App真实事件序列覆盖ImageView、delta、canonical `lastAgentMessage`、有序FIFO、72/38宽reflow和10行VT100 overflow；tail sentinel在两种屏幕均唯一保留，partial final被替换。
- M4.5 snapshot：隔离`/tmp/c26-current-target`生成并审阅long-final VT100和Teal/40 heading visual snapshots；共享target写到clean worktree的结果已作废且未复制。
- M4.6 SDK：TypeScript组构建并注入current VSP binary；Python新增`CODEX_PATH`低优先级override、current-checkout进程合同和命名组。继承SDK Bazel workflow改用真实`vsp-codex`label。

该实现检查点时尚未进入M5；发布、安装、service restart均未执行。

## M4 Completion

- 全workspace `cargo nextest list --workspace --list-type binaries-only`在隔离target成功完成，覆盖约200个Cargo harness binaries；M3记录的三个test-build blocker已归零。
- Bazel四个新增binary-root targets通过：CLI 215、state 3、app-server 74、exec 45，共337个tests；同时修正了一个从未被Bazel执行过的`codex resume` stale expectation。
- Parallel oracle首次强制barrier时揭示真实产品问题：Classic shell可并行，ZshFork单会话串行但handler仍宣称parallel-safe。产品已改为仅Classic声明并行；sync/shell barrier和backend capability unit test通过；冗余mixed timing case删除。
- Python current-checkout model/list揭示pinned schema不识别`max`/`ultra`。生成器现在保持pinned-runtime重现合同和现有enum API，同时接受`max`、`ultra`和未来非空effort；generator drift和current process均通过。
- M4最终证据：Python全套128 pass / 38 explicit real-provider skip；Python current 44/44；TypeScript abort current-binary 5/5；Linux loopback denial 1/1；permission preapproval 1/1；remote literal notification 1/1；C26 App/heading snapshots各1/1且无`.snap.new`。

M5开始；仍未发布、安装或重启。

## M5 Final Validation

- 相关命名组全部真实执行：protocol 273/273、rollout 107/107、core permission/parallel/
  plan-stale、TUI heading/final-VT100/cleanup regressions、app-server remote notification 均通过。
- `build-contracts` 的 Cargo binary tests、doctests、Bazel query 与四个 binary-root targets 全通过；
  首次发现并修复的 `feedback_tags!` doctest 现在可编译。
- selected core/TUI/app-server-client/linux-sandbox/CLI Clippy 退出0；严格 lint 暴露的 await-held
  guard、redundant clone/borrow/closure 与 test helper `expect` 已按等价语义修复。
- `just vsp-ci` 完整通过：build、CLI help/version、interactive TUI 和 app-server smoke 共7项。
- Cargo native bwrap group 因本机缺 `libcap.pc` 显式失败；Bazel bwrap、linux unit 与 integration
  三个原生 targets 随后全部通过。前者记录为 host prerequisite gap，不伪装为pass。
- `just fmt-check`、`git diff --check`、workspace pending-snapshot scan通过，无`.snap.new`。

## S2 Real TUI Acceptance

- 按`test-tui`流程使用真实PTY、`RUST_LOG=trace`、Cycle专用log_dir与临时`CODEX_HOME`。
- 100x34 truecolor终端中，H1居中、H2双侧rule、H3左对齐；长token安全换行，Rust代码和
  `End marker.`完整可见，composer/status line没有覆盖 transcript。
- 文本与Enter分开发送；`/exit`正常退出，process status 0；trace无ERROR/panic/fatal。
- S2通过。用户已明确要求“该发布发布、该完成完成”，进入下一不可变版本发布和安装。

## Release And Closure

- alpha.4在GitLab pipelines 354/355绿灯后发布；真实安装暴露Official migration 46相对
  VSP 40的兼容尾部被exact-equality guard误拒。
- 修复要求共同migration prefix逐项同checksum，并显式检查`threads`/`backfill_state`所需列；
  共同前缀漂移和缺列继续硬失败。
- Live迁移先备份两个DB，neutralize 63个历史rollout；复核0 pending。alpha.4不移动，恢复版
  alpha.5使用新commit/tag/release。
- alpha.5 release commit/tag均为`07b1fff4bc`，pipeline 358通过；远端archive SHA-256为
  `55945d913660978bbfd177abcde04eac90925bfba7ee53affc4fe77b2f1d3a30`。
- 公开installer成功安装alpha.5；session migration返回0 pending / 1925 compatible，随后installed
  binary状态为0 pending / 1927 neutral；两个本机入口和7项package smoke通过。
