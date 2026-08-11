## Unreleased

## v0.145.0-vsp.11.1-alpha.7 - 2026-08-11

### Fixes

- 恢复OpenAI-compatible `spawn_agent` provider wire shape，撤下Alpha 6泄漏到
  `/responses` tool schema的VSP私有`routing_class`和routing diagnostics，修复VSPLab稳定502。
- Worker Routing继续在客户端内部按trusted policy默认解析`standard` candidate；当semantic
  reroute改变child配置时，`fork_turns=all`在Exec场景转换为no-history、其他场景转换为最近
  3轮，避免full-history override冲突和ephemeral root fork失败。

### Validation

- `spawn_agent` schema 6/6、Multi-Agent V2 spawn 19/19、V2 prompt assembly 2/2通过。
- 真实VSPLab V2 root smoke返回`ok`；Subagent smoke记录`standard-luna-max`内部解析和Luna
  child session启动，无502或spawn error，root最终返回`root-ok`。
- Alpha 7仅发布Linux x86-64 GNU hotfix；macOS和Windows不阻塞。

## v0.145.0-vsp.11.1-alpha.6 - 2026-08-11

### Features

- `gpt-5.6-sol`采用精简VSP基础提示；默认权限、Multi-Agent、Apps/Plugins与Workflow
  runtime policy合并为更少、更明确的developer上下文，Multi-Agent V2默认主动委派。
- System Skills按effective name与`Repo > User > System > Admin`解析，默认metadata budget封顶
  2,000 tokens；激活的Skill跨compaction与resume保留，并新增`vsp-provider`。
- `spawn_agent`新增`mechanical`、`long_context`、`standard`、`explore`、`critical`、
  `escalation`语义路由；trusted home policy离线解析provider/model/reasoning/tier，失败时
  继承parent并返回candidate skip reasons，不允许repository注入路由策略。
- Preview release builder支持Linux x64、macOS ARM64和Windows x64的target-specific archive，
  并严格聚合同版本多target manifest；新增手动GitHub Actions三平台原生构建workflow。

### Fixes

- 移除unsolicited plugin recommendation user fragment，保留显式list/install工作流。
- 修复cross-provider Worker Routing错误复用parent service-tier catalog的问题；semantic routing
  与显式model/reasoning/tier或role-pinned模型发生冲突时现在会明确拒绝。
- 修复release-dir manifest在构建同版本第二个平台时覆盖首个平台distribution的问题。

### Validation

- System Skills 121/121；prompt assembly 2/2；Skill protocol 2/2；compaction/lifecycle 19/19；
  Worker Routing 6/6；Multi-Agent V2 spawn 19/19；显式plugin list/install 1/1。
- `just vsp-ci`七项CLI/TUI/app-server smoke通过；release/package/installer/session checks 52/52，
  全仓format、diff与Workflow YAML检查通过。
- 真实VSPLab首轮为21,321 input tokens、3,840 cached tokens，相对27,536下降22.6%；非工具
  指令从77,567字符降至约55,565（28.4%）。因Stage C AGENTS/runtime按用户决定延期，未达到
  原25% token与40%字符目标；live Subagent smoke在进入spawn前被VSPLab连续502阻断。

## v0.145.0-vsp.11.1-alpha.5 - 2026-08-10

### Fixes

- Session compatibility migration 现在要求 Official/VSP 的共同 migration 前缀逐项同 checksum，
  并显式验证实际读写的 `threads`/`backfill_state` 列；允许任一 DB 拥有兼容的尾部新 migration。
- 修复 Official DB 已前进到 migration 46、VSP DB 停在 40 时，alpha installer 将兼容 schema
  误报为不匹配并跳过历史 rollout neutralization 的问题。

### Validation

- Session compatibility 单测 9/9；完整 release checks 48/48；Ruff check/format。
- Live Official 1-46 / VSP 1-40 status 通过；带双 DB 备份 neutralize 63 个历史 rollout，
  复核 `to_neutralize=0`、`already_neutral=1925`。
