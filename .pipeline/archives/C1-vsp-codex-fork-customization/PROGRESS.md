# Hypo-Workflow Progress

更新时间：2026-05-25T14:12:00+08:00

## 当前状态

Milestone 6 已本地完成，等待用户审核测试：VSP 品牌和官方提示清理已落地；`resume` 兼容共享官方 Codex Sessions 的问题也已修复。M6 后续补丁已收敛 `/provider` 列表并打上 VSP 版本号。

M2 已提交并推送到 GitLab：`04db73422a621e891fd0dc361f36a0972561da7c`。GitLab pipeline #136 已通过，当前 CI 只自动运行 `format`，TUI 回归 job 保持 manual/allow_failure，避免 runner 冷构建网络依赖导致邮件噪音。

执行边界：M6 只处理 VSP 品牌、官方残留提示/文案、以及共享 Sessions 下的 resume provider 过滤兼容；不删除底层官方代码。MCP、Skills、Plugins、OpenAI-compatible Provider、`exec-server --remote` 保留。

## 结果摘要

- 构建入口通过：`cargo build --bin codex`、`./target/debug/codex --version`。
- 非 TUI 目标测试通过：`codex-cli`、`codex-config`、`codex-model-provider-info`、`codex-model-provider`、`codex-login`、`codex-exec-server`。
- 原始 `codex-tui` 失败不是业务回归，而是测试环境假设不 hermetic：
  - `/tmp/project` 被宿主机 `/tmp/.git` 影响，project name 从预期的 `project`/`my-project` 漂移为 `tmp`。
  - Unix socket IPC 测试在默认 tempdir 下触发生产权限校验，父目录权限不满足安全要求。
- 已修复并验证：`just test -p codex-tui` 通过，2614 passed、4 skipped。
- ignored tmux resize/reflow smoke 通过：3 passed、2615 skipped。
- scoped lint/fix 通过：`just fix -p codex-tui` 通过。
- 格式化 gate 已修复：`just fmt` 通过。修复方式是让 Ruff 在 `sdk/python` 目录下以 `uv --no-project --with 'ruff>=0.15.8'` 运行，只安装 Ruff 工具，不安装 SDK 运行时依赖，从而避开当前 Linux glibc 环境缺失 `openai-codex-cli-bin==0.131.0a4` wheel 的问题。
- SDK 默认安装/发布模型仍有后续风险：`openai-codex-cli-bin==0.131.0a4` 没有 Linux glibc/manylinux wheel；这不再阻塞 `just fmt`，但若要让 `uv sync`/默认 SDK 安装在本机通过，需要单独设计运行时依赖方案。
- 无 `.snap.new` 残留。
- M2 新增 TUI 用户模拟 baseline：
  - 输入 `/mo` 后 slash command popup/filter snapshot。
  - 从 slash popup 选择 `/model` 后打开 model picker snapshot。
  - 普通 prompt 输入并 Enter 后产生 `Op::UserTurn`。
- GitLab CI 已收敛：
  - 新增仓库级 `.gitlab-ci.yml`，避免继续触发旧 Auto DevOps `build/test/code_quality` 默认 job。
  - 远端唯一在线 runner 需要 `latex-runner` tag 且 `run_untagged=false`，新 CI 已配置默认 tag。
  - `glab ci lint .gitlab-ci.yml` 通过。
  - 已删除旧的 9 个 pending/failed Auto DevOps pipelines。
  - M2 推送后的 pipeline #136 已通过。

## M3 执行计划

- test worker：先补 `vsp-codex` CLI 入口、`~/.vsp-codex` 默认 home、TUI 旧 home 导入提示的测试和 snapshot。
- implement worker：只改 Cargo/CLI/just/npm wrapper/home-dir/TUI startup import 生产代码，不写测试。
- audit worker：最终只读审查入口一致性、旧 `~/.codex` 写入遗漏、CI/merge 成本和 M4-M6 越界风险。

## Worker Lifecycle

