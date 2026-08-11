# C8/M6 Remote SSH Acceptance Checklist

Purpose: record the manual validation expected after implementation lands. This checklist does not claim release, publish, tag, VSIX packaging, or plugin source changes.

## Preconditions

- Run from a VSCode Remote SSH window whose extension host is under `~/.vscode-server`.
- Use the installed `openai.chatgpt` extension app-server path captured in M2.
- Confirm the current repository/session is `/home/heyx/Codex-VSP`.
- Do not modify installed extension source while collecting evidence.

## Commands And Observations

1. `vsp-codex vscode status`
   - Expected: reports Remote SSH server Machine settings when present, matching M1 settings priority.
   - Record: command output scope/path and whether the plugin app-server command is installed.

2. Resume a VSP session from the VSCode plugin.
   - Expected: session loads through the app-server protocol without provider metadata failure, matching M3.
   - Record: visible session title/id prefix, provider/resume error absence, and relevant app-server log path.

3. Trigger a Workflow Request Input decision in the plugin path.
   - Expected: native Ask UI only when `nativeAskUi: true` is proven; otherwise readable Markdown fallback from M5.
   - Record: whether native Ask or Markdown fallback appeared, plus the visible heading and answer controls.

4. Inspect Workflow Markdown surface output.
   - Expected: readable Markdown summary, no raw UUID/protocol/debug clutter, no duplicated machine-translation blocks.
   - Record: visible headings and any redacted screenshot/log excerpt reference.

5. Inspect Subagent prompt/report surface or Markdown substitute.
   - Expected: translated or derived content is labeled; original prompt/report evidence remains visible; paths, commands, and code fences are preserved.
   - Record: prompt/report labels, one path example, one command example, and whether original evidence is intact.

6. Inspect Plan/SubPlan progress.
   - Expected: current SubPlan progress and root Plan progress are displayed separately, for example `2/5 | 5/6`, not mixed as one total.
   - Record: exact progress text observed.

## Pass Signal

- Remote SSH plugin resume works.
- Request Input is visible and actionable through native Ask or the Markdown fallback.
- Workflow, Subagent, Plan/SubPlan, and tool-output summaries are readable Markdown without raw protocol clutter.

## Fail Signal

- Resume fails under Remote SSH.
- Request Input is invisible or unusable.
- Workflow/Subagent output appears as raw protocol/debug/transcript text.
- Progress counts are mixed across root plan and subplan.