- GitLab `vsp-smoke` pipeline #356 通过；alpha.5 将重新验证 archive、远端 SHA 与安装器。

## v0.145.0-vsp.11.1-alpha.4 - 2026-08-09

### Features

- H1/H2/H3 使用独立、宽度安全的 TUI surface；长 token、CJK、quote/list container、
  Setext streaming 和 OSC 8 链接均保持正确显示列与层级。
- 新增 14 个行为导向 test groups、Cargo/Bazel binary-root build contracts、current-checkout
  Python/TypeScript SDK 合同和真实 Linux sandbox/permission/parallel oracles。

### Fixes

- `turn/completed` 透传 canonical final text；TUI 在 item completion 缺失或 active stream 未完成时
  恢复完整最终回复，避免图片/partial delta 边界后的缺尾和重复。
- thread events 改用单一 FIFO，消除 channel-full spawned sends 的顺序竞争。
- Classic shell 仅在真实并行 backend 上声明 parallel-safe；ZshFork 保持串行。
- 修复 binary-root Bazel test 遗漏、compile-data 漂移、doctest 编译和 strict Clippy 拒绝项。

### Validation

- Protocol 273/273、rollout 107/107、focused core/TUI/app-server regressions 和 reviewed snapshots。
- Cargo/Bazel build contracts、3 个 Bazel native sandbox targets、selected package Clippy、fmt 和
  pending-snapshot hygiene。
- `just vsp-ci` 7 项 smoke checks；真实 100x34 PTY 验证标题、长 token、代码块和 final tail。

## v0.145.0-vsp.11.1-alpha.3 - 2026-08-08

### Features

- 新增独立 `$CODEX_HOME/model-providers.toml` Responses provider addon layer；第三方 provider 定义、credential env reference 和默认选择不写入官方 `config.toml`。
- `/model` 统一展示官方 base provider 与 addon provider tabs，并以 `(provider_id, model_id, reasoning_effort)` 事务切换；缺 credential 的 provider 可发现但不可请求。
- App-server v2 新增 server-owned `modelProvider/list` / `modelProvider/write`，连接式 TUI 不直接读写 remote server 的配置文件。
- Per-turn provider catalog、resume、fork 和 manual compact 保持 provider-local model metadata、Base URL 与 endpoint。

### Fixes

- 修复 TUI quote/markdown heading hierarchy、stream finalize refresh、resize transcript replay 贴底和 streamed/canonical CJK whitespace reconciliation。
- Provider/model 持久化失败时回滚 thread settings，避免 runtime 与默认选择形成半状态。

### Validation

- App-server protocol 271 unit tests + 2 schema fixture tests。
- Core addon contract 13 tests、provider-local metadata 1、manual compact endpoint 1、app-server public JSON-RPC provider ownership 1。
- C24 TUI provider catalog/transaction/popup focused tests、4 个 provider snapshots，以及 resize/CJK regressions。
- Release builder tests、release binary/package/SHA smoke、rustfmt、diff 和 pending snapshot hygiene。

## v0.140.0-vsp.8.2 - 2026-06-30

本条目记录已发布的 C20 consultation-first 内部预览版；tag、main、GitLab Release、release asset 和 stable manifest 指向 `v0.140.0-vsp.8.2`。

### Workflow

- 同步 C20 consultation-first action boundary：`discussion`、`background`、`idea`、`complaint`、`question`、`solution-discussion` 类输入先输出 Mini-contract（`我的理解` -> `问题原因` -> `推荐方案`），不直接编辑文件或启动执行。
- 保留自动执行边界：明确 imperative 且有 concrete target 的请求仍可直接执行；`可以`、`确认`、`OK`、`go ahead`、`apply it` 等计划后确认只授权已展示范围。
- 将该边界同步到 Hypo-Workflow/OpenCode adapter 指导面、bundled model catalog 的 `base_instructions` 和 `instructions_template` 路径，并用 `codex-models-manager` focused test 锁定。

### Fixes