| Role | Worker | Status | Evidence |
| --- | --- | --- | --- |
| test | Descartes (`019e59c9-f18a-7ec3-8eef-f7621d4ff9b7`) | completed/closed | 已生成 `.pipeline/reports/test-baseline.md` 和 `.pipeline/reviews/001-test-baseline-test.md` |
| implement | Ampere (`019e59ca-0b51-7201-abf1-2aed05114835`) | completed/closed | 已生成 `.pipeline/reviews/001-test-baseline-implement.md`，确认 M1 无实现改动 |
| audit | Dirac (`019e59e0-481f-7730-badb-9491d2562407`) | completed/closed | 已生成 `.pipeline/reviews/001-test-baseline-audit.md`，裁决 `pass` |
| fix-audit | Hypatia (`019e59f2-c4a1-76b0-a0ce-ec148b9daab2`) | completed/closed | 已生成 `.pipeline/reviews/001-test-baseline-fix-audit.md`，裁决 `pass` |
| test | Schrodinger (`019e5a0b-d30f-7533-b5d4-c5c828c462f1`) | completed/closed | 已生成 `.pipeline/reviews/002-tui-user-simulation-test.md`，新增 3 个用户模拟测试与 2 个 snapshot |
| implement | Leibniz (`019e5a0c-1ac9-74c2-8b8e-74e2c1d62805`) | completed/closed | 已生成 `.pipeline/reviews/002-tui-user-simulation-implement.md`，确认 M2 无生产实现 |
| audit | James (`019e5a17-3054-7bf1-85b4-f3bfe2f0f9c7`) | completed/closed | 已生成 `.pipeline/reviews/002-tui-user-simulation-audit.md`，裁决 `pass` |

## 下一步

- M3 已提交并推送：`38628c937a feat: add vsp-codex entrypoint`，GitLab pipeline #137 已通过。
- 已将 `/home/heyx/.local/bin/vsp-codex` 链接到 `/home/heyx/Codex-VSP/codex-rs/target/debug/vsp-codex`；`~/.local/bin` 已在 PATH 中，现在可在任意仓库直接运行 `vsp-codex`，cwd 保持为当前仓库。
- 后续共享策略已确定：
  - Sessions：VSP Codex 与官方 Codex 复用 session 资产，优先共享 `sessions` 与 `session_index.jsonl`；谨慎处理 sqlite/backfill 状态，避免两个进程同时写 sqlite 造成锁冲突。
  - Skills/Plugins：共享可复用资产，如 `skills`、`plugins`/`plugins/cache`、`vendor_imports/skills`。
  - 不共享完整配置与认证：`config.toml`、`auth.json`、`.credentials.json`、provider/base_url/login 配置由 VSP Codex 独立维护，避免官方 OAuth/Cloud 配置回流。
- 下一步进入 M4 前，应把上述共享策略落成代码或启动迁移逻辑，并补对应回归测试。

## 2026-05-25 / M3.5 shared Codex assets patch

- `/hw:resume` 选择 active continuation，先执行 M4 前置补丁：共享原 Codex 的 sessions、skills、plugins 资产，但继续隔离 VSP 的 config/auth/provider。
- 本补丁不是原始 M4。原始 M4 仍是 Provider 配置与 `/provider`，包括后续处理启动 API Key/OAuth 选择界面。
- 已实现：
  - 默认 `~/.vsp-codex` 且 `CODEX_HOME` 未设置时，自动链接 sibling `~/.codex` 的 `sessions`、`archived_sessions`、`session_index.jsonl`。
  - 共享用户 skills 的直接子项，但跳过 `skills/.system`，避免 VSP 系统技能缓存覆盖官方 Codex。
  - 共享 `plugins/cache`、`plugins/data`、`.tmp/plugins`、`.tmp/plugins.sha` 和 `vendor_imports/skills`。
  - 不共享 `config.toml`、`auth.json`、`.credentials.json`，且不覆盖任何已存在的 VSP 目标。
  - 删除旧 TUI 首启“复制完整 Codex home”的 prompt、测试和 snapshot。
