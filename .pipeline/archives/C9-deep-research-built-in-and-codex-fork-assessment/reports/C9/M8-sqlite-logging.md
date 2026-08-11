# C9 M8 SQLite Logging Convergence Report

## conclusion

M8 结论：PASS。当前 VSP `0.140.0-vsp.8` SQLite persistent logging 行为已与 official alpha.25 的关键降噪修复收敛，并保留了 VSP 额外的 in-layer `target == "log"` 防护。基于已通过的 M8 test evidence，本 implement worker 不做源码、测试源码、manifest、evidence、review、state/cycle/log/PROGRESS、prompt 或 release artifacts 修改；M8 只写入本收口报告。

M8 没有执行历史 SQLite DB 清理、vacuum 或 rotation；这些工作明确不在 M8 范围内。M8 也没有执行 release、push、tag、publish、远程发布或其他 remote action。

## official alpha.25 fix comparison

M7 baseline 指向 upstream tag `rust-v0.143.0-alpha.25` / commit `d634f56262`。与 SQLite logging 相关的 official fixes 包括：

- `d81fcdf8ef` / #26396：降低 OpenTelemetry SDK debug logs 导致的 SQLite contention。
- `e98d43ac37` / #29457：从 persistent logs 中过滤 noisy targets。
- `91e6da943b` / #29432：停止记录每个 Responses WebSocket event。
- `63f8f547c0` / #29599：停止持久化 bridged log events。

VSP 现有 hotfix 已覆盖 alpha.25 的核心 persistent-log filtering 方向：默认过滤 `log`、`codex_otel.log_only`、`codex_otel.trace_safe`，并在 app-server/TUI 初始化时把过滤器接入 SQLite log layer。VSP 还额外保留 `LogDbLayer::on_event()` 内部的 `target == "log"` guard，用于防止外层 tracing filter 被 bridge/bypass path 绕过。

因此 M8 的收敛策略不是盲目替换当前 VSP hotfix，而是确认当前行为与 alpha.25 等价或更强，并保留 VSP 本地更强的防护点。

## current VSP behavior

当前 VSP logging 行为来自既有 hotfix：

- `codex_state::log_db::default_filter()` 默认拒绝 `log`、`codex_otel.log_only`、`codex_otel.trace_safe`。
- app-server 和 TUI SQLite tracing 初始化使用该默认 filter。
- `LogDbLayer::on_event()` 对 `metadata.target() == "log"` 做 in-layer drop，即使外层 filter permissive，也不把 bridged `target="log"` events 写入 SQLite。
- `opentelemetry_sdk` 等低层 noisy logs 继续被过滤，避免重复进入 persistent SQLite sink。

这与 M7 debug report 的根因一致：历史本机 `~/.vsp-codex/logs_2.sqlite` / WAL 中，主要放大来源是 `target='log'` bridged events 与 `codex_otel.*` persistent log events。

## whether code changed in M8

本 implement worker 未在 M8 中修改源码或测试源码。M8 沿用此前已经落地的 VSP hotfix，并基于 test worker 已通过的 unit validation 与 runtime smoke evidence 编写本 convergence 报告。

本次写入范围仅限：

- `.pipeline/reports/C9/M8-sqlite-logging.md`

未修改：

- `codex-rs/state/src/log_db.rs`
- `codex-rs/state/src/log_db_filter_tests.rs`
- `codex-rs/app-server/src/lib.rs`
- `codex-rs/tui/src/lib.rs`
- manifest、release artifacts、prompt、review、evidence JSON、state/cycle/log/PROGRESS

## unit validation

M8 test worker 已通过 targeted unit validation：

```bash
cargo test --manifest-path codex-rs/Cargo.toml -p codex-state filter -- --nocapture
```

结果：PASS，`8 passed; 0 failed`。覆盖信号包括：

- `default_filter_disables_persistent_log_bridge_targets`
- `sqlite_sink_drops_tracing_log_bridge_events_even_without_outer_filter`
- `sqlite_sink_drops_low_level_opentelemetry_sdk_logs`

这些单测证明默认 filter 会拒绝已知 noisy persistent-log targets，并证明即使外层过滤器被配置为 permissive，SQLite sink 内层仍会丢弃 `target="log"` bridged events。

