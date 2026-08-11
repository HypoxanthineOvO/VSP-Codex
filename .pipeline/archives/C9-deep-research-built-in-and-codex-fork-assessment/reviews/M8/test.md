# C9 M8 Test Review - SQLite persistent log runtime I/O smoke

## Verdict

PASS。

当前 VSP hotfix 满足 M8 test worker 验证要求：`codex-state` 过滤单测通过；真实 runtime smoke 使用当前源码构建后的 `vsp-codex 0.140.0-vsp.8`，在隔离 `CODEX_HOME` / `CODEX_SQLITE_HOME` 下触发 app-server SQLite runtime 初始化，创建 `logs_2.sqlite`，且没有持久化 `log`、`codex_otel.log_only`、`codex_otel.trace_safe` noisy targets。

实现阶段如果代码已保持当前状态，可以只做报告/集成收口；不需要为 M8 test 信号追加源码修复。

## Commands

- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-state filter -- --nocapture`
  - 结果：PASS，8 passed / 0 failed。
  - 覆盖信号包括：
    - `default_filter_disables_persistent_log_bridge_targets`
    - `sqlite_sink_drops_tracing_log_bridge_events_even_without_outer_filter`
    - `sqlite_sink_drops_low_level_opentelemetry_sdk_logs`
- `cargo build --manifest-path codex-rs/Cargo.toml -p codex-cli --bin vsp-codex`
  - 结果：PASS。
  - 构建后版本：`vsp-codex 0.140.0-vsp.8`。
- Runtime smoke:
  - `CODEX_HOME=/tmp/c9-m8-sqlite-smoke.2QsbHk/codex-home CODEX_SQLITE_HOME=/tmp/c9-m8-sqlite-smoke.2QsbHk/sqlite-home RUST_LOG=trace timeout -s INT 5s codex-rs/target/debug/vsp-codex app-server --listen off`
  - 结果：退出码 1，耗时 210ms。该退出码可接受，因为 `--listen off` 在初始化后按设计报 `no transport configured; use --listen or enable remote control`；本 smoke 的通过依据是 SQLite DB 创建、可查询、且 noisy target 为 0。

## Evidence Path

- `.pipeline/evidence/C9/M8-sqlite-io-smoke.json`

核心证据：

- `logs_2.sqlite`：0 bytes -> 49152 bytes。
- `logs_2.sqlite-wal`：0 bytes -> 0 bytes。
- `SELECT COUNT(*) FROM logs`：0。
- `SELECT COUNT(*) FROM logs WHERE target IN ('log','codex_otel.log_only','codex_otel.trace_safe')`：0。
- JSON verdict：PASS。

## Pass / Fail Signals

Pass signals:

- 单测证明默认 filter 禁用 `log`、`codex_otel.log_only`、`codex_otel.trace_safe`。
- 单测证明即使外层 filter permissive，`LogDbLayer::on_event()` 仍丢弃 `target="log"` bridge event。
- 单测证明低级 `opentelemetry_sdk` TRACE/DEBUG 不写入 SQLite sink。
- 真实 app-server 路径创建了隔离 `logs_2.sqlite`，说明 smoke 不是纯静态检查。
- 真实 DB 查询中 noisy target count 为 0。
- WAL 没有在 bounded smoke 后残留增长。

Fail signals that were explicitly rejected:

- 旧 `target/debug/vsp-codex` 在第一次尝试中显示 `0.140.0-vsp.0`，且 `remote-control` 被 VSP official-service guard 拦截，未进入 SQLite 初始化；该尝试记录为 FAIL/BLOCKED discarded attempt，未作为通过证据。
- 两次 `cargo run` 尝试被编译时间消耗，未产生 runtime SQLite 证据；均记录为 DISCARDED，未作为通过证据。

## Pseudo-Test Rejection

本次 review 不接受伪 smoke：

- 没有手工插入 SQLite rows。
- 没有用单测临时 runtime 代替真实 CLI/app-server path。
- 没有把未创建 DB 的命令判为通过。
- 最终 PASS 来自当前源码构建后的真实二进制、隔离 SQLite home、文件大小观测和 `sqlite3` 查询结果。

## Risks

- `app-server --listen off` 是无网络、短路径 smoke，能验证 app-server 初始化和 SQLite log layer 装配，但不覆盖长时间交互会话或 provider streaming 场景。
- 此 smoke 的 `logs` 表行数为 0；它证明初始化路径没有写入 noisy targets，也证明 DB/WAL 没有放大，但不是高流量压力测试。
- 历史 `~/.vsp-codex/logs_2.sqlite` 清理、vacuum 或 rotation 不在 M8 test worker 范围内，未处理。
- 后续 audit worker 仍应核对最终 diff 是否保留 `default_filter()`、app-server/TUI filter 接入，以及 `LogDbLayer::on_event()` 的 `target == "log"` guard。
