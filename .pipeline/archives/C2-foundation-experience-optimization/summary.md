# Cycle C2：基础体验优化

- Cycle：C2
- 名称：基础体验优化
- 类型：feature
- 状态：completed
- 开始时间：2026-05-25T22:30:12+08:00
- 结束时间：2026-05-26T07:07:17+08:00
- Preset：tdd
- 归档路径：`.pipeline/archives/C2-foundation-experience-optimization/`
- 知识摘要：`.pipeline/archives/C2-foundation-experience-optimization/knowledge-summary.md`

## Milestones

| Milestone | 一句话结果 |
|---|---|
| M1 | `vsp-codex` 默认 YOLO 权限路径与底栏 Context 显示完成，并保留显式配置覆盖。 |
| M2 | Subagent 默认策略完成：上限 32、中文昵称池、默认鼓励使用，并保留 model/reasoning metadata。 |
| M3 | TUI 增加活跃 Subagent 常驻卡片与键盘详情查看入口。 |
| M4 | 接入 Hypo-Translate 运行时翻译，且 hidden/transcript-only reasoning 不翻译、不泄露。 |
| M5 | MCP 正常启动提示降噪，并完成第一轮高频 TUI/Slash Command 静态汉化。 |
| M6 | 完成集成回归、实际 TUI smoke、官方 remote/cloud/auth 噪声门控和发布前修复。 |
| Release prep | 准备 `0.134.0-vsp.2`：修复 resume 大历史闪烁、Context 显示、Read File 高亮和 Subagent hash 兜底。 |

## 关键数据

- `just fmt`：通过。
- `just test -p codex-tui`：最终记录为 2657 passed，4 skipped。
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`：No pending snapshots。
- `just test -p codex-cli --test vsp_entrypoint`：8 passed。
- `just test -p codex-exec`：116 passed。
- `just test -p codex-model-provider-info`：21 passed。
- `just test -p codex-core-plugins startup_remote_plugin_sync remote_plugin`：18 passed，181 skipped。
- `just fix -p codex-cli -p codex-exec -p codex-tui -p codex-core-plugins -p codex-model-provider-info`：通过。
- `cargo build -p codex-cli --bin vsp-codex`：通过。
- `vsp-codex --version`：`vsp-codex 0.134.0-vsp.2`。
- `git diff --check`：通过。
- full workspace 本机尝试：9786 passed、155 failed、5 timed out、21 skipped；已记录为环境/负载/旧官方路径风险，非 C2 目标阻断。

## 延后事项

无。C2 state 中 M1-M6 均为 completed。

## 改动摘要

C2 将 VSP Codex 的基础交互体验从 C1 的 fork 可用状态推进到更适合日常使用的默认形态：本地默认 YOLO、Context Window 可见、Subagent 默认可用且可观察、运行时翻译与静态汉化接入、MCP 启动噪声降级，并在实际 TUI smoke 中收敛官方 ChatGPT/Cloud/remote plugin 的默认噪声。

## 技术思路

实现策略以 VSP 入口层和 TUI 展示层为主，尽量避免重写核心官方架构：默认权限在 VSP entrypoint/config normalization 层处理；Subagent 策略保留现有协作协议，只调整默认上限、工具说明和展示元数据；翻译作为非阻断展示缓存，不写回 rollout hidden source；MCP 成功启动提示从主 Working 区迁移，失败路径保持显眼；发布准备阶段以实际二进制 smoke 兜底单元测试未覆盖的启动体验。

## 修改文件 / 模块

- CLI/exec 默认策略：`codex-rs/cli/`、`codex-rs/exec/`。
- TUI 状态与展示：`codex-rs/tui/src/`，包括 bottom pane、footer、chat widget、history cell、Subagent panel、Hypo-Translate、MCP startup 和 snapshots。
- Core/Subagent 策略：`codex-rs/core/src/` 多 agent 工具说明、默认限制与 metadata 保留。
- Provider/remote plugin 门控：`codex-rs/core-plugins`、`codex-rs/model-provider-info` 等相关路径。
- 发布文档：`README.md`、`docs/vsp-codex.md`、`CHANGELOG.md`。
- Workflow 证据：`.pipeline/reports/`、`.pipeline/reviews/`、`.pipeline/plan-state/`。

## 测试设计

测试按用户可见风险分层：默认策略用 CLI/exec/TUI focused tests 覆盖；Subagent 策略用 core 与 TUI 状态测试覆盖；TUI 文案、布局和 panel 变化用完整 `codex-tui` 与 insta snapshot pending 检查覆盖；翻译路径用 fake SSE 与 hidden/transcript-only 防泄露测试覆盖；最终用实际 `vsp-codex` TUI smoke 验证启动噪声、Provider 脱敏、remote plugin 门控和默认权限状态。

## 验证结果

C2 目标路径均已通过 focused 验证，并完成发布准备。唯一非绿色的大范围验证是本机 full workspace `just test`，失败集中在环境/负载敏感和旧官方路径，不作为 C2 目标阻断；相关风险已在 C2 PROGRESS 与 release prep 报告中记录。

## 预期结果

用户运行 `vsp-codex` 时应默认进入无审批全权限的 VSP 本地体验，不再被 OAuth/API Key/Base URL 官方启动噪声打断；底栏可以看到 Context 使用情况；Subagent 更容易被模型主动使用且能在 TUI 中观察；中文用户可以看到更完整的高频中文文案；翻译服务可用时可获得中文辅助阅读，服务不可用时回退原文。

## 遇到的问题

- 单元测试未覆盖真实启动噪声，需要实际 TUI smoke 才能发现 OAuth/API Key/Base URL 和 provider token 泄露风险。
- Hypo-Translate 初始实现需要多轮修复，重点是 SSE 事件形态、长连接 fragment 和 delta-only pending 行为。
- transcript-only reasoning 必须视为私密内容，不能被翻译或通过 transcript overlay 泄露。
- full workspace 本机测试存在环境/负载敏感失败，不宜直接等同于 C2 功能回归。

## 风险 / 后续

- 官方 remote/cloud/plugin 相关底层路径仍保留，只做 VSP 默认门控；后续合并 upstream 时需关注这些 gate。
- 静态汉化仍是第一轮高频覆盖，不是完整 i18n。
- 翻译能力仍依赖本机 Hypo-Translate 服务，后续可继续优化失败诊断、缓存策略和切换体验。
- C3 可在 C2 的交互基础上推进 Ask User Tools，把模型主动询问用户的能力产品化并跨 TUI/GUI 复用。
