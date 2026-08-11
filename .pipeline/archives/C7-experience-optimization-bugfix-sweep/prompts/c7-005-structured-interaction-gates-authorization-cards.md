# M5 Structured Interaction Gates And Authorization Cards

## Objective

Make required Workflow decisions, confirmations, and authorization-like prompts use structured `request_user_input` cards with clear TUI rendering instead of ordinary chat text.

## Source Of Truth

- P1 issue: Issue 10 structured Request Input and confirmation/authorization cards.
- P2 technical solution: define a Workflow interaction-gate contract; required user decisions use `request_user_input`; true permission expansion uses `request_permissions` or approval overlays.
- Local evidence: `codex-rs/protocol/src/request_user_input.rs`, `codex-rs/protocol/src/request_permissions.rs`, `codex-rs/tui/src/bottom_pane/request_user_input/`, `codex-rs/tui/src/bottom_pane/approval_overlay.rs`, and `codex-rs/core/src/workflow_routing.rs`.

## Card Types To Support

At minimum, implement or codify these card templates:

1. Plan confirmation card: approve the generated plan or request revision before P3/P4 transitions.
2. Phase transition card: continue, stop, or revise when moving between Workflow phases such as P2 -> P3 or P4 -> execution.
3. Scope selection card: choose one or more implementation scopes, release channels, test levels, or affected surfaces.
4. Revision/rejection card: reject, revise, provide feedback, or cancel without burying the decision in chat prose.
5. Authorization-intent card: confirm user intent for high-impact Workflow choices while still routing real permission grants to `request_permissions` or approval overlays.
6. Release/update confirmation card: update now, skip this time, skip this version, view notes, or cancel.

The implementation may add more types, but it must keep these semantic categories distinct. A confirmation card is not a permission grant; real sandbox/network/destructive/external approvals remain controlled by permission/approval mechanisms.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns failing tests, fixtures, assertions, and snapshots for Workflow guidance, Request Input confirmation cards, authorization wording, zh-CN localization, narrow terminal layout, and no-card ordinary chat behavior.
  - Write boundary: test/snapshot files and `.pipeline/reviews/C7/M5/test.md` only.
  - Expected artifact: `.pipeline/reviews/C7/M5/test.md`.
- `implement`
  - Scope: owns production prompt/runtime guidance, Request User Input card templates/rendering polish, Workflow gate routing, and any helper APIs needed by later Plan/Updater milestones.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C7/M5/implement.md`.
- `audit`
  - Scope: read-only review of approval bypass risk, card semantics, localization, prompt/tool guidance, and ordinary chat non-regression.
  - Must reject changes that represent true permission grants as ordinary confirmation cards.
  - Expected artifact: `.pipeline/reviews/C7/M5/audit.md`.

Workers must not revert user or other-worker edits. Required user decisions must use `request_user_input` when available. Destructive operations, system-level dependency installation, network/remote side effects, and release publication require explicit confirmation/authorization.

## Technical Solution

Define a small Workflow interaction-gate contract:

- Informational status: no answer required; render as status/prose.
- Optional clarification: may remain conversational unless a structured answer would materially improve safety or UX.
- Required decision: use `request_user_input` when the tool is available.
- Permission/authorization: use `request_permissions` or the existing approval overlay for real grants; `request_user_input` may confirm intent but must not bypass enforcement.

Add reusable card templates with concise summaries, clear descriptions, explicit options, cancellation/rejection paths, selected/default state handling, and zh-CN wording. Update Workflow runtime guidance so required confirmations cannot be asked only in assistant prose when `request_user_input` is available.

## Technical Route

1. Inspect the existing `request_user_input` protocol and TUI overlay, including queueing, question kinds, secrets, options, and snapshots.
2. Inspect `request_permissions` and approval overlay behavior to define the boundary between intent confirmation and permission grant.
3. Add Workflow interaction-gate classification in runtime/prompt guidance and any helper metadata used by Workflow routing.
4. Add or document card templates for plan confirmation, phase transition, scope selection, revision/rejection, authorization-intent, and release/update confirmation.
5. Polish Request User Input card rendering where needed: title, summary, description, option labels, selected/default states, cancellation, narrow-terminal wrapping, and zh-CN copy.
6. Add tests/snapshots proving Workflow gates are represented as structured cards and ordinary chat is not over-questioned.

Non-goals: replacing exec/patch approval overlays; weakening sandbox/network/destructive/external side-effect approvals.

## Research Required

- Resolved by local evidence: `request_user_input`, TUI overlay, zh-CN snapshots, `request_permissions`, approval overlay, and Workflow routing guidance all exist in the repo.
- No external service research required.
- Implementation-time design remains for exact card copy and visual affordances.

## Risks And Alternatives

- Risk: cards are overused and make normal chat bureaucratic. Mitigation: strict card requirement only applies to required Workflow/high-impact decisions.
- Risk: users confuse confirmation with permission grants. Mitigation: card copy and routing must clearly distinguish intent confirmation from actual permission authorization.
- Risk: the model still asks plain-text confirmations. Mitigation: update Workflow guidance, add tests around routing/guidance text, and ensure P3/P4 prompts require `request_user_input` for gates.
- Rejected alternative: rely only on assistant wording. That is not testable and does not guarantee consistent TUI cards.

## Validation Path

Required validation:

- `cargo test -p codex-core request_user_input workflow_routing -- --nocapture` with new guidance/fixture cases.
- `cargo test -p codex-tui request_user_input approval_overlay -- --nocapture` with confirmation/authorization zh-CN snapshots.
- Snapshot: plan confirmation, release-update confirmation, and scope-choice gates render as cards with explicit choices rather than assistant-only prose.
- `cargo fmt --check`.
- `git diff --check`.

Manual acceptance:

- Run a P2/P4 confirmation and a release/update prompt.
- Verify the user answers through a card and can reject, revise, cancel, or skip clearly.

Pass signal: required Workflow decisions render as `request_user_input` cards with explicit choices; the six required card types are covered; confirmation/authorization-intent/real permission grant language remains distinct.

Fail signal: required gates are asked only in assistant prose, true permissions are represented as ordinary confirmation cards, ordinary chats are over-questioned, or zh-CN card snapshots regress.

## Audit Focus

- Verify no approval or permission enforcement is bypassed.
- Verify required Workflow decisions use structured cards when available.
- Verify card language clearly separates confirmation, authorization intent, permission grant, and ordinary clarification.
- Verify ordinary non-Workflow chats are not forced into unnecessary cards.