- `vsp-codex app-server --listen <official ~/.codex/app-server-control socket>` 和 `app-server proxy --socket-path <official socket>` 会自动 remap 到 `~/.vsp-codex/app-server-control`，避免 Desktop/Remote APP 绑定回落到官方 Codex socket。
- `vsp-codex vscode repair --yes` 可单独修复远端 VSCode ChatGPT 插件 bundled `codex` shim；VSCode wrapper 启动时会做一次 quiet self-heal，避免插件更新后丢失 VSP shim。
- 安装器创建兼容 `codex` 入口时改用 wrapper，固定 `CODEX_HOME=${HOME}/.vsp-codex`，让 Remote APP/插件从兼容入口启动时仍绑定 VSP Codex。
- app-server 对历史 thread API 输出做 provider fallback：空白或未知 `model_provider` 不再泄露给客户端，而是回到当前配置 provider，避免恢复旧 session 时破坏插件/remote 体验。

### Docs and Packaging

- 更新 VSP Codex 用户指南和 Slash Commands 文档，记录 consultation-first 与 `/hw:*` helper 的协作边界。
- 新增 sync/doc release note：`.pipeline/release/c20-consultation-first-sync/release-notes.md`。
- 同步 README、安装指南、用户指南、versioned release notes 和 stable manifest 到 `v0.140.0-vsp.8.2` GitLab Release downloads。
- 安装命令使用 `--version 0.140.0-vsp.8.2-preview`；release 包 SHA256 为 `427a101bfecfa123c6795c12d6a2ca4b5a26952b6f6b61ba9a654106e20cb65b`。

### Validation

- `python3 -m json.tool opencode.json .opencode/opencode.json codex-rs/models-manager/models.json`
- `node --check .opencode/runtime/hypo-workflow-hooks.js`
- `node --check .opencode/runtime/hypo-workflow-status.js`
- `node --check .opencode/plugins/hypo-workflow.ts`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-models-manager bundled_models -- --nocapture`
- `cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex`
- `codex-rs/target/release/vsp-codex --version`
- `python3 scripts/build_vsp_preview_release.py --entrypoint-bin codex-rs/target/release/vsp-codex --bwrap-bin /usr/bin/bwrap --cargo-profile release --force`
- `sha256sum -c .pipeline/release/v0.140.0-vsp.8.2/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`
- isolated installer smoke for `vsp-codex` and compatibility `codex`
- VSCode CLI binding regression
- app-server thread resume provider fallback regression
- `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check`
- `git diff --check`

## v0.140.0-vsp.8.1 - 2026-06-26

本条目记录已发布的内部预览版；tag、main/发布分支、GitLab Release 和 release asset 已创建。

### Upstream Base

- 记录 upstream Codex Rust 基线收敛到 `rust-v0.143.0-alpha.25`，保留 `vsp.8` 系列语义。
- alpha.25 中与 VSP 不等价或高风险的 protocol/app-server、core runtime、TUI/CLI/VSCode、plugin/release 差异继续按 preserve/defer 管理，不声明 full upstream merge。

### Fixes

- SQLite 日志写入收敛：过滤 `log`、`codex_otel.log_only`、`codex_otel.trace_safe` 等 noisy target，并保留 VSP in-layer `target=log` guard，降低 persistent log 写入放大。
- Plugin/Remote APP 会话可见性启动路径修复：VSCode Remote bundled `codex` shim 漂移可被诊断，Remote APP 的兼容 `codex` wrapper 可回到 VSP Codex 启动路径。
- VSCode bundled codex shim/diagnostic 修复：`vsp-codex vscode install/status/rollback` 持续维护官方 ChatGPT 插件 bundled CLI 指向 VSP wrapper，并暴露漂移状态。
- 安装器和 updater 持续维护 `vsp-codex` 与 `codex` 兼容入口；显式 VSP manifest update path 保留，普通官方 update 入口继续被 VSP gate 禁用。

### Docs and Packaging

- 同步 README、安装指南、用户指南和 GitLab release notes 到已发布 tag `v0.140.0-vsp.8.1`。
- 安装命令使用 `--version 0.140.0-vsp.8.1-preview`；release 包 SHA256 为 `0ff6e0d34daf6265e23cb99d21b69d9acbbcce4df2067c168f72f0e7a82e567e`。
- 已生成并上传 Linux x64 GNU/glibc preview release asset；stable release manifest 已指向 `v0.140.0-vsp.8.1`。

### Validation

- `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check`
- `cargo check --manifest-path codex-rs/Cargo.toml -p codex-cli --bin vsp-codex`
- `cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex`
- `codex-rs/target/release/vsp-codex --version`
- `python3 scripts/build_vsp_preview_release.py --entrypoint-bin codex-rs/target/release/vsp-codex --bwrap-bin /usr/bin/bwrap --cargo-profile release --force`
- `sha256sum -c .pipeline/release/v0.140.0-vsp.8.1/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`
- isolated installer smoke for `vsp-codex` and compatibility `codex`
- VSCode CLI test suite
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-state filter`
- SQLite runtime smoke confirmed zero noisy persistent log rows.

