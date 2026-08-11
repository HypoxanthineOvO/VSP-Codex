# C8 M2 VSCode Plugin Protocol Capability Map

Generated: 2026-05-30T14:44:13+08:00

## Evidence Base

- Prompt: `.pipeline/prompts/c8-002-vscode-plugin-protocol-capability-map.md`
- Black-box evidence: `.pipeline/reviews/C8/M2/test.md`
- Remote SSH extension root: `/home/heyx/.vscode-server/extensions/openai.chatgpt-26.519.32039-linux-x64`
- Installed extension version: `openai.chatgpt` version `26.519.32039`
- Remote SSH extension logs: `/home/heyx/.vscode-server/data/logs/*/exthost*/openai.chatgpt/*.log`
- Recent app-server spawn evidence: `/home/heyx/.vscode-server/data/logs/20260528T204628/exthost6/openai.chatgpt/Codex.log:2`

No plugin source, installed extension files, production code, tests, workflow state, prompts, progress, logs, locks, or remote resources were modified for this report.

## Native Plugin-Supported / Evidence Found

| Capability | Evidence | Routing guidance |
|---|---|---|
| Remote SSH extension launches Codex app-server | Manifest entry `main: ./out/extension.js`; logs contain `[CodexMcpConnection] Spawning codex app-server`; installed under the Remote SSH `.vscode-server` extension root. | M3 provider/resume can rely on the app-server protocol path as the primary plugin integration path. |
| Experimental app-server API negotiation | Installed `out/extension.js` initialize payload includes `capabilities: { experimentalApi: true, requestAttestation: false }`. | Experimental app-server requests and notifications may be usable where the plugin has matching handlers. |
| Command execution approval requests | Protocol exposes `item/commandExecution/requestApproval`; webview bundle contains matching handler strings and approval UI mapping evidence. | M5 should use the native command approval path where supported. |
| File change approval requests | Protocol exposes `item/fileChange/requestApproval`; webview bundle contains matching handler strings and patch approval evidence, including approval id / `grantRoot` handling. | M5 should use the native file approval path where supported. |
| Permissions approval requests | Protocol exposes `item/permissions/requestApproval`; webview bundle contains matching synthetic permissions request UI evidence. | M5 should use the native permissions request path, with focused validation for advanced choices or scopes. |
| Flat plan updates | Protocol exposes `turn/plan/updated`; webview bundle transforms plan updates into a `todo-list` UI item. | M6 can send flat structured plan/status updates, but should still include Markdown summaries for portability. |
| Proposed plan text streaming | Protocol has `ThreadItem::Plan` and `item/plan/delta`; bundle handles `item/plan/delta` and `proposed-plan`. | Use for proposed-plan prose, not as the only source of checklist state. |
| Command/tool output surface | Protocol has command output deltas and aggregated output; bundle maps command output into execution items and has previous-turn expansion UI evidence. | Prefer native command/tool output items for logs and transcript expansion; summarize very large output. |
| Dynamic tool calls | Protocol includes `item/tool/call` and `ThreadItem::DynamicToolCall`; bundle handles dynamic tool call items and dynamic tool requests. | Native support appears present, but validate UX per tool before relying on specialized rendering. |
| Subagent/collab lifecycle items | Protocol includes `ThreadItem::CollabAgentToolCall`; event mapping covers spawn/send/wait/close/resume; bundle renders `multi-agent-action`. | Use native lifecycle items for M6 where available, but do not assume every subagent report surface is rich-rendered. |
| MCP elicitation | Protocol includes `mcpServer/elicitation/request`; bundle contains matching elicitation request and response handling. | Native elicitation can be considered supported by installed-plugin evidence. |

## App-Server-Supported But Plugin Support Unknown

| Capability | App-server evidence | Unknown / hard gate |
|---|---|---|
| Native Request Input / Ask UI | App-server protocol declares `item/tool/requestUserInput`; request payloads support `singleChoice`, `multiChoice`, `freeText`, and `ranking`; webview bundle contains `item/tool/requestUserInput` and `userInput` handling strings. | `nativeAskUi` is the hard gate for native Request Input. It was not found in the installed extension initialize evidence. App-server transport skips native Ask delivery unless `native_ask_ui` is true, so M5 should default to Markdown Ask fallback unless a live trace proves otherwise. |
| Structured plan state with nested subplan | Source `TurnPlanState` supports root items with one-level `subplan`; protocol tests exercise richer `state` payloads. | Installed stable TypeScript / bundle evidence confirmed flat plan rendering but did not confirm nested subplan rendering. M6 should pair structured plan/subplan payloads with Markdown summaries. |
| Command approval `additionalPermissions` / `availableDecisions` | Source supports these experimental fields; transport tests show experimental schema can retain them. | Stable generated TypeScript omits some experimental fields, and bundle rendering of these exact fields was not definitive. Use conservatively until live plugin trace or UI proof confirms behavior. |
| Full subagent prompt/report display | App-server maps collab agent lifecycle and output items. | Bundle evidence confirms lifecycle UI, not complete evidence for all prompt/report display expectations. Pair native lifecycle with Markdown report summaries. |
| Live JSON-RPC behavior under Remote SSH | Logs prove app-server lifecycle. | No full JSON-RPC trace was captured, so runtime delivery order, exact payloads, and UI state transitions remain partially unverified. |

