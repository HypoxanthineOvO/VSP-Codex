# v0.145.0-vsp.9.4

**Hotfix**: stop `codex_http_client::transport` TRACE bodies from bloating `logs_2.sqlite`

## What changed

- Filter `codex_http_client::transport` target at `WARN` level in the sqlite log sink, preventing full HTTP request/response bodies (up to 7 MB each) from being written as TRACE events.
- Truncate `feedback_log_body` column to 64 KB as a defence-in-depth measure against any future large-payload target.
- Fix pre-existing test compile error in `threads.rs` (`history_mode` field).

## Context

Upstream bug cluster: openai/codex#28224, #24275, #17320, #22444, #20213.
The per-partition retention caps (`LOG_PARTITION_SIZE_LIMIT_BYTES=10MB`, `LOG_PARTITION_ROW_LIMIT=1000`) are scoped per `thread_id`/`process_uuid`, so many concurrent sessions with large HTTP bodies still inflated total `logs_2.sqlite` size past 1 GB, causing backfill lease stalls and complete UI freezes.

## Affected files

- `codex-rs/state/src/log_db.rs` — transport filter + body truncation
- `codex-rs/state/src/log_db_filter_tests.rs` — tests for both
- `codex-rs/state/src/runtime/threads.rs` — test compile fix

## Upgrade notes

After updating, existing bloated `logs_2.sqlite` files will not shrink automatically.
Run `sqlite3 ~/.codex/logs_2.sqlite 'DELETE FROM logs; VACUUM;'` (or `~/.vsp-codex/logs_2.sqlite`) to reclaim disk space immediately.
