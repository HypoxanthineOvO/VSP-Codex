# C5 Sync / Docs / Release Prep Report

时间：2026-05-28T22:49:25+08:00

## 结论

C5 已完成 sync、docs 和 release prep，可进入提交、打 tag、推送与远端发布门控。

本次 release 版本确定为 `0.134.0-vsp.5`，因为 `v0.134.0-vsp.4` 已存在并用于 VSCode 空 `model_provider` hotfix。

## Sync

- 已运行 `hypo-workflow sync --repair --platform opencode --project /home/heyx/Codex-VSP`。
- 已刷新 compact/derived 视图和 `PROJECT-SUMMARY.md`。
- Sync 只处理派生产物，没有把 sync 当作 milestone runner。

## Docs

- `README.md` 快速安装命令切到 C5 预览包。
- `docs/vsp-codex.md` 用户指南切到 C5 预览包，并保留 Thinking、Markdown、命令面板、Resume、VSCode 接入说明。
- `docs/install.md` 安装文档切到 C5 预览包，并说明一条命令安装、GitLab raw 下载权限、安装器参数和 C5 体验更新。

安装命令使用：

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | sh -s -- \
  --archive-url https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/.pipeline/release/C5-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz \
  --sha256 5221b3410cf3db67cc7d3618a446b6ac42956bd18a8b7aab5ff024a15d7a8598 \
  --target x86_64-unknown-linux-gnu \
  --version 0.134.0-vsp.5-preview \
  --vscode \
  --no-launch \
  --yes
```

## Release Prep

- `codex-rs/Cargo.toml` 升到 `0.134.0-vsp.5`。
- `codex-rs/Cargo.lock` 中 workspace package 版本同步到 `0.134.0-vsp.5`。
- `codex-cli/package.json` 升到 `0.134.0-vsp.5`。
- `CHANGELOG.md` 新增 `v0.134.0-vsp.5 - 2026-05-28`，覆盖主题选择器、命令面板、复制最新代码块、Resume metadata cache 和预览包准备。

## Preview Package

- Package directory: `.pipeline/release/C5-preview/package`
- Archive: `.pipeline/release/C5-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz`
- SHA256: `5221b3410cf3db67cc7d3618a446b6ac42956bd18a8b7aab5ff024a15d7a8598`
- Binary version: `vsp-codex 0.134.0-vsp.5`

## Validation

- `cargo build --manifest-path codex-rs/Cargo.toml --bin vsp-codex`: passed.
- `.pipeline/release/C5-preview/package/bin/vsp-codex --version`: `vsp-codex 0.134.0-vsp.5`.
- Extracted archive smoke test: `vsp-codex 0.134.0-vsp.5`.
- `sha256sum -c .pipeline/release/C5-preview/vsp-codex-preview-linux-x64-gnu-debug.tar.gz.sha256`: passed.
- `just test -p codex-tui resume_picker`: passed 98/98.
- `just test -p codex-rollout state_db_only`: passed 1/1.
- `just test -p codex-app-server thread_list_state_db_only_returns_sqlite_without_jsonl_repair`: passed 1/1.
- `just test -p codex-thread-store read_thread_applies_sqlite_thread_name read_thread_uses_sqlite_metadata_for_rollout_without_user_preview list_threads_preserves_sqlite_title_search_results`: passed 3/3.
- `just test -p codex-tui command_palette`: passed 7/7.
- `just test -p codex-tui markdown_render`: passed 127/127.
- `git diff --check`: passed.
- `git tag --list v0.134.0-vsp.5`: no existing local tag.

## Remaining Gate

- C5 state is still `completed_pending_acceptance`.
- Commit, tag `v0.134.0-vsp.5`, push `main`, push tag, and optional GitLab Release asset publication have not been executed in this prep step.
