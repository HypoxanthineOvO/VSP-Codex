# C8 VSCode Remote SSH Session Compatibility And Plugin UX Progress

更新时间：2026-06-07T18:12:53+08:00

## 当前状态

- 当前 Cycle：C8 `VSCode Remote SSH Session Compatibility And Plugin UX`
- 类型：feature
- Preset：tdd
- 阶段：lifecycle_release
- 当前 Milestone：C8 completed；GitLab release published for `v0.134.0-vsp.6`
- 当前步骤：C9 post-release correctness backfill 已验证，未发布
- 已完成 Milestone：M1, M2, M3, M4, M5, M6
- Worker Separation：strict；每个执行 Prompt 均预分配 `test` / `implement` / `audit`
- Gate 策略：C8 已完成并进入发布准备。用户已确认 GitLab commit/tag/push/release 备份发布范围；GitHub 外部发布仅准备草稿，仍需单独确认。破坏性操作、系统级依赖安装和未授权外部副作用继续保留确认/授权门禁。
- Prompts：6
- Release：`v0.134.0-vsp.6` 已发布到 GitLab fork `heyx/Codex`，commit `2070ca9d2fb678076522b6b4391d0a99d3756cc8`。C8 preview archive `.pipeline/release/C8-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz`，SHA-256 `666ee5841238914d200a496339d5d103b220fb933484defd3b4299fe959fa3bc`。GitHub 外部发布未执行。
- C9 follow-up：`0.134.0-vsp.7` 当前仍是本地未提交/未发布工作区。已补齐用户指出的遗漏：GitLab/local Subagent 默认命名使用 VSP Lab 内部名字；Plan/SubPlan 计数分离且无 active SubPlan 时不再混合 root/subplan；structured SubPlan active work 会参与 Plan freshness；Hypo-Workflow `.pipeline` 状态文件变化会触发 `update_plan` freshness reminder。
- Patch 轨道：P001 `VSCode wrapper pins VSP home` 已关闭。`vsp-codex vscode install` 现在写入 `$HOME/.vsp-codex/vscode/vsp-codex` wrapper，启动真实 CLI 前强制 `CODEX_HOME=$HOME/.vsp-codex`，避免官方 VSCode 插件继承 `$HOME/.codex` 造成 provider 配置混用。
- Session 兼容迁移：用户明确授权后，已直接修正 `$HOME/.codex/sessions` 中 43 个历史 session 文件的 `session_meta.payload.model_provider` 非官方值；备份位于 `$HOME/.codex/session-provider-backups/20260607T174417+0800`。全量验证结果：`OpenAI` 2347、缺失 129、`TTAPI-Codex` 0、`vsp` 0，JSONL 解析无错误。
- State DB 兼容补丁：继续排查 `Model provider `` not found` 后发现 official `$HOME/.codex/state_5.sqlite` 仍有 4 条空 `threads.model_provider` 缓存。已备份 SQLite 并更新为空值到 `OpenAI`；official DB 现在 `OpenAI` 1829、空 0，VSP DB 空 0。
- Patch 发布：P001 已推送到 GitLab `origin/main`，并打 tag `v0.134.0-vsp.7.1`。远端 `main` 和 tag 解引用均指向 `b403d14a999fd2896b7033aed3f7874983643c0d`。

## Patch 轨道

| Patch | 状态 | 严重级 | 发现于 | 摘要 |
| --- | --- | --- | --- | --- |
| P001 | closed | normal | Patch | VSCode wrapper pins VSP home |

## Integration Sync

2026-05-30T12:38:16+08:00：C18 Hypo-Workflow integration sync 开始。范围限定在已确认文件清单内，同步 `/hw:audit`、`/hw:quality`、`/hw:optimize` helper coverage、ordinary conversation capture 候选分类、目标文档记录和 focused tests；不复制源仓库运行态 `.pipeline/state.yaml`、`.pipeline/cycle.yaml` 或 `.pipeline/log.yaml`。

2026-05-30T12:50:38+08:00：C18 Hypo-Workflow integration sync 的 Codex-VSP 侧适配完成并通过验证：`cargo fmt --check` 通过（仅既有 stable rustfmt warning），`cargo test -p codex-tui workflow_slash_routing` 9 项通过，`cargo test -p codex-core workflow_conversation_capture` 10 项通过，`cargo test -p codex-core workflow_routing` 10 项通过，`git diff --check` 通过。

## 初始范围

本 Cycle 关注 VSCode Remote SSH 下 VSP-Codex session 兼容与插件体验优化。

包含：

- Remote SSH VSCode server Machine settings 优先级与诊断。
- 官方 VSCode 插件黑盒协议能力矩阵。
- VSP provider/session resume 兼容契约，修复空 provider resume 失败。
- 官方 Codex 发现 VSP sessions 的安全策略。
- 插件侧 Request Tool/Input、authorization-intent 和 fallback Markdown UX。
- Markdown Workflow surface 与真实 Remote SSH 插件端到端验收。

当前 P3 产物：

- `.pipeline/prompts/c8-001-remote-ssh-vscode-integration-probe.md`
- `.pipeline/prompts/c8-002-vscode-plugin-protocol-capability-map.md`
- `.pipeline/prompts/c8-003-provider-resume-compatibility-contract.md`
- `.pipeline/prompts/c8-004-session-discovery-strategy-official-codex.md`
- `.pipeline/prompts/c8-005-request-tool-request-input-plugin-ux.md`
- `.pipeline/prompts/c8-006-markdown-workflow-surface-remote-ssh-acceptance.md`

## Recent Activity

### Patch

- 2026-06-07T18:12:53+08:00：P001 patch update 已推送。`origin/main` 从 `8cc0394c80` 推进到 `b403d14a99`，新增 GitLab tag `v0.134.0-vsp.7.1` 指向 `fix(P001): pin VSCode launch to VSP home`；本地 session/SQLite 迁移记录未包含在该 tag。
- 2026-06-07T17:55:03+08:00：补齐 provider 兼容迁移的 SQLite 缓存层。`$HOME/.codex/state_5.sqlite` 中 4 条空 `threads.model_provider` 已备份后更新为 `OpenAI`，`pragma integrity_check` 为 `ok`；JSONL 复扫仍无空/`vsp`/`TTAPI-Codex` provider。
- 2026-06-07T17:47:32+08:00：按用户明确授权完成历史 session provider 兼容迁移。已备份 43 个受影响 JSONL 到 `$HOME/.codex/session-provider-backups/20260607T174417+0800`，仅移除 `session_meta.payload.model_provider` 中的 `vsp` / `TTAPI-Codex` 字段；复扫确认官方不支持的 provider 已清零，全量 `jq` 解析 `stderr_lines=0`。
- 2026-06-07T17:15:00+08:00：P001 closed。修复 VSCode official extension 继承 `CODEX_HOME=$HOME/.codex` 启动 VSP CLI 的路径；`vscode install` 现在写入 `$HOME/.vsp-codex/vscode/vsp-codex` wrapper 并固定 VSP home。验证通过：`codex-cli --test vscode` 14/14、focused wrapper regression、`cargo fmt --check --package codex-cli`、scoped `git diff --check`、debug `vsp-codex` build；本机远端 VSCode Machine settings 已刷新到 wrapper 路径。

### C8

- 2026-05-30T13:37:59+08:00：创建 C8 planning state，主题为 `VSCode Remote SSH Session Compatibility And Plugin UX`，执行策略为 strict Worker Separation，P4 之前要求 Subagent prompt audit。
- 2026-05-30T13:43:31+08:00：完成 C8 P2/P3 产物生成：新增架构记录、P2/P3 checkpoint 和 6 个执行 Prompt。下一步必须完成 `.pipeline/reviews/C8/prompt-audit.md` 后进入 P4 确认。
- 2026-05-30T13:50:47+08:00：Subagent prompt audit 完成，结论 `pass`，无 P4 前阻塞项。C8 进入 P4 pending confirmation。
- 2026-05-30T14:15:02+08:00：用户通过 Request Input 卡片确认 C8 P4。C8 状态推进到 `ready_to_start`，执行尚未开始，下一步可 `/hw:start` 从 M1 开始。
- 2026-05-30T14:17:54+08:00：用户启动 `/hw:start`。已创建执行租约并进入 M1 `Remote SSH VSCode Integration Probe` 的 `write_tests`，后续按 strict Worker Separation 自动推进。
- 2026-05-30T14:22:43+08:00：M1 `test` worker 孟轲完成并关闭。新增 both-exist Remote SSH settings priority 红测，主代理复跑 `cargo test -p codex-cli vscode -- --nocapture` 确认 8 passed / 2 failed，失败点为 resolver 仍选择 local user settings；M1 进入 `implement`。
- 2026-05-30T14:26:51+08:00：M1 `implement` worker 孔丘完成并关闭。生产实现将 VSCode settings resolver 调整为优先 `.vscode-server/data/Machine/settings.json`；主代理 green validation 通过：`cargo test -p codex-cli vscode -- --nocapture` 10 项、`cargo fmt --check --package codex-cli`、`git diff --check`。
- 2026-05-30T14:30:43+08:00：M1 只读 audit worker 孔丘 the 2nd 返回 `pass` 并关闭。已写 `.pipeline/reports/016-remote-ssh-vscode-integration-probe.md`，M1 标记 completed。按连续执行策略自动进入 M2 `VSCode Plugin Protocol Capability Map`。
- 2026-05-30T14:42:10+08:00：M2 `test` worker 孟轲 the 2nd 完成并关闭。已写 `.pipeline/reviews/C8/M2/test.md`，确认 Remote SSH 插件目录与日志存在，app-server/approval/permission/plan/tool/collab 处理有证据；`nativeAskUi` 未在 initialize payload 中发现，因此原生 Request Input 标为 gated/unknown，Markdown Ask fallback 是安全默认。M2 进入 `implement`。
- 2026-05-30T14:46:59+08:00：M2 `implement` worker 孟轲 the 3rd 完成并关闭。已生成 `.pipeline/reports/017-vscode-plugin-protocol-capability-map.md` 和实现报告，明确四类能力矩阵、`nativeAskUi` hard gate、M3-M6 路由指导；`git diff --check` 通过。M2 进入 `review_code`。
- 2026-05-30T14:49:55+08:00：M2 只读 audit worker 孔丘 the 3rd 返回 `pass` 并关闭。M2 标记 completed，按连续执行策略自动进入 M3 `Provider And Resume Compatibility Contract`。
- 2026-05-30T15:07:33+08:00：M3 `test` worker 孟轲 the 4th 完成并关闭。新增 provider/resume focused tests；主代理复跑 `cargo test -p codex-app-server c8_m3_thread_resume -- --nocapture` 确认 2 passed / 1 failed。失败点：persisted `model_provider: "missing_provider"` 未报错，resume 成功但 top-level `modelProvider` 与 `thread.modelProvider` 不一致。M3 进入 `implement`。
- 2026-05-30T15:17:36+08:00：M3 `implement` worker 孔丘 the 4th 完成并关闭。生产实现把非空 provider metadata 从 resumed rollout history 提升到 config loading 验证；主代理 green validation 通过：M3 focused app-server 3 项、rollout VSP provider omission、fmt check 和 `git diff --check`。
- 2026-05-30T15:22:46+08:00：M3 只读 audit worker 孔丘 the 5th 返回 `pass` 并关闭。已写 `.pipeline/reports/018-provider-resume-compatibility-contract.md`，M3 标记 completed。按用户要求，本 Cycle 暂停在 M4 之前，下一步 `/hw:resume` 从 M4 `write_tests` 开始。