## v0.140.0-vsp.8 - 2026-06-17

### Upstream Base

- 合并 upstream Codex Rust `rust-v0.140.0-alpha.18` 作为新的 VSP 基线。
- 保留 VSP provider/config、`vsp-codex` CLI、VSCode Remote SSH、Hypo-Workflow `/hw`/Question Tool/Markdown/plan freshness、Subagent UX、plugin/skills compatibility 和 sandbox/permissions safety defaults。
- `voice/realtime` 与 `remote-control` 记录为非 VSP 必保功能，不作为本版本升级阻塞项。

### Docs and Packaging

- 同步 README、安装指南、用户指南和稳定更新 manifest 到 `v0.140.0-vsp.8` GitLab Release downloads。
- 重建 Linux x64 GNU/glibc preview release 包 `.pipeline/release/v0.140.0-vsp.8/vsp-codex-preview-linux-x64-gnu.tar.gz`。

### Validation

- `cargo metadata --manifest-path codex-rs/Cargo.toml --no-deps --format-version=1`
- `cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex`
- `codex-rs/target/release/vsp-codex --version`
- `python3 scripts/build_vsp_preview_release.py --entrypoint-bin codex-rs/target/release/vsp-codex --bwrap-bin /usr/bin/bwrap --cargo-profile release --force`
- `sha256sum -c .pipeline/release/v0.140.0-vsp.8/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`

### Same-Version HotFixes

- 2026-06-22：发布 VSCode Plugin HotFix，默认 thread/start 覆盖旧配置中的 `on-request` / `read-only` 为 VSP Full Access，native Ask 客户端不再显示合成 Question Context，自动压缩摘要不再作为可见 assistant 回复输出且压缩后继续显示真实回复，更新 release asset SHA256 为 `973471e85537fa2a1bf8ba568e3df6027673ec71b1039041d9a97b084ffb9704`。

## v0.134.0-vsp.7.2 - 2026-06-08

### Docs and Packaging

- 发布 C19 Hypo-Workflow Plan adapter 更新，新增 `/hw:plan:technical-stack` 与 `/hw:plan:architecture` OpenCode 命令映射，保留 Discover / Technical Stack / Architecture / Decompose / Generate 可见 gate 纪律。
- 同步 README、安装指南、用户指南和稳定更新 manifest 到 `v0.134.0-vsp.7.2` GitLab Release downloads。
- 重建 Linux x64 GNU/glibc preview release 包 `.pipeline/release/v0.134.0-vsp.7.2/vsp-codex-preview-linux-x64-gnu.tar.gz`。

### Validation

- `python3 -m json.tool opencode.json .opencode/hypo-workflow.json`
- stale scan for old `0.134.0-vsp.7.1` release surface references
- `git diff --check`
- `cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex`
- `codex-rs/target/release/vsp-codex --version`
- `python3 scripts/build_vsp_preview_release.py --release-dir .pipeline/release/v0.134.0-vsp.7.2 --entrypoint-bin codex-rs/target/release/vsp-codex --bwrap-bin /usr/bin/bwrap --cargo-profile release --force`
- `sha256sum -c .pipeline/release/v0.134.0-vsp.7.2/vsp-codex-preview-linux-x64-gnu.tar.gz.sha256`
- `pnpm prettier --check` not run: target pnpm environment has no `prettier` command and node v20 does not satisfy package engine >=22.