## runtime smoke evidence

M8 runtime smoke 使用当前源码构建后的真实 CLI/app-server path，而不是伪造 SQLite rows 或只做静态检查。

构建命令：

```bash
cargo build --manifest-path codex-rs/Cargo.toml -p codex-cli --bin vsp-codex
```

结果：PASS；构建后版本输出为 `vsp-codex 0.140.0-vsp.8`。

Runtime smoke 命令：

```bash
CODEX_HOME=/tmp/c9-m8-sqlite-smoke.2QsbHk/codex-home CODEX_SQLITE_HOME=/tmp/c9-m8-sqlite-smoke.2QsbHk/sqlite-home RUST_LOG=trace timeout -s INT 5s codex-rs/target/debug/vsp-codex app-server --listen off
```

结果：退出码 1，耗时 210ms。该退出码可接受，因为 `--listen off` 在初始化后按设计报 `no transport configured; use --listen or enable remote control`；本 smoke 的判断依据是 SQLite DB 创建、查询结果与 noisy target rejection。

核心 evidence：

- `logs_2.sqlite`: `0` bytes -> `49152` bytes。
- `logs_2.sqlite-wal`: `0` bytes -> `0` bytes。
- `SELECT COUNT(*) FROM logs`: `0`。
- noisy targets 查询总数：`0`。
- 检查的 noisy targets：`log`、`codex_otel.log_only`、`codex_otel.trace_safe`。
- evidence verdict：PASS。

Test worker 明确拒绝了未进入 SQLite 初始化或被编译时间消耗的 discarded attempts；最终 PASS 来自真实构建后的 `vsp-codex app-server` 路径、隔离 `CODEX_HOME` / `CODEX_SQLITE_HOME`、生成的 SQLite 文件和 `sqlite3` 查询。

## decision on retaining VSP in-layer `target=log` guard

M8 决策：保留 VSP in-layer `target == "log"` guard。

理由：

- alpha.25 的外层过滤方向与 VSP hotfix 一致，但 VSP 已经证明本地历史放大主要来自 `target='log'` bridged events。
- 单测 `sqlite_sink_drops_tracing_log_bridge_events_even_without_outer_filter` 覆盖了外层 filter permissive 时的 sink-level 防护。
- 保留内层 guard 不改变正常 user/thread diagnostics 的持久化策略，却能防止 tracing-log bridge 或未来初始化路径绕过外层 filter 后重新写入 high-volume `log` target。
- M7 equivalence matrix 已将 SQLite persistent log filtering 列为 M8 replacement/convergence candidate，同时要求仅在 alpha.25 覆盖不足时保留 VSP guard；M8 test evidence 支持该 guard 是本地更强保护。

## risks/follow-up

- M8 runtime smoke 是无网络、短路径 app-server 初始化 smoke；它验证 SQLite layer 装配、DB 创建、noisy target rejection 和 WAL 无残留增长，但不是长时间交互会话或 provider streaming 压力测试。
- Smoke 中 `logs` 表行数为 `0`，说明 bounded 初始化路径没有写入 noisy rows；这不等同于覆盖所有高流量真实会话场景。
- 历史 `~/.vsp-codex/logs_2.sqlite`、`~/.vsp-codex/logs_2.sqlite-wal`、`~/.codex/logs_2.sqlite`、`~/.codex/logs_2.sqlite-wal` 的清理、checkpoint、vacuum 或 rotation 不在 M8 范围，未执行。
- 已运行的旧 VSP/Codex 进程若仍使用旧二进制，需要在用户明确安排后重启或替换二进制；M8 implement worker 未执行 service restart。
- Full upstream sync 到 `rust-v0.143.0-alpha.25` 仍是后续较大 Cycle/Milestone 工作，M8 只处理 SQLite logging convergence，不处理 protocol/app-server、core runtime、TUI、CLI、plugins、docs/CI/release 的大范围 merge。
- 本 worker 未发生 release、push、tag、publish、远程发布或其他 remote side effect。

## modified files

- `.pipeline/reports/C9/M8-sqlite-logging.md`
