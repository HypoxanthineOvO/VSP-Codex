# M5 MCP 提示迁移与静态汉化

## Goal
把 `Booting MCP server: context7` 这类启动提示从主 Working/status 区移走，不删除用户配置或 MCP 功能；同时完成静态汉化第一轮：TUI 文案、Slash Command 解释、设置页面、常见 Error/Warning。中文风格简洁，专业术语保留英文。

## Subworker Assignment Plan
Subworkers are authorized for this milestone. Worker separation is strict.

- `test`: 先补 snapshot/CLI output/error 文案测试，覆盖 MCP 启动提示不再污染主 Working 区、失败仍可见、Slash Command 描述汉化、设置页面常用项汉化。测试 worker 不改文案实现。
- `implement`: 实现 MCP 启动提示迁移和静态汉化。实现 worker 负责文案表/渲染点修改，不删除 MCP 配置或 context7 配置。
- `audit`: 审核是否误删 MCP、是否遗漏高频英文提示、是否把动态翻译范围与静态汉化混在一起。
- Main agent: 整合补丁，运行 snapshot tests，记录范围和未覆盖项。

## Technical Solution
MCP 启动过程仍可正常发生，但普通成功启动提示不在主 Working/status 区持续显示。推荐展示策略：

- `/mcp` 或现有 MCP 管理界面可查看启动状态。
- 启动失败、认证失败、工具加载失败等异常仍在主界面可见。
- 普通 `Booting MCP server: context7` 只进入低噪声日志/详情，不打断用户主工作流。

静态汉化不依赖 Hypo-Translate。优先处理用户高频看到的固定文案：

- TUI welcome、empty state、tips、buttons、status、common labels。
- Slash Command 名称保留英文命令，description/help 改中文。
- 设置页、Provider 相关页面、model/provider/status labels。
- 常见 Error/Warning 的固定前缀和解释。

## Technical Route
1. 搜索 MCP startup 文案、`Booting MCP server`、`context7` 相关渲染路径，区分 success/info/error。
2. 搜索 TUI 静态英文文案、slash command descriptions、settings labels、common error/warning constructors。
3. 建立测试清单，先锁定需要变化和不能变化的地方。
4. 修改 MCP startup routing：成功 info 降噪，失败 warning/error 保持可见。
5. 分批替换静态文案，保留英文专业术语和命令名。
6. 更新 snapshots，记录未覆盖的低频文案作为后续 backlog。

## Research Required
不需要外部 research。需要本地代码探索：

- MCP startup event 的 severity/source 是否已存在。
- Slash Command description 的注册位置。
- TUI settings/help/error 文案是否集中管理，是否需要抽一个轻量文案模块。

## Risks And Alternatives
- 风险：过度隐藏 MCP 状态让用户无法诊断。替代方案是保留失败显眼，成功只移到详情页。
- 风险：汉化范围过大导致 snapshot 爆炸。替代方案是按高频路径完成第一轮，并把低频路径记录为 backlog。
- 风险：硬编码中文散落全 repo。替代方案是对高频 TUI 文案使用局部 helper/module，但不引入大型 i18n 框架。

## Validation Path
- `just fmt` from `codex-rs`.
- `just test -p codex-tui` for UI snapshots.
- Run scoped tests for slash command/settings crates if separate.
- Manual smoke: start TUI with context7 configured, confirm主工作区不显示普通 booting 提示；制造或模拟 MCP failure 时确认错误仍可见。

## Audit Focus
- MCP 功能和用户配置没有被删除。
- 静态汉化覆盖 P1 指定范围的高频路径。
- 专业术语和 command names 保留英文。
- Error/Warning 汉化不能掩盖关键技术细节。

## Deliverables
- MCP startup noise reduction.
- Static Chinese localization pass for TUI, Slash Command help, settings, common errors/warnings.
- Snapshot and report listing covered paths and deferred low-frequency strings.
