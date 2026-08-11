# P003: Official 与 VSP 会话互通
- 严重级: high
- 状态: pending_acceptance
- 发现于: Patch after C10
- 创建时间: 2026-07-11 CST
- 关联: VSP/Official session compatibility
- resolved_by: dade17b232
- worker_separation: unavailable_in_current_tool_surface
- worker_lifecycle: test=main, implement=main, audit=unavailable

## 症状

`~/.vsp-codex/sessions` 已链接到 `~/.codex/sessions`，但两端各自维护
`state_5.sqlite`。Official 0.144.1 的 picker 只匹配默认 `OpenAI` provider；VSP rollout
则持久化内部 provider id `vsp`。结果是共享文件存在，但 picker、SQLite metadata 和
resume provider 恢复仍然分叉。

## 临时方案

对历史 VSP rollout 首条 `session_meta` 中的 provider 做等长原位中性化：
`"vsp"` 替换为 `null `。迁移前校验两套 DB migration checksum 完全一致，并使用
SQLite backup API 创建一致性备份；迁移后将两套 backfill 状态重置为 pending，分别由
Official 和 VSP 以各自默认 provider 重建索引。

当前工具面没有可用 subagent 委派入口，无法满足该项目 strict worker separation；本 Patch
不得声称独立 test/implement/audit，完成前保持待审计状态。

## 本机执行结果

- 首轮备份：`~/.vsp-codex/session-compat-backups/20260711-200106/`
- 补充扫描备份：`~/.vsp-codex/session-compat-backups/20260711-200504/`
- 两轮共中性化 575 个 `session_meta` provider 位置；第二轮覆盖同一 rollout 内后续重复写入的 canonical metadata。
- Official 与 VSP 分别完成 backfill 后均索引 2789 个 thread，双方独有 thread 均为 0。
- Official 恢复 VSP 会话时使用 `OpenAI`，VSP 恢复 Official 会话时使用 `vsp`；双向 `thread/read` 与 `thread/resume` 均成功。
- VSP 探针同时暴露一个既有 Notion OAuth `invalid_grant`，与会话恢复无关。

## 回滚与边界

数据库可从对应备份目录恢复；rollout 的等长原位中性化不会移动后续字节，且 `null` provider
由客户端按自身默认 provider 回填，因此通常无需恢复 rollout。迁移脚本依赖 Python 3、SQLite
和 `pread`/`pwrite`，当前发布目标仅覆盖 Linux/macOS；Windows 需要单独实现等价迁移。

## 小版本目标

1. VSP 新建 rollout 不再持久化内部 provider id `vsp`，改为 provider-neutral metadata。
2. 显式使用默认 `CODEX_HOME=~/.vsp-codex` 时仍建立共享 session assets。
3. 将历史迁移工具打包进 standalone release，并纳入安装/更新后的兼容步骤。
4. 增加 Official/VSP 双向发现与恢复回归测试，发布为连续补丁版本 `0.145.0-vsp.9.2`。

## 持久修复实现

- `codex-rollout` 对内部 provider `vsp` 写入 provider-neutral `session_meta`；空 provider 和其他 provider 的既有语义不变。
- wrapper 显式设置默认 `CODEX_HOME=~/.vsp-codex` 时，home-dir 仍会非覆盖地链接 sessions、archive 和 index。
- standalone VSP 包携带 `vsp-session-compat.py`，安装器在两套 DB 都存在时自动执行。
- 新版首次启动在 marker 缺失时静默迁移；脚本 stdout 被捕获，不会污染 app-server JSON-RPC。并发首次启动由文件锁串行化，失败不阻止 VSP 启动并保留重试提示。
- 新增 `vsp-codex session-compat status|apply` 作为显式诊断、重试和自定义 home 入口。

## 需验证的主要行为

1. 同一个新 VSP rollout 在默认 provider 为 `vsp` 和 `OpenAI` 时都能被列表发现。
2. 历史 rollout 迁移保持文件长度和后缀字节不变，并覆盖同一文件内重复 canonical `session_meta`。
3. 迁移前必须校验 DB schema fingerprint 并备份双库；marker 缺失的中断恢复需重置 backfill，marker 存在时不得重复备份。
4. 自动迁移不得向 stdout 输出内容；显式命令应透传 action、Official home、VSP home 和 backup root。
5. 安装包缺少迁移资源时拒绝安装；资源完整时安装器自动执行，且现有 VSP assets 不被覆盖。

## 当前验证

- `codex-rollout`: 99 unit + 8 Official discovery tests passed。
- `codex-utils-home-dir`: 12/12 passed。
- `codex-cli --test session_compat`: 2/2 passed。
- migration Python tests: 7/7 passed，包含 ResourceWarning 严格模式。
- package layout: 2/2 passed；Unix installer: 16/16 passed。
- Python compile、`sh -n`、rustfmt 和 `git diff --check` passed。
- 本机最终脚本报告 0 candidates / 518 already-neutral，并写入一次性 marker；没有新增备份或重置已完成的 backfill。

## Release Candidate

- Version: `0.145.0-vsp.9.2`
- Upstream: `rust-v0.145.0-alpha.4`
- Archive: `342370072` bytes
- SHA256: `3078c348e3ea76ac39459125c8f165a6b9328d93ee628bd4345d5942d1781a38`
- release/source binary SHA256: `c38aec5a6c3dd5f7188055c46cb990a2516a3d57aca7029771a90c2489ecaab6`
- bundled/source migration SHA256: `f498b6da8ff896771d7d45bbeebb7ab343cfd9f11e98445384f7ee2d5219d59d`
- Stable manifest 共 9 个唯一 distribution entries，`vsp.9.2` 位于首项。
- 临时 HOME 通过完整 installer 路径安装；`vsp-codex`、兼容 `codex` wrapper 均报告 `vsp.9.2`。

## 发布与分发

- Release commit: `49ba73d56c2cb7494755bb4fd4531a2b4171f446`
- Tag: `v0.145.0-vsp.9.2`
- GitLab Release: `https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.9.2`
- branch `c10-upstream-0.145.0-alpha.4` 与 `main` 已推送；tag 解引用到 release commit。
- Release API 暴露 archive 与 sha256 两个 direct assets；完整远端 archive 下载 SHA256 匹配 `3078c348...1a38`。
- 本机从 Release URL 重新安装，四个 VSP 入口均报告 `0.145.0-vsp.9.2`；Official 保持 `0.144.1`。
- 安装时迁移报告 0 candidates / 518 already-neutral，未重复备份或重置 backfill。
- 两套 state DB 均为 2789 threads，Official-only 与 VSP-only 均为 0；裸 update 正确报告无更高版本。

由于当前工具面没有独立 audit worker，本 Patch 保持 `pending_acceptance`，不声称独立审计通过。
