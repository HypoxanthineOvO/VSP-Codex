# C4 体验优化 + 发布 Progress

更新时间：2026-05-28T13:28:04+08:00

## 当前状态

- 当前 Cycle：C4 `体验优化 + 发布`
- 类型：feature
- Preset：tdd
- 阶段：completed，7/7 milestones 已完成；等待用户验收
- Prompts：7
- Worker Separation：strict，`test` / `implement` / `audit` 分离，已授权 `/hw:start` 和 `/hw:resume`

## Scope

本 Cycle 定义为“能发布给大家用的 VSCode/TUI preview”，不是完整体验优化大版本。

Blocker：

1. VSCode 官方插件自动接入和 rollback。
2. 至少 Linux x64 预览包、quickstart、smoke evidence。
3. Rename 写 canonical session/thread metadata，并在 TUI/app-server/VSCode 可见。
4. Thinking 只接真实 summary/raw reasoning 能力；不伪造。
5. 必要操作不依赖 VSCode 终端冲突键。

包含但不作为完整系统重构：

1. Markdown 已有 renderer 的颜色补齐。
2. 默认淡色块配色和输入框明显化。
3. 性能测量与低风险启动优化。

顺延：

1. 完整 Color Theme picker、三组主题持久化和完整 fallback matrix。
2. 完整 command palette/focus stack/键盘复制选择系统。
3. resume metadata cache、recent preview cache、progressive stale-cache verification。
4. 自研 VSCode 插件或 patch 官方插件 UI。

## Milestones

1. M1 Release Spine + VSCode Hijack：completed
2. M2 Canonical Rename：completed
3. M3 Reasoning Capability Gate：completed
4. M4 VSCode-Terminal-Safe Keymap + Composer Prominence：completed
5. M5 Markdown Color Refinement + Default Subtle Blocks：completed
6. M6 Performance Evidence + Low-Risk Startup Fixes：completed
7. M7 Preview Release Validation：completed

## Generated Prompts

- `.pipeline/prompts/001-release-spine-vscode-hijack.md`
- `.pipeline/prompts/002-canonical-rename.md`
- `.pipeline/prompts/003-reasoning-capability-gate.md`
- `.pipeline/prompts/004-vscode-terminal-safe-keymap-composer.md`
- `.pipeline/prompts/005-markdown-color-default-blocks.md`
- `.pipeline/prompts/006-performance-evidence-startup.md`
- `.pipeline/prompts/007-preview-release-validation.md`

## Plan Artifacts

- `.plan-state/p0-configure.yaml`
- `.plan-state/p1-discover.yaml`
- `.plan-state/decompose.yaml`
- `.plan-state/technical-route.md`

## Recent Activity

