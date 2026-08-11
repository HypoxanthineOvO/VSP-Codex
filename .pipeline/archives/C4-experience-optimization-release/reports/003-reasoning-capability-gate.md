# M3 Reasoning Capability Gate Report

## Outcome

Implemented the focused runtime fix for TUI transcript conversion. Empty or unsupported reasoning items no longer create a Thinking/Reasoning transcript block or fallback transcript output.

## Files Changed By Implement Worker

- `codex-rs/tui/src/resume_picker/transcript.rs`
- `.pipeline/reviews/C4/M3/implement.md`
- `.pipeline/reports/003-reasoning-capability-gate.md`

## Capability Status

- Real app-server v2 reasoning summary/raw notification mappings are covered by the test worker and pass locally.
- TUI transcript conversion preserves real summary rendering when raw is hidden.
- TUI transcript conversion preserves real raw rendering when raw is visible.
- Empty reasoning items are skipped and do not synthesize fake Thinking output.
- Tool/progress fallback transcript items remain separate fallback cells and are not converted into reasoning.

## Config Compatibility

Current config does not yet provide a dedicated `off / summary / raw` visibility enum. Compatibility remains based on existing fields:

- `show_raw_agent_reasoning = true` maps to raw-visible behavior.
- `show_raw_agent_reasoning = false` or unset maps to summary-visible/raw-hidden behavior.
- Existing `hide_agent_reasoning` remains the suppression path for agent reasoning output.

No `ConfigToml` or schema change was required for the M3 red test.

## Validation

- `cd codex-rs && just fmt` passed.
- `cd codex-rs && just test -p codex-tui reasoning` passed: 59 passed, 2623 skipped.
- `cd codex-rs && just test -p codex-app-server-protocol reasoning` passed: 5 passed, 219 skipped.
- `cd codex-rs && just test -p codex-core resume_replays_reasoning_summary_without_raw_when_raw_reasoning_disabled` passed: 1 passed, 2567 skipped.
- `cd codex-rs && just fix -p codex-tui` passed.

## Known Residuals

- Broader `just test -p codex-core reasoning` was not rerun by this worker to avoid known unrelated timeouts already recorded by the test worker:
  - `suite::remote_models::remote_models_long_model_slug_is_sent_with_high_reasoning`
  - `suite::compact::auto_compact_runs_when_reasoning_header_clears_between_turns`
- No snapshot updates were made by this worker.