### C7

- 2026-05-29T20:06:25+08:00：创建 C7 规划种子，主题为 `Experience Optimization And Bugfix Sweep`。
- 2026-05-29T21:00:31+08:00：P2 修订确认。C7 从 7 个 Milestone 调整为 8 个 Milestone，新增 M5 `Structured Interaction Gates And Authorization Cards`，要求 required Workflow decisions 使用 `request_user_input` 卡片；后续 Plan/SubPlan 和 Updater 依赖该交互契约。
- 2026-05-29T21:17:55+08:00：进入 P3 Generate。生成 8 个 C7 执行 Prompt、C7 active state/cycle 元数据、`c7-generate.yaml` 和架构基线 `.pipeline/architecture/c7-experience-optimization-bugfix-sweep.md`。下一步必须完成 `.pipeline/reviews/C7/prompt-audit.md` 后才能进入 P4 确认。
- 2026-05-29T21:31:51+08:00：Subagent prompt audit 复核完成，结论 `pass`；三条非阻塞建议已应用并复核，无剩余 P4 阻塞项。C7 进入 P4 pending confirmation。
- 2026-05-29T23:42:02+08:00：用户通过 Request Input 卡片确认 C7 P4。C7 状态推进到 `ready_to_start`，执行尚未开始，下一步可 `/hw:start` 从 M1 开始。
- 2026-05-29T23:48:49+08:00：用户确认 C7 不设置普通 Milestone Gate 并开始 `/hw:start`。已创建执行租约，进入 M1 `Resume And Scrollback Stability` 的 `write_tests`，后续按 strict Worker Separation 自动推进。
- 2026-05-29T23:51:54+08:00：启动 M1 `test` worker 孟轲（agent id `019e746f-2631-7830-a3af-54a128686d03`），范围限制为 resume/reflow/scrollback 相关测试和 `.pipeline/reviews/C7/M1/test.md`，禁止生产代码编辑。
- 2026-05-29T23:59:22+08:00：等待 M1 `test` worker 仍未完成；已刷新执行租约并发送收敛指令，要求保留最小真实红测、写 `.pipeline/reviews/C7/M1/test.md` 后结束。
- 2026-05-30T00:03:30+08:00：M1 `test` worker 完成并关闭。新增 focused red test `duplicate_height_settling_does_not_keep_postponing_resize_reflow`；主代理复跑确认失败符合预期，进入 `implement`。
- 2026-05-30T00:04:52+08:00：启动 M1 `implement` worker 孔丘 the 2nd（agent id `019e747a-dad7-7320-bca2-e72282435cec`），范围限制为生产 TUI replay/reflow 相关代码和 `.pipeline/reviews/C7/M1/implement.md`，禁止编辑测试。
- 2026-05-30T00:11:07+08:00：M1 `implement` worker 完成并关闭。生产实现新增 pending observed-size tracking，并在 `handle_draw_size_change` 中合并重复同尺寸 startup-settling reflow scheduling；focused resize 验证通过。`cargo fmt --check` 仍因 test-owned `app/tests.rs` 格式化差异未通过，需 test-fix。
- 2026-05-30T00:12:14+08:00：启动 M1 `test-fix` worker 孟轲 the 2nd（agent id `019e7481-d25d-74a1-a516-9fd473afb288`），只允许机械格式化 `codex-rs/tui/src/app/tests.rs` 并更新 M1 test report。
- 2026-05-30T00:15:22+08:00：M1 `test-fix` worker 完成并关闭；`cargo fmt --check` 恢复通过。主代理 green 验证通过：duplicate startup-settling red test、height shrink resize test、`resize_reflow` 过滤套件、`cargo fmt --check`、`git diff --check` 全部通过；tmux smoke tests 仍按配置 ignored。
- 2026-05-30T00:16:39+08:00：启动 M1 只读 `audit` worker 孟轲 the 3rd（agent id `019e7485-d808-7d11-b39d-27cd2a551ca0`），复查实现边界、测试真实性、resize 回归风险和 worker separation evidence。
- 2026-05-30T00:19:38+08:00：M1 audit 返回 `pass` 并关闭。已写 `.pipeline/reviews/C7/M1/audit.md` 与 `.pipeline/reports/007-resume-and-scrollback-stability.md`，M1 标记 completed。按连续执行策略自动进入 M2 `Subagent Identity, Role Policy, And Name Leases` 的 `write_tests`。
- 2026-05-30T00:22:22+08:00：启动 M2 `test` worker 孔丘 the 3rd（agent id `019e748b-157e-7690-bd87-68836c946257`），范围限制为 Subagent display names、lease behavior、lifecycle labels、active panel consistency 和 role/model/reasoning precedence 红测。
- 2026-05-30T00:29:08+08:00：等待 M2 `test` worker 时一度未返回；已刷新执行租约。随后收到 worker 完成通知，产出三条 focused red tests 和 `.pipeline/reviews/C7/M2/test.md`。
- 2026-05-30T00:32:01+08:00：M2 `test` worker 完成并关闭。主代理复跑确认三条红测按预期失败：lifecycle history 仍显示 UUID、active panel fallback 名称碰撞、MultiAgentV2 role + explicit reasoning effort 被错误拒绝。进入 `implement`。
- 2026-05-30T00:33:23+08:00：启动 M2 `implement` worker 孟轲 the 4th（agent id `019e7495-17a7-73e1-b031-5e7746e9a3e2`），范围限制为 TUI Subagent display-name resolver/name leases、core spawn role/default/explicit override 顺序和 `.pipeline/reviews/C7/M2/implement.md`，禁止编辑测试。
- 2026-05-30T00:39:43+08:00：等待 M2 `implement` worker 仍未完成；已刷新执行租约并发送收敛指令，要求只完成三条红测对应的最小生产修复和 `.pipeline/reviews/C7/M2/implement.md`。
- 2026-05-30T00:55:11+08:00：M2 `implement` worker 已完成并关闭；三条 focused green 测试通过，`active_subagents` 更宽 TUI 过滤测试通过，`git diff --check` 通过。但 `cargo fmt --check` 发现两个生产文件格式化差异，`cargo test -p codex-core multi_agent -- --nocapture` 在 `spawn_agent_service_tier_inheritance_preserves_supported_or_configured_tiers` 触发 stack overflow。已启动 M2 validation-fix `implement` worker 孔丘 the 4th（agent id `019e74a8-c93b-7f52-9296-47e01e0f7f16`），只允许修复 M2 生产文件和实现报告。
- 2026-05-30T01:06:50+08:00：M2 validation-fix worker 仍在运行。主代理观察到 worker diff 中出现临时 `stack_size` 线程包裹方案，已发送收敛指令，要求优先定位根因并移除该 workaround，除非报告能证明这是唯一安全方案。
- 2026-05-30T01:13:51+08:00：首个 M2 validation-fix worker 孔丘 the 4th 被关闭时仍为 running，未产出最终报告；其中间 diff 暂不视为可信结果。已启动 replacement validation-fix `implement` worker 孔丘 the 5th（agent id `019e74ba-3d77-74a1-807a-3de94c9d92f3`），授权在 M2 生产文件范围内清理/替换上一位留下的 tentative stack-size workaround。
- 2026-05-30T01:29:17+08:00：replacement validation-fix worker 孔丘 the 5th 返回并关闭；其 `tokio::task::spawn` 调度边界方案未通过主代理验证。`cargo fmt --check` 通过，`multi_agent_v2_spawn_role_allows_explicit_reasoning_effort_override` 通过，但 `spawn_agent_service_tier_inheritance_preserves_supported_or_configured_tiers` 仍 stack overflow。下一轮必须移除无效调度 workaround，回到 role/config layer 或 service-tier 合成路径做根因修复。
- 2026-05-30T01:31:57+08:00：启动 M2 root-cause `implement` worker 孟轲 the 5th（agent id `019e74ca-d4ea-70a0-bc81-3d74061ef5bc`），要求先移除 `tokio::task::spawn` 调度 workaround，再定位 service-tier inheritance 三个子场景中的真实 stack overflow 根因。
- 2026-05-30T01:52:00+08:00：M2 root-cause `implement` worker 孟轲 the 5th 多轮等待后仍未返回，被关闭时状态为 running，且未追加 Round 3 报告。其留下的候选改动将由主代理先验证，未验证前不进入 audit。
- 2026-05-30T02:12:22+08:00：启动新的 M2 root-cause `implement` worker 孟轲 the 6th（agent id `019e74eb-2e4f-7e03-b2ee-16d467a24a6c`），接手默认测试栈 stack overflow 修复；主代理保持编排/只读定位，绿测前不进入 audit。
- 2026-05-30T02:29:12+08:00：M2 root-cause `implement` worker 孟轲 the 6th 被关闭时仍为 running，未追加新报告。其留下 `session/session.rs` 候选修复，主代理将先做集成验证。
- 2026-05-30T02:34:38+08:00：集成验证显示 `session/session.rs` 候选仍未修复默认栈 stack overflow；最新 gdb 栈转移到 subagent 初始任务的 `goal_runtime_apply(TurnStarted)` -> rollout materialization。已启动 replacement `implement` worker 孔丘 the 6th（agent id `019e7504-1942-7583-9702-7218bf7acca6`）处理剩余 root-cause。
- 2026-05-30T02:45:43+08:00：replacement `implement` worker 孔丘 the 6th 完成并关闭，追加 Round 4 报告。主代理 green 验证通过：service-tier inheritance 默认栈测试、v2 role+reasoning 测试、TUI lifecycle/name-lease focused tests、`active_subagents` broader tests、`cargo fmt --check` 和 `git diff --check`。M2 进入只读 audit。
- 2026-05-30T02:54:27+08:00：M2 只读 audit 返回 `needs_changes`：active fallback 名称可能与 active explicit nickname 碰撞；explicit `service_tier=default` 可能被错误视为 unsupported。M2 返回 audit-fix 流程。
- 2026-05-30T02:59:50+08:00：M2 audit-fix test worker 孟轲 the 7th 完成并关闭，补充 explicit nickname lease collision 与 `service_tier=default` 两条红测；随后启动 implement worker 孟轲 the 8th 修复两项阻塞问题。
- 2026-05-30T03:08:37+08:00：M2 audit-fix 实现和主代理回归验证通过。验证覆盖两条 audit-fix 测试、原始 service-tier inheritance、v2 role+reasoning、TUI lifecycle/name-lease、`active_subagents`、`cargo fmt --check` 和 `git diff --check`。
- 2026-05-30T03:09:49+08:00：启动 M2 只读复审 worker 孔丘 the 8th（agent id `019e7524-5156-7550-af44-8362dc88ab7c`），复查两条 needs_changes 是否真正关闭，以及 M2 原始身份/角色/租约语义是否回归。
- 2026-05-30T03:12:48+08:00：M2 复审返回 `pass` 并关闭。已追加 `.pipeline/reviews/C7/M2/audit.md` recheck，写入 `.pipeline/reports/008-subagent-identity-role-policy-name-leases.md`，M2 标记 completed。按连续执行策略进入 M3 `Tool And Collaboration Output Rendering Boundaries` 的 `write_tests`。
- 2026-05-30T03:24:50+08:00：启动 M3 `test` worker 孟轲 the 9th（agent id `019e7532-2b63-7252-bfd6-b43ac6a1b73a`），范围限制为 TUI history-cell translation-boundary tests 和 `.pipeline/reviews/C7/M3/test.md`，禁止生产代码编辑。
- 2026-05-30T03:29:18+08:00：M3 `test` worker 完成并关闭。新增三条真实红测，覆盖 completed exec/MCP/collab lifecycle/status 不应 append translation；新增一条正向 guard 确认 reasoning/prose translation 仍然启用。主代理正在复跑红测。
- 2026-05-30T03:30:10+08:00：主代理复跑确认 M3 红测状态：exec/MCP/collab translation-boundary 三条失败符合预期，reasoning/prose translation guard 通过。已启动 M3 `implement` worker 孔丘 the 9th（agent id `019e7537-7abe-7ba3-a9f9-30bd8eea422f`），范围限制为 TUI 生产渲染/翻译边界和实现报告。
- 2026-05-30T03:39:38+08:00：M3 `implement` worker 完成并关闭。主代理 green 验证通过：三条 translation-boundary 红测转绿，reasoning/prose translation guard 仍通过，`hypo_translate` 8 项通过，`translated` 过滤 7 项通过，`cargo fmt --check` 和 `git diff --check` 通过。宽 `history_cell` 过滤仍有既有版本 snapshot 漂移，暂列非阻塞审计输入。已启动 M3 只读 audit worker 孟轲 the 10th（agent id `019e7540-1f9f-7261-8db1-e92686da729e`）。
- 2026-05-30T03:41:55+08:00：M3 audit 返回 `pass` 并关闭。已写 `.pipeline/reviews/C7/M3/audit.md` 与 `.pipeline/reports/009-tool-collaboration-output-rendering-boundaries.md`，M3 标记 completed。按连续执行策略进入 M4 `Subagent Prompt And Report Viewer` 的 `write_tests`。
- 2026-05-30T04:59:13+08:00：M4 audit recheck 返回 `pass` 并关闭。已写 `.pipeline/reports/010-subagent-prompt-report-viewer.md`，M4 标记 completed。按连续执行策略进入 M5 `Structured Interaction Gates And Authorization Cards`。
- 2026-05-30T05:13:16+08:00：M5 `test` worker 孟轲 the 14th 完成并关闭。新增 core Workflow interaction guidance 红测和 TUI Workflow request_user_input card 红测；主代理确认 core guidance 与 TUI helper 缺失按预期失败。
- 2026-05-30T05:25:31+08:00：首个 M5 `implement` worker 孔丘 the 14th 被关闭时仍为 running；其留下部分 TUI helper 但缺 core guidance/report，已记录为生命周期不完整并由 replacement worker 接手。
- 2026-05-30T05:33:01+08:00：replacement M5 `implement` worker 孟轲 the 15th 完成并关闭，确认 core guidance 与 `workflow_cards` helper 满足红测；snapshot test worker 孔丘 the 15th 接受三张 Workflow card 快照。主代理 green 验证通过：core `workflow_interaction` 3 项、TUI `workflow_` 10 项、`cargo fmt --check`、`git diff --check`。已启动只读 audit worker 孟轲 the 16th。
- 2026-05-30T05:36:14+08:00：M5 audit worker 孟轲 the 16th 返回 `pass` 并关闭。已写 `.pipeline/reviews/C7/M5/audit.md` 与 `.pipeline/reports/011-structured-interaction-gates-authorization-cards.md`，M5 标记 completed。按连续执行策略进入 M6 `Workflow Plan/SubPlan Live Surface` 的 `write_tests`。
- 2026-05-30T05:38:49+08:00：启动 M6 `test` worker 孔丘 the 16th（agent id `019e75ac-dea0-7a80-a902-ef13ba977997`），范围限制为 Plan/SubPlan live surface 相关测试、snapshots 和 `.pipeline/reviews/C7/M6/test.md`，禁止生产代码编辑。
- 2026-05-30T05:43:58+08:00：M6 `test` worker 仍在运行；已刷新执行租约并发送收敛指令，要求保留最小真实红测、运行命令并写 `.pipeline/reviews/C7/M6/test.md` 后结束。
- 2026-05-30T05:46:55+08:00：M6 `test` worker 完成并关闭。主代理复跑确认 Plan panel 当前混合计数 `6/15` 且展开未来 M7/M8 SubPlan；core Workflow guidance 缺少 phase/worker/validation/final response update_plan freshness triggers，并仍有 model-visible prompt 路径的 deprecated cwd allowance。已启动 M6 `implement` worker 孔丘 the 17th（agent id `019e75b4-4611-7ac1-abd2-f1f248e8b9cb`）。
- 2026-05-30T05:57:25+08:00：M6 `implement` worker 仍在运行并已留下 Plan panel 与 session cwd-detection 候选 diff；已刷新执行租约并发送收敛指令，要求完成最小 guidance 修复、验证和 `.pipeline/reviews/C7/M6/implement.md`。
- 2026-05-30T06:02:26+08:00：首个 M6 `implement` worker 孔丘 the 17th 被关闭时仍为 running，未写实现报告；其候选生产 diff 暂不视为完整结果。已启动 replacement `implement` worker 孟轲 the 17th（agent id `019e75c2-6635-70a1-afea-1df2c8bdc701`）接手验证、最小修正和报告。
- 2026-05-30T06:30:30+08:00：replacement M6 `implement` worker 孟轲 the 17th 完成并关闭。生产侧修复 split progress、未来 SubPlan 折叠、Workflow update_plan freshness guidance 和 model-visible cwd detection；core routing、slash routing、`plan_panel`、fmt 和 diff checks 通过。剩余 Workflow focused test 为 snapshot-only 差异，已启动 `test-fix` worker 孔丘 the 18th（agent id `019e75dc-2fa0-7393-89bc-e8f0e3bb77a8`）收口。
- 2026-05-30T06:31:58+08:00：M6 `test-fix` worker 孔丘 the 18th 完成并关闭。已将 Workflow Cycle Plan surface snapshot 裁剪为 Plan surface，移除 `.snap.new`，并把 GREEN 证据追加到 `.pipeline/reviews/C7/M6/test.md`。
- 2026-05-30T06:35:16+08:00：主代理 M6 最终 green validation 通过：Workflow Cycle Plan surface snapshot、core `workflow_routing`、TUI `workflow_slash_routing`、TUI `plan_panel`、`cargo fmt --check` 和 `git diff --check` 均通过。已进入 `review_code`，准备启动 M6 只读 audit worker；剩余 `turn_context.cwd` deprecated warning 是已知非阻塞 conversation-capture 债务。
- 2026-05-30T06:37:05+08:00：启动 M6 只读 audit worker 孟轲 the 18th（agent id `019e75e2-2ffc-7060-9077-0f16c10f77c8`），复核 Plan/SubPlan split progress、未来 SubPlan 折叠、普通 Plan 非回归、Workflow `update_plan` freshness guidance、slash routing 非捕获、model-visible cwd detection 与测试/报告证据。
- 2026-05-30T06:43:15+08:00：M6 只读 audit worker 孟轲 the 18th 返回 `pass` 并关闭。已保存 `.pipeline/reviews/C7/M6/audit.md` 和 `.pipeline/reports/012-workflow-plan-subplan-live-surface.md`，M6 标记 completed。
- 2026-05-30T06:43:15+08:00：按连续执行策略进入 M7 `VSP Release Channels, Naming Metadata, And Updater` 的 `write_tests`；已完成现有 updater、update prompt/action、install context、Request Input 更新卡片和 Subagent 名字池入口的初步只读梳理。
- 2026-05-30T06:45:13+08:00：启动 M7 `test` worker 孔丘 the 19th（agent id `019e75e9-9f1f-7a11-9d69-b2ddc4de7979`），范围限制为 release/updater/channel metadata 相关测试、fixtures、snapshots 和 `.pipeline/reviews/C7/M7/test.md`，禁止生产代码编辑。
- 2026-05-30T06:51:24+08:00：M7 `test` worker 首轮等待仍未完成；已刷新执行租约并发送收敛指令，要求保留最小 focused red tests、记录失败证据并写 `.pipeline/reviews/C7/M7/test.md` 后结束。
- 2026-05-30T06:58:00+08:00：首个 M7 `test` worker 孔丘 the 19th 第二轮等待仍未完成，被关闭时仍为 running。其留下 `update_versions.rs` 和 `update_action.rs` 的 test-only 候选红测，但缺报告；下一步启动 replacement test worker 补齐红测证据与 `.pipeline/reviews/C7/M7/test.md`。
- 2026-05-30T06:58:45+08:00：启动 M7 replacement `test` worker 孟轲 the 19th（agent id `019e75f6-07f9-7a13-a7ee-5dd6fc28bf69`），范围限制为确认首个 worker 留下的 release/updater 红测、必要时补一条 focused metadata/card 测试，并写 `.pipeline/reviews/C7/M7/test.md`。
- 2026-05-30T07:05:48+08:00：M7 replacement `test` worker 孟轲 the 19th 完成并关闭，报告写入 `.pipeline/reviews/C7/M7/test.md`。新增红测覆盖 VSP manifest metadata/selection、官方 endpoint 禁止、checksum mismatch 不切 current、standalone VSP update action、结构化更新卡片选项和 channel 默认名字池优先级。
- 2026-05-30T07:05:48+08:00：主代理复跑 `cargo test -p codex-tui c7_m7 --no-run`，按预期 compile-red，失败集中在缺失 `VspReleaseManifest`/`VspReleaseQuery`/`VspReleaseChannel`/`VspReleaseSource`/`VspSelectedRelease`/`VspStandaloneUpdatePlan`/`UpdateAction::from_vsp_manifest_release`/release-channel metadata API。M7 进入 `implement`。
- 2026-05-30T07:07:05+08:00：启动 M7 `implement` worker 孟轲 the 20th（agent id `019e75fd-a596-7491-8c0b-1a31c2945d21`），范围限制为 production updater/channel metadata/card helper/docs 和 `.pipeline/reviews/C7/M7/implement.md`，禁止编辑测试。
- 2026-05-30T07:12:45+08:00：M7 `implement` worker 首轮等待仍未完成；已刷新执行租约并发送收敛指令，要求优先完成最小生产 API、聚焦验证和 `.pipeline/reviews/C7/M7/implement.md`。
- 2026-05-30T07:19:23+08:00：M7 `implement` worker 孟轲 the 20th 第二轮等待仍未完成，被关闭时仍为 running 且未写实现报告。为避免阻塞，主代理接手 bounded production implementation，后续通过独立 green validation 与只读 audit 补风险控制。
- 2026-05-30T08:22:35+08:00：主代理完成 M7 生产实现与 audit-fix。最终验证通过：`cargo test -p codex-tui c7_m7 -- --nocapture` 9 项、`cargo test -p codex-cli c7_m7 -- --nocapture` 1 项、`cargo check -p codex-cli --release --bin vsp-codex`、`cargo fmt --check --package codex-cli --package codex-tui`、`git diff --check`。第一次只读 audit 指出 manifest-selected URL 官方端点封锁不完整和 VSP history notice 官方 updater fallback；已修复并由 `庄周 the 20th` 复审为 `pass`。
- 2026-05-30T08:22:35+08:00：M7 标记 completed，报告写入 `.pipeline/reports/013-vsp-release-channels-naming-updater.md`。VSP 更新路径现在由 manifest 驱动，archive/notes URL 与 manifest URL 均拒绝官方 OpenAI/GitHub/chatgpt 更新端点；history notice 在 VSP 模式不 fallback 官方 updater action，也不展示官方 release 链接。
- 2026-05-30T08:22:35+08:00：按连续执行策略进入 M8 `Appearance Entry Point And Final UX Polish` 的 `write_tests`。M8 将统一 `/theme` 外观入口、保留 `/ui-theme` 快捷路径，并完成 C7 最终体验回归。
- 2026-05-30T08:32:13+08:00：M8 `test` worker 孔丘 the 20th 完成并关闭，报告写入 `.pipeline/reviews/C7/M8/test.md`。主代理复跑确认 `cargo test -p codex-tui c7_m8 --no-run` compile-red，唯一失败点为缺失 `build_appearance_picker_params(...)`，符合预期。
- 2026-05-30T08:32:13+08:00：启动 M8 `implement` worker 孔丘 the 21st（agent id `019e764b-8fc3-7410-81ac-47a1ccc7b3e8`），范围限制为 Appearance picker、`/theme`/`/ui-theme` 生产入口、command palette 文案、docs 和 `.pipeline/reviews/C7/M8/implement.md`，禁止编辑测试。

