---
kind: progress
cycle: C26-tui-heading-ci-review
plan: PLAN.md
status: closed
updated: 2026-08-10T01:17:05+08:00
current: complete
next: 无；alpha.5进入alpha channel使用与soak
---

# C26 TUI 标题与回复完整性 CI 审核进度

## 当前状态

M4/M5与S2真实体验均已完成。alpha.4已在双GitLab pipeline绿灯后不可变发布，远端asset与SHA验证通过；本机安装暴露Official DB migrations 1-46与VSP 1-40的兼容超集被session-compat脚本过严拒绝。修复版已在live数据上发现并带备份neutralize 63个历史rollout，复核0待处理；当前准备alpha.5恢复发布，不移动alpha.4 tag。

## 任务状态

| ID | 阶段 | 状态 | 当前证据 | 下一步 |
| --- | --- | --- | --- | --- |
| `M1` | 聚焦 C26、冻结证据与双基线 | `completed` | clean worktree 固定为 `cd7921547e`；当前标题 diff 为 tracked 205+/66- 加新增文件 443 行；运行版本、rollout 和工具链已记录 | 已完成 |
| `M2` | 全部测试与 CI 的 exhaustive test-quality audit | `completed` | 131 packages、217 Cargo harnesses、12,526+ Rust cases、614 snapshots、29 workflows/94 jobs；源码语义未检查项 0 | 已完成 |
| `M3` | 主线程复核 inventory、重现与处置分类 | `completed` | 3 Cargo build blockers、3 Bazel load blockers；12,458 cases 实跑为 12,131 pass / 327 fail；suite 边界已分类 | 已完成 |
| `S1` | 人工审核 exhaustive audit 报告 | `completed` | 用户接受新报告；不接受当前产品/测试状态，并要求先暂停 | 已完成 |
| `M4` | 修复重新确认的测试设计与产品缺陷 | `completed` | 全workspace test binaries build/list成功；Bazel四个binary-root target 337/337；真实E2E、Linux、Python/TS current artifact与snapshot均通过；发现并修复ZshFork parallel声明和Python effort wire drift | 已完成 |
| `M5` | 零失败严格复验 | `completed` | 相关命名组、protocol 273、rollout 107、Clippy、build-contracts、Bazel native sandbox、fmt/snapshot与`just vsp-ci`通过；Cargo native bwrap缺`libcap.pc`明确记录为host gap | 已完成 |
| `S2` | 最终真实体验验收 | `completed` | 真实100x34 PTY验证H1/H2/H3、长token、代码块与末尾完整；`/exit`状态0 | 已完成 |
| `R1` | 不可变发布、安装与恢复验证 | `completed` | alpha.5 `07b1fff4bc`；pipeline 358；remote/install SHA与entrypoints通过；session status 0 pending | 已完成 |

## 时间线

