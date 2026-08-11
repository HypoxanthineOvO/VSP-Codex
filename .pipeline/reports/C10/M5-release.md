# C10 M5 VSP9 发布候选报告

Status: `published`; final candidate rebuilt, verified, published, and remotely downloaded.

## 结论

`0.145.0-vsp.9` 已完成本地 release candidate 构建与验证。代码基线为
`rust-v0.145.0-alpha.4`，最终只读 fetch 截止点为
`upstream/main@5c19155cbd`。Question Tool 跨终端 ownership、GitLab 自动更新、
官方 upstream Cherry Pick 提醒、Full Access 默认值、compaction 可见性、VSCode
文件链接和 release provenance 均已验证。最终审计发现的 listener-loss handoff 与
resolution cleanup 缺口已在 `7ffab1c7ae` 修复并重新通过 GREEN。最终 ordinal 提交已用
`codex-rollout` 与 `codex-app-server-protocol` 全 crate 回归验证。

发布 commit `79cc6564b9` 已推送到 branch/main，annotated tag `v0.145.0-vsp.9` 已推送，
GitLab Release 与 tar/sha256 assets 已创建并完成完整远端下载校验。本机安装不包含在
本次加速发布中。

## 技术方案

### Question ownership

- App-server 用 handoff lock 串行化 Ask 注册、回答、Resume、unsubscribe 和 disconnect。
- Resume 原子迁移 pending Question 的 UI 与回答所有权；旧连接的延迟回答被拒绝。
- 断线或退订按连接历史恢复前一个 live owner，并复用原 request ID。
- Listener missing/closed 的降级路径复用同一完整 handoff：旧端 resolved、新端 replay、
  回答权限迁移；已接受回答在 listener 丢失时仍定向 resolved 并清理 resolution target。
- Running Resume 只恢复 live pending request；cold cross-process Resume 才从 rollout
  创建 continuation，因此已回答卡片不会复活。
- Native Ask 不再附带合成的 Question Context；fallback 客户端仍能完成 turn。

### 权限、压缩与文件链接

- Plugin thread start 在无显式权限时默认 `never` + `danger-full-access`；top-level 和
  request config 的显式权限优先。
- Local auto-compaction 只暴露 ContextCompaction lifecycle，不暴露内部 summary；
  TurnComplete、TurnAborted 和 Error 均清理抑制状态，真实 final reply 继续输出。
- VSCode integrated terminal 抑制 file OSC8，让 Ctrl+单击由当前 VSCode local/remote
  window 处理。

### 自动更新与 upstream 提醒

- 默认发现源是 GitLab raw stable manifest；环境变量与 install context 可覆盖。
- 最新兼容 release 决定 VSP 更新，精确已安装 release 决定 upstream baseline。
- 通知显示版本与 UpdateAction 必须来自同一 manifest release。
- Startup cache hit 与 background refresh 都可通知当前 TUI session，并去重。
- 存在更高 VSP release 时优先提示 VSP 更新；仅当 VSP 已最新时提示官方 Cherry Pick。
- Stable manifest 即分发发现服务，不需要额外云服务，但客户端必须能匿名读取该 URL。

### Upstream 与发布来源

- alpha.4 baseline 以 merge commit `820f9b14b8` 纳入。
- alpha.4 后 10 个提交按 upstream 顺序纳入到本地 `47bb38a787`；7 个 patch-id 完全
  一致，3 个保留 VSP provider/cache、remote-plugin guard 或相邻 discovery/test context。
- VSP listener-loss 审计修复独立提交为 `7ffab1c7ae`，不会改变 10 个 upstream
  Cherry Pick 的 patch-id 结论。
- VSP rollout fixture 参数漂移独立修复为 `ecf799926b`。
- Builder 显式写入 `upstream_tag=rust-v0.145.0-alpha.4`，并按
  `(version, channel, target)` 替换 stable manifest 条目。
- Cargo workspace 与 npm package version 在打包前必须一致。

## 修改模块

- `codex-rs/app-server`: native Ask/fallback、pending ownership、Resume/断线恢复、
  权限默认值、compaction event visibility。
- `codex-rs/tui`: GitLab updater、upstream reminder、VSCode file links、IDE IPC、
  model availability 与 alpha.4 后官方 TUI 变更。
- `codex-rs/core`, `codex-rs/thread-store`, `codex-rs/rollout`,
  `codex-rs/app-server-protocol`, `codex-rs/model-provider`: reasoning、response IDs、
  分页持久化、durable ordinals、history projection 与 provider 兼容。
- `codex-rs/exec-server`, `codex-rs/memories/write`, `codex-rs/ext/extension-api`:
  alpha.4 基线前后保留的 upstream reliability 变更。
- `scripts/build_vsp_preview_release.py` 与 `.pipeline/release/`: version/provenance、
  archive、checksum、versioned manifest 和 stable manifest。

## 测试设计与结果