### C6

- 2026-05-28T23:07:16+08:00：创建 C6，范围定为 Compact 机制、上下文管理与 Workflow 定制化接入。尚未进入 milestone decomposition。
- 2026-05-28T23:44:58+08:00：进入 `$hypo-workflow:plan` 的交互式 P1 Discover。P0 配置复用项目设置：中文输出、Codex 平台、交互式规划、strict Worker Separation，且 `/hw:start` 和 `/hw:resume` 的执行子工作器已授权。
- 2026-05-28T23:49:51+08:00：完成 P1 Discover 第 1 轮。用户优先选择“Workflow 变成 VSP-Codex 的内建能力”；成功信号包括 Compact 返回空也能像未压缩一样继续、P3/Milestone/Explore 能自动派 Subagent、普通对话也能后续整理归档；初始验证重点是空 Compact 恢复和 YAML 读写稳定性。
- 2026-05-28T23:52:14+08:00：完成 P1 Discover 第 2 轮。用户希望所有适合分工的场景都能自动派 Subagent；Compact 空返回恢复优先使用全局目标、Plan Prompt、Architecture、State 和最近 N 条对话；普通对话归档形态待进一步解释后决定。
- 2026-05-28T23:54:45+08:00：用户接受普通对话记录的建议形态：默认 Journal + Inbox。
- 2026-05-28T23:56:29+08:00：完成 P1 Discover 第 3 轮。Compact 空返回恢复优先级为 Architecture/项目摘要、全局/当前目标、最近 N 条对话、state/cycle、Plan Prompt、compact logs；自动 Subagent 仅在破坏性操作和系统级依赖安装前必须停下来问；长主题普通对话应自动升级 Mini-Cycle。
- 2026-05-28T23:57:51+08:00：完成 P1 Discover 第 4 轮。内建化最终要覆盖系统提示/运行时策略、原生 Workflow 工具、Slash 命令、Compact/Resume 运行时和普通对话捕获；落地顺序优先解决 Compact 空返回；Workflow 工具优先做结构化 YAML、State/Progress/Log API、Question/Plan 桥接、Subagent 生命周期 API，Lock/并发暂不优先。
- 2026-05-29T00:00:32+08:00：进入 P2 Decompose 并生成检查点 `.pipeline/plan-state/c6-p2-decompose.md`。初版拆为 Compact fallback context pack、结构化 Workflow 文件 API、Native Workflow runtime routing、普通对话捕获、Workflow 命令/UX polish 五个 Milestone。
- 2026-05-29T00:03:47+08:00：进入 P3 Generate，生成 5 个执行 Prompt，并记录新规则：以后 Workflow 生成 Prompt 后默认用 Subagent 做 prompt audit；本轮也必须完成该审查后才能进入 P4。
- 2026-05-29T00:07:31+08:00：启动 Prompt Audit Subagent（第一次带 agent_type 的 full-history fork 被工具拒绝，随后以默认 full-history fork 成功启动，agent id `019e6f57-130b-7b62-ac40-6ceedcb0478f`）。
- 2026-05-29T00:07:59+08:00：完成 P3 本地 YAML 校验，并修复 `.pipeline/state.yaml` 中重复 `milestones` key。
- 2026-05-29T00:09:17+08:00：Prompt Audit Subagent 返回 pass；已保存 `.pipeline/reviews/C6/prompt-audit.md`，并应用非阻塞建议：M2-M5 补强 test 角色措辞，M1-M5 均补充破坏性操作和系统级依赖安装需要用户确认。
- 2026-05-29T00:13:45+08:00：用户确认 P4。C6 的 5 个执行 Prompt 已接受，Cycle 进入 `ready_to_start`，后续可用 `/hw:start` 从 M1 开始执行。
- 2026-05-29T00:15:25+08:00：执行 `/hw:start`，创建 `.pipeline/.lock` 执行租约，进入 M1 `write_tests`，按 strict Worker Separation 准备启动 `test` worker。
- 2026-05-29T00:16:16+08:00：启动 M1 `test` worker（agent id `019e6f5f-27a3-7f31-933d-276a03fa6db9`），范围限制为 compact 相关测试/夹具/断言/快照和 `.pipeline/reviews/C6/M1/test.md`，禁止生产代码编辑。
- 2026-05-29T00:19:11+08:00：等待 M1 `test` worker 红测输出 120 秒未完成；刷新执行租约心跳，主代理继续保持编排状态，不编辑测试或生产代码。
- 2026-05-29T00:22:53+08:00：第二轮等待 `test` worker 仍未完成；工作树已有 `compact.rs`/`compact_remote.rs` 测试文件改动但缺报告。已发送收敛指令，要求写 `.pipeline/reviews/C6/M1/test.md` 并结束。
- 2026-05-29T00:27:39+08:00：`test` worker 完成并关闭。产物包括 `codex-rs/core/tests/suite/compact.rs`、`codex-rs/core/tests/suite/compact_remote.rs` 和 `.pipeline/reviews/C6/M1/test.md`；未编辑生产代码。进入 `implement`。
- 2026-05-29T00:29:01+08:00：启动 M1 `implement` worker（agent id `019e6f6a-d3b1-7dc3-a1c5-394f295691ff`），范围限制为生产/运行时代码和 `.pipeline/reviews/C6/M1/implement.md`，禁止测试资产编辑。
- 2026-05-29T00:33:38+08:00：等待 `implement` worker 240 秒未完成；工作树已有 `compact.rs`、`compact_remote.rs`、`lib.rs`、`workflow_context.rs` 生产改动但缺报告。已发送收敛指令，要求写 `.pipeline/reviews/C6/M1/implement.md` 并结束。
- 2026-05-29T00:38:09+08:00：关闭未收敛的 `implement` worker 时其仍为 running，已记录为 `close_failed`/生命周期不完整；其留下生产改动但无报告。将启动 replacement implement worker 接手并补齐报告。
- 2026-05-29T00:39:16+08:00：启动 replacement `implement` worker（agent id `019e6f74-365c-70c3-9922-60a8c3c8edcf`），不 fork 全历史，范围限制为当前生产实现和 `.pipeline/reviews/C6/M1/implement.md`。
- 2026-05-29T00:44:50+08:00：replacement `implement` worker 仍未收敛，被关闭时为 running 且未写实现报告。M1 在 strict Worker Separation 下阻塞；已移除 `.pipeline/.lock` 并写入 `.pipeline/continuation.yaml`，安全恢复命令为 `/hw:resume`。
- 2026-05-29T14:06:49+08:00：用户明确确认 M1 implement 降级为单代理实现并要求继续。重新创建 `.pipeline/.lock`，恢复执行，主代理接手生产实现；后续仍将使用独立验证/审查尽量补回风险控制。
- 2026-05-29T14:50:47+08:00：主代理完成 M1 单代理实现并更新 `.pipeline/reviews/C6/M1/implement.md`。聚焦验证通过：`cargo fmt --check`（仅既有 nightly-only rustfmt 配置警告）、本地 empty compact fallback、remote v1 empty filtered fallback、remote v2 marker-only fallback、`workflow_context` builder 单测、`git diff --check`。已启动只读 audit worker `019e727f-cb82-7e81-9d6f-f41dddfb4a4e`，进入 `review_code`。
- 2026-05-29T14:54:40+08:00：只读 audit worker 返回 pass，无阻塞发现，并认可用户确认 implement 降级后的 M1 完成条件。已写 `.pipeline/reviews/C6/M1/audit.md` 和 `.pipeline/reports/001-compact-fallback-context-pack.md`；M1 标记 completed。由于 Cycle 配置 `continuous_execution_after_start=false`，执行停在 M2 `not_started`，等待下一步启动。
- 2026-05-29T15:29:49+08:00：用户确认后续不再设置 Milestone Gate，要求继续工作直到完成。已将 C6 `continuous_execution_after_start` 调整为 true，恢复执行租约，进入 M2 `Structured Workflow File API` 的 `write_tests`。
- 2026-05-29T15:32:00+08:00：启动 M2 `test` worker（agent id `019e72a5-4569-7960-be3a-1d26e37aef6c`），范围限制为 `codex-rs/core/tests/**` 与 `.pipeline/reviews/C6/M2/test.md`，禁止生产代码编辑。
- 2026-05-29T15:37:05+08:00：等待 M2 `test` worker 240 秒未完成；已发送收敛指令，要求运行最小红测命令、写 `.pipeline/reviews/C6/M2/test.md` 并结束。
- 2026-05-29T15:38:05+08:00：M2 `test` worker 完成并关闭。新增 `codex-rs/core/tests/suite/workflow_files.rs` 并注册测试模块；红测命令 `cargo test -p codex-core workflow_files --no-run` 因缺少 `codex_core::workflow_files` API 按预期失败。进入 `implement`。
- 2026-05-29T15:39:14+08:00：启动 M2 `implement` worker（agent id `019e72ac-31a2-7aa3-b4a9-01902a01adaf`），范围限制为生产代码和 `.pipeline/reviews/C6/M2/implement.md`，禁止测试资产编辑。
- 2026-05-29T15:45:19+08:00：等待 M2 `implement` worker 300 秒未完成；已发送收敛指令，要求运行最小验证命令、写 `.pipeline/reviews/C6/M2/implement.md` 并结束。
- 2026-05-29T15:52:34+08:00：M2 `implement` worker 完成并关闭。新增 `codex_core::workflow_files` 生产 API、导出模块并为 core crate 启用 workspace `serde_yaml` 依赖；worker 侧验证 `cargo test -p codex-core workflow_files --no-run`、`cargo test -p codex-core workflow_files` 和 `cargo fmt --check` 通过。进入主代理 green 验证。
- 2026-05-29T15:53:32+08:00：主代理 green 验证通过：`cargo test -p codex-core workflow_files` 7 项通过，`cargo fmt --check` 退出码 0（仅既有 nightly-only rustfmt 警告），`git diff --check` 通过。进入 `review_code`，准备启动只读 audit worker。
- 2026-05-29T15:54:18+08:00：启动 M2 只读 audit worker（agent id `019e72b9-fcb7-7fb2-bbc0-09e959092028`），范围为路径安全、YAML 验证、无部分写语义、测试真实性和 worker lifecycle evidence。
- 2026-05-29T15:56:46+08:00：M2 audit worker 返回 `needs_changes` 并关闭。阻塞点：`write_unprotected_yaml` 未处理 `.pipeline` 内部已有目录符号链接，可能通过 symlink parent 写出项目本地 `.pipeline`。M2 返回 `implement` 做最小安全修复和回归测试。
- 2026-05-29T15:57:59+08:00：启动 M2 needs_changes 修复轮 `test` worker（agent id `019e72bd-5417-72d2-91c4-cdd43b9a2da8`），只允许补充 symlink-parent escape 回归测试和测试报告。
- 2026-05-29T15:59:47+08:00：M2 修复轮 `test` worker 完成并关闭。新增 Unix-only `pipeline_child_symlink_parent_escape_is_denied` 回归测试；命令 `cargo test -p codex-core pipeline_child_symlink_parent_escape_is_denied` 按预期失败，证明当前生产实现仍允许 symlink-parent escape。
- 2026-05-29T16:00:32+08:00：启动 M2 needs_changes 修复轮 `implement` worker（agent id `019e72bf-ad7c-7af0-b76e-d1583eefd580`），只允许修复 `codex-rs/core/src/workflow_files.rs` 并追加实现报告。
- 2026-05-29T16:03:57+08:00：M2 修复轮 `implement` worker 完成并关闭。`write_unprotected_yaml` 现在会拒绝 `.pipeline` 下已有 symlink 祖先目录，并在创建缺失目录后确认最终 parent canonical path 仍位于 canonical `.pipeline` 下；worker 侧验证 symlink 回归测试通过，`workflow_files` 8 项通过，fmt 通过。
- 2026-05-29T16:04:49+08:00：主代理修复轮 green 验证通过：`cargo test -p codex-core workflow_files` 8 项通过，`cargo fmt --check` 退出码 0（仅既有 nightly-only rustfmt 警告），`git diff --check` 通过。进入 M2 audit 复审。
- 2026-05-29T16:08:40+08:00：M2 audit recheck 返回 `pass` 并关闭；M2 标记 completed，写入 `.pipeline/reports/002-structured-workflow-file-api.md`。由于用户已确认不设置后续 Gate，立即进入 M3 `Native Workflow Runtime Routing` 的 `write_tests`。
- 2026-05-29T16:11:51+08:00：启动 M3 `test` worker（agent id `019e72ca-03f7-7701-b982-b21896b5df6b`），范围限制为 runtime context / skill injection / slash routing 相关测试和 `.pipeline/reviews/C6/M3/test.md`，禁止生产代码编辑。
- 2026-05-29T16:18:06+08:00：等待 M3 `test` worker 300 秒未完成；已发送收敛指令，要求提交最小真实红测、运行最小命令、写 `.pipeline/reviews/C6/M3/test.md` 并结束。
- 2026-05-29T16:24:36+08:00：M3 `test` worker 留下红测和报告，但关闭时仍为 running，记录为 `closed_while_running`。将启动 replacement test worker 复跑并确认测试证据，不编辑生产代码。
- 2026-05-29T16:25:33+08:00：启动 M3 replacement `test` worker（agent id `019e72d6-700d-76e1-9d5d-20a49bffcb5f`），只复跑/确认红测并追加测试报告，不编辑生产代码。
- 2026-05-29T16:28:34+08:00：M3 replacement `test` worker 完成并关闭。红测证据确认：core `workflow_routing` 为 1 passed / 3 failed，TUI `/hw:plan` routing API 缺失导致 compile-red，均符合 M3 预期。启动 M3 `implement` worker（agent id `019e72d9-5787-77f0-91d0-f57c103894a0`），范围限制为 core/tui 生产代码和 `.pipeline/reviews/C6/M3/implement.md`。
- 2026-05-29T16:35:50+08:00：等待 M3 `implement` worker 360 秒未完成；已发送收敛指令，要求运行最小验证、写 `.pipeline/reviews/C6/M3/implement.md` 并结束。
- 2026-05-29T16:42:39+08:00：M3 `implement` worker 留下 core Workflow runtime guidance 和 TUI `/hw:*` parser 生产改动，但未写实现报告且关闭时仍为 running，记录为 `closed_while_running`。将启动 replacement implement worker 接手验证和报告。
- 2026-05-29T16:44:19+08:00：启动 M3 replacement `implement` worker（agent id `019e72e7-ae54-7170-959b-19a5f62fa4f0`），接手验证/最小修正现有生产改动并写实现报告。
- 2026-05-29T16:46:18+08:00：M3 replacement `implement` worker 完成并关闭。core `workflow_routing` 4 项通过，TUI `/hw:plan` routing 编译通过；剩余 `cargo fmt --check` 问题是测试模块排序，主代理已做机械集成修正。进入 green 验证。
- 2026-05-29T16:47:21+08:00：主代理 M3 green 验证通过：`cargo test -p codex-core workflow_routing -- --nocapture` 4 项通过，`cargo test -p codex-tui hw_plan_slash_command_is_recognized_as_native_workflow_route --no-run` 通过，`cargo fmt --check` 退出码 0（仅既有 nightly-only rustfmt 警告），`git diff --check` 通过。进入 audit。
- 2026-05-29T16:48:16+08:00：启动 M3 只读 audit worker（agent id `019e72eb-5fb4-7d83-9164-6e0308ae2bb9`），范围为 activation guard、prompt size、permission gates、slash compatibility 和 Worker Separation evidence。
- 2026-05-29T16:49:30+08:00：恢复后发现原 M3 audit worker `019e72eb-5fb4-7d83-9164-6e0308ae2bb9` 已无法查询且未写审计产物，已如实记录为 lifecycle incomplete，并启动 replacement 只读 audit worker `019e72ec-efee-7180-ad37-da7f4b0d8bb0`。
- 2026-05-29T16:50:43+08:00：M3 green 复验通过：core `workflow_routing` 4 个真实集成测试通过且无 skip，TUI `/hw` route 编译测试通过，`cargo fmt --check` 退出码 0（仅既有 nightly-only rustfmt 警告），`git diff --check` 通过。
- 2026-05-29T16:51:52+08:00：M3 replacement audit 返回 `needs_changes`。阻塞点：`codex-rs/tui/src/workflow_slash_routing.rs` 会把 `/review`、`/compact`、普通文本等非 Workflow 输入也解析为 Hypo-Workflow route。已保存 `.pipeline/reviews/C6/M3/audit.md`，M3 返回修复轮。
- 2026-05-29T16:52:44+08:00：启动 M3 修复轮 test worker `019e72ef-be7b-7940-8f2a-d3fa5b0e182e`，范围限制为 TUI workflow slash routing 测试和 `.pipeline/reviews/C6/M3/test.md`，用于补充非 Workflow 输入的负向红测。
- 2026-05-29T16:54:18+08:00：M3 修复轮 test worker 完成并关闭。新增 `/hw:*` 与 `$hypo-workflow:*` 正向测试，以及 `/review`、`/compact`、`/model`、普通文本、空输入的负向测试；`cargo test -p codex-tui workflow_slash_routing --no-run` 按预期 compile-red，因为生产 parser 仍返回非 Option 的 always-route 类型。
- 2026-05-29T16:54:52+08:00：启动 M3 修复轮 implement worker `019e72f1-b080-7ac2-b6f9-cd8ba927cb0a`，范围限制为 `codex-rs/tui/src/workflow_slash_routing.rs` 和实现报告，修复 parser 只接受显式 Workflow 前缀。
- 2026-05-29T16:56:11+08:00：M3 修复轮 implement worker 完成并关闭。`parse_workflow_slash_command` 现在返回 `Option<WorkflowSlashRoute>`，仅 `/hw:` 和 `$hypo-workflow:` 返回 route；无关 slash、普通文本和空输入返回 `None`。worker 侧 TUI routing 4 项通过。
- 2026-05-29T16:57:43+08:00：M3 修复轮 green 验证通过：core `workflow_routing` 4 项通过，TUI `workflow_slash_routing` 4 项通过，`cargo fmt --check` 退出码 0（仅既有 stable rustfmt warning），`git diff --check` 通过。进入复审。
- 2026-05-29T16:58:03+08:00：启动 M3 修复轮只读复审 worker `019e72f4-a8ba-7f51-9254-5ab207c5b0e5`，重点复查 parser 负向行为、正向路由、测试覆盖、core guidance gate 和 Worker Separation evidence。
- 2026-05-29T16:59:32+08:00：M3 只读复审返回 `pass`。确认 parser 只接受 `/hw:` 与 `$hypo-workflow:`，负向测试覆盖无关 slash/普通文本/空输入；core runtime guidance 仍由 `.pipeline/` gate 控制，权限门和 Subagent 指引保留。
- 2026-05-29T16:59:58+08:00：M3 `Native Workflow Runtime Routing` 完成，报告写入 `.pipeline/reports/003-native-workflow-runtime-routing.md`。按用户要求不设置后续 Gate，立即进入 M4 `Ordinary Conversation Capture` 的 `write_tests`。
- 2026-05-29T17:00:47+08:00：启动 M4 test worker `019e72f7-62ca-7fe3-9d89-bde24ff49d38`，范围限制为 core 测试与 `.pipeline/reviews/C6/M4/test.md`，覆盖 Journal、Inbox、Mini-Cycle、去重和 compact fallback inclusion 红测。
- 2026-05-29T17:04:51+08:00：M4 test worker 完成并关闭。新增 5 个红测，覆盖 bounded Journal append、conservative Inbox item、long-topic Mini-Cycle auto-upgrade、completed turn replay dedupe、compact fallback inclusion；`cargo test -p codex-core workflow_conversation_capture --no-run` 按预期 compile-red，因为缺少生产 capture API 和公开 fallback 测试入口。
- 2026-05-29T17:05:46+08:00：启动 M4 implement worker `019e72fb-b13a-7d30-99a8-a21e05853c93`，范围限制为生产 capture 模块、必要导出/fallback 可见性、可选 session hook 和实现报告，禁止编辑测试。
- 2026-05-29T17:11:19+08:00：M4 implement worker 已留下生产改动但未写实现报告，发送收敛指令要求运行聚焦验证、写 `.pipeline/reviews/C6/M4/implement.md` 并结束。
- 2026-05-29T17:15:31+08:00：M4 implement worker 仍未收敛，被关闭时仍为 running。其留下生产改动且 `cargo test -p codex-core workflow_conversation_capture --no-run` 已通过，但缺实现报告和完整 lifecycle，需 replacement implement worker 验证/补齐。
- 2026-05-29T17:16:09+08:00：启动 M4 replacement implement worker `019e7305-2eca-7a12-87f8-e5dd3b532470`，接手验证/最小修正现有生产改动并补齐 `.pipeline/reviews/C6/M4/implement.md`。
- 2026-05-29T17:18:03+08:00：M4 replacement implement worker 完成并关闭。`workflow_conversation_capture` 聚焦测试 5 项通过；`cargo fmt --check` 仍失败，但 diff 仅在 test worker 拥有的 `codex-rs/core/tests/suite/mod.rs` 与 `workflow_conversation_capture.rs`。
- 2026-05-29T17:19:06+08:00：启动 M4 test-fix worker `019e7307-e2c1-7020-9a09-f36d8f39b17a`，只允许机械格式化 M4 test-owned 文件并复跑 fmt/聚焦测试。
- 2026-05-29T17:20:26+08:00：M4 test-fix worker 完成并关闭。测试文件格式化完成；`cargo fmt --check` 和 `cargo test -p codex-core workflow_conversation_capture` 均通过。
- 2026-05-29T17:21:07+08:00：M4 green 验证通过：`workflow_conversation_capture` 5 项通过，`cargo fmt --check` 退出码 0（仅既有 stable rustfmt warning），`git diff --check` 通过。进入 audit。
- 2026-05-29T17:21:46+08:00：启动 M4 只读 audit worker `019e730a-640c-7151-ac39-fd217c0ef036`，范围为隐私/脱敏、bounded summaries、去重、formal Cycle 隔离、subagent/internal turn 分类、compact fallback inclusion、API surface 和 evidence。
- 2026-05-29T17:24:52+08:00：M4 audit 返回 `needs_changes`。阻塞点：capture API 尚未接入真实 root session turn completion；Inbox/Mini-Cycle 缺少足够边界；脱敏覆盖过窄；长 `turn_id`/`completion_event_id` 被截断后去重不稳定。已保存 `.pipeline/reviews/C6/M4/audit.md`，M4 进入修复轮。
- 2026-05-29T17:26:14+08:00：启动 M4 audit-fix test worker `019e730e-68a1-7b40-9256-df6cf7e4ab2b`，补充 root turn-completion 自动捕获、长 id 去重、广义脱敏和边界红测。
- 2026-05-29T17:30:35+08:00：M4 audit-fix test worker 完成并关闭。新增 4 个红测：真实 root session `TurnComplete` 后被动写入 Journal/Inbox、长 ID replay 去重、常见 secret 脱敏、Inbox/Mini-Cycle 有界输出；聚焦测试 5 passed / 4 failed，符合 audit finding。
- 2026-05-29T17:31:47+08:00：启动 M4 audit-fix implement worker `019e7313-80f9-7283-88bd-cbd2ee9ec89f`，修复 root turn completion capture 接入、长 ID 去重、广义脱敏和有界输出。
- 2026-05-29T17:45:18+08:00：M4 audit-fix implement worker 完成并关闭。已接入 root/non-subagent `TurnComplete` 被动捕获，仅 `.pipeline/` 项目生效；长 ID 去重、常见 secret 脱敏、Inbox/Mini-Cycle bounded window 均完成。
- 2026-05-29T17:51:46+08:00：M4 修复轮 green 验证通过：`workflow_conversation_capture` 9 项通过，`cargo fmt --check` 退出码 0，`git diff --check` 通过。测试命令仍有一个 `turn_context.cwd` deprecated warning，交给复审判断是否需要清理。
- 2026-05-29T17:52:27+08:00：启动 M4 修复轮只读复审 worker `019e7326-6e06-7850-8a38-8941d801ef8b`，复查 root turn-completion capture、长 ID 去重、脱敏、有界输出、fallback inclusion、API surface 和 evidence。
- 2026-05-29T17:55:42+08:00：M4 修复轮只读复审返回 `pass`。确认 root/non-subagent `TurnComplete` 已被动捕获，`.pipeline` gate 生效，长 ID 去重、脱敏、有界输出、fallback inclusion 与 formal Cycle 隔离均通过；`turn_context.cwd` deprecated warning 为非阻塞。
- 2026-05-29T17:56:22+08:00：M4 `Ordinary Conversation Capture` 完成，报告写入 `.pipeline/reports/004-ordinary-conversation-capture.md`。按连续执行策略进入 M5 `Workflow Command And UX Polish` 的 `write_tests`。
- 2026-05-29T17:57:31+08:00：启动 M5 test worker `019e732b-2829-7381-99f0-cf05986afe9c`，检查 Workflow command/status/docs seam，并优先补 focused red tests；范围限制为测试/检查产物和 `.pipeline/reviews/C6/M5/test.md`。
- 2026-05-29T18:03:08+08:00：M5 test worker 首轮等待后尚未产出 M5 工件，已发送收敛指令，要求添加 focused tests 或写明检查报告并结束，不编辑生产/docs。
- 2026-05-29T18:04:45+08:00：M5 test worker 完成并关闭。新增 `workflow_command_palette_entries` 与 `workflow_status_summary` 的红测，覆盖 Workflow 项目可发现性、非 Workflow 不误导，以及 M1-M4 已验证功能的状态摘要；`cargo test -p codex-tui workflow_slash_routing --no-run` 按预期 compile-red。
- 2026-05-29T18:06:04+08:00：启动 M5 implement worker `019e7332-dd27-7f43-b21c-ed7a73ec96aa`，范围为 TUI Workflow helper、docs/changelog 和实现报告，不编辑测试。
- 2026-05-29T18:09:26+08:00：M5 implement worker 完成并关闭。新增 Workflow command/status helper，更新 README、VSP guide、slash command docs 与 changelog；`cargo test -p codex-tui workflow_slash_routing` 6 项通过，`git diff --check` 通过；`cargo fmt --check` 仍仅在 test-owned `workflow_slash_routing.rs` 有一行格式化 diff。
- 2026-05-29T18:10:58+08:00：启动 M5 test-fix worker `019e7337-5750-7112-91cd-7eeb5dce151e`，只允许机械格式化 `codex-rs/tui/tests/suite/workflow_slash_routing.rs` 并复跑 fmt/focused tests。
- 2026-05-29T18:12:18+08:00：M5 test-fix worker 完成并关闭。测试文件格式化完成；`cargo fmt --check` 和 `cargo test -p codex-tui workflow_slash_routing` 均通过。
- 2026-05-29T18:13:13+08:00：M5 green 验证通过：TUI `workflow_slash_routing` 6 项通过，`cargo fmt --check` 退出码 0，`git diff --check` 通过。进入 audit。
- 2026-05-29T18:13:52+08:00：启动 M5 只读 audit worker `019e733a-02fe-70a0-80b1-e2b26cd55e07`，复查 helper 行为、docs/changelog 诚实性、非 Workflow UX、slash 兼容性与 evidence。
- 2026-05-29T18:15:25+08:00：M5 audit 返回 `pass`。确认 Workflow command/status helper 行为正确，非 Workflow 项目不显示 active Workflow entries，docs/changelog 明确 helper-level discoverability 限制且没有过度承诺。
- 2026-05-29T18:16:02+08:00：M5 `Workflow Command And UX Polish` 完成，报告写入 `.pipeline/reports/005-workflow-command-ux-polish.md`。C6 全部 5 个 Milestone 完成，Cycle summary 写入 `.pipeline/reports/006-cycle-summary.md`。
- 2026-05-29T18:18:03+08:00：收尾同步 `.pipeline/cycle.yaml` 与 `.pipeline/continuation.yaml` 为 completed，清除过期 `/hw:resume` 待办指针，避免后续误恢复到 M3。