- 验证：
  - `just fmt`: passed
  - `just test -p codex-utils-home-dir`: passed
  - `just test -p codex-tui app::tests::startup`: passed
  - `just test -p codex-tui`: passed
  - `cargo check -p codex-core -p codex-tui`: passed
  - `TMPDIR=/var/tmp/codex-vsp-clean-tmp just test -p codex-core codex_home_is_not_loaded_as_project_layer_from_home_dir project_layers_disabled_when_untrusted_or_unknown`: passed
  - `cargo insta pending-snapshots --workspace-root .`: no pending snapshots
  - `just fix -p codex-utils-home-dir`: passed
  - `just fix -p codex-tui`: passed
  - `just fix -p codex-core`: passed
  - `git diff --check`: passed
- 审查：
  - 第一轮 audit 发现 Windows symlink 权限和并发 `AlreadyExists` 竞态，已修复为非致命/幂等。
  - 最终 audit 未发现阻塞问题，残余风险是 Windows symlink 行为未在 Windows 真机覆盖。

## 2026-05-25 / M4 Provider config and `/provider`

- `/hw:resume` 已执行原始 M4：Provider 配置、启动初始化页改造、TUI `/provider` 选择器、当前 thread provider 同步。
- 已实现：
  - 启动 OAuth/API Key 选择页改造成 VSP Provider 初始化页，要求 Base URL + API Key。
  - 默认写入 `model_provider = "vsp"` 和 `[model_providers.vsp]`，使用 Responses wire API，禁用 OpenAI auth/websocket。
  - `vsp` provider 缺失或不完整时强制进入初始化页，即使当前选中的是其他 provider。
  - 新增 `/provider` slash command，列出现有 provider，描述 Base URL 与凭据来源，不展示 API Key。
  - `/provider` 切换时先严格同步当前 app-server thread，失败不更新 UI、不提示成功。
  - `thread/settings/update` 支持 `model_provider`，后续 turn 会使用新的 provider/base_url。
  - 修复预热 `ModelClientSession` 复用旧 provider 的问题；provider 不匹配时丢弃预热 session。
  - app-server protocol 增加 `modelProvider` 参数序列化和 experimental schema 覆盖。
- Subagent：
  - test：`019e5b25-6470-78b1-9883-3b5d5364cf7d`，新增真实 base_url 回归测试，先红后绿。
  - implement：`019e5b44-116b-7242-963d-0f91088820d1`，修复旧 provider 预热 session 复用。
  - audit：`019e5b55-c609-7451-867f-61aa32b3e539`，确认无 token 泄漏和 thread 假成功；提出的 schema/初始化策略问题已修复。
- 验证：
  - `just fmt`: passed
  - `just test -p codex-app-server thread_settings_update`: passed
  - `just test -p codex-tui chatwidget::tests::user_simulation`: passed
  - `just test -p codex-app-server-protocol`: passed
  - `cargo check -p codex-tui -p codex-core -p codex-app-server -p codex-app-server-protocol`: passed
  - `cargo insta pending-snapshots --workspace-root .`: no pending snapshots
  - `git diff --check`: passed
  - `just fix -p codex-app-server-protocol`: passed
  - `just fix -p codex-tui`: passed
  - `just fix -p codex-protocol -p codex-core -p codex-app-server`: passed
- 残余风险：
  - `just test -p codex-core` 全量 crate 测试仍有大量与环境/超时相关失败；本次 M4 以 app-server 真实 `/responses` base_url 回归覆盖 provider 切换核心路径。
- 下一步：
  - 用户实际进入 TUI 审核初始化页和 `/provider`。
  - 审核通过后进入 M5：隐藏或禁用官方 Cloud/OAuth/Codex Remote 等入口，保留 MCP/Skills/Plugins/OpenAI-compatible BaseURL+API Key。

## 2026-05-25 / M5 Hide or disable official-service entrypoints

- 已执行 M5：隐藏或禁用官方 OpenAI/Codex 服务入口，同时保留后续合并官方更新所需的内部代码结构。
- Subagent：
  - audit：`019e5d3c-71cb-7da1-a3f6-ea6530c79210`，只读梳理官方服务入口，确认 P0/P1 清理范围。
