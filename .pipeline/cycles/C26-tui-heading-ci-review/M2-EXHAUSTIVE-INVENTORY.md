---
kind: exhaustive-test-inventory
cycle: C26-tui-heading-ci-review
milestone: M2
status: completed
updated: 2026-08-09T19:49:27+08:00
---

# M2 全仓测试、Snapshot 与 CI 可信度清单

## 结论

本轮已经把 Cargo package、test target、test source/helper/fixture、snapshot asset、Bazel target expansion 和 CI job 做了双向对账。源码语义审阅未检查项为 0；macOS、Windows、Wine、remote executor、真实 provider 与真实外网行为没有在本机动态执行，作为平台验证边界单列，不算已通过。

当前测试体系不能作为 C26 验收证据。问题不是单一的 42 个 TUI 失败，而是 build/load 阻断、silent-pass、same-source oracle、mock-only、全局状态、错误 binary 依赖、动态 production fixture、弱 snapshot、平台缺口和 CI 覆盖错位共同存在。

## 完整性证明

| 面 | 已枚举 | 审阅状态 |
| --- | ---: | --- |
| Cargo workspace packages | 131 | 131/131 |
| Cargo targets | 248 | 248/248 |
| test-enabled Cargo harness targets | 217 | 217/217 |
| Rust test/source/helper/fixture files | 1,686 | 1,686/1,686 |
| Rust cases | 至少 12,526 | 12,487 可列出；39 被 2 个 test crate 编译错误阻断；sample 为 0-case compile harness |
| 非 Rust first-party tests | 33 files / 308 logical cases | 33/33 |
| 外部 Insta snapshots | 614 | 614/614 |
| Inline snapshot assertions | 19 / 7 files | 19/19 |
| GitHub workflows | 29 | 29/29 |
| GitHub composite/actions | 11 | 11/11 |
| GitHub jobs | 94 | 94/94 |
| GitLab jobs | 1 | 1/1 |
| Bazel BUILD packages | 131 | 131/131 static mapping；3 个 package load blocker 已动态复现 |

测试源 scanner 的上界是 1,734 个 source/harness 文件：1,686 Rust、28 Python、8 TypeScript、6 shell、6 Bazel/Starlark。它不是通过数，而是确保所有测试相关文件都进入人工 domain review 的 routing inventory。

### Rust domain reconciliation

| Domain | Files / tests | 主要结论 |
| --- | --- | --- |
| TUI | 223 test/support files；3,226 tests | production 替代路径、theme race、ignored real terminal tests、弱 VT100、动态 model fixture |
| Core + protocol + rollout + proxy | 287 files；3,476 functions / 3,489 logical cases | 795 silent network returns、same-source oracle、host/global state、external Workflow suite、remote/platform错位 |
| App-server + protocol/client + exec + exec-server | 215 test files；1,835 tests | public JSON-RPC缺口、silent-pass、fixed-sleep假覆盖、ignored regression、same-source wire mock |
| 其余 Rust packages | 115 packages；785 sources；4,214 declarations | 20 zero-test crates、81+ silent-pass paths、mock-only real seams、wrong binaries、cross-platform fixtures |

Core 的最后缺口已返工完成：A-M 为 498 test functions / 502 logical cases，N-Z 为 467 functions / 476 logical cases，support/auxiliary 为 40/40；protocol、rollout、proxy 和 core/src 全部逐文件逐测试审阅。Snapshot 正文由独立 snapshot lane 覆盖，未重复计数。

## 静态候选上界

这些数量用于 completeness 和人工 routing，不能自动判定每一处有问题：