## Markdown Fallback Likely Safe

| Capability | Evidence | Recommended use |
|---|---|---|
| Request Input fallback | `request_user_input_fallback.rs` emits numbered Markdown-like prompt text and parses numbered replies; `bespoke_event_handling.rs` uses fallback when no native Ask UI connection exists. | M5 should use Markdown Ask fallback by default for Hypo-Workflow `request_user_input` until `nativeAskUi: true` is proven in live initialize evidence. |
| Agent Markdown/text messages | `ThreadItem::AgentMessage` renders assistant text; bundle imports Markdown rendering helpers. | Use Markdown for workflow reports, Ask fallback, acceptance summaries, and compatibility notes. |
| Structured plan explanation summaries | Plan updates support explanations, and ordinary assistant text is rendered. | M6 should send native flat plan updates when useful and also include Markdown summaries for subplans and milestone state. |
| Subagent and report summaries | Assistant messages and command output surfaces are available even when rich subagent report rendering is uncertain. | Use Markdown summaries as the durable surface for M3-M6 handoffs and final acceptance reports. |
| Large or uncertain tool output | Native command output exists, but transcript expansion behavior was not fully traced. | Prefer native output items for raw logs and Markdown summaries for decisions, risks, and next actions. |

## Unsupported / Explicit Unknown / Requires Plugin Change

| Capability | Status | Reason |
|---|---|---|
| Native Request Input without `nativeAskUi` | Unsupported for routing assumptions | App-server checks `native_ask_ui` before delivery. Installed initialize evidence did not include `nativeAskUi`. |
| Attestation-based UX | Unsupported in installed evidence | Installed initialize payload sets `requestAttestation: false`; do not route C8 UX through attestation. |
| Legacy approval methods `applyPatchApproval` / `execCommandApproval` | Legacy fallback only | Protocol still contains legacy paths, but bundle evidence indicates legacy approval request methods are ignored/logged rather than first-class plugin UX. |
| Guaranteed nested subplan rendering | Explicit unknown | App-server supports nested plan state, but installed-plugin rendering evidence was not found. |
| Exact native Ask UI in installed extension `26.519.32039` | Explicit unknown | Bundle has latent request input strings, but initialize evidence lacks the hard-gate `nativeAskUi` capability. |
| Full Remote SSH JSON-RPC transcript claims | Explicit unknown | Logs do not contain complete request/notification transcripts, and no live trace was captured. |
| Improvements requiring plugin UI/source changes | Requires plugin change | Any feature not expressible through existing app-server requests, notifications, Markdown assistant text, or command/tool output requires plugin work outside C8 M2. |

## Routing Guidance For M3-M6

- M3 provider/resume can rely on the Remote SSH extension app-server path. It should not assume Request Input native UI, because `nativeAskUi` was not found in installed initialize evidence.
- M5 should use native command approval, file approval, permissions approval, MCP elicitation, dynamic tool call, and command output paths where supported. For Request Input, M5 should default to Markdown Ask fallback unless live initialize/trace evidence proves `nativeAskUi: true`.
- M6 should use flat `turn/plan/updated` plan updates where helpful, but pair structured plan/subplan payloads with Markdown summaries because nested subplan rendering is unknown in the installed plugin.
- Cross-milestone reports should keep capability claims evidence-labeled: native plugin-supported, app-server-supported/plugin-unknown, Markdown fallback, or unsupported.

## Validation And Limitations

- Validation: generated from `.pipeline/reviews/C8/M2/test.md`; no new probe scripts or code changes were added.
- Validation command: `git diff --check -- .pipeline/reviews/C8/M2/implement.md .pipeline/reports/017-vscode-plugin-protocol-capability-map.md`
- Result: passed.
- Limitation: installed extension bundle is minified, so UI behavior was inferred from protocol strings and bundle tokens, not readable source-level component review.
- Limitation: no full JSON-RPC trace was captured from a live Remote SSH plugin session.
- Limitation: no plugin source modifications, installed plugin modifications, dependency installs, artifact fetches, publications, or remote side effects were performed.
- Required follow-up validation for changing the Request Input classification: capture installed extension initialize evidence showing `nativeAskUi: true`, or obtain equivalent plugin-side proof that app-server native Ask delivery is enabled.