- 已实现：
  - CLI help 隐藏 `login`、`logout`、`cloud`、`remote-control`、`update`。
  - 上述 CLI 命令运行时返回 VSP 禁用提示。
  - `app-server daemon enable-remote-control` 和 `app-server daemon bootstrap --remote-control` 被禁用；直接 `app-server --remote-control` 会忽略该 flag 并以非 remote-control 模式启动。
  - TUI `/feedback`、`/logout` 从 slash command 中隐藏；即使内部直接 dispatch 也只显示 VSP 禁用提示，不触发上传或登出事件。
  - `/provider` 过滤 `requires_openai_auth` 的官方 provider；直接选择这类 provider 也会被拒绝。
  - `/status` 隐藏 ChatGPT usage link 和 ChatGPT 登录引导，API Key 显示为通用已配置状态。
  - rate-limit prefetch、TUI update check、update popup、announcement prewarm 不再请求官方 OpenAI/GitHub 发布渠道。
  - `doctor updates` 不再探测 GitHub/Homebrew/npm，只报告 VSP 已禁用官方更新检查。
- 保留：
  - MCP、Skills、Plugins。
  - OpenAI-compatible BaseURL/API Key Provider。
  - `vsp-codex exec-server --remote`。
- 验证：
  - `just fmt`: passed
  - `cargo check -p codex-cli -p codex-tui`: passed
  - `just test -p codex-cli official_service_cli_commands_are_disabled`: passed
  - `just test -p codex-cli vsp_codex_help_uses_vsp_codex_usage`: passed
  - `just test -p codex-cli login_with_api_key_is_disabled_in_vsp`: passed
  - `just test -p codex-cli login_with_access_token_is_disabled_in_vsp`: passed
  - `just test -p codex-cli update_check_does_not_probe_official_release_channels`: passed
  - `just test -p codex-tui official_service_commands_are_hidden`: passed
  - `just test -p codex-tui official_service_commands_do_not_resolve_for_dispatch`: passed
  - `just test -p codex-tui slash_logout_is_disabled_in_vsp`: passed
  - `just test -p codex-tui slash_feedback_is_disabled_in_vsp`: passed
  - `just test -p codex-tui user_typing_provider_slash_command_opens_provider_picker`: passed
  - `just test -p codex-tui prefetch_rate_limits_is_gated_on_chatgpt_auth_provider`: passed
  - `just test -p codex-tui official_openai_auth_providers_are_hidden_from_vsp_selection`: passed
  - `just test -p codex-tui vsp_provider_is_allowed_even_when_complete`: passed
  - `just test -p codex-tui status_model_provider_uses_bedrock_runtime_base_url_and_gates_usage_link`: passed
  - `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`: no pending snapshots
  - `just fix -p codex-cli -p codex-tui`: passed
  - `git diff --check`: passed
- 残余风险：
  - 官方服务底层代码按低 merge 成本策略保留，只在入口处 gate；未来官方更新合并时冲突预计小于直接删除。
  - Plugins/app-link 内仍有 ChatGPT Marketplace 等字样，因为插件系统本体按用户要求保留；这放到 M6 品牌和内容清理。
  - `exec-server --remote` 保留后，上游内部认证路径可能仍提到 ChatGPT/Agent Identity；这是用户明确要求保留的例外。
- 下一步：
  - 用户审核 M5 后，进入 M6：VSP 品牌、官方残留 prompt/文案清理、中文简洁风格统一。

## 2026-05-25 / M5.5 Resume compatibility and M6 VSP branding

- 用户要求先处理 `resume` 看不到现有 sessions 的问题，再继续 M6。
- 已修复：
  - `~/.vsp-codex/sessions` 已共享官方 `~/.codex/sessions`，但 TUI resume picker 原先按当前 provider 过滤，导致 legacy sessions 的 `OpenAI` provider metadata 被隐藏。
  - VSP 本地 resume picker 现在不再按 provider 过滤；`vsp-codex resume --all` 可浏览所有共享 sessions，普通 `vsp-codex resume` 仍按当前 cwd 过滤。
