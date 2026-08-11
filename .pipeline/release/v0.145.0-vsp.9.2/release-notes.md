# VSP-Codex v0.145.0-vsp.9.2

Linux x86_64 GNU session compatibility hotfix，基于 `rust-v0.145.0-alpha.4`。

## 修复

- VSP-Codex 与 Codex Official 现在共享同一种 provider-neutral rollout metadata；两端会按各自默认 provider 建立独立索引，不再互相隐藏会话。
- wrapper 显式设置默认 `CODEX_HOME=~/.vsp-codex` 时仍会链接 sessions、archived sessions 和 session index；config、auth 与 state DB 继续隔离。
- 历史 VSP 会话会在新版首次启动时自动迁移。迁移先校验两套 SQLx schema、备份双库，再以等长原位方式中性化 provider，避免移动正在追加的 JSONL 字节。
- 自动迁移使用文件锁和一次性 marker，且捕获脚本 stdout，不会污染 VSCode app-server JSON-RPC。
- 新增 `vsp-codex session-compat status|apply`，用于检查、显式重试或指定自定义 home/backup 目录。
- standalone package 与安装器内置同一迁移工具；全新安装和修复安装会在检测到两套现有 state DB 时自动执行。

## 更新已有安装

```shell
vsp-codex update
vsp-codex session-compat apply
```

第二条命令是可重复执行的一次性确认；正常情况下新版首次启动也会静默完成同一迁移。迁移成功后重新加载 VSCode 窗口，并重启仍在运行的 Codex Official/VSP-Codex 进程，让两端完成各自 backfill。

该更新要求终端能够访问内网 `gitlab.vsplab.cn`，并要求 Python 3。迁移备份位于 `~/.vsp-codex/session-compat-backups/`。

## 全新安装或修复安装

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh -s -- \
  --archive-url https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.9.2/downloads/vsp-codex-preview-linux-x64-gnu.tar.gz \
  --sha256 3078c348e3ea76ac39459125c8f165a6b9328d93ee628bd4345d5942d1781a38 \
  --target x86_64-unknown-linux-gnu \
  --version 0.145.0-vsp.9.2-preview \
  --manifest-url https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/.pipeline/release/vsp-release-manifest.json \
  --vscode \
  --no-launch \
  --yes
```

## 制品

- Archive: `vsp-codex-preview-linux-x64-gnu.tar.gz`
- Size: `342370072` bytes
- SHA256: `3078c348e3ea76ac39459125c8f165a6b9328d93ee628bd4345d5942d1781a38`
- Target: `x86_64-unknown-linux-gnu`
- Channel: `lab`
- Upstream baseline: `rust-v0.145.0-alpha.4`

## 验证

- 本机临时迁移后 Official/VSP 均索引 2789 个 thread，双方独有为 0；双向 `thread/read` 与 `thread/resume` 实测通过。
- `codex-rollout` 107/107、home-dir 12/12、session compatibility CLI 2/2。
- migration Python 7/7、package layout 2/2、Unix installer 16/16。
- release `cargo check`、release build、rustfmt、CI docs、Python compile、shell syntax 和 diff check 通过。
- 包内自动迁移 stdout 隔离、marker 幂等、中断后 backfill 恢复和资源完整性均有直接回归。

本 release 仅发布 Linux x86_64 GNU 制品；Windows 尚不提供历史会话迁移，macOS archive 尚未发布。
