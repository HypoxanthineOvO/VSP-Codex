# C8/M6 Audit Worker Report

Verdict: `needs_changes`

## Findings

### Critical

1. M6 helper is not wired into live plugin thread emission, so the plugin path is not yet coherent.
   - Evidence: the M6 objective requires making Workflow output readable in the VSCode plugin and completing the Remote SSH plugin acceptance path, including resume, Request Input, Markdown surfaces, and Subagent prompt/report inspection (`.pipeline/prompts/c8-006-markdown-workflow-surface-remote-ssh-acceptance.md:3`-`12`, `:78`-`:86`).
   - Evidence: the new renderer is only exported and tested. `rg` finds production references only at `codex-rs/app-server/src/workflow_markdown_surface.rs:2` and `:43`, plus module export at `codex-rs/app-server/src/lib.rs:105`; no thread emission path constructs or calls `render_workflow_markdown_surface`.
   - Evidence: the implement report explicitly records this residual risk: "It does not yet wire the helper into a live plugin thread emission path" (`.pipeline/reviews/C8/M6/implement.md:69`-`72`).
   - Impact: focused helper tests can pass while real Remote SSH plugin users still do not see Workflow/Subagent Markdown through the app-server thread. Given the M6 objective says the plugin path should be coherent, this is blocking for M6 acceptance.

### Warning

1. Remote SSH acceptance is documented as a future checklist, not completed acceptance evidence.
   - Evidence: the prompt requires final C8 acceptance to include real Remote SSH plugin resume/interaction (`.pipeline/prompts/c8-006-markdown-workflow-surface-remote-ssh-acceptance.md:12`, `:78`-`:82`, `:90`).
   - Evidence: `.pipeline/reviews/C8/M6/acceptance-checklist.md` is honest about being a checklist and asks the acceptance owner to record command output, visible headings, prompt/report labels, and exact progress text (`:1`-`:36`), but it contains no recorded observed results yet.
   - Impact: this is acceptable as a checklist artifact, but not sufficient to claim final manual acceptance has passed.

### Info

1. The standalone Markdown helper satisfies the covered M6 rendering contract.
   - Evidence: it renders a top-level Markdown heading and summary (`codex-rs/app-server/src/workflow_markdown_surface.rs:43`-`50`), separate Plan/SubPlan counts (`:73`-`:88`), Subagent labels for `Prompt Original`, `Report Original`, and `Derived Translation` (`:90`-`:110`), concise tool output with command, excerpt, and omitted-line count (`:113`-`:137`), and appends the M5 request input fallback Markdown verbatim (`:64`-`:68`).
   - Evidence: tests assert readable headings, paths, command/code fence preservation, request input fallback inclusion, and separate progress text (`codex-rs/app-server/tests/suite/v2/workflow_markdown_surface.rs:69`-`87`, `:112`-`:139`).

2. The helper avoids the tested raw protocol/debug clutter and duplicate translation markers.
   - Evidence: the focused test rejects UUID/protocol/debug markers, `raw_transcript`, duplicated translation labels, and mixed count examples (`codex-rs/app-server/tests/suite/v2/workflow_markdown_surface.rs:94`-`109`).
   - Evidence: `strip_duplicate_heading` removes a duplicate same-label `####` heading before appending labeled Markdown (`codex-rs/app-server/src/workflow_markdown_surface.rs:140`-`158`).

3. M5 request input fallback inclusion remains compatible with M6.
   - Evidence: M5 fallback emits `## Workflow Decision`, native Ask fallback guidance, `**Answer controls**`, explicit Workflow decision labels, and authorization-intent safety wording when applicable (`codex-rs/app-server/src/request_user_input_fallback.rs:13`-`60`).
   - Evidence: recovered pending ask fallback emits a `ThreadItem::AgentMessage` containing `fallback_prompt(&params)` for clients without native Ask UI (`codex-rs/app-server/src/request_processors/thread_lifecycle.rs:809`-`815`).

