# M3 Provider And Resume Compatibility Contract

## Objective

Fix and document the provider/session resume contract so the VSCode plugin can resume VSP-Codex sessions without failing on `Model provider `` not found`, while keeping VSP provider metadata private where needed.

## Source Of Truth

- User symptom: Remote SSH VSCode plugin resume fails with `failed to load configuration: Model provider `` not found`.
- Known code: `codex-rs/rollout/src/recorder.rs` intentionally omits VSP provider from persisted session metadata; test `vsp_provider_is_omitted_from_persisted_session_meta` covers that behavior.
- P2 technical solution: missing/empty provider should fallback to current request or VSP default provider; unknown non-empty provider remains diagnostic error.

## Subworker Assignment Plan

Status: authorized, strict worker separation.

- `test`
  - Scope: owns red tests for app-server resume/list/read/fork provider handling and recorder metadata fixtures.
  - Write boundary: tests/fixtures and `.pipeline/reviews/C8/M3/test.md` only.
- `implement`
  - Scope: owns production provider normalization/resolution and minimal diagnostic improvements.
  - Must not create, edit, rewrite, or satisfy tests owned by `test`.
  - Expected artifact: `.pipeline/reviews/C8/M3/implement.md`.
- `audit`
  - Scope: read-only review of provider privacy, resume behavior, unknown-provider errors, and plugin-facing diagnostics.
  - Expected artifact: `.pipeline/reviews/C8/M3/audit.md`.

Workers are not alone in the codebase. Do not revert user edits or another worker's edits; adapt around them and stop on scope conflicts.

Required user decisions must use `request_user_input` when available. Do not expose private provider/auth metadata to official Codex or plugin logs unless explicitly justified and confirmed.

## Technical Solution

Normalize provider metadata at app-server resume boundaries. Treat absent provider and empty-string provider as unset. For VSP sessions whose persisted metadata omits the provider, resume should resolve a legal provider from current request overrides, configured VSP defaults, or a documented fallback. Unknown non-empty provider names should still fail with a clear diagnostic. Persisted session metadata should continue avoiding VSP-private provider details unless a later milestone explicitly changes the security model.

## Technical Route

1. Inspect app-server resume/list/read/fork code paths and config-loading boundaries.
2. Inspect rollout recorder metadata persistence and state-db/session metadata read path.
3. Add tests proving empty provider does not call config loading with `""`.
4. Add tests proving omitted VSP provider resumes through a legal fallback without leaking private provider metadata.
5. Add tests proving unknown non-empty provider still errors clearly.
6. Implement a shared provider normalization/resolution helper at the boundary where app-server constructs resume config.
7. Update diagnostics so plugin-visible errors name missing provider resolution, not an empty provider id.

Non-goals: forcing VSP provider into public session metadata, changing auth storage, or solving official Codex discovery before M4.

## Research Required

- Local code evidence is enough to start.
- Implementation-time research remains for the exact app-server method that passes provider into config loading.

## Risks And Alternatives

- Risk: fallback provider could resume with the wrong provider. Mitigation: use current request/configured VSP default first and test precedence.
- Risk: exposing provider metadata would improve compatibility but leak VSP-private implementation details. Mitigation: keep metadata private and fix boundary normalization.
- Alternative: persist provider unconditionally. Rejected unless M4 proves it is safe and necessary.

## Validation Path

Required validation:

- Focused app-server provider/resume tests.
- Focused rollout recorder metadata tests.
- `cargo fmt --check`
- `git diff --check`

Manual acceptance:

- In Remote SSH VSCode, resume a VSP-Codex session that previously failed with `Model provider `` not found`.

Pass signal: missing/empty provider resumes through a legal fallback, unknown non-empty provider still errors clearly, and private provider metadata is not leaked.

Fail signal: empty provider still reaches config loader or the fix requires public provider persistence without explicit approval.

## Audit Focus

- Verify the compatibility contract is implemented at the boundary, not as scattered string patches.
- Verify private VSP provider/auth data remains protected.
- Verify plugin-visible diagnostics are actionable.

