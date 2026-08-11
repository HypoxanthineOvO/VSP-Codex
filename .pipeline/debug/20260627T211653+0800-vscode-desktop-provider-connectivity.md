# Debug: VSCode Plugin / Desktop / Provider Connectivity

> Language: zh-CN | Timezone: Asia/Shanghai | Time: 2026-06-27T21:16:53+08:00

## Symptom

用户报告 VSCode Plugin 经常连不到 VSP-Codex，Codex Desktop / Remote APP 也会连不上；另一次新会话显示 provider 为 ``，并触发 `Model provider `` not found` 类错误。

## Context

- 当前发布编号保持 `0.140.0-vsp.8.1`。
- C9 已完成，本轮作为独立 `/goal` debug 修复处理，没有重新打开 C9 milestone。
- 发现最新 Remote VSCode ChatGPT extension `openai.chatgpt-26.623.41415-linux-x64` 的 bundled `codex` 回到了官方 `codex-cli 0.142.3`。
- 发现旧 Desktop/Remote APP 启动链仍使用 Volta 官方 `@openai/codex` 和 `~/.codex/app-server-control/desktop-ssh-websocket-v0.sock`。
- `~/.vsp-codex/state_5.sqlite` 空 provider 计数为 0；旧官方 `~/.codex/state_5.sqlite` 仍有 1 条历史空 provider。

## Hypotheses And Validation

1. VSCode Remote extension 更新后新增 bundled `codex`，旧 `vscode install` 只报警不自愈。
   - 结论：确认。`vsp-codex vscode status --json` 初始 `configured:false`，新 extension bundled binary 输出官方 `codex-cli 0.142.3`。
   - 修复：新增 `vsp-codex vscode repair --yes`，并让 VSCode wrapper 每次启动前静默 repair 新增 extension shims。

2. Desktop/Remote APP 外部脚本显式传入 `~/.codex/app-server-control/*` socket，即使 `codex` 解析到 VSP 也仍落回旧 socket。
   - 结论：确认。清理官方进程后外部 supervisor 自动重启了 VSP binary，但命令行仍包含 `--listen unix:///home/heyx/.codex/...` / `--sock /home/heyx/.codex/...`。
   - 修复：VSP CLI 在 `app-server --listen unix://...` 和 `app-server proxy --sock ...` 入口自动把默认官方 `~/.codex/app-server-control/*` socket remap 到 `~/.vsp-codex/app-server-control/*`。

3. 兼容 `~/.local/bin/codex` 裸 symlink 不设置 `CODEX_HOME`，导致官方 home 污染继续影响 Desktop。
   - 结论：确认。原本 `~/.local/bin/codex` 是直接指向 VSP binary 的 symlink。
   - 修复：installer 现在把兼容 `codex` 写成 shell wrapper，固定 `CODEX_HOME=${VSP_CODEX_HOME:-$HOME/.vsp-codex}` 后再 exec VSP；本机也已应用该 wrapper。

4. provider 空值仍可能从 VSP 新会话持久化链路进入。
   - 结论：未复现为 VSP 新写入。已有 `effective_model_provider_id_for_persistence()`、thread start/resume/fork 空 provider override 保护均通过测试；本轮额外收紧 API 输出层，旧历史中的空/未知 provider 不再原样返回给客户端。

## Root Cause

本轮问题不是单一 provider 持久化 bug，而是启动路径和 socket home 双重漂移：

- VSCode Remote extension 自动更新后产生新 bundled official `codex`，之前只在手动 `vscode install` 时修复，无法持续自愈。
- Desktop/Remote APP 的外部脚本会显式使用 `~/.codex/app-server-control/*` socket；即使 PATH 先解析到 VSP binary，旧 socket 参数仍会把运行态引回官方 home。
- 裸 symlink 形式的兼容 `codex` 没有设置 `CODEX_HOME`，进一步放大 `~/.codex` 与 `~/.vsp-codex` 混用。
- 旧官方 `~/.codex` state DB 内仍存在历史空 provider 记录，解释了用户看到 provider 为 `` 的来源；VSP DB 当前为空 provider 计数 0。

## Fix Applied

### Change Summary

已修复 VSCode Remote extension 漂移、Desktop/Remote APP socket home 漂移、兼容 `codex` home 污染，以及 API response 中空/未知 provider 透出问题。本机已重新构建并分发 `vsp-codex 0.140.0-vsp.8.1`。

### Technical Approach

- 增加显式 `vsp-codex vscode repair --yes [--quiet]`，只修 wrapper 和 remote extension shims，不改 VSCode settings backup。
- VSCode wrapper 启动前执行一次 guarded quiet repair，防止新安装的 `openai.chatgpt-*` extension 再次回到官方 bundled binary。
- VSP CLI 对默认官方 app-server socket 路径做内部 remap，覆盖外部脚本已经展开 `CODEX_HOME` 的情况。
- installer 改为用 wrapper 生成兼容 `codex`，而不是 symlink。
- app-server API thread/list/read/resume/fork 输出层过滤空/未知 provider，回退到当前 configured provider。

