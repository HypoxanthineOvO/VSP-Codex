# M3 TUI Ask 多题型与隐私展示

## Objective

扩展现有 bottom pane `request_user_input` overlay，原生支持多选、短答/自由文本、order-only ranking、request/question 说明和 secret 圆点掩码。

## Source Of Truth

- P2: `.pipeline/plan-state/decompose.yaml` -> `M3`
- Depends on M1/M2 answer and question shapes.
- Key files:
  - `codex-rs/tui/src/bottom_pane/request_user_input/`
  - `codex-rs/tui/src/history_cell/request_user_input.rs`
  - `codex-rs/tui/src/app/app_server_requests.rs`

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Owns TUI state-machine tests and insta snapshots.
  - Must cover request summary, per-question description, multi-select, short/free text, ranking, secret masking, and narrow wrapping.
  - Must verify visible and raw history output for secret masking.
- `implement`
  - Owns overlay state/render/submit changes and history rendering updates.
  - Must keep implementation in request_user_input/bottom pane modules, not central `chatwidget.rs`.
- `audit`
  - Checks UI layout, keyboard ergonomics, secret leakage, and answer shape integrity.
  - Must reject if ranking adds per-item notes or if secret appears in snapshot/raw lines.

## Technical Solution

Introduce question-kind specific answer state:

- single choice: existing highlighted selection.
- multi choice: selected option-id set.
- short/free text: composer draft.
- ranking: ordered option ids.

Render request-level summary and optional per-question description above the answer module. Secret values are masked with bullet characters in all user-visible history/transcript/log output.

## Technical Route

1. Refactor `AnswerState`.
   - Replace single selected index assumption with kind-specific state.
   - Keep queue, multi-question navigation, and unanswered confirmation.
2. Extend input handling.
   - Multi-select toggle.
   - Ranking reorder.
   - Free/short text composer commit.
3. Extend render/layout.
   - Show request summary.
   - Show per-question description.
   - Add kind-specific footer hints.
   - Preserve tight/narrow layout constraints.
4. Update submit path.
   - Emit structured response.
   - Preserve legacy answer strings where required by compatibility.
5. Update `RequestUserInputResultCell`.
   - Secret/private answers render as bullets in display and raw lines.
6. Add/update snapshots.

## Research Status

Resolved. Existing overlay has:

- `AnswerState` with selection and draft.
- queueing and multi-question behavior.
- `submit_answers` building `ToolRequestUserInputResponse`.
- snapshot coverage under `bottom_pane/request_user_input/snapshots`.

## Risks And Alternatives

- Risk: ranking keybindings conflict with list navigation. Mitigation: explicit footer hints and tests.
- Risk: multi-select/free text crowds narrow terminals. Mitigation: kind-specific layout and wrapping.
- Risk: secret leaks through raw history. Mitigation: test both display and raw paths.

## Validation Path

- `just test -p codex-tui`
- `cargo insta pending-snapshots -p codex-tui`

Pass signal:

- New Ask kinds render and submit correctly.
- Ranking returns stable order-only answer.
- Secret/private answers are masked in all user-visible outputs.

## Completion Criteria

- TUI supports all C3 Ask kinds.
- Snapshots are reviewed and accepted if intended.
- M3 report is written to `.pipeline/reports/003-tui-ask-multikind-privacy.md`.