## Next Step

2026-05-30T08:46:44+08:00：C7 M8 green validation 通过：`cargo test -p codex-tui c7_m8 -- --nocapture` 7 项通过，`theme_picker` 14 项通过，`ui_theme_picker` 1 项通过，`command_palette` 8 项通过，`cargo fmt --check --package codex-tui` 与 `git diff --check` 均通过。仅出现既有 `turn_context.cwd` deprecated warning 和 stable rustfmt 忽略 unstable import granularity 的 warning。随后进入 M8 只读 audit。

2026-05-30T08:54:54+08:00：M8 只读 audit worker `孟轲 the 21st` 返回 `pass`，无阻塞 finding。已写 `.pipeline/reports/014-appearance-entry-final-ux-polish.md` 和 `.pipeline/reports/015-cycle-summary.md`。最终回归通过：`workflow_slash_routing` 8 项、`plan_panel` 5 项、`request_user_input` 85 项、`c7_m7` 9 项。C7 全部 8 个 Milestone 已完成，后续可按新需求进入下一个 Cycle 或 follow-up plan。

2026-05-30T17:14:05+08:00：用户通过 `/hw:resume` 恢复 C8。已创建执行租约 `.pipeline/.lock`，将 C8 从 M3 后暂停状态切回执行中，并进入 M4 `Session Discovery Strategy For Official Codex` 的 `write_tests`。