## v0.134.0-vsp.7.1 - 2026-06-07

### Fixes

- VSCode wrapper 启动官方 ChatGPT 插件时固定 `CODEX_HOME=$HOME/.vsp-codex`，避免插件进程继承官方 `~/.codex` 后读取到空 provider 并触发 `Model provider `` not found`。

### Docs and Packaging

- 发布 Linux x64 GNU/glibc preview release 包，并把 README、安装指南、用户指南和稳定更新 manifest 指向 `v0.134.0-vsp.7.1` GitLab Release downloads。
- 顶层 `.pipeline/release/vsp-release-manifest.json` 以 `0.134.0-vsp.7.1` 为最新 lab release，同时保留 `0.134.0-vsp.7` 和 `0.134.0-vsp.6` 历史条目。

### Validation

- `cargo build --manifest-path codex-rs/Cargo.toml --release --bin vsp-codex`
- `codex-rs/target/release/vsp-codex --version`
- `python3 scripts/build_vsp_preview_release.py --release-dir .pipeline/release/v0.134.0-vsp.7.1 --entrypoint-bin codex-rs/target/release/vsp-codex --bwrap-bin /usr/bin/bwrap --cargo-profile release --force`

## v0.134.0-vsp.7 - 2026-06-02

### Features

- VSCode 插件恢复对话时，app-server 各入口（thread/start、resume、fork、运行态 resume mismatch 日志、settings/update）会把空或纯空白的 `model_provider` 规范化为未设置，修复官方/插件客户端恢复 VSP session 时的 `Model provider `` not found`。
- 更新确认改为 chat 内 Workflow 卡片：VSP 模式发现新版本时通过 `request_user_input` 呈现可操作卡片（立即更新 / 跳过此版本 / 查看说明），合成 turn 在 App 事件循环本地拦截，替换此前不可达的旧 `UpdatePromptScreen` 死代码路径。
- Subagent 查看器侧栏宽度阈值修正，使其可进入并排显示态；翻译范围从仅 Reasoning 扩展到 Prompt/Report（Original/Translated 双栏）。
- `/hw:*` Workflow 路由数据注入 command palette（Ctrl+Space 可见可用），不再是零引用数据。
- resume 闪烁抑制守卫在 resume 路径实际置位启用，消除恢复会话时的重绘闪烁。
- GitLab/local Subagent 内置/默认 fallback 改用 VSP Lab 内部命名；Plan/SubPlan 进度分别按当前 SubPlan 与 root Plan 计数，无 active SubPlan 时使用 root-only 计数；结构化 SubPlan active work 纳入 Plan freshness，`.pipeline` 状态文件变化会把 active Plan 标为 stale 直到 `update_plan`。

### Fixes

- `vsp-codex vscode install` / `status` 在远端 `.vscode-server` Machine 作用域下，明确警告 `chatgpt.cliExecutable` / `chatgpt.localeOverride` 为 application-scoped、会被 VSCode 静默忽略，并给出本地 VSCode 客户端 User settings 应填的 key/value；写入与备份行为保持向后兼容。

### Docs

- 新增 `.pipeline/review-20260531/` 只读审计（需求落实追溯、工程审计 PASS_WITH_WARNINGS、质量评分卡），并归档 `correctness-gaps.md` 记录 done-but-wrong 项的真实闭环状态。

### Validation