| Rule | Matches / files |
| --- | ---: |
| snapshot macro | 339 / 71 |
| absolute user path | 215 / 43 |
| parent/sibling path | 51 / 17 |
| env mutation | 55 / 23 |
| cwd mutation | 4 / 2 |
| external URL literal | 1,790 / 353 |
| loopback URL literal | 277 / 88 |
| sleep | 344 / 174 |
| wall clock | 852 / 255 |
| ignored test attrs | 27 / 18；nextest 实际列出 29 ignored cases |
| should_panic | 12 / 5；7 个无 expected message |
| contains assertion | 1,625 / 273 |
| field-level assert_eq candidate | 10,737 / 792 |
| cfg(test) function | 200 / 114 |

绝对路径多数是合法 parser fixture；URL 多数是本地 mock payload；field assertion 很多是精确 API 合同。只有经人工复核的实例进入 findings。

## Build 与 target blockers

### Cargo workspace

1. 默认 `cargo nextest list --workspace` 在 `codex-bwrap` build script 因系统缺 `libcap.pc` 失败。使用仓库提供的 `CODEX_SKIP_BWRAP_BUILD=1` 仅用于继续 inventory，不代表 bwrap 路径通过。
2. `install-context/src/lib.rs:568` 的测试仍按旧 API 传 5 个参数，当前函数只有 4 个参数。
3. `thread-manager-sample/src/main.rs:177` 手写 `Config`，缺 `addon_model_providers` 与 `base_model_provider_id`。它没有 tests，但 binary test harness 无法编译。
4. `app-server-client/src/lib.rs:1058` 仍写旧 `cloud_requirements` 字段；`:1975`、`:2041` 的 question fixtures 缺 3 个新字段。

排除以上 3 个 package 后，nextest 列出 214 个 binaries、110 个 packages、12,487 cases；两个被阻断的 test crates另有 39 个 tests。

### Bazel required CI

动态 `bazel query` 已逐个复现：

1. `tui/BUILD.bazel:18,27` 在同一调用重复 `extra_binaries`。
2. `core/BUILD.bazel:15,53` 同样重复 `extra_binaries`。
3. `exec-server/BUILD.bazel:22` 传入 `defs.bzl:181` 未支持的 `unit_test_args`。

因此 required Bazel PR CI 当前在 load 阶段就失败，不能到达 tests。理论 expansion 为 510 个 test-rule instances；当前 keep-going query 只能实现 441 个。

Bazel 还漏掉 CLI main 110、state logs client 3、app-server main 1、exec main 1，共 115 个 binary unit tests，并漏 2 个 doctests；另一方面把 `exec/tests/event_processor_with_json_output.rs` 的 30 cases 每个平台重复执行。

## 严格执行结果

正式命令使用与 GitHub full nextest 相同的 8 MiB stack，关闭重试并禁止 snapshot 更新：

```text
CI=1 INSTA_UPDATE=no RUST_MIN_STACK=8388608 CODEX_SKIP_BWRAP_BUILD=1 \
cargo nextest run --workspace \
  --exclude codex-install-context \
  --exclude codex-thread-manager-sample \
  --exclude codex-app-server-client \
  --no-fail-fast --retries 0
```

结果：12,458 non-ignored cases executed；12,131 passed；327 failed；0 timeout；0 stack overflow；29 ignored 未执行。

| Binary | Failures |
| --- | ---: |
| `codex-core::all` | 214 |
| `codex-tui` | 42 |
| `codex-app-server::all` | 24 |
| `codex-core` unit | 15 |
| `codex-network-proxy` | 7 |
| `codex-linux-sandbox::all` | 7 |
| `codex-mcp-server::all` | 3 |
| `codex-exec-server::exec_process` | 3 |
| other 9 binaries | 12 |

一次先行诊断漏设 `RUST_MIN_STACK`，产生 840 个 stack-overflow abort；该结果已作废，不用于产品/CI结论。它只说明裸 `cargo nextest` 与仓库/CI命令不等价。

## Findings 分类

### P0 - CI 与 workspace 无法完整加载/编译

- 三个 Bazel package load blockers使 PR required CI 必败。
- 三个 Cargo package test-build blockers使 full workspace test无法完整列出或执行。
- CLI 多个 tests及 Bazel benchmark仍请求不存在的 `codex`/`codex-rs` binary，当前 target只有 `vsp-codex`。

