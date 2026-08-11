# SQLite Log Write Amplification Investigation

## Conclusion

VSP Codex `v0.140.0-vsp.8` inherited the upstream persistent SQLite log amplification risk. The local machine has large existing diagnostic log databases:

- `~/.vsp-codex/logs_2.sqlite`: about 1.75 GB
- `~/.vsp-codex/logs_2.sqlite-wal`: about 548 MB
- `~/.codex/logs_2.sqlite`: about 1.13 GB
- `~/.codex/logs_2.sqlite-wal`: about 51 MB

The dominant retained targets in `~/.vsp-codex/logs_2.sqlite` were `log` bridged events and `codex_otel.*` persistent log events. This matches the reported class of issue: high-volume diagnostic events being persisted into the dedicated SQLite logs database rather than only useful user/thread diagnostics.

## Official Upstream Status

Fetched `upstream` tags and refs on 2026-06-26. The latest visible Rust prerelease tag is `rust-v0.143.0-alpha.25`; there is no local `rust-v0.144*` tag after fetch. GitHub Releases also identifies `0.143.0-alpha.25` as the latest prerelease and `0.142.2` as the latest stable release.

Relevant upstream fixes:

- `d81fcdf8ef` / #26396, `Reduce SQLite contention from OpenTelemetry SDK debug logs`: present in stable `rust-v0.142.2` and latest prerelease.
- `e98d43ac37` / #29457, `Filter noisy targets from persistent logs`: present in stable `rust-v0.142.2` and latest prerelease.
- `91e6da943b` / #29432, `Stop logging every Responses WebSocket event`: present in stable `rust-v0.142.2` and latest prerelease.
- `63f8f547c0` / #29599, `Stop persisting bridged log events`: present in `rust-v0.143.0-alpha.6` through `rust-v0.143.0-alpha.25`; not contained in `rust-v0.142.2`.

The local VSP database shows `target='log'` is the largest retained target, so the #29599 behavior is important for VSP even though stable `0.142.2` contains earlier noise filters.

## Full Upstream Merge Assessment

Created branch `c9-upstream-0.143.0-alpha.25-vsp8` and attempted `git merge rust-v0.143.0-alpha.25 --no-edit`. The merge was intentionally aborted after conflict assessment because the conflict surface was broad: app-server protocol schemas, app-server request processing, core session/compaction state, rollout, exec, TUI plugin/status/resize surfaces, snapshots, and docs.

`HEAD..rust-v0.143.0-alpha.25` is 418 upstream commits while VSP has 38 local commits after that ancestor path. `HEAD..upstream/main` touches about 2397 files. A complete fork rebase/merge to current upstream should be a separate larger upgrade Cycle, not a narrow hotfix.

## Technical Change

Applied the relevant official persistent-log filtering behavior locally while preserving VSP version metadata as `0.140.0-vsp.8`:

- Added `codex_state::log_db::default_filter()` with:
  - default TRACE retention for normal targets,
  - `log` disabled,
  - `codex_otel.log_only` disabled,
  - `codex_otel.trace_safe` disabled.
- Applied this filter at both app-server and TUI tracing initialization points before attaching the SQLite log layer.
- Added an in-layer guard in `LogDbLayer::on_event()` that drops `metadata.target() == "log"` even if an outer filter is bypassed by `tracing-log` bridge behavior.
- Added tests proving:
  - the default filter disables the known persistent-log bridge targets,
  - low-level `opentelemetry_sdk` logs remain dropped,
  - `target="log"` bridge events are dropped even when an outer permissive filter is installed.

Modified files:

- `codex-rs/state/src/log_db.rs`
- `codex-rs/state/src/log_db_filter_tests.rs`
- `codex-rs/app-server/src/lib.rs`
- `codex-rs/tui/src/lib.rs`

No package version was changed. Local VSP remains `0.140.0-vsp.8`.

## Validation

Passed:

- `cargo fmt --manifest-path codex-rs/Cargo.toml --all --check`
- `cargo test --manifest-path codex-rs/Cargo.toml -p codex-state filter -- --nocapture`
  - 8 filtered tests passed, including the 3 log-filter tests.
- `cargo check --manifest-path codex-rs/Cargo.toml -p codex-app-server -p codex-tui`
  - Passed with existing dead-code warnings in `codex-core` and `codex-tui`.
- `git diff --check`

Attempted but stopped:

- A nonessential runtime smoke with temporary `CODEX_HOME` and `cargo run -p codex-cli --bin vsp-codex -- --help` was stopped after it spent over a minute compiling. No temp process or temp directory remained. The behavior is covered by unit tests and crate checks.

## Risks And Follow-Up

- Existing `logs_2.sqlite` and WAL files already contain large historical rows. The code fix prevents the high-volume targets from being persisted going forward, but it does not shrink existing SQLite files. A follow-up maintenance step should safely checkpoint, vacuum, or rotate `~/.vsp-codex/logs_2.sqlite` and `~/.codex/logs_2.sqlite` after ensuring no running Codex process is using them.
- Already-running VSCode/plugin/app-server processes must be restarted after a rebuilt/deployed binary is available; otherwise they will keep using the old log behavior.
- Full upstream sync to `rust-v0.143.0-alpha.25` remains unresolved and should be planned separately because the full merge conflict surface is large.
