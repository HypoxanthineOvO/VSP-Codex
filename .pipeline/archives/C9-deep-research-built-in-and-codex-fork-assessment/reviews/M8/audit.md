# C9 M8 Audit Review - SQLite logging convergence final state

## verdict

PASS，未发现需要阻断 M8 的问题。

M8 最终状态满足本 milestone 的审计条件：runtime smoke 不是伪造检查，证据来自当前源码构建后的 `vsp-codex app-server` 路径、隔离 `CODEX_HOME` / `CODEX_SQLITE_HOME`、真实生成的 `logs_2.sqlite` 文件和 SQLite 查询；源码 diff 仍限于此前 SQLite hotfix 相关的四个文件；报告正确说明保留 VSP in-layer `target == "log"` guard 的原因；未虚假声称已清理历史 DB、完成 release、push/tag/publish 或完成 full alpha.25 merge。

## reviewed_refs

- `.pipeline/prompts/c9-008-sqlite-logging-convergence-and-runtime-io-smoke.md`
- `.pipeline/evidence/C9/M8-sqlite-io-smoke.json`
- `.pipeline/reviews/C9/M8/test.md`
- `.pipeline/reports/C9/M8-sqlite-logging.md`
- `codex-rs/state/src/log_db.rs`
- `codex-rs/state/src/log_db_filter_tests.rs`
- `codex-rs/app-server/src/lib.rs`
- `codex-rs/tui/src/lib.rs`

只读验证命令：

- `python3 -m json.tool .pipeline/evidence/C9/M8-sqlite-io-smoke.json`
- `rg -n "release|publish|tag|push|vacuum|cleanup|clean|delete|历史|alpha\\.25|full|merge|0\\.143|logs_2|target == \\\"log\\\"|target=log|target='log'" ...`
- `git diff -- codex-rs/state/src/log_db.rs codex-rs/state/src/log_db_filter_tests.rs codex-rs/app-server/src/lib.rs codex-rs/tui/src/lib.rs`
- `git diff --check`

`git diff --check` 无输出。

## findings

- 阻断问题：无。
- Runtime SQLite/WAL evidence 真实有效。`M8-sqlite-io-smoke.json` 记录的最终通过命令使用 `cargo build` 后的 `codex-rs/target/debug/vsp-codex app-server --listen off`，隔离 home 下创建 `logs_2.sqlite`，并查询 DB：`total_rows = 0`、`noisy_target_total = 0`、`logs_2.sqlite` 从 `0` 增至 `49152` bytes、`logs_2.sqlite-wal` 保持 `0` bytes。证据还明确把未创建 DB 的旧 binary/remote-control 尝试和被编译耗尽窗口的 `cargo run` 尝试列为 discarded attempts，没有把它们当作通过信号。
- VSP in-layer `target=log` guard 被正确保留并解释。`log_db::default_filter()` 默认关闭 `log`、`codex_otel.log_only`、`codex_otel.trace_safe`；app-server 和 TUI 都接入 `log_db::default_filter()`；`LogDbLayer::on_event()` 内部仍对 `metadata.target() == "log"` 直接 return。单测 `sqlite_sink_drops_tracing_log_bridge_events_even_without_outer_filter` 覆盖了外层 filter permissive 时的 sink-level 防护，支持报告中“外层 filter 可能被 bridge/bypass path 绕过，因此保留内层 guard”的解释。
- M8 报告没有越界声明。报告明确说历史 `~/.vsp-codex` / `~/.codex` SQLite DB 清理、checkpoint、vacuum、rotation 不在 M8 范围且未执行；也明确说未发生 release、push、tag、publish、远程发布或其他 remote side effect；full upstream sync 到 `rust-v0.143.0-alpha.25` 仍是后续工作。
- 当前源码 diff 限于审计指定的四个 SQLite hotfix 文件：`codex-rs/state/src/log_db.rs`、`codex-rs/state/src/log_db_filter_tests.rs`、`codex-rs/app-server/src/lib.rs`、`codex-rs/tui/src/lib.rs`。未看到 M8 implement worker 追加新的源码改动；`.pipeline/reports/C9/M8-sqlite-logging.md` 也声明 implement 阶段只写收口报告。

## risks

- Runtime smoke 是短初始化路径，不是长时间会话或 provider streaming 压力测试。它足以证明当前 app-server 初始化会装配 SQLite log layer、创建真实 SQLite DB、没有写入 noisy targets、WAL 在 bounded smoke 后无残留增长；但不能证明所有高流量真实会话都无写放大。
- `logs` 表总行数为 `0`，所以该 smoke 对“noisy target rejection”的 runtime 信号是负向观测：初始化路径未持久化 noisy rows。更强的后续验证可设计一个受控 runtime 事件注入路径，在真实二进制路径中同时产生 retained target 与 noisy target，再查询 retained/noisy 分布。
- 工作区存在大量既有 `.pipeline/` 未提交/未跟踪状态和四个源码文件 diff；本 audit 只核对 M8 指定对象，没有审计整个 C9 工作区或后续 M9-M12 风险。

## required follow-up

- M8 不需要阻断修复，可以进入后续 Workflow 收口/验收。
- 后续 milestone 若要覆盖真实高流量场景，应追加长时 session 或 provider streaming SQLite/WAL regression smoke，并设置明确的 byte/sample 阈值。
- 历史用户 DB 清理、checkpoint、vacuum、rotation 和服务重启仍需单独计划，不能由 M8 PASS 隐含完成。
- Full alpha.25 merge/release 仍需由后续 M9-M12 或独立 release 流程处理，不能把 M8 SQLite convergence 解释为完整 upstream merge 或发布完成。
