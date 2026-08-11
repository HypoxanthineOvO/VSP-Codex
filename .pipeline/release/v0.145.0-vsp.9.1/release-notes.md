# VSP-Codex v0.145.0-vsp.9.1

Linux x86_64 GNU lab hotfix，基于 `rust-v0.145.0-alpha.4` 和 vsp.9 已发布功能。

## 修复

- standalone 安装现在可以直接运行 `vsp-codex update`。
- 裸更新命令从当前安装包的 `codex-package.json` 读取 GitLab manifest、channel 和
  target，并复用现有 archive 下载、SHA256 校验和原子安装流程。
- 显式 `vsp-codex update --from-vsp-manifest ...` 保持兼容。
- debug 构建返回明确的不可更新提示，不再误报为 VSP 官方服务被禁用。

## 更新已有安装

```shell
vsp-codex update
```

该命令要求终端能够访问内网 `gitlab.vsplab.cn`。如果当前安装早于本热修且裸命令仍
显示官方服务禁用，请先使用下面的完整安装命令升级一次；从 vsp.9.1 起可直接使用裸命令。

## 全新安装或修复安装

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh -s -- \
  --archive-url https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.9.1/downloads/vsp-codex-preview-linux-x64-gnu.tar.gz \
  --sha256 9dcc14a0f2d6cbbafdbc5e294094cc956a54942a00620aa66ee95f96164eb465 \
  --target x86_64-unknown-linux-gnu \
  --version 0.145.0-vsp.9.1-preview \
  --manifest-url https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/.pipeline/release/vsp-release-manifest.json \
  --vscode \
  --no-launch \
  --yes
```

## 制品

- Archive: `vsp-codex-preview-linux-x64-gnu.tar.gz`
- Size: `352485875` bytes
- SHA256: `9dcc14a0f2d6cbbafdbc5e294094cc956a54942a00620aa66ee95f96164eb465`
- Target: `x86_64-unknown-linux-gnu`
- Channel: `lab`
- Upstream baseline: `rust-v0.145.0-alpha.4`

## 验证

- CLI debug 路径 1/1、release standalone manifest discovery 1/1。
- TUI VSP updater 10/10。
- release builder 与 installer 19/19。
- `cargo check -p codex-cli`、rustfmt、CI docs 和 diff check 通过。
- 源码与归档中的 `vsp-codex` 二进制 SHA256 一致，均报告
  `vsp-codex 0.145.0-vsp.9.1`。

本 release 仅发布 Linux x86_64 GNU 制品，不包含 Windows 或 macOS archive。