- Ask integration 24/24；Ask focused unit 5/5；app-server lib 282/282。
- Full Access defaults 2/2、显式 request config override 1/1、local compaction 1/1。
- VSP updater release profile 18/18；cache 2/2；update action 2/2。
- IDE IPC 10/10；VSCode native file-link 1/1；model availability 4/4。
- Memories 39/39；extension API 11/11；exec-server 全 package targets 通过。
- Core workspace-roots 6 unit + 2 integration；reasoning 13 unit + 23 integration；
  review integration 10/10。
- 最终 ordinal delta：`codex-rollout` 99/99；`codex-app-server-protocol` 271/271。
- Release builder 4/4；archive binary、metadata、members 和 SHA256 验证通过。
- rustfmt、CI docs、diff、JSON、YAML、conflict marker 和 pending snapshot 检查通过；
  app-server strict Clippy 在定点忽略 2 条既有、非本次 diff 告警后通过。

完整命令与环境说明见 `.pipeline/reviews/C10/M5/test.md`。

## 发布产物

- Version: `0.145.0-vsp.9`
- Upstream baseline: `rust-v0.145.0-alpha.4`
- Upstream cutoff: `5c19155cbd`
- Local HEAD: `ecf799926b`
- Archive: `vsp-codex-preview-linux-x64-gnu.tar.gz`
- Size: `352415153` bytes
- SHA256: `bbc6ecd5abbee8ea8273e57db80086bb05d3ba4dde6804937f1f5559db121a72`
- Intended release URL:
  `https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.9`

该 URL 将在本报告提交后创建并做远端 checksum 验证。

## 预期用户效果

- 终端 B Resume 后接管待回答卡片；终端 A 不能再用旧答案解除 working 状态。
- B 断线或退订后，仍在线的 A 恢复同一卡片；已回答卡片不被再次 Resume 复活。
- Plugin 默认 Full Access，但用户显式选择更严格权限时保持该选择。
- 自动压缩不显示内部 summary，也不会吞掉压缩后的真实回复。
- VSCode Ctrl+单击在当前 VSCode window 中打开本地或 remote 文件。
- 已安装客户端通过 GitLab stable manifest 发现新 VSP release；维护者在 VSP 已最新时
  获得官方 upstream 更新提醒。

## 问题与处理

- 三轮 Ask 审计发现 disconnect、unsubscribe、fallback、listener-loss 和 resolution
  races；统一 handoff lock、listener ordering、降级 replay 与 resolution target cleanup
  后，5 个 focused unit、24 个集成和 282 个 app-server lib 测试全部通过。
- alpha.4 后 IDE IPC 测试在主机 `umask 002` 下因 socket parent 为 0775 被严格校验拒绝，
  server thread 随后阻塞于 accept。测试夹具改用 0700 目录和 bounded accept；生产校验未放宽。
- 一个过宽的 `review` filter 运行 98 个 Guardian tests 时，当前仓库 `.pipeline/` 指令进入
  2 个模型可见请求快照。生成的 `.snap.new` 被拒绝并删除；目标 review integration 10/10 通过。
- 最终 fetch 三次发现 upstream 前进，先补齐 7 个、再补齐 2 个、最后补齐 ordinal
  生产提交；每次都重新打开
  GREEN，未沿用过期证据。
- release builder 必须显式传 alpha.4 tag，避免多 release 历史下 nearest-tag provenance 歧义。
- 严格 Clippy 首轮只命中 `thread_processor.rs` 两条既有、非本次 diff 的
  needless-lifetime/borrow；定点 allow 后其余 app-server lib/tests 无告警。
- Listener-loss 降级分支共享同一 handoff 已通过 helper 测试与独立代码审查，但当前
  自动化没有直接故障注入 closed listener 后再跑 native/fallback replay；测试报告已
  收窄为实际覆盖范围。

## 风险与后续

1. Live Question ownership 迁移是单 app-server 进程能力。跨进程 Resume 可创建 continuation，
   但不能主动关闭另一台机器上旧进程的 UI 卡片。
2. Updater 在启动和 stale-cache refresh 时检查，不是常驻秒级推送。
3. GitLab raw manifest 必须对安装客户端可访问；私有化或网络策略会阻断自动发现。
4. 当前候选只包含 Linux x86_64 GNU；Windows/macOS archive 和实机安装未覆盖。
5. 工作区 `.pipeline/chat`、inbox 和 mini-cycle 运行文件保持未提交，不属于 release commit。
6. Closed-listener native/fallback replay、completion loss 与 compaction Error cleanup
   尚缺直接故障注入测试；现有 GREEN 不应被解读为这些路径都有端到端覆盖。

## 发布执行

用户已显式授权。顺序固定为：提交最终 evidence/artifacts -> push branch/main ->
创建 `v0.145.0-vsp.9` tag -> 上传 archive/sha256 -> 创建 GitLab Release -> 验证远端
checksum -> 本地安装 -> 验证 `vsp-codex`、compat `codex` 和 standalone 三入口 ->
检查 GitLab pipeline。