- 2026-05-28T01:37:44+08:00：用户通过 `$hypo-workflow:start` 确认开始执行 C4。
- 2026-05-28T01:37:44+08:00：创建执行租约 `.pipeline/.lock`，M1 进入严格 Worker Separation 执行。
- 2026-05-28T01:39:10+08:00：启动 M1 `test` 工作者 `019e6a84-b564-7a82-a652-ada340132a38`，限定写入 CLI 测试和 `.pipeline/reviews/C4/M1/test.md`。
- 2026-05-28T01:49:27+08:00：`test` 工作者完成并关闭；新增 `codex-rs/cli/tests/vscode.rs` 和 `.pipeline/reviews/C4/M1/test.md`，红测信号为缺少 `vsp-codex vscode install`。
- 2026-05-28T01:50:18+08:00：启动 M1 `implement` 工作者 `019e6a8e-f75f-78c3-805e-e2ec24293f7f`，限定写入 CLI production、M1 report 和 implement review artifact。
- 2026-05-28T02:01:08+08:00：实现 worker 仍在运行；主代理仅刷新租约和保持编排，不接管生产实现。
- 2026-05-28T02:02:49+08:00：`implement` 工作者完成并关闭；`just test -p codex-cli vscode` 通过 7/7，随后运行 `just fix -p codex-cli` 和 `just fmt`。
- 2026-05-28T02:04:21+08:00：主代理复验 `cd codex-rs && just test -p codex-cli vscode`，7/7 通过，bench smoke 也正常结束。
- 2026-05-28T02:06:07+08:00：`audit` 工作者完成并关闭，结论 `needs_changes`；主要问题是 rollback 可能恢复错误备份或错误 scope。
- 2026-05-28T02:06:47+08:00：启动 M1 `test` retry 工作者 `019e6a9e-0970-7ca3-bd99-dbfc06ecc285`，限定补 rollback safety 测试。
- 2026-05-28T02:18:46+08:00：`test` retry 工作者完成并关闭；新增 rollback scope safety 红测，目前 8 个 vscode 测试中 7 通过、1 个按预期失败。
- 2026-05-28T02:19:27+08:00：启动 M1 `implement` retry 工作者 `019e6aa9-937c-7543-a851-dd8279296cc3`，限定修复 rollback scope safety。
- 2026-05-28T02:22:16+08:00：`implement` retry 工作者完成并关闭；rollback 改为先解析当前 scope，只选择该 scope 前缀的备份；`just test -p codex-cli vscode` 通过 8/8，随后 `just fix -p codex-cli` 通过。
- 2026-05-28T02:25:22+08:00：`audit` retry 工作者完成并关闭，结论 `pass`；M1 无剩余 blocking finding。
- 2026-05-28T02:25:56+08:00：M1 标记 completed，恢复指针写入 M2 Canonical Rename。
- 2026-05-28T10:19:01+08:00：用户要求本 Cycle 自动连续执行；创建新执行租约并启动 M2 Canonical Rename。
- 2026-05-28T10:19:49+08:00：启动 M2 `test` 工作者 `019e6c61-6598-7b21-aff1-e9275ceb7123`，限定写入 rename 测试和 `.pipeline/reviews/C4/M2/test.md`。
- 2026-05-28T10:33:32+08:00：M2 `test` 工作者仍在运行；主代理只刷新租约并做只读追踪，不接管测试。
- 2026-05-28T10:35:56+08:00：M2 `test` 工作者完成并关闭；新增 app-server restart/empty-name 测试和 rollout blank-entry 测试，focused coverage 全绿。
- 2026-05-28T10:36:38+08:00：启动 M2 `implement` 工作者 `019e6c70-c597-7a80-94b5-501ed6498526`，确认生产路径或产出 no-op implementation evidence。
- 2026-05-28T10:39:27+08:00：M2 `implement` 工作者完成并关闭；确认生产 runtime no-op，写入 M2 report 和 implement evidence。
- 2026-05-28T10:42:17+08:00：M2 `audit` 工作者完成并关闭，结论 `pass`；M2 标记 completed，并自动进入 M3 Reasoning Capability Gate。
- 2026-05-28T10:43:30+08:00：启动 M3 `test` 工作者 `019e6c76-f75f-7c51-8467-3c248ef6e6e8`，限定写入 reasoning tests/snapshots 和 `.pipeline/reviews/C4/M3/test.md`。
- 2026-05-28T10:52:19+08:00：M3 `test` 工作者完成并关闭；新增 app-server-protocol/core/TUI reasoning 覆盖，红点为空 reasoning item 仍渲染 transcript 输出。
- 2026-05-28T10:58:57+08:00：M3 `implement` 工作者完成并关闭；修复 TUI transcript 中空 reasoning item 不再生成 Thinking/Reasoning block 或 fallback 输出。
- 2026-05-28T11:02:10+08:00：M3 `audit` 工作者完成并关闭，结论 `pass`；M3 标记 completed，并自动进入 M4。
- 2026-05-28T11:04:02+08:00：启动 M4 `test` 工作者 `019e6c89-ddda-70c0-a3ef-ebcc45b3b7cd`，限定写入 keymap/request_user_input/composer tests 和 snapshots。
- 2026-05-28T11:12:16+08:00：M4 `test` 工作者完成并关闭；红测锁定 `list.move_up` 默认冲突键、Ask ranking/multi-question Ctrl 提示、composer focused/unfocused 渲染相同。
- 2026-05-28T11:13:48+08:00：启动 M4 `implement` 工作者 `019e6c92-c94c-77c1-afab-3993c1e11b16`，限定写入 keymap/request_user_input/chat_composer/footer production 和 M4 implement/report artifacts。
- 2026-05-28T11:23:03+08:00：M4 `implement` 工作者完成并关闭；新安全测试已通过，但旧 keymap/request_user_input 断言和快照仍要求 Ctrl 默认/提示，进入 test retry。
- 2026-05-28T11:23:55+08:00：启动 M4 `test` retry 工作者 `019e6c9c-1083-7010-a4d6-60fece55fa25`，限定更新旧 keymap/request_user_input 测试断言和相关快照。
- 2026-05-28T11:27:54+08:00：M4 `test` retry 工作者完成并关闭；主代理复验 `keymap` 108/108、`request_user_input` 81/81、`chat_composer` 186/186 全部通过，且无 pending snapshots。
- 2026-05-28T11:28:47+08:00：启动 M4 `audit` 工作者 `019e6ca0-8169-70b0-bbfd-0965cee833c4`，只读检查默认键位、Ask 提示、composer focus 和 worker 范围。
- 2026-05-28T11:31:42+08:00：M4 `audit` 工作者完成并关闭，结论 `needs_changes`；footer shortcut overlay 仍显示 `ctrl + c to exit` 和 `⌥ + ,/.` Alt-only reasoning 快捷键，现有测试未覆盖实际渲染格式。
- 2026-05-28T11:32:32+08:00：启动 M4 第二轮 `test` retry 工作者 `019e6ca3-efc0-7bf1-9aa4-27b944699a76`，限定补 footer overlay 实际渲染格式红测。
- 2026-05-28T11:35:09+08:00：M4 第二轮 `test` retry 工作者完成并关闭；`just test -p codex-tui footer` 按预期红，失败信号为 overlay 仍显示 `ctrl + j`、`ctrl + c`、`⌥ + ,/.`。
- 2026-05-28T11:36:10+08:00：启动 M4 第二轮 `implement` retry 工作者 `019e6ca7-3aea-7283-9d03-dcd1de6b2d6e`，限定修复 footer shortcut overlay production 和相关快照/report。
- 2026-05-28T11:44:24+08:00：M4 第二轮 `implement` retry 工作者完成并关闭；主代理复验 `footer` 42/42、`chat_composer` 186/186、`keymap` 108/108、`request_user_input` 81/81 全部通过，且无 pending snapshots。
- 2026-05-28T11:45:18+08:00：启动 M4 第二轮 `audit` 工作者 `019e6caf-a103-7480-a2c9-bcbc89002a36`，只读复查 footer overlay finding 和 M4 全量验收点。
- 2026-05-28T11:49:02+08:00：M4 第二轮 `audit` 工作者完成并关闭，结论 `pass`；`just fix -p codex-tui` 和 `just fmt` 通过。M4 标记 completed，并自动进入 M5。
- 2026-05-28T11:50:20+08:00：启动 M5 `test` 工作者 `019e6cb4-3b21-7a91-b898-5d84a4484d73`，限定写入 Markdown/history-cell 测试、快照和 `.pipeline/reviews/C4/M5/test.md`。
- 2026-05-28T11:58:28+08:00：M5 `test` 工作者完成并关闭；红测锁定 strong/list/table Markdown 颜色不足，以及 assistant/reasoning 默认块 accent 不可区分。
- 2026-05-28T11:59:24+08:00：启动 M5 `implement` 工作者 `019e6cbc-9109-7463-b75f-aaeb95ac45b4`，限定修复 Markdown production style palette、history-cell 默认 accent 和 M5 implement/report artifacts。
- 2026-05-28T12:09:54+08:00：M5 `implement` 工作者仍在运行；主代理刷新租约，未接管生产实现。
- 2026-05-28T12:13:25+08:00：M5 `implement` 工作者完成并关闭；主代理复验 `markdown_render` 122/122、`history_cell` 121/121 通过，且无 pending snapshots。
- 2026-05-28T12:14:14+08:00：启动 M5 `audit` 工作者 `019e6cca-17f5-7bd1-85f7-80e786cde747`，只读检查 Markdown palette、history block accent、scope creep 和 layout 风险。
- 2026-05-28T12:17:25+08:00：M5 `audit` 工作者完成并关闭，结论 `needs_changes`；`legacy_unconstrained()` 会让普通 `**Strong**`、`*Emphasis*`、`- item` 路径仍缺少 foreground color。
- 2026-05-28T12:18:13+08:00：启动 M5 `test` retry 工作者 `019e6ccd-cbb1-78c0-8aed-d293a4d8f117`，限定补最小 Markdown 输入路径红测。
- 2026-05-28T12:20:26+08:00：M5 `test` retry 工作者完成并关闭；`markdown_render` 按预期红，新增 3 个 minimal-path 测试失败。
- 2026-05-28T12:21:17+08:00：启动 M5 `implement` retry 工作者 `019e6cd0-9801-74a1-9e8f-c1dd427a1838`，限定修复 `markdown_render` production 样式回退和 M5 implement/report artifacts。
- 2026-05-28T12:25:40+08:00：M5 `implement` retry 工作者完成并关闭；新增 minimal foreground 测试已通过，但旧 `markdown_render_tests.rs` exact-style 断言仍期望无 foreground，`markdown_render` 仍红。
- 2026-05-28T12:26:32+08:00：启动 M5 第二轮 `test` retry 工作者 `019e6cd5-5fcd-7682-8f8d-7204f0708370`，限定更新旧 Markdown exact-style 断言/相关快照。
- 2026-05-28T12:30:34+08:00：M5 第二轮 `test` retry 工作者完成并关闭；主代理复验 `markdown_render` 125/125、`history_cell` 121/121 通过，且无 pending snapshots；`legacy_unconstrained` 回退已移除。
- 2026-05-28T12:31:27+08:00：启动 M5 第二轮 `audit` 工作者 `019e6cd9-e391-7420-a808-426567dab5ac`，只读复查 legacy fallback blocker 是否消除和 M5 验收点。
- 2026-05-28T12:38:18+08:00：M5 第二轮 `audit` 工作者完成并关闭，结论 `pass (retry)`；主代理补充移除 disallowed `.yellow()`，`just fmt` 和 `just fix -p codex-tui` 通过，M5 标记 completed。
- 2026-05-28T12:38:18+08:00：自动进入 M6 Performance Evidence + Low-Risk Startup Fixes，准备启动 `test` 工作者为 cold start、resume/history 和 VSCode initialize 添加可复现实证覆盖。
- 2026-05-28T12:39:39+08:00：启动 M6 `test` 工作者 `019e6ce1-6b53-7382-8bf4-d3c530f7e9e4`，限定测试/fixture/evidence 脚本和 `.pipeline/reviews/C4/M6/test.md`，禁止 production 实现和 cache 系统。
- 2026-05-28T12:43:23+08:00：M6 `test` 工作者仍在运行；主代理刷新租约并继续只读梳理，不接管测试实现。
- 2026-05-28T12:48:15+08:00：M6 `test` 工作者完成并关闭；新增 resume picker 首屏 lazy-load 测试和 VSCode initialize 慢插件 warmup 非阻塞测试，`codex-tui startup` 45/45 与 `codex-app-server initialize` 14/14 通过。
- 2026-05-28T12:49:23+08:00：启动 M6 `implement` 工作者 `019e6cea-5364-7f70-b729-33cd90a65ab3`，限定 release evidence runner/report hook 和 M6 implement/report 工件。
- 2026-05-28T12:53:05+08:00：M6 `implement` 工作者仍在运行；主代理刷新租约，未接管实现。
- 2026-05-28T12:59:49+08:00：M6 `implement` 工作者完成并关闭；新增 `scripts/collect_performance_evidence.py` 和 `.pipeline/evidence/performance-startup.json`，主代理复验 py_compile、runner smoke 和 diff hygiene 通过。
- 2026-05-28T13:00:37+08:00：启动 M6 `audit` 工作者 `019e6cf4-a033-7160-9f57-31d59043a6ab`，只读审查 evidence runner 质量、skip 诚实性、无 cache 系统和冷启动证据语义。
- 2026-05-28T13:02:12+08:00：M6 `audit` 工作者完成并关闭，结论 `needs_changes`；`cold_tui_start` 当前把 bracketed-paste escape 当成 passed，需要 retry 修正为 meaningful visible text 或 structured partial/skipped。
- 2026-05-28T13:03:15+08:00：启动 M6 `implement` retry 工作者 `019e6cf6-fca3-7d62-894e-b9f81254b8b5`，限定修复 cold TUI start evidence 语义。
- 2026-05-28T13:06:11+08:00：M6 `implement` retry 工作者完成并关闭；`cold_tui_start` 现在剥离 ANSI/control 后等待 meaningful visible text，控制字节-only 时记录 `skipped`；主代理复验 py_compile、runner smoke、JSON parse 和 diff hygiene 通过。
- 2026-05-28T13:07:13+08:00：启动 M6 `audit` retry 工作者 `019e6cfa-a1fe-7a80-8e9f-41e82372ce15`，只读复查 cold TUI start blocker 是否消除。
- 2026-05-28T13:08:16+08:00：M6 `audit` retry 工作者完成并关闭，结论 `pass`；M6 标记 completed，并自动进入 M7 Preview Release Validation。
- 2026-05-28T13:09:39+08:00：启动 M7 `test` 工作者 `019e6cfc-e581-7f72-abfb-f274639e5aca`，限定发布验证矩阵、focused tests/smokes 和 `.pipeline/reviews/C4/M7/test.md`。
- 2026-05-28T13:11:12+08:00：M7 `test` 工作者仍在运行；主代理刷新租约，并完成只读 package builder / VSCode command 路径梳理。
- 2026-05-28T13:16:46+08:00：M7 `test` 工作者完成并关闭；focused matrix 通过，但 `git diff --check` 发现 request_user_input snapshot trailing whitespace，进入 test retry 修复 hygiene。
- 2026-05-28T13:17:50+08:00：启动 M7 `test` retry 工作者 `019e6d04-602e-7733-bba2-38fa24acf61b`，限定修复 snapshot trailing whitespace 和复验 diff hygiene。
- 2026-05-28T13:18:45+08:00：M7 `test` retry 工作者完成并关闭；`git diff --check` 和 TUI pending snapshots 均通过，进入 release artifact/report implement。
- 2026-05-28T13:19:38+08:00：启动 M7 `implement` 工作者 `019e6d05-ff4a-7733-99f0-bf35d2bbcf53`，限定 Linux x64 preview artifact、checksum、quickstart/known limitations 和 release report。
- 2026-05-28T13:25:37+08:00：M7 `implement` 工作者完成并关闭；产出 Linux x64 GNU debug preview 包、checksum 和 release report，主代理复验 checksum、archive 内容、解包 `vsp-codex --version` 和 scoped diff hygiene 通过。
- 2026-05-28T13:26:32+08:00：启动 M7 `audit` 工作者 `019e6d0c-4e67-7033-b556-e4d3944075c8`，只读审查 preview artifact、release evidence、已知限制和 worker separation。
- 2026-05-28T13:28:04+08:00：M7 `audit` 工作者完成并关闭，结论 `pass`；M7 标记 completed，C4 执行完成，预览包和 release evidence 已落盘。

## Next Step

等待用户验收 C4。预览包位于 `.pipeline/release/C4-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz`。