4. The Remote SSH manual checklist is honest about scope and avoids unsupported claims.
   - Evidence: it explicitly says it does not claim release, publish, tag, VSIX packaging, or plugin source changes (`.pipeline/reviews/C8/M6/acceptance-checklist.md:1`-`4`) and tells the validator not to modify installed extension source (`:5`-`:10`).

5. Worker separation and validation evidence are sufficient for the helper-level work.
   - Evidence: M6 prompt defines strict `test`, `implement`, and `audit` boundaries (`.pipeline/prompts/c8-006-markdown-workflow-surface-remote-ssh-acceptance.md:14`-`31`).
   - Evidence: test worker owned tests/checklist/report and recorded initial red coverage (`.pipeline/reviews/C8/M6/test.md:1`-`31`); implement worker owned production helper/export and report only (`.pipeline/reviews/C8/M6/implement.md:5`-`25`).
   - Evidence: main-agent validation reported focused app-server Markdown surface tests, request input tests, `cargo fmt --check --package codex-app-server`, and scoped `git diff --check` all passing.

## Validation Reviewed

- `cargo test -p codex-app-server --test all workflow_markdown_surface -- --nocapture`: reported 2 passed.
- `cargo test -p codex-app-server --test all request_user_input -- --nocapture`: reported 11 passed.
- `cargo fmt --check --package codex-app-server`: reported passed with existing stable rustfmt warning.
- `git diff --check -- ...`: reported passed.

## Required Follow-Up

Wire `render_workflow_markdown_surface` or equivalent M6 formatting into the app-server/plugin-facing thread emission path for Workflow/Subagent output, then add or extend a test that observes the emitted plugin thread item rather than only calling the helper directly. After that, complete the Remote SSH manual acceptance checklist with real observed evidence.

---

# C8/M6 Audit Recheck

Verdict: `pass`

## Findings

No blocking findings remain.

## Evidence

- Live unsupported native Ask fallback now calls `workflow_surface_fallback_prompt(params)` before emitting `ThreadItem::AgentMessage` in `codex-rs/app-server/src/bespoke_event_handling.rs`.
- Recovered pending unsupported Ask fallback replay uses the same wrapper in `codex-rs/app-server/src/request_processors/thread_lifecycle.rs`.
- The wrapper preserves the M5 fallback body by rendering `render_request_input_workflow_markdown_surface(fallback_prompt(params))` in `codex-rs/app-server/src/request_user_input_fallback.rs`.
- The live emission test observes the plugin-facing `agentMessage` text and requires `## Workflow Surface`, `### Request Input`, the M5 fallback content, answer controls, and authorization-intent safety copy.

## Commands Reviewed

```bash
cargo test -p codex-app-server --test all workflow_required_decisions_without_native_ask_ui_render_polished_markdown_cards -- --nocapture
cargo test -p codex-app-server --test all request_user_input -- --nocapture
cargo test -p codex-app-server --test all workflow_markdown_surface -- --nocapture
cargo fmt --check --package codex-app-server
git diff --check -- codex-rs/app-server/src/bespoke_event_handling.rs codex-rs/app-server/src/request_processors/thread_lifecycle.rs codex-rs/app-server/src/request_user_input_fallback.rs codex-rs/app-server/src/workflow_markdown_surface.rs .pipeline/reviews/C8/M6/implement.md
```

Result: all passed. The only noted warnings were existing stable rustfmt `imports_granularity = Item` warnings.

## Scope Notes

- `.pipeline/reviews/C8/M6/acceptance-checklist.md` is a manual validation checklist, not a claim that Remote SSH manual acceptance has already completed.
- `.pipeline/reviews/C8/M6/implement.md` explicitly keeps Remote SSH manual acceptance as pending and separates the audit-fix implementation from the initial helper implementation.
- Worker separation evidence is adequate for test, implement, and audit-fix review coverage.
