verdict: pass

# C4/M3 Audit Review

## reviewed_refs

- `.pipeline/prompts/003-reasoning-capability-gate.md`
- `.pipeline/reviews/C4/M3/test.md`
- `.pipeline/reviews/C4/M3/implement.md`
- `.pipeline/reports/003-reasoning-capability-gate.md`
- `codex-rs/app-server-protocol/src/protocol/event_mapping.rs`
- `codex-rs/core/tests/suite/resume.rs`
- `codex-rs/tui/src/resume_picker.rs`
- `codex-rs/tui/src/resume_picker/transcript.rs`
- `codex-rs/protocol/src/items.rs`
- `codex-rs/core/src/session/turn.rs`
- `codex-rs/tui/src/chatwidget/replay.rs`
- `codex-rs/tui/src/chatwidget/protocol.rs`
- `codex-rs/config/src/config_toml.rs`
- `codex-rs/core/src/config/mod.rs`

## findings

- Pass: no fake reasoning path found in the reviewed runtime. TUI transcript conversion only creates a reasoning cell from `ThreadItem::Reasoning`; tool/progress/fallback items stay in `fallback_transcript_cell` and are rendered as plain tool/status cells, not Thinking/Reasoning. Empty reasoning content is skipped and no generic fallback transcript cell is added when the transcript only contains skipped empty reasoning.
  Evidence: `codex-rs/tui/src/resume_picker/transcript.rs:82`, `codex-rs/tui/src/resume_picker/transcript.rs:92`, `codex-rs/tui/src/resume_picker/transcript.rs:99`, `codex-rs/tui/src/resume_picker/transcript.rs:103`, `codex-rs/tui/src/resume_picker/transcript.rs:110`.

- Pass: app-server v2 reasoning mapping remains the VSCode compatibility path. Core `EventMsg::ReasoningContentDelta`, `EventMsg::AgentReasoningSectionBreak`, and `EventMsg::ReasoningRawContentDelta` map directly to v2 `ReasoningSummaryTextDelta`, `ReasoningSummaryPartAdded`, and `ReasoningTextDelta` notifications. Tests cover thread/turn/item ids plus summary/content indices.
  Evidence: `codex-rs/app-server-protocol/src/protocol/event_mapping.rs:361`, `codex-rs/app-server-protocol/src/protocol/event_mapping.rs:370`, `codex-rs/app-server-protocol/src/protocol/event_mapping.rs:379`, `codex-rs/app-server-protocol/src/protocol/event_mapping.rs:527`, `codex-rs/app-server-protocol/src/protocol/event_mapping.rs:553`, `codex-rs/app-server-protocol/src/protocol/event_mapping.rs:575`.

- Pass: raw/summary visibility is compatible with legacy `show_raw_agent_reasoning`. Stored reasoning summary is always replayed through legacy events; raw content is replayed only when `show_raw_agent_reasoning` is true. Resume picker mirrors that behavior by mapping true to raw-visible and false/unset to summary-visible/raw-hidden.
  Evidence: `codex-rs/protocol/src/items.rs:457`, `codex-rs/protocol/src/items.rs:466`, `codex-rs/tui/src/resume_picker.rs:506`, `codex-rs/tui/src/resume_picker.rs:5886`, `codex-rs/tui/src/resume_picker.rs:5943`, `codex-rs/tui/src/resume_picker.rs:6045`, `codex-rs/core/tests/suite/resume.rs:239`, `codex-rs/core/tests/suite/resume.rs:304`.

- Pass with residual: full `off / summary / raw` config enum was not implemented. This is acceptable for M3 because the implemented behavior preserves the old compatibility contract: raw can be enabled, summary remains visible when raw is disabled, and the existing `hide_agent_reasoning` suppression path was not changed. No `ConfigToml` or schema mutation was needed for the M3 red test.
  Evidence: `.pipeline/reviews/C4/M3/implement.md:22`, `.pipeline/reviews/C4/M3/implement.md:46`, `codex-rs/config/src/config_toml.rs:326`, `codex-rs/config/src/config_toml.rs:331`, `codex-rs/core/src/config/mod.rs:3496`.

## validation

- Accepted implement worker evidence:
  - `cd codex-rs && just fmt` passed.
  - `cd codex-rs && just test -p codex-tui reasoning` passed: 59 passed, 2623 skipped.
  - `cd codex-rs && just test -p codex-app-server-protocol reasoning` passed: 5 passed, 219 skipped.
  - `cd codex-rs && just test -p codex-core resume_replays_reasoning_summary_without_raw_when_raw_reasoning_disabled` passed: 1 passed, 2567 skipped.
  - `cd codex-rs && just fix -p codex-tui` passed.

- Test worker evidence was sufficient for the broader initial scan. The two broader `just test -p codex-core reasoning` timeouts were recorded before implementation as existing failures outside the new M3 coverage:
  - `suite::remote_models::remote_models_long_model_slug_is_sent_with_high_reasoning`
  - `suite::compact::auto_compact_runs_when_reasoning_header_clears_between_turns`

- The M3-specific red signal from the test worker, `resume_picker::tests::thread_to_transcript_cells_skips_empty_reasoning_items`, was addressed by the implement worker and reported passing in the scoped TUI reasoning run.

## residual risks

- No snapshot updates were made. The current M3 surface is covered by assertion-based transcript conversion tests rather than rendered snapshot tests, which is acceptable for this focused transcript behavior but weaker for visual review of a future collapsed Thinking UI.

- The dedicated `off / summary / raw` config enum remains unimplemented. Treat this as a follow-up product/API cleanup unless a later milestone requires an explicit single-field visibility API.

- This audit did not rerun tests independently because this worker is constrained to read-only review plus writing this audit artifact. Validation relies on the test and implement worker reports plus source inspection.

## scope boundary

- Worker separation was respected in this audit: no code, tests, reports, schemas, or generated artifacts were modified.
- The only file written by this worker is `.pipeline/reviews/C4/M3/audit.md`.
- `ConfigToml` and `codex-rs/core/config.schema.json` were not modified by M3 implementation; no schema regeneration was required for the reviewed change.