- `cargo test -p codex-cli --test vscode`（13 通过）
- `cargo test -p codex-app-server --test all -- resume`（40 通过）
- `cargo test -p codex-tui -- active_subagents command_palette release_update suppress_session_configured_redraw`（24 通过）
- `cargo test -p codex-tui workflow`（9 通过）
- `cargo test -p codex-core -- built_in_roles_use_vsp_lab_nickname_candidates_for_internal_distribution structured_subplan_active_work_keeps_plan_freshness_trackable workflow_state_file_change_marks_active_plan_stale_until_update_plan plan_update_gate`（6 通过）
- `cargo fmt --check`（仅既有 stable rustfmt warning）
- `git diff --check`

## v0.134.0-vsp.6 - 2026-05-30

### Features

- VSCode Remote SSH 安装/status/rollback 现在优先处理远端 VSCode server Machine settings，并补充 app-server initialize probe。
- VSCode 插件恢复 VSP-authored session 时会规范化空 provider/model metadata，避免 `Model provider `` not found`。
- 新增官方 Codex discovery sanitized export API，生成可读 manifest/artifact，同时剥离 provider/auth/Workflow 私有字段。
- Request Input 现在保留显式 Workflow metadata；不支持 native Ask UI 的插件客户端会收到可读 Markdown fallback，并区分 authorization-intent 与真实权限授权。
- 新增 Workflow Markdown surface contract；unsupported native Ask fallback 与 recovered pending fallback replay 会输出 `Workflow Surface` wrapper，并保留 M5 fallback 内容。
- `/theme` 现在打开统一 Appearance 入口，可继续进入 UI palette 或 syntax theme；`/ui-theme` 保持为 UI palette 快捷入口，`[tui].theme` 和 `[tui].ui_theme` 仍分别持久化。
- 增加 Hypo-Workflow 项目辅助的 TUI helper surface：`/hw:*` 与 `$hypo-workflow:*` 只在匹配前缀时解析为 Workflow route，并提供 Workflow 项目下的 `/hw:status`、`/hw:plan`、`/hw:resume` discoverability entries。
- Workflow 状态 helper 只报告已验证能力：`.pipeline` 检测、compact fallback、ordinary conversation capture，以及 Prompt 写作默认 Subagent 审核。
- 同步 C18 指令质量入口：`/hw:audit`、`/hw:quality`、`/hw:optimize` 可被 Workflow helper 解析/发现，普通对话 capture 会把 audit/quality/optimize/sync 反馈分类为候选记录；integration sync 仍不是用户命令。

### Docs and Packaging

- README、用户指南和安装文档同步 C8 预览安装命令、VSCode Remote SSH 接入和 Request Input 插件 fallback 说明。
- 准备 Linux x64 GNU/glibc debug 预览包，用于内部试用和 GitLab Release 资产分发。
- 补充 Hypo-Workflow 运行时说明：`.pipeline` guard、短 guidance 注入、compact fallback、普通对话捕获、slash parser/helper status 和 destructive/system dependency/remote side effect 安全边界。
- 记录当前限制：`/hw:*` 命令面板 discoverability 仍是 helper 层能力，完整 TUI command palette 接线需要后续显式集成。
- 记录 C18 同步边界：Quality/Optimize 属于 Workflow 指令入口，integration sync 是源仓库更新后的目标适配流程。

### Validation

- `cargo test -p codex-app-server --test all request_user_input -- --nocapture`
- `cargo test -p codex-app-server --test all workflow_markdown_surface -- --nocapture`
- `cargo test -p codex-app-server --test all thread_resume -- --nocapture`
- `cargo test -p codex-rollout --test official_codex_discovery`
- `cargo test -p codex-cli --test vscode`
- `cargo fmt --check --package codex-app-server --package codex-app-server-protocol --package codex-protocol --package codex-core --package codex-rollout --package codex-cli --package codex-tui`
- `git diff --check`

## v0.134.0-vsp.5 - 2026-05-28

### Features

- 增加 `/ui-theme` 主题选择器，并把主题持久化到 `[tui].ui_theme`；TUI Markdown、输出块 accent 和输入框 focused 状态会跟随主题色变化。
- 增加 `/commands` 命令面板和默认 `Ctrl+Space` 快捷键，避开 VSCode terminal 中常见的 `Ctrl+J` / `Ctrl+K` 冲突。
- 命令面板新增 `Copy latest code block`，可以复制最近 assistant 消息里的最后一个 fenced code block。
- Resume/Fork 历史列表首屏优先读取 SQLite metadata cache，再做同 token 的 canonical verification；过期 metadata 会替换当前行，不重复插入。