2026-05-30T17:23:12+08:00：M4 test worker `孟轲 the 5th` 完成并关闭。新增 official Codex discovery 红测和 fixture，覆盖 sanitized export、field classification 和禁止 raw/symlink 共享。主代理复跑 `cargo test -p codex-rollout --test official_codex_discovery`，按预期 compile-red 于缺少 `codex_rollout::official_codex_discovery`。

2026-05-30T17:31:51+08:00：M4 implement worker `孟轲 the 6th` 完成并关闭。新增 `codex_rollout::official_codex_discovery` sanitized export/manifest API。主代理 green 验证通过：`official_codex_discovery` 4 项通过，`cargo fmt --check --package codex-rollout` 通过（仅既有 stable rustfmt warning），`git diff --check` 通过。进入只读 audit。

2026-05-30T17:38:16+08:00：M4 只读 audit worker `孔丘 the 6th` 返回 `needs_changes`。阻塞点：实现仅为导出 API，不能过度承诺自动官方发现/导入；sanitized artifact 未经 normal rollout reader 验证；`session_index.thread_name` 可能泄漏敏感字符串；source traversal 会跟随 sessions 内 symlink；artifact basename 可能碰撞覆盖。进入修复轮 `write_tests_fix`。

2026-05-30T17:45:30+08:00：M4 修复轮 test worker `孔丘 the 7th` 完成并关闭。新增 audit 回归测试。主代理复跑 `cargo test -p codex-rollout --test official_codex_discovery`，8 项中 6 项通过、2 项失败：`session_index.thread_name` 泄漏 `.pipeline`，以及重复 basename 产物路径碰撞覆盖。进入 `implement_fix`。

