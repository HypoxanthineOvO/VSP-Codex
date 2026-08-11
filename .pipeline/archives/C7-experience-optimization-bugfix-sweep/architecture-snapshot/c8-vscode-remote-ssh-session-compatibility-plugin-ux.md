# C8 VSCode Remote SSH Session Compatibility And Plugin UX

创建时间：2026-05-30T13:43:31+08:00

## 目标

C8 聚焦 VSCode Remote SSH 场景下的 VSP-Codex 插件体验、session 兼容和 Workflow 交互表面。目标不是发布新版本，而是形成可实现、可验证的兼容契约：Remote SSH 窗口应优先配置远端 VSCode server；VSP-Codex session 应能被插件稳定 resume；官方 Codex 对 VSP session 的发现策略要有隐私边界；Request Tool/Input 和 Markdown Workflow 输出要尽量在不改插件源码的前提下获得更好的体验。

## 用户需求摘要

- 用户主要在 Remote SSH 的 VSCode 窗口里使用插件，VSP-Codex 和 sessions 位于远端主机。
- 当前 VSCode 插件 resume VSP-Codex session 会报 `Model provider `` not found`。
- 官方 Codex 也无法找到 VSP-Codex sessions，需要调研能否统一 session discovery。
- 插件曾有 Request Tool 能力，但以前受限于 Plan；希望探索不改插件本身时能否通过服务端协议和输出形态优化体验。
- Prompt 生成后默认由 Subagent 审核；本 Cycle 从 C8 开始继续执行该规则。

## 关键边界

- 不做发布、打 tag、推远端或触发外部发布副作用。
- 官方 VSCode 插件默认视为黑盒/VSIX client。除非本 Cycle 证据证明必须修改插件源码，并且用户再次确认，否则不 fork、不改插件。
- Remote SSH 优先级高于本机 VSCode 用户配置：`vsp-codex vscode install/status` 应优先处理远端 `.vscode-server/data/Machine/settings.json`。
- 对官方 Codex 兼容保持 research-first：不得为了可发现性直接泄露 VSP 私有 provider、token、Workflow 内部记录或敏感 metadata。
- Workflow confirmation、authorization intent、release/update confirmation 等 required decisions 优先使用 `request_user_input`；真实权限授权仍由审批/权限机制处理。

## Milestone Map

1. **M1 Remote SSH VSCode Integration Probe**：修正/验证 VSCode CLI 配置探测顺序，优先 Remote SSH server Machine settings，并产出诊断。
2. **M2 VSCode Plugin Protocol Capability Map**：以 VSIX/日志/协议黑盒调研插件支持的 app-server 能力，包括 `item/tool/requestUserInput`、approval、permission、Markdown thread item 等。
3. **M3 Provider And Resume Compatibility Contract**：定义 VSP provider 与 session resume 的兼容契约，修复空 provider 或缺失 provider 导致的 resume 失败。
4. **M4 Session Discovery Strategy For Official Codex**：设计官方 Codex 发现 VSP sessions 的安全策略，分级处理共享、镜像、导入导出和 metadata 脱敏。
5. **M5 Request Tool And Request Input UX For Plugin**：将 Workflow required decisions、ordinary clarification、authorization intent 和 tool/permission approvals 映射到插件可理解的协议/回退输出。
6. **M6 Markdown Workflow Surface And Remote SSH Acceptance**：统一 Subagent prompt/report、tool output、Plan/SubPlan、Workflow report 的 Markdown surface，并完成真实 Remote SSH 插件验收。

## 已知代码线索

- `codex-rs/cli/src/vscode_cmd.rs` 的 `resolve_settings_scope()` 当前偏向本地 user settings，再考虑 `.vscode-server`，需要为 Remote SSH 主场景重新排序。
- `codex-rs/cli/tests/vscode.rs` 已有本地和 server settings helper 的测试，可扩展覆盖 Remote SSH 优先级。
- app-server protocol 已存在实验性的 `item/tool/requestUserInput`。
- `codex-rs/app-server/tests/suite/v2/request_user_input.rs` 已有 native/fallback 行为测试。
- VSP provider 当前会从持久 session meta 中省略，相关逻辑在 `codex-rs/rollout/src/recorder.rs` 与测试 `vsp_provider_is_omitted_from_persisted_session_meta`。

## 验证策略

- 每个 Milestone 使用 strict Worker Separation：`test` 先写真实红测或调研验证，`implement` 只做生产实现，`audit` 只读复核。
- M1-M5 需要 focused tests 或黑盒证据；M6 必须包含真实 Remote SSH VSCode 插件 resume/interaction 验收。
- `cargo fmt --check` 和 `git diff --check` 是每个实现 Milestone 的基本验证。
- 涉及外部副作用、远端发布、系统级依赖安装、破坏性操作时必须停下确认。

