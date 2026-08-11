# M1 默认 YOLO 与底栏 Context

## Goal
让所有 `vsp-codex` 本地入口默认进入 YOLO 行为，并在 TUI 底栏右侧展示 Context Window 使用信息。用户不需要再手动输入 `--dangerously-bypass-approvals-and-sandbox` 或额外授权参数；TUI 状态栏需要继续保持当前工作目录、分支、Provider 等信息，同时新增可快速扫读的上下文占用信息。

## Subworker Assignment Plan
Subworkers are authorized for this milestone. Worker separation is strict.

- `test`: 先补回归测试，覆盖 TUI 启动参数默认值、exec/TUI 入口权限策略、底栏渲染快照、Context Window 百分比/缺失数据显示。测试 worker 只负责测试与 snapshot，不实现产品逻辑。
- `implement`: 在测试存在后实现默认 YOLO 策略和底栏 Context Window 渲染。实现 worker 负责 CLI/TUI 入口、配置合并、状态栏数据接线，不改测试断言方向。
- `audit`: 审核默认权限是否只作用于 VSP Codex 入口，是否误伤显式配置、CI、测试环境或官方兼容路径；审核 UI 是否在窄屏下稳定。
- Main agent: 协调 worker、整合补丁、运行格式化与 scoped tests、记录证据。

## Technical Solution
把 YOLO 作为 `vsp-codex` 品牌入口的默认运行策略，而不是要求用户手动传危险参数。配置优先级应保持清楚：显式 CLI 参数和配置文件仍然可以覆盖默认值；默认值只在用户没有显式选择审批/沙箱策略时生效。

Context Window 信息放在 TUI 底栏右侧。显示内容优先使用已有 token/context usage 数据源；如果当前请求尚未产生 usage，则显示稳定占位，不制造闪烁。底栏需要兼容中文字体宽度、窄屏截断和已有 model/provider/directory/git branch 信息。

## Technical Route
1. 查找 `--dangerously-bypass-approvals-and-sandbox`、approval policy、sandbox policy、TUI startup config 的解析路径，确认 TUI 与 `codex exec` 是否共享同一配置合并层。
2. 增加回归测试，先证明当前默认不是 YOLO，底栏没有 Context Window。
3. 在 VSP entrypoint/config normalization 层设置默认 approval/sandbox 组合，避免散落在多个 UI 调用点。
4. 查找 TUI 底栏组件与 usage 数据模型，在最靠近渲染层的位置新增 Context Window 文本。
5. 更新/新增 `insta` 快照，覆盖正常宽度与窄屏。
6. 运行 scoped format/fix/test。

## Research Required
不需要外部 research。需要本地代码探索：

- `codex-rs/cli` 或 VSP wrapper 如何区分 `vsp-codex` 与兼容命令。
- TUI bottom pane/footer/status line 的现有布局与 snapshot 体系。
- usage/context window 数据当前在 protocol/core/TUI 中的传递路径。

## Risks And Alternatives
- 风险：YOLO 默认如果在 core 层无差别启用，可能影响测试或非 VSP 兼容路径。替代方案是在 VSP-branded binary/CLI normalization 层启用。
- 风险：底栏右侧空间不足，Context Window 可能挤压 model/provider/path。替代方案是按优先级截断右侧信息，只保留百分比。
- 风险：usage 数据在某些状态不存在。替代方案是显示 `Context --` 或隐藏右侧段，不能 panic。

## Validation Path
- `just fmt` from `codex-rs` after Rust edits.
- `just fix -p codex-tui` if TUI Rust changes are substantial.
- `just test -p codex-tui` for TUI snapshots and footer tests.
- If CLI crate is changed, run its scoped package tests as discovered in the workspace.
- Manual smoke: from another repo run `vsp-codex` without dangerous flags and verify it launches with YOLO behavior and bottom status shows Context Window.

## Audit Focus
- Default YOLO applies to all intended `vsp-codex` local entrypoints, including TUI and exec paths.
- Explicit user config/CLI choices still override defaults.
- No changes to `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` or `CODEX_SANDBOX_ENV_VAR`.
- Bottom bar does not overlap, wrap incorrectly, or break Chinese text width.

## Deliverables
- Tests/snapshots for default YOLO and Context Window display.
- Product changes implementing the default and bottom status update.
- Evidence in milestone report with commands, screenshots or terminal smoke notes, and any known residual risk.