2026-05-30T17:53:10+08:00：M4 修复轮 implement worker `孟轲 the 7th` 完成并关闭。已清理 `session_index.thread_name` 可疑标题并为重复 source basename 生成去重 artifact 路径。主代理 green 验证通过：`official_codex_discovery` 8 项通过，`cargo fmt --check --package codex-rollout` 通过（仅既有 stable rustfmt warning），`git diff --check` 通过。进入 audit 复审。

2026-05-30T17:56:52+08:00：M4 audit 复审 worker `孔丘 the 8th` 返回 `pass`。已写 `.pipeline/reports/019-session-discovery-strategy-official-codex.md`，M4 标记 completed。按用户“做完这个 Milestone 后先暂停”的要求，C8 暂停在 M5 `Request Tool And Request Input UX For Plugin` 的 `write_tests` 前；安全恢复命令为 `/hw:resume`。

2026-05-30T20:54:22+08:00：用户要求继续完成全部 C8。已创建执行租约 `.pipeline/.lock`，将 C8 从 M5 前暂停状态切回执行中，并进入 M5 `Request Tool And Request Input UX For Plugin` 的 `write_tests`。

2026-05-30T21:08:21+08:00：M5 test worker `孟轲 the 8th` 完成并关闭。新增 app-server Request Input 红测，覆盖 native Workflow Ask 元数据、Markdown fallback 卡片、authorization-intent 安全边界和普通澄清不强制卡片。主代理复跑 `cargo test -p codex-app-server --test all request_user_input -- --nocapture`，稳定为 6 passed / 2 failed，进入 `implement`。