### Modified Files / Modules

- `codex-rs/cli/src/vscode_cmd.rs`
- `codex-rs/cli/tests/vscode.rs`
- `codex-rs/cli/src/main.rs`
- `codex-rs/app-server/src/request_processors/thread_processor.rs`
- `codex-rs/app-server/tests/suite/v2/thread_resume.rs`
- `scripts/install/install.sh`
- 本机运行态：`~/.vsp-codex/packages/standalone/current/bin/vsp-codex` 已覆盖；`~/.local/bin/codex` 已改为 wrapper；`~/.local/bin/vsp-codex` 仍指向 current binary。

### Test Design

- 单元/集成测试覆盖 VSCode repair、自愈 wrapper、socket remap、provider 空/未知 fallback。
- 真实运行态验证覆盖 extension bundled binaries、VSP daemon socket、Desktop/Remote socket remap、state DB 空 provider count。
- Worker separation 降级：当前工具规则不允许在用户未显式要求 subagent 时 spawn subagent；本轮未声称独立 worker 验证。

### Validation Results

- `cargo test -p codex-cli vsp_app_server_socket`：2 passed。
- `cargo test -p codex-cli --test vscode -- --test-threads=1`：19 passed。
- `cargo test -p codex-cli --test vscode vscode_install_remote_scope_repoints_running_bundled_elf -- --test-threads=1`：1 passed；此前并行运行同一测试出现一次环境性 `Text file busy`，串行复验通过。
- `cargo test -p codex-app-server model_provider`：unit 11 passed，integration 6 passed。
- `cargo test -p codex-core effective_model_provider_id_for_persistence_never_returns_blank`：1 passed。
- `cargo build -p codex-cli --bin vsp-codex --release`：通过。
- 本机 `codex --version` / `vsp-codex --version`：均为 `vsp-codex 0.140.0-vsp.8.1`。
- `vsp-codex vscode repair --yes` 修复 4 个 `openai.chatgpt-*` bundled `codex`；全部 bundled path `--version` 返回 `vsp-codex 0.140.0-vsp.8.1`。
- `vsp-codex vscode status --json`：`configured:true`；仍提示 Remote Machine settings 的 VSCode application-scope 限制。
- `ss -xlpn`：当前监听 socket 只在 `~/.vsp-codex/app-server-control/app-server-control.sock` 和 `~/.vsp-codex/app-server-control/desktop-ssh-websocket-v0.sock`。
- `~/.vsp-codex/state_5.sqlite`：空 provider thread count 为 0。

### Expected Result

- VSCode Remote extension 更新后，如果任何 VSP wrapper/shim 被启动，会自动 repair 新增 bundled `codex`。
- Desktop/Remote APP 即使命令行仍显式传 `~/.codex/app-server-control/*`，VSP CLI 也会内部 remap 到 `~/.vsp-codex/app-server-control/*`。
- 新会话、resume、fork、list/read API 不再向客户端返回空 provider 或未知 provider。
- 本机后续新启动的 Plugin/Desktop/Remote APP 应连接 VSP-Codex 状态和会话。

### Problems Encountered

- 严格 worker separation 无法满足：subagent tool 明确要求用户显式请求 subagent/委派才可 spawn。
- VSCode full test 并行运行时 `vscode_install_remote_scope_repoints_running_bundled_elf` 曾出现一次 `Text file busy`；串行全套复验通过。
- 外部 Desktop supervisor 会自动重启 `codex app-server` / `proxy`，因此单纯 kill 旧进程不足，必须做 CLI socket remap。

### Risks / Follow-Up

- `~/.codex/state_5.sqlite` 仍有 1 条历史官方空 provider 记录；本轮未修改官方 DB。VSP 运行态已经不再使用该 DB。
- VSCode `chatgpt.cliExecutable` 是 application-scoped；远端 Machine settings 仍会提示本地 VSCode User settings 限制。这是 VSCode 约束，不影响本轮 remote bundled shim 修复。
- 已运行的 VSCode/Remote windows 可能仍保留旧连接状态；当前进程已切到 VSP binary/socket，但用户侧 UI 若仍异常，优先 reload VSCode window 或重启 Desktop/Remote APP。

## Architecture Impact

需要在后续 release checklist 中保留两条规则：

1. 兼容 `codex` 入口必须是设置 VSP `CODEX_HOME` 的 wrapper，不应回退为裸 symlink。
2. app-server/proxy 对默认官方 `~/.codex/app-server-control/*` socket 的 remap 是 VSP Desktop/Remote APP 兼容边界，不能在后续 upstream merge 中删除。