### P1 - 默认 PASS 不能证明测试执行过

- `core/tests/common/lib.rs:536-631` 的 skip macros普通 return；仅 core 即有 `skip_if_no_network!` 795 calls / 86 files，另有 sandbox、remote-env、Wine和平台 returns。
- 其余 Rust范围至少81条默认路径因 sandbox、binary或capability缺失普通返回，包括登录/LM Studio/Ollama、PTY、MCP server、Landlock、UDS、PowerShell与CLI bwrap。
- 两个本地 mock app-server tests在私有环境变量缺失时直接成功退出；缺失真实外网并非合理原因。
- 27 个 ignore attrs展开为29 cases；扣除 child-process入口、live API和手工tmux后，仍有关键 compaction、pending-input、review、unified-exec、Windows shell/ConPTY等真实行为被永久关闭。

### P1 - same-source、mock-only 与无效 oracle

- Python SDK固定 `openai-codex-cli-bin==0.137.0a4`，CI安装并测试旧 published runtime；schema、client、harness来自同一版本，完全不验证 current checkout。
- app-server-client notification test用同一 `ServerNotification` 类型生成并解析 JSON；不能发现真实 server/client wire drift。
- core request-permissions的“预批准”测试收到 approval request 后主动 Approved；grant完全失效仍通过。
- core Ask round-trip没有第二 response或 function-call-output assertion；回答不回传模型也可能通过。
- 两个 parallelism tests阈值1.6s，而串行执行约0.5s；完全串行仍通过。
- Windows symlink-race security smoke检查了错误的局部 outside 路径；真实逃逸目标被写入也可能通过。
- Linux network sandbox只断言任意 non-zero exit；command-not-found、DNS或远端故障都能冒充 sandbox denial。
- TypeScript abort test在正常结束路径自抛 sentinel，又由同一 catch接受；abort失效仍可通过。
- Cloud Tasks真实 HTTP client、OS keyring、analytics delivery、backend client、notify backend、sleep-inhibitor OS backend等关键seam只有mock或零测试。

### P1 - 测试走了非生产路径

- TUI `markdown_stream.rs` 的23个collector tests有22个走test-only renderer/simulator，只有1个走生产 source boundary。
- Core无条件编入 `test_sync_tool`，甚至可由model metadata在产品中暴露；同时大批 tests测试test-only renderer、test mode和改短的timeout/lease参数。
- C26新增最终回复 App test用 `ImageView` 而非 `ImageGeneration`，canonical只有两行、24行终端且手工处理两个event；没有复现真实绘图后长回复overflow/scrollback截断。
- `lastAgentMessage` 只有内部 `handle_turn_complete` producer test，没有真实 `codex-app-server` JSON-RPC进程级通知测试。

### P1 - 环境、全局状态与时间依赖

- TUI仍有5个未guard/未serial的全局 theme 写；任意572个TUI snapshot可与其竞态。
- ChatGPT test修改process cwd；CLI test替换PATH；core tests修改CODEX_HOME、stdin、connector cache、test mode；多数保护不覆盖其他test binaries或panic路径。
- TUI helper `tempdir().keep()` 被约740个调用点使用，测试持续泄漏home目录。
- 344个sleep候选；高置信问题包括per-character typing helper、git mtime sleeps、fixed negative windows、bind/drop/rebind TOCTOU及无timeout accept/recv。
- Core `wait_for_mcp_server` 无timeout；其他helper把调用方timeout强制clamp到至少10秒。

### P1 - 外部 Workflow contract 放错 suite