2026-05-30T21:20:56+08:00：M5 implement worker `孔丘 the 9th` 完成并关闭。生产改动保留 Workflow Ask 元数据并改进 unsupported-native Ask Markdown fallback。主代理 green 验证通过：`cargo test -p codex-app-server --test all request_user_input -- --nocapture` 8 passed；`cargo fmt --check --package codex-app-server --package codex-app-server-protocol --package codex-protocol --package codex-core` 通过（仅既有 stable rustfmt warning）；`git diff --check` 通过。进入只读 audit。

2026-05-30T21:28:00+08:00：M5 audit worker `孟轲 the 9th` 返回 `needs_changes` 并关闭。阻塞点：Workflow metadata 由 question id 派生，而不是从 request_user_input 工具入参显式保留，可能误标普通问题或丢失合法 Workflow metadata。M5 进入 audit-fix 测试轮。

2026-05-30T21:30:04+08:00：M5 audit-fix test worker `孟轲 the 10th` 完成并关闭。新增 3 个红测覆盖显式 Workflow metadata 非保留 id 保留、保留 id 普通问题不误标、fallback 使用显式 metadata。主代理复跑 `cargo test -p codex-app-server --test all request_user_input -- --nocapture`，稳定为 8 passed / 3 failed，进入 `implement_fix`。

2026-05-30T21:46:00+08:00：M5 audit-fix implement worker `孔丘 the 10th` 完成部分生产修复后因范围阻塞而停止。它已将显式 Workflow metadata 接入 core/app-server/protocol/tool schema，但新增 core 字段要求 `codex-rs/core/src/mcp_skill_dependencies.rs` 与 `codex-rs/core/src/mcp_tool_call.rs` 的既有构造器补默认字段。将启动 replacement implement worker 接手这个收尾范围并复跑验证。

2026-05-30T21:45:29+08:00：M5 replacement audit-fix implement worker `孔丘 the 11th` 完成并关闭。显式 `workflowDecisionType` / `intentOnly` / `grantsApproval` 现在从 core request model 透传到 app-server v2 question，并且普通 MCP prompts 明确不带 Workflow metadata。主代理 green 验证通过：`cargo test -p codex-app-server --test all request_user_input -- --nocapture` 11 passed；fmt check 与 git diff check 均通过（仅既有 stable rustfmt warning / deprecated warning）。进入 audit 复审。

2026-05-30T21:49:12+08:00：M5 audit 复审 worker `孟轲 the 11th` 返回 `pass`。已写 `.pipeline/reports/020-request-tool-request-input-plugin-ux.md`，M5 标记 completed。按用户要求继续完成全部 C8，立即进入 M6 `Markdown Workflow Surface And Remote SSH Acceptance` 的 `write_tests`。

2026-05-30T22:02:00+08:00：M6 test worker `孔丘 the 12th` 完成并关闭。新增 app-server `workflow_markdown_surface` 契约红测和 Remote SSH acceptance checklist。主代理确认当前 focused 命令 compile-red 于缺少 `codex_app_server::workflow_markdown_surface`，进入 `implement`。

2026-05-30T22:00:38+08:00：M6 implement worker `孟轲 the 12th` 完成并关闭。新增 app-server `workflow_markdown_surface` helper/contract。主代理 green 验证通过：`workflow_markdown_surface` 2 项、`request_user_input` 11 项、`cargo fmt --check --package codex-app-server` 与 `git diff --check` 均通过。进入只读 audit，重点确认 helper 是否需要接入 live plugin thread emission。

2026-05-30T22:03:30+08:00：M6 audit worker `孟轲 the 13th` 返回 `needs_changes` 并关闭。阻塞点：Markdown surface helper 只被导出和单测覆盖，没有接入 live app-server/plugin thread emission。M6 进入 audit-fix 测试轮。

