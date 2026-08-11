# GitHub Release Draft: v0.134.0-vsp.6

Status: draft only

Do not publish this GitHub release without a separate explicit confirmation.

## Title

VSP-Codex v0.134.0-vsp.6

## Draft Notes

This release prepares the C8 VSCode Remote SSH and plugin compatibility work for external presentation.

### Highlights

- Remote SSH VSCode integration now prioritizes remote VSCode server Machine settings for `vsp-codex vscode` status/install/rollback.
- VSCode plugin resume compatibility now normalizes empty provider/model metadata for VSP-authored sessions.
- Official Codex session discovery has a sanitized export path that avoids raw VSP session sharing and strips private provider/auth/Workflow fields.
- Request Input supports explicit Workflow metadata and readable Markdown fallback output for clients without native Ask UI support.
- Workflow Markdown surface wrapping is wired into live app-server plugin-facing fallback output.

### Validation

- `cargo test -p codex-app-server --test all request_user_input -- --nocapture`
- `cargo test -p codex-app-server --test all workflow_markdown_surface -- --nocapture`
- `cargo test -p codex-app-server --test all thread_resume -- --nocapture`
- `cargo test -p codex-rollout --test official_codex_discovery`
- `cargo test -p codex-cli --test vscode`
- package smoke for Linux x64 GNU/glibc debug preview artifact

### Artifact Note

The prepared local/GitLab artifact is a Linux x64 GNU/glibc debug preview package. A public GitHub release should decide whether to publish this preview artifact, build a release-profile artifact, or wait for a broader target matrix.