- `core/tests/suite/mod.rs:66` 把7个 `hypo_workflow_host_contract` tests无条件放入默认 `core::all`。
- `hypo_workflow_root()` 默认拼 sibling `../Hypo-Workflow`；4项读取外部manifest/fixtures，3项扫描源码字符串。
- GitHub/Bazel没有 checkout、env或runfiles映射。默认 full test的实际执行已按预期失败。
- 正确边界：本仓 consumer行为用固定repo-owned fixture留default；真实跨仓compatibility使用固定release/digest的独立mandatory job。

### P2 - Snapshot inventory

- 外部 assets 614；活跃映射593；明确orphan 21；stale provenance但仍活跃76；`.snap.new`为0。
- 21个orphan：message_queue 4；chatwidget 5；diff_render 5；resume_picker 2；status_indicator 2；bottom-pane 3。
- 26组/53个payload完全重复；至少tight-height和remote-image-selected没有观察到场景声称的状态。
- 25个通用UI snapshots通过production bundled catalog共享 `gpt-5.6-sol`；catalog变化会制造跨模块漂移。
- 31个snapshots序列化`0s`；部分依赖“创建后立即render”而非注入clock。
- CI没有显式 `cargo insta pending-snapshots`；full Cargo也没有独立 snapshot/worktree hygiene gate。

## CI coverage 结论

- GitLab required `vsp-smoke` 只有 `cargo build --locked` 和CLI/TUI首帧/app-server smoke，不运行test、Clippy或snapshot。它成功不能证明C26正确。
- GitHub PR blocking以Bazel为Rust主门禁；完整Cargo nextest/Clippy只在postmerge。因此Bazel漏掉的115个binary tests不是merge-blocking。
- `repo-checks.yml`固定旧 `0.133.0-alpha.4` 和历史 workflow run `26201494185`；它测试旧远程artifact，不是current commit。
- nextest全局 `retries=1`；Bazel shard自动 `flaky=True`，首轮失败可以被重试掩盖。
- macOS Cargo只跑arm64；Linux x64只有remote-env lane；Linux arm64 Bazel tests整组关闭；平台矩阵不能报告成等价全覆盖。
- `.bazelrc` 用硬编码test-name substring跳过code-mode、ConPTY与PowerShell cases；fork无BuildBuddy时的fallback执行形态不同。
- Python SDK 38个“real integration”默认skip且仍用pinned runtime和硬编码远程model；另有95个tracked non-Rust cases不在默认blocking CI。

## Suite 边界处置

### 留在 default 且必须真实执行

- 纯transform/schema/serialization、TempDir+SQLite/FS deep equality、loopback protocol/state machine、固定synthetic model/wire fixtures。
- loopback不等于外网；不能因 `CODEX_SANDBOX_NETWORK_DISABLED` 普通return。

### 重写 oracle/harness

- same-source expected、contains-only结构、允许null/empty的negative、固定短观察窗、错误binary、全局env/cwd/theme、test-only production替代路径。
- 对公开JSON/JSON-RPC/CLI输出使用literal或独立fixture反序列化后whole-object equality。

### 独立 mandatory suites

- remote executor、Docker/POSIX remote、zsh-fork/DotSlash、OS sandbox/login shell/shell snapshot、真实git/python/helper-binary、live provider/API、Windows/Wine平台合同、跨仓Workflow compatibility。
- 独立suite不是豁免；缺prerequisite必须infra failure或runner-visible skip，并校验discovered/executed case count。

## 未执行边界与本轮产物

- 未动态执行macOS、Windows、Wine、remote executor、真实provider或真实外网；相关源码与CI routing已审。
- 非Rust测试做了collection与语义审阅，没有全套动态执行。
- 本轮没有修改或接受任何Rust、CI或snapshot文件。
- 正式run生成3个未跟踪 `codex-rs/core/.tmp*` 目录；精确删除被安全hook拦截，未绕过。需用户授权后单独清理。
- `/tmp/c26-nextest-list-excluding-blockers.json`、`/tmp/c26-nextest-run-ci.stderr` 与 `/tmp/c26_test_quality_inventory.json` 是本机审计证据，不进入源码提交。