| Time (Asia/Shanghai) | State | Detail |
| --- | --- | --- |
| 2026-08-09 12:43 | started | 创建 C26，继承 C23 渲染基线与 C25 alpha.3 CI 基线；限定为标题 diff 审核，不含发布、安装或无关基线修复。 |
| 2026-08-09 13:52 | confirmed | 用户确认严格 CI 归因与多 subagent 交叉审核；Plan 纳入真实回复截断、测试可信度、双基线与两个人工 Stone。 |
| 2026-08-09 13:58 | baseline | 冻结 clean alpha.3 detached worktree、当前标题文件哈希/规模、工具版本与共享 target 串行测试策略；M1 完成，进入 M2。 |
| 2026-08-09 14:10 | audited | 四路只读 reviewer 完成；共享 target 暴露 manifest/snapshot root 污染，废弃 50/47 归因并改用独立 target；M2 完成，进入 M3。 |
| 2026-08-09 14:20 | waiting-review | clean/current 独立 target 双跑稳定，47 exact failures 与 fingerprints 相同；M3 完成，S1 审计报告等待人工判断。 |
| 2026-08-09 16:43 | accepted | 用户接受 S1，并明确扩大 M4 到 5 个 non-hermetic Workflow tests；其余 42 个 baseline failures 保持独立后续边界。 |
| 2026-08-09 17:39 | implemented | M4 完成：canonical final fallback、有序事件 relay、Setext reflow、heading 边界、hermetic Workflow fixtures 与真实 App/image/VT100 测试均已实现并 focused GREEN；进入 M5。 |
| 2026-08-09 18:11 | waiting-review | M5 完成：最终 TUI 全量稳定为 42 个共同 baseline failures，无新增失败；check、focused、schema、snapshot/diff hygiene 通过，Clippy 保持既有 2+8 lint。进入 S2，等待真实候选启动授权与体验验收。 |
| 2026-08-09 18:44 | rejected | 用户拒绝 M5/S2：核心“全部测试与 CI 的硬编码/弱 oracle 审计”没有完成；仅比较失败集合和抽查 snapshot 不合格。C26 回退 M2，旧 M5 报告失效，重新建立 exhaustive inventory。 |
| 2026-08-09 19:49 | waiting-review | redo audit 完成：源码语义未检查项归零；131 Cargo packages、217 harnesses、12,526+ Rust cases、614 snapshots、29 workflows/94 jobs 全部对账。正式零重试运行 12,458 cases，12,131 pass / 327 fail；另有 3 Cargo build blockers、3 Bazel load blockers、29 ignored。新的 S1 报告等待用户判断。 |
| 2026-08-09 20:00 | accepted-paused | 用户接受新的 exhaustive audit，但明确要求先暂停。接受范围仅为审计结论与分阶段方向；当前产品/测试状态未获验收，M4 修复未开始。 |
| 2026-08-09 21:32 | resumed | 用户明确恢复C26并要求完成/发布；Session聚焦C26，进入M4.1。发布仅在M5/S2通过后执行。 |
| 2026-08-09 22:23 | implemented | M4.1-M4.3完成：修复Cargo fixture drift、Bazel load/compile_data、错误binary与115个binary-root target遗漏；新增build-contracts/SDK/Linux sandbox命名组；重写same-source notification、permission preapproval、parallelism与network-denial弱oracle。C26 App长final已通过ImageView、FIFO、canonical completion、72/38宽VT100 overflow/reflow并生成已审snapshot。 |
| 2026-08-09 22:58 | milestone-complete | M4完成。全workspace test binaries在隔离target成功build/list；Bazel CLI/state/app-server/exec binary-root 337/337。真实执行发现ZshFork虽串行却声明parallel-safe，并修为仅Classic支持parallel；barrier tests通过。Python pinned schema保持可重现，同时effort兼容max/ultra/future值；全套128通过、38个real-provider显式skip，current-checkout 44/44；TypeScript current binary 5/5。进入M5。 |
| 2026-08-09 23:50 | milestone-complete | M5完成。修复共享target snapshot污染、hidden shell fixture、doctest与strict Clippy拒绝项；protocol 273/273、rollout 107/107、相关core/TUI/app-server组、build-contracts、Bazel native sandbox与`just vsp-ci`全部通过。Cargo native bwrap因本机缺`libcap.pc`保持显式host prerequisite gap。 |
| 2026-08-09 23:57 | accepted | S2真实TUI通过：100x34 PTY中H1居中、H2双侧rule、H3左对齐，长token安全换行，代码块和`End marker.`完整，composer不重叠；trace无错误，`/exit`状态0。进入不可变发布。 |
| 2026-08-10 00:46 | release-recovery | alpha.4在pipeline #354/#355绿灯后发布，archive SHA为`fc6361bc...fab26cd`并成功安装；安装器随后发现Official migration 46相对VSP 40的兼容尾部被exact-equality guard误拒。修复为共同migration prefix严格一致+所需schema列检查；单测9/9、Ruff通过。live status发现63项，备份至`~/.vsp-codex/session-compat-backups/20260810-004550`后全部neutralize，复核0待处理。准备新不可变alpha.5。 |
| 2026-08-10 01:17 | closed | alpha.5 pipeline 356-358通过；候选、package/source hash、远端整包SHA、GitLab Release与公开installer通过。本机VSP/compat入口均为alpha.5，installed smoke 7/7，session status 0 pending / 1927 neutral。C26关闭。 |