- M6 已实现：
  - 首启欢迎语改为中文 VSP Codex 品牌。
  - session header、`/status` 卡片、非交互 human output 改为 `VSP Codex`。
  - 官方 GPT 可用性 NUX、模型迁移 prompt、announcement prewarm、随机官方 tooltip 在 VSP 下禁用。
  - Plugin/App-link UI 保留功能，但将 `OpenAI Curated` 显示为 `VSP Curated`，将 `Install/Manage on ChatGPT` 改为通用 app 文案。
  - Plugin detail 使用 marketplace display label，避免继续显示原始官方 marketplace 名称。
  - device-code login prompt 做了品牌一致性处理；官方 login 入口仍按 M5 禁用。
- 验证：
  - `just fmt`: passed
  - `just test -p codex-tui status_snapshot`: passed
  - `just test -p codex-tui session_header`: passed
  - `just test -p codex-tui clear_ui`: passed
  - `just test -p codex-tui plugins_popup`: passed
  - `just test -p codex-tui vsp_local_picker_does_not_filter_by_current_provider`: passed
  - `just test -p codex-tui prepare_startup_tooltip_override_is_disabled_for_vsp`: passed
  - `just test -p codex-tui model_migration_prompt_shows_for_hidden_model`: passed
  - `just test -p codex-tui app_link_view`: passed
  - `just test -p codex-tui session_info_availability_nux_tooltip_snapshot`: passed
  - `just test -p codex-tui marketplace_add_success_refreshes_to_new_marketplace_tab`: passed
  - `just test -p codex-tui plugin_detail_popup`: passed
  - `just test -p codex-tui tooltips`: passed
  - `cargo check -p codex-exec -p codex-login -p codex-tui`: passed
  - `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`: no pending snapshots
  - `just fix -p codex-tui -p codex-exec -p codex-login`: passed
  - `cargo build --bin vsp-codex`: passed
  - `git diff --check`: passed
  - `/home/heyx/.local/bin/vsp-codex --version`: `vsp-codex 0.0.0`
- 报告：
  - `.pipeline/reviews/006-vsp-branding-resume.md`
- 残余说明：
  - 一些 `OpenAI`/`ChatGPT` 字符串仍留在测试、注释、禁用官方路径和 OpenAI Responses API 兼容描述中；VSP 运行时路径已 gate 掉官方提示。

## 2026-05-25 / M6.1 Provider list and VSP version patch

- 用户要求：
  - `/provider` 里的其他内置 Provider 可以删掉。
  - 给 VSP Codex 打版本号，最好能关联官方版本并标记 VSP fork 版本。
- 版本方案：
  - 采用 `官方基线-vsp.递增号` 的 SemVer prerelease 形式。
  - 当前版本设为 `0.134.0-vsp.1`：`0.134.0` 表示本 fork 跟随的官方 Codex Rust 版本线，`vsp.1` 表示 VSP fork 在该基线上的第 1 个内部版本。
  - 后续只做 VSP 自有修复时递增为 `0.134.0-vsp.2`；后续合并官方新版本线时切到类似 `0.135.0-vsp.1`。
- 已实现：
  - `/provider` 选择器不再展示内置 OpenAI、Amazon Bedrock、Ollama、LM Studio Provider。
  - 用户自定义 OpenAI-compatible Provider 仍保留，只要不是官方内置 ID 且不要求 OpenAI auth。
  - workspace package version 和 `Cargo.lock` 更新为 `0.134.0-vsp.1`。
  - 已重建 `/home/heyx/.local/bin/vsp-codex` 指向的 debug 二进制。
- 验证：
  - `just test -p codex-tui built_in_providers_are_hidden_from_vsp_selection`: passed
  - `just test -p codex-tui custom_provider_is_allowed`: passed
  - `just test -p codex-tui user_typing_provider_slash_command_opens_provider_picker`: passed
  - `cargo build --bin vsp-codex`: passed
  - `just fmt`: passed
  - `just fix -p codex-tui -p codex-cli`: passed
  - `cargo insta pending-snapshots --manifest-path tui/Cargo.toml`: no pending snapshots
  - `git diff --check`: passed
  - `/home/heyx/.local/bin/vsp-codex --version`: `vsp-codex 0.134.0-vsp.1`