### Docs and Packaging

- README、用户指南和安装文档同步 C5 预览安装命令、VSCode 插件接入说明和体验更新说明。
- 准备 Linux x64 GNU/glibc debug 预览包，用于内部试用和 GitLab raw/Release 资产分发。

### Validation

- `just test -p codex-tui resume_picker`
- `just test -p codex-rollout state_db_only`
- `just test -p codex-app-server thread_list_state_db_only_returns_sqlite_without_jsonl_repair`
- `just test -p codex-thread-store read_thread_applies_sqlite_thread_name read_thread_uses_sqlite_metadata_for_rollout_without_user_preview list_threads_preserves_sqlite_title_search_results`
- `git diff --check`

## v0.134.0-vsp.4 - 2026-05-28

### Fixes

- 修复 VSCode 插件恢复旧 chat/session 时报 `Failed to resume chat` 的问题：历史 metadata 中空的 `model_provider` 不再作为显式 provider 覆盖写入。
- 配置加载现在会把空白 `model_provider` 视为未设置，继续回退到有效配置或默认 `openai`，避免出现 `Model provider `` not found`。

### Validation

- `just fmt`
- `just test -p codex-core load_config_ignores_empty_model_provider_override`
- `just test -p codex-core load_config_uses_default_model_provider_when_configured_provider_is_empty`
- `just test -p codex-app-server merge_persisted_resume_metadata_ignores_empty_model_provider`
- `just fix -p codex-core`
- `just fix -p codex-app-server`
- `git diff --check`

## v0.134.0-vsp.3 - 2026-05-28

### Features

- 增加 Ask User Tools，支持模型以结构化方式发起单选、多选、填空和排序问题；TUI 可显示原生问答卡片，unsupported client 会降级为普通对话，pending Ask 可在 resume 后恢复。
- 将 Plan 更新升级为 structured Plan/SubPlan state，并在 TUI 中增加常驻 Plan panel；压缩、resume 或长工具调用后会提醒模型刷新计划状态。
- Ask UI 完成中文卡片式体验重构，强化大标题、小问题、说明和选项之间的层级，并支持 Other / Chat About This 语义。
- 统一 Subagent spawn、活跃列表和详情中的昵称与角色显示；Subagent nickname candidates 支持顶层和 role-level 配置，公共默认池改为中国古代人物名。
- 增加官方 VSCode ChatGPT 插件接入探索与本地劫持入口，安装器可通过 `--vscode` 自动写入 `chatgpt.cliExecutable` 并支持 status/rollback。
- Rename 现在写入 canonical session metadata，TUI resume 历史与 VSCode/app-server 可读到同一组 session name。
- Markdown 渲染补齐 heading、strong、inline code 和无序列表层级符号；无序列表按层级渲染为实心圆点、空心圆点和小方块。
- TUI 输出块采用主题色衍生的浅色块配色，输入框和不同输出块的边界更清晰。
- VSCode 终端快捷键冲突避让，Composer 不再默认占用 `Ctrl+J`/`Ctrl+K` 这类 VSCode 高频键位。

### Fixes

- Thinking 能力明确按可用事件输出：有原生 reasoning 内容时进入 thinking 事件；仅有 summary 时显示 summary；两者都没有时不伪造 Thinking。
- 预览安装脚本支持 direct archive URL、checksum 校验、launch/no-launch、非交互安装和已有 Codex 冲突保护。
- 启动、resume 列表、历史会话打开和 VSCode app-server initialize 补充性能证据采集脚本与报告。

### Validation

