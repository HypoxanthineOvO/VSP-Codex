# C2 P2 Technical Route: 基础体验优化

状态：`generated`  
时间：2026-05-25T23:28:45+08:00  
来源：[p1-discover.yaml](p1-discover.yaml)

本阶段已生成执行 prompt。用户已要求后续不设置 Gate；之后输入 Start 即视为执行确认，执行应从 M1 连续推进到 M6，只在真实阻塞、重复验证失败、缺少外部输入或最终人工验收时停下。

## 总体顺序

1. M1 默认 YOLO 与底栏 Context
2. M2 Subagent 核心策略
3. M3 Subagent 常驻卡片与键盘查看
4. M4 Hypo-Translate 运行时翻译
5. M5 MCP 提示迁移与静态汉化
6. M6 集成回归与实际验收

顺序理由：先固定运行策略和核心状态契约，再做 TUI 面板与翻译，最后做大范围汉化和集成回归，减少 snapshot 和 UI 状态互相干扰。

## M1 默认 YOLO 与底栏 Context

目标：所有 `vsp-codex` 入口默认无审批全权限运行，并在底部状态栏右侧显示 `Context x%`。

范围：
- TUI 与 `vsp-codex exec` 都默认 YOLO。
- 默认效果等价于 `approval_policy=never` + `danger-full-access`。
- 用户显式配置仍应能覆盖。
- Context 复用现有 footer/status line 右侧 indicator 和 Context 数据项。

技术方案：
- 在 VSP 入口层注入默认运行策略，避免让用户打 `--dangerously-bypass-approvals-and-sandbox`。
- Context 走现有 `ContextRemaining` / `ContextUsed` / `ContextWindowSize` 能力，不重建底栏。

验证：
- `just test -p codex-cli`
- `just test -p codex-tui`
- `just exec -- true`

审计重点：
- 默认 YOLO 不误伤非 VSP 入口。
- 显式用户配置优先级正确。
- 底栏窄屏无重叠。

## M2 Subagent 核心策略

目标：移除 Subagent 显式授权软限制，默认鼓励使用；上限 32；中文昵称池；默认派任务式上下文。

范围：
- 删除工具说明里的 “only if user explicitly asks”。
- 默认最多 32 个 Subagent。
- 中文名字池：李萌、杨超等按 P1 记录。
- 默认不 full-history fork，保留显式 full-history 选项。
- 保留并强化 `model` / `reasoning_effort` / `service_tier` 指定能力。

技术方案：
- 修改 `multi_agents_spec` 的模型可见说明。
- 调整 `multi_agent_v2` 默认并发/数量映射。
- 放宽 `nickname_candidates` 校验，允许中文，仍拒绝空名、重复名、控制字符。
- 默认 spawn 策略避开 full-history，从而允许指定 role/model/reasoning。

验证：
- `just test -p codex-core`
- `just test -p codex-tui`

审计重点：
- 没有残留显式授权软限制。
- 32 表示 32 个 Subagent，不是误算总线程。
- full-history 显式模式仍可用。

## M3 Subagent 常驻卡片与键盘查看

目标：TUI 展示活跃 Subagent 卡片，并可用键盘查看 Subagent 命令、返回内容和状态详情。

范围：
- 卡片格式参考：`杨超 · GPT 5.5 xhigh · Audit · 运行中 · 24h`。
- 只显示活跃或未 close 的 Subagent。
- close 后从常驻卡片消失。
- 复用现有 agent navigation / collab event 数据。

技术方案：
- 新增独立 TUI 模块管理 `SubagentPanelState`，避免继续膨胀 `chatwidget.rs`。
- 从 spawn/wait/close 事件更新状态。
- 插入 bottom pane 的运行状态区域。
- 扩展 `/subagents` 或相关选择器实现详情查看。

验证：
- `just test -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui`

审计重点：
- 状态来源不分叉。
- 中文宽度和窄屏布局正确。
- close 后没有残留。

## M4 Hypo-Translate 运行时翻译

目标：接入本机 Hypo-Translate，把 reasoning、Subagent 内容、工具摘要/返回内容翻译成中文。

范围：
- 默认服务：`http://127.0.0.1:8998`。
- 优先 `POST /translate/stream`。
- 只显示 `delta`、`revise`、`commit`、`passthrough` 等可见事件。
- 不显示、不持久化 `source_hidden`。
- 服务不可用时显示原文，不阻塞 TUI。
- 提供 raw/original 切换。

技术方案：
- 翻译是 TUI 展示层增强，不写回 rollout hidden source。
- 新增 Hypo-Translate client 和 SSE reducer。
- 用渲染缓存承载翻译结果。
- 只接入三类动态内容，不全局翻译。

验证：
- `just test -p codex-tui`
- `just test -p codex-core`
- fake SSE 服务覆盖事件解析和失败降级。

审计重点：
- hidden/source 原文不泄露、不持久化。
- 网络失败不阻塞 UI。
- 翻译范围不扩散。

## M5 MCP 提示迁移与静态汉化

目标：正常 MCP 启动不占主界面 Working 区；完成第一轮高频汉化。

范围：
- `Booting MCP server: context7` 这类正常启动提示不显示在主 Working 区。
- MCP 功能和失败提示保留。
- 汉化 TUI 高频文案、Slash Command 解释、设置页面、常见 Error/Warning。
- 保留英文术语：Subagent、Context Window、Provider、MCP、YOLO。

技术方案：
- MCP 启动状态与主 status header 解耦。
- 成功启动静默或只进入 `/mcp` 状态。
- 失败/取消仍写入 warning/history。
- 汉化采用局部文案替换，不先引入完整 i18n 框架，降低合并成本。

验证：
- `just test -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui`

审计重点：
- 不删除 MCP/context7 功能。
- 失败路径仍可诊断。
- 中文术语一致且不破坏布局。

## M6 集成回归与实际验收

目标：把 C2 所有体验改动跑通到真实二进制和回归测试。

范围：
- 统一 format、fix、snapshot 和聚焦测试。
- 实际运行 `vsp-codex`、`vsp-codex exec`、`/subagents`、`/mcp`、翻译降级。
- 检查 CI/依赖 lock 状态。
- 输出验收报告和遗留项。

验证：
- `just fmt`
- `just test -p codex-cli`
- `just test -p codex-core`
- `just test -p codex-tui`
- `just vsp-codex --version`
- `just exec -- true`

审计重点：
- 测试证据真实有效。
- hidden/source 翻译原文不泄露。
- CI 和依赖 lock 状态正确。

## 未解决假设

- M4 如果新增 Rust 依赖或 feature，需要同步 Bazel lock。
- 如果 core/shared 改动超过聚焦测试覆盖，应在 M6 前询问用户是否运行完整 `just test`。
- 汉化先覆盖高频 TUI 面，不承诺一次性翻完整仓库低频文档。
