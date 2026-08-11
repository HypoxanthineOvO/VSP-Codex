# M4 Session Discovery Strategy For Official Codex

## Objective

Define and implement the safest practical strategy for official Codex to discover or import VSP-Codex sessions, without leaking VSP Workflow internals, provider secrets, or private runtime metadata.

## Source Of Truth

- User issue: official Codex cannot find VSP-Codex sessions.
- User accepted research-first depth: preserve Workflow metadata if safe; strip, summarize, or private-gate unsafe VSP metadata.
- P2 technical solution: choose from explicit export/import, sanitized mirror/index, or controlled shared discovery after field-level risk classification.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns fixture sessions, safe/unsafe field classification tests, mirror/import assertions, and `.pipeline/reviews/C8/M4/test.md`.
  - Write boundary: tests/fixtures/review artifact only.
- `implement`
  - Scope: owns production/session discovery strategy implementation or documented minimal safe path if implementation is deferred by evidence.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C8/M4/implement.md`.
- `audit`
  - Scope: read-only privacy/security review of discovery strategy, stripped fields, official Codex compatibility claims, and migration risk.
  - Expected artifact: `.pipeline/reviews/C8/M4/audit.md`.

Workers are not alone in the codebase. Do not revert user edits or another worker's edits; adapt around them and stop on scope conflicts.

Required user decisions must use `request_user_input` when available. Any broad sharing of `~/.vsp-codex` into `~/.codex`, external publication, or secret-adjacent metadata exposure requires explicit confirmation.

## Technical Solution

First classify VSP session fields into safe, derived, private, and secret categories. Then implement or prepare the least risky compatibility path. Preferred default is a sanitized mirror/index or explicit export/import that lets official Codex see resumable conversation metadata without direct access to `.pipeline` runtime, provider/auth internals, or sensitive Workflow records. Keep native VSP Workflow continuity intact.

## Technical Route

1. Inspect VSP and official Codex session root defaults, session file layout, and current sharing/symlink helpers.
2. Build a field-level risk table for session metadata, rollout metadata, Workflow metadata, provider/auth fields, and conversation content.
3. Add tests with representative VSP session fixtures proving unsafe fields are stripped or summarized.
4. Choose one default strategy:
   - explicit export/import
   - sanitized mirror/index
   - config-gated shared discovery
5. Implement the smallest strategy that satisfies discovery without broad private sharing.
6. Add diagnostics explaining where official Codex should look and which fields are intentionally omitted.
7. If implementation cannot be safe within C8, produce a concrete deferred design with tests for the sanitizer layer.

Non-goals: copying all `.pipeline` runtime into official Codex, exposing VSP provider/auth data, or changing official Codex behavior outside this repository without confirmation.

## Research Required

- Must inspect current VSP CODEX_HOME behavior and official `~/.codex` asset sharing.
- Must identify exactly which session fields official Codex requires for listing/resume.

## Risks And Alternatives

- Risk: sanitized sessions list but cannot resume. Mitigation: separate discovery from resume and document required missing fields.
- Risk: too much stripping destroys useful Workflow continuity. Mitigation: keep Workflow summaries or references only when safe.
- Risk: symlink-based sharing leaks private files. Mitigation: prefer sanitized mirror/export over raw directory sharing.

## Validation Path

Required validation:

- Safe/unsafe field tests or sanitizer fixture tests.
- Discovery strategy report with chosen path and rejected alternatives.
- `cargo fmt --check` if production Rust changes occur.
- `git diff --check`

Manual acceptance:

- Official Codex can discover/import at least one VSP session in the selected safe mode, or the report clearly explains the blocking plugin/official-client limitation.

Pass signal: discovery improves without private metadata leakage.

Fail signal: the implementation relies on raw sharing of VSP private runtime or makes unsupported assumptions about official Codex.

## Audit Focus

- Verify privacy classification is field-level, not hand-wavy.
- Verify no secret/provider/auth data leaks into official Codex-visible files.
- Verify Workflow continuity is preserved for VSP.

