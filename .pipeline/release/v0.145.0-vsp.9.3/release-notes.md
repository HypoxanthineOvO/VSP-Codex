# VSP-Codex v0.145.0-vsp.9.3

Hypo-Workflow Host Contract v1 宿主收敛版本，基于 `rust-v0.145.0-alpha.4`。

## 变化

- Workflow 命令表面改为从已安装 Hypo-Workflow plugin 的 release manifest 动态发现，并严格限制为九个公共命令。
- 新增只读 Host Status projection reader，递归拒绝未知字段、敏感字段、损坏状态和不兼容 contract。
- 移除 Codex 内旧 Workflow state/file writer、conversation capture 和 compact fallback，避免宿主与 Skill 双重拥有状态。
- TUI palette、slash routing 和状态摘要统一消费 Host Contract；Codex 原生 Ask、Plan/SubPlan 与 Subagent 能力保持不变。

## 安装或更新

```shell
vsp-codex update
```

全新安装或修复安装：

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh -s -- \
  --archive-url https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.9.3/downloads/vsp-codex-preview-linux-x64-gnu.tar.gz \
  --sha256 d8558ecf42b5f55703438f41948bb0fad776ff5b9bccc2884ea6ba66eb32d80d \
  --target x86_64-unknown-linux-gnu \
  --version 0.145.0-vsp.9.3-preview \
  --manifest-url https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/.pipeline/release/vsp-release-manifest.json \
  --vscode --no-launch --yes
```

## 制品

- Archive: `vsp-codex-preview-linux-x64-gnu.tar.gz`
- Size: `342586353` bytes
- SHA256: `d8558ecf42b5f55703438f41948bb0fad776ff5b9bccc2884ea6ba66eb32d80d`
- Target: `x86_64-unknown-linux-gnu`
- Channel: `lab`
- Upstream baseline: `rust-v0.145.0-alpha.4`

## 验证

- `cargo check -p codex-core -p codex-tui` 通过。
- Host Contract `5/5`、Workflow routing `4/4`、TUI routing `8/8`、palette regression `1/1` 通过。
- release profile entrypoint 与 code-mode host 从 9.3 源码重新编译；归档 checksum、布局和解包 `--version` smoke 通过。
- 系统缺少 `libcap.pc`，因此 bundle 内 `bwrap` 复用了同 target、同 upstream baseline 的 9.2 已发布资源；未安装系统依赖，也未复用旧 entrypoint。

本版本只发布 Linux x86_64 GNU 制品。