- `just fmt`
- `just test -p codex-cli vscode`
- `just test -p codex-app-server initialize`
- `just test -p codex-app-server thread_read`
- `just test -p codex-tui startup`
- `just test -p codex-tui keymap`
- `just test -p codex-tui markdown_render`
- `cargo insta pending-snapshots --manifest-path codex-rs/tui/Cargo.toml`
- `git diff --check`
- `sh -n scripts/install/install.sh`
- Linux x64 GNU/glibc preview package smoke tested with `vsp-codex --version`.
- Full workspace `just test` was deferred for CI/manual release validation.

## v0.134.0-vsp.2 - 2026-05-26

### Features

- 默认进入 VSP YOLO 权限体验：TUI 与 `vsp-codex exec` 默认使用 `approval_policy=never` 和 `danger-full-access`，不再要求额外输入危险模式参数。
- 默认启用并鼓励 Subagent 使用，最大线程数提升到 32，并使用 VSP 中文姓名池生成 Subagent 昵称。
- 增加 TUI Subagent 常驻卡片，显示昵称、模型、reasoning effort、角色、状态和运行时长；`Ctrl+O` 可打开详情。
- 接入 Hypo-Translate 运行时翻译，支持把 Agent 输出、reasoning summary 和 Subagent 文本异步翻译为中文，同时保留英文技术术语。
- 底栏和状态界面增加 Context Window 使用量展示，格式统一为 `已用量/总量 百分比`。
- 对 Slash Commands、设置页、状态页、启动页、MCP 提示和常见用户可见文案进行第一轮 VSP 中文化与品牌整理。

### Fixes

- 降低 MCP 启动噪声，普通 `Booting MCP server: context7` 不再占用主状态标题。
- 修复 `vsp-codex resume` 进入大历史会话时终端长时间闪烁的问题；初始 scrollback 只写入最近渲染行，完整 transcript 数据仍保留。
- 修复 Subagent 面板在 metadata 未到达时显示 hash/thread id 的问题，并在终态事件缺少 `agents_states` 时清理残留行。
- 修复 Read File 输出缺少语法高亮的问题，普通输出和 transcript 视图都会按文件扩展名复用现有高亮。
- 修复 hidden / transcript-only reasoning 可能被翻译或在 transcript overlay 中暴露的问题。
- 默认门控官方 remote plugin、ChatGPT auth 和 Cloud 相关路径，避免 VSP 启动时出现官方登录噪声。
- Provider debug 输出默认脱敏，避免 inline token 泄露到日志。

### Validation

- `just fmt`
- `just test -p codex-tui`
- `just test -p codex-tui plan_mode`
- `just test -p codex-cli --test vsp_entrypoint`
- `just test -p codex-exec`
- `just test -p codex-model-provider-info`
- `just test -p codex-core-plugins startup_remote_plugin_sync`
- `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`
- `just fix -p codex-tui`
- `cargo build -p codex-cli --bin vsp-codex`
- `git diff --check`

## v0.134.0-vsp.1 - 2026-05-25

### Features

- Add the `vsp-codex` entrypoint and default `~/.vsp-codex` home.
- Share existing Codex sessions, skills, and plugin assets without sharing official auth/config files.
- Add VSP Provider initialization and `/provider` selection for OpenAI Responses compatible Base URL + API Key providers.
- Hide or disable official Cloud, OAuth, update, feedback, and app remote-control entrypoints.
- Apply VSP Codex branding to primary TUI startup, status, session, plugin, and human-output surfaces.

### Fixes

- Fix shared-session resume by listing local sessions across legacy provider metadata.
- Hide built-in OpenAI, Amazon Bedrock, Ollama, and LM Studio providers from the VSP Provider picker.
- Keep `just fmt` working in the local Linux environment by running Python SDK Ruff checks without installing the unavailable runtime wheel.

### Validation

- `just fmt`
- `just test -p codex-tui user_typing_provider_slash_command_opens_provider_picker`
- `just test -p codex-tui vsp_local_picker_does_not_filter_by_current_provider`
- `just test -p codex-cli official_service_cli_commands_are_disabled`
- `just test -p codex-app-server thread_settings_update`
- `just test -p codex-app-server-protocol`
- `cargo build --bin vsp-codex`