2026-05-30T22:14:00+08:00：M6 audit-fix test worker `孔丘 the 13th` 完成并关闭。扩展 live unsupported native Ask fallback 测试，要求实际 plugin-facing `ThreadItem::AgentMessage` 包含 M6 Workflow Surface wrapper 且保留 M5 fallback 内容；当前 focused 测试按预期失败，进入 `implement_fix`。

2026-05-30T22:21:15+08:00：M6 audit-fix implement 验证通过：live fallback emission 测试、`request_user_input` 11 项、`workflow_markdown_surface` 2 项、`cargo fmt --check --package codex-app-server` 与 scoped `git diff --check` 均通过。已启动只读复审 worker `孟轲 the 14th`，重点复查 live plugin-facing fallback wiring 和 Remote SSH acceptance checklist 是否诚实。

2026-05-30T22:23:33+08:00：M6 只读复审 worker `孟轲 the 14th` 返回 `pass`。确认 live unsupported native Ask fallback 与 recovered pending Ask fallback replay 均使用 M6 Workflow Markdown surface wrapper；M6 报告写入 `.pipeline/reports/021-markdown-workflow-surface-remote-ssh-acceptance.md`。Remote SSH 手工验收仍明确为 checklist 待真实环境执行，不作为已完成发布验收声明。进入 C8 final validation。

2026-05-30T22:31:31+08:00：C8 final validation 发现一个编译回归：`cargo test -p codex-cli --test vscode` 编译 `codex-tui` 时，`tui/src/bottom_pane/request_user_input/workflow_cards.rs` 的 `ToolRequestUserInputQuestion` literal 缺少 M5 新增的 `workflow_decision_type`、`intent_only`、`grants_approval` 字段。已启动 final-validation implement-fix worker `孔丘 the 15th` 做窄修复。

2026-05-30T22:41:52+08:00：C8 final-validation fix 完成并通过主验证：TUI 侧 `ToolRequestUserInputQuestion` literals 已补齐新增 metadata 字段并默认为 `None`；`cargo test -p codex-cli --test vscode` 10 项通过，`cargo fmt --check --package codex-tui`、多包 fmt check 与 `git diff --check` 均通过。已启动只读复审 worker `孟轲 the 15th` 复查该窄修复语义。

2026-05-30T22:44:30+08:00：C8 全部完成。final-validation fix 只读复审 worker `孟轲 the 15th` 返回 `pass`；确认 TUI 侧所有 `ToolRequestUserInputQuestion` literal 均包含 M5 新 metadata 字段，`None` 默认不会误标 Workflow 决策，也不会清空 app-server 真实 metadata。Cycle summary 写入 `.pipeline/reports/022-cycle-summary.md`，C8 状态标记 completed。

2026-05-30T23:57:14+08:00：进入 C8 后 release prep。版本提升到 `0.134.0-vsp.6`，生成 Linux x64 GNU/glibc debug preview 包 `.pipeline/release/C8-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz`，SHA-256 为 `666ee5841238914d200a496339d5d103b220fb933484defd3b4299fe959fa3bc`。包内 `vsp-codex --version`、archive checksum、解压后 smoke、`rg`/`bwrap`/`zsh` 资源检查均通过；GitHub 外部发布仅写草稿 `.pipeline/reports/024-github-release-draft-v0.134.0-vsp.6.md`，未发布。

2026-05-31T00:14:46+08:00：v0.134.0-vsp.6 发布前最终验证通过：`request_user_input` 11 项、`workflow_markdown_surface` 2 项、`thread_resume` 37 项、`official_codex_discovery` 8 项、`codex-cli --test vscode` 10 项均通过；多包 `cargo fmt --check` 通过（仅既有 stable rustfmt warning），`git diff --check` 通过。只读 release prep audit worker 返回 pass，指出的 `.pipeline/PROGRESS.md` 顶部状态和 `PROJECT-SUMMARY.md` 过期信息已修正。

2026-05-31T00:20:00+08:00：本地 release commit/tag 已准备，但 GitLab 远端发布被认证阻塞：`git push origin main` 和 `git push origin v0.134.0-vsp.6` 均返回 `Permission denied (publickey,password)`；HTTPS + `glab auth git-credential` 也被 GitLab HTTP Basic 拒绝。`glab` API 可用但远端查不到本地 release commit，因此未创建 GitLab Release，避免错误指向旧远端 ref。

2026-05-31T13:01:44+08:00：v0.134.0-vsp.6 已发布到 GitLab fork `heyx/Codex`。`main` 和 tag `v0.134.0-vsp.6` 均指向 commit `2070ca9d2fb678076522b6b4391d0a99d3756cc8`；GitLab Release 地址为 `https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.134.0-vsp.6`。由于 GitLab uploads endpoint 对 257 MB archive 返回 `413 Request Entity Too Large`，Release assets 改为链接到 tag 下的仓库文件：`vsp-codex-preview-linux-x64-gnu-debug.tar.gz` 和 `.sha256`。GitHub upstream 未推送，GitHub Release 未发布。

2026-05-31T20:32:08+08:00：修复 VSCode 插件点击历史会话后仍报 `Failed to resume chat: Model provider `` not found` 的实际路径。根因是插件请求参数本身可能携带空白 `modelProvider`，C8 之前只归一化了 persisted history 里的空 provider。已在 app-server 请求边界将空白 `model_provider` 视为 unset，覆盖 `thread/start`、`thread/resume`、`thread/fork`、running resume mismatch logging 和 `thread/settings/update`。验证通过：`thread_processor_behavior_tests` 41 项、`thread_settings_build_params` 2 项、`cargo fmt --check --package codex-app-server`、scoped `git diff --check`；只读 Subagent 复审结论 `pass`。

2026-05-31T23:07:25+08:00：完成 C9 自查补齐。用户指出“Subagent 名字、Plan 计数器、提醒自己更新 Plan”未真正完成后，派出两个只读 Subagent 审核并按 findings 修复：内部 GitLab/local built-in/default Subagent 命名统一到 VSP Lab 名字，古代人名仅保留为未来外部 GitHub fork/config 备用；修正 stale TUI explorer fallback 测试；Plan 面板/terminal title 保持 `当前 SubPlan | 当前 Plan` 分离计数，无 active SubPlan 时只显示 root Plan 计数；Plan freshness 识别 structured SubPlan active work，并在 `.pipeline/state.yaml`、`.pipeline/cycle.yaml`、`.pipeline/log.yaml`、`.pipeline/PROGRESS.md` 于 `update_plan` 后变化时注入 Workflow Plan 刷新提醒。验证通过：Subagent 命名 core/TUI focused tests、Plan panel/status focused tests、structured subplan freshness、Workflow state-file freshness、`plan_update_gate`、多包 `cargo fmt --check`（仅既有 stable rustfmt warning）和 `git diff --check`。

2026-06-07T19:38:15+08:00：完成 `v0.134.0-vsp.7.1` release surface 发布。此前 18:12 的 7.1 tag-only 记录已由正式发布提交 supersede：`main` 和 tag `v0.134.0-vsp.7.1` 现在均指向 `a7c3ec1c0cd6b05f08bf66172ef525e65aa1bd63`；README、安装文档、用户指南和稳定更新 manifest 均指向 GitLab Release downloads 路径 `https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.134.0-vsp.7.1/downloads/vsp-codex-preview-linux-x64-gnu.tar.gz`，SHA-256 为 `d33501b66d7e2ec5ac2d9ce7d3f3a01f0726b1abb5150754ba354b8002f8f630`。GitLab Release 已创建并挂载 package/sha256 assets；远程 main raw README 和 manifest 已验证显示 7.1。

2026-06-07T19:59:28+08:00：本机安装切换到 `0.134.0-vsp.7.1` standalone release。`~/.local/bin/vsp-codex` 现在指向 `~/.vsp-codex/packages/standalone/current/bin/vsp-codex`，`vsp-codex --version` 返回 `0.134.0-vsp.7.1`；VSCode wrapper `~/.vsp-codex/vscode/vsp-codex` 固定 `CODEX_HOME=/home/heyx/.vsp-codex` 后 exec 本机可见命令。随后修复用户用旧 debug 版本新建会话产生的官方 SQLite provider cache 空值：备份 `$HOME/.codex/state_5.sqlite*` 到 `$HOME/.codex/session-provider-backups/20260607T195928+0800/sqlite-empty-provider-fix`，将 2 条 `threads.model_provider=''` 回填为 `OpenAI`。复扫验证：官方 state DB `OpenAI=1831, EMPTY_ROWS=0`，VSP state DB `vsp=271, OpenAI=107, EMPTY_ROWS=0`，两库 `pragma integrity_check=ok`，共享 JSONL sessions/index 未发现空 provider 或旧 `vsp`/`TTAPI-Codex` provider。

2026-06-08T21:22:31+08:00：按 Hypo-Workflow C19 目标适配 Gate 刷新 OpenCode/Hypo adapter 表面：新增 `/hw:plan:technical-stack` 与 `/hw:plan:architecture`，移除用户态 `/hw:plan:confirm`，将 generated agents 的 Plan checkpoint 文案从 `P1/P2/P3/P4` 改为 `Discover / Technical Stack / Architecture / Decompose / Generate`，并投影 Four-Rule Discipline 与 Plan Gate Visibility。验证通过：`opencode.json` 和 `.opencode/hypo-workflow.json` JSON 解析、`.opencode/plugins/hypo-workflow.ts` import、旧 `plan:confirm` / `P1/P2/P3/P4` 扫描无命中、`git diff --check`。`pnpm prettier --check` 未执行成功：目标仓当前 pnpm 环境缺少 `prettier` 命令且 node v20 不满足 package engine >=22。

2026-06-08T21:54:54+08:00：进入 C19 发布执行。目标版本为 `v0.134.0-vsp.7.2`，仅推送 GitLab `origin git@gitlab.vsplab.cn:heyx/Codex.git`，不推 upstream。发布范围包含 C19 OpenCode adapter、新增 Plan phase 命令映射、README/安装/用户文档、版本元数据、release manifest、`.pipeline/release/v0.134.0-vsp.7.2/` 产物，以及本文件和 `.pipeline/log.yaml` 的 Workflow 记录；运行态 `.pipeline/chat*`、`.pipeline/inbox`、`.pipeline/knowledge`、`.pipeline/patches.compact.md` 排除提交。用户已确认接受 `pnpm prettier --check` 因 Node/prettier 环境无法运行的验证缺口。
