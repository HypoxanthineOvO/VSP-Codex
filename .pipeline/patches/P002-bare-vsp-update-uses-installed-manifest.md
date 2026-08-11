# P002: 裸 VSP update 使用安装 manifest
- 严重级: normal
- 状态: pending_acceptance
- 发现于: Patch after C10
- 创建时间: 2026-07-11 15:03 CST
- 改动: 裸 `vsp-codex update` 在 VSP 官方服务禁用时，改为读取当前 standalone 安装的 `codex-package.json` 并复用现有 GitLab manifest 更新路径；显式 `--from-vsp-manifest` 保持兼容；debug build 返回明确不可用提示。
- 测试: CLI debug 1/1；standalone release manifest discovery 1/1；TUI VSP updater 10/10；release builder + installer 19/19；rustfmt、CI docs、diff check 通过。
- 关联: v0.145.0-vsp.9.1
- resolved_by: 59a5088eb4
- related: []
- supersedes: []
- worker_separation: simplified_by_user_confirmation
- worker_lifecycle: test=main, implement=main, audit=not_delegated
- execution_note: 用户明确确认该低风险热修不启动独立 subworker，由主 Agent 直接实现和聚焦验证；Patch 不自动声明独立三方审查。

## 目标

让 standalone VSP-Codex 用户可以直接运行 `vsp-codex update`。命令应读取当前安装包
`codex-package.json` 中的 GitLab manifest、channel 和 target，选择新版本并沿用现有
archive/SHA256 安全更新路径；显式 `--from-vsp-manifest` 长参数继续兼容。

## 实现

- `codex-rs/cli/src/main.rs`: 当 `vsp_official_services_disabled()` 为真且用户没有传显式
  manifest 参数时，调用既有 `run_vsp_manifest_update_from_install_context()`。
- `codex-rs/cli/tests/update.rs`: 新增 release-only standalone 布局回归，使用有界本地 HTTP
  server 提供同版本 manifest，验证裸命令确实读取安装元数据并返回“没有更高版本”。
- debug 构建继续拒绝真实更新，但错误不再误导为官方服务禁用。

## 验证结论

RED 基线复现裸命令在进入安装上下文前被 `reject_vsp_official_service()` 拒绝。GREEN 后，
release 二进制从 `packages/standalone/releases/<version-target>/codex-package.json` 读取
manifest URL、channel 和 target，并完成版本选择；下载、SHA256 校验和原子安装仍复用
现有 updater，没有引入第二套更新实现。

由于用户明确要求快速修复并确认不启动 subworker，本 Patch 由主 Agent 完成 test、implement
和聚焦验证，未执行独立 audit，因此按工作流约束保持 `pending_acceptance`。

## 发布与本地分发

- Release commit: `dee80c2ed6`
- Tag: `v0.145.0-vsp.9.1`
- GitLab Release: `https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.9.1`
- 远端 SHA 文件与 stable manifest 均返回
  `9dcc14a0f2d6cbbafdbc5e294094cc956a54942a00620aa66ee95f96164eb465`。
- 本机通过远端制品从 vsp.8.2 升级；`vsp-codex`、兼容 `codex`、standalone current
  和 VSCode wrapper 均报告 `0.145.0-vsp.9.1`。
- 本机裸 `vsp-codex update` 实际读取远端 manifest，并按预期返回
  `No newer VSP release matched this install.`。
