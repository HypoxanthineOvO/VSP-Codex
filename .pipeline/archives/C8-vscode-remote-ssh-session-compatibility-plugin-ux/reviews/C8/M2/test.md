# C8 M2 Test Worker Report

Generated: 2026-05-30T14:39:31+08:00

## Scope

This report records black-box probe evidence for the installed Remote SSH VSCode extension and app-server protocol capability map. No production code, plugin source, installed plugin files, workflow state, logs, or prompts were edited.

Changed file:

- `.pipeline/reviews/C8/M2/test.md`

No probe fixture was added.

## Commands Run

- `sed -n '1,220p' .pipeline/state.yaml`
- `sed -n '1,260p' .pipeline/prompts/c8-002-vscode-plugin-protocol-capability-map.md`
- `git status --short`
- `find /home/heyx/.vscode-server/extensions/openai.chatgpt-26.519.32039-linux-x64 -maxdepth 3 -type f ...`
- `find /home/heyx/.vscode-server/data/logs -path '*openai.chatgpt*' -type f`
- `sed -n '1,260p' /home/heyx/.vscode-server/extensions/openai.chatgpt-26.519.32039-linux-x64/package.json`
- `file /home/heyx/.vscode-server/extensions/openai.chatgpt-26.519.32039-linux-x64/bin/linux-x86_64/codex /home/heyx/.vscode-server/extensions/openai.chatgpt-26.519.32039-linux-x64/out/extension.js`
- `rg -l 'item/tool/requestUserInput|requestUserInput|ToolRequestUserInput|request_user_input' /home/heyx/.vscode-server/extensions/openai.chatgpt-26.519.32039-linux-x64/out /home/heyx/.vscode-server/extensions/openai.chatgpt-26.519.32039-linux-x64/webview/assets`
- `rg -l 'item/commandExecution/requestApproval|item/fileChange/requestApproval|item/permissions/requestApproval|PermissionsRequest|request_permissions|approvalId|availableDecisions|additionalPermissions' ...`
- `rg -l 'turn/plan/updated|item/plan/delta|subplan|planItems|PlanUpdate|update_plan|proposed_plan' ...`
- `rg -n 'nativeAskUi|capabilities:\\{experimentalApi' /home/heyx/.vscode-server/extensions/openai.chatgpt-26.519.32039-linux-x64/out/extension.js`
- `rg -n 'item/tool/requestUserInput|latestUserInputRequest|replyWithUserInputResponse|turn/plan/updated|type:\\`todo-list\\`' .../webview/assets/*.js`
- `rg -n 'requestUserInput|PermissionsRequest|approval|DynamicTool|FunctionCallOutput|plan|subplan|markdown|item/tool|ServerRequest|ClientRequest|initialize' codex-rs/app-server-protocol codex-rs/app-server`
- `sed -n ... codex-rs/app-server-protocol/src/protocol/v1.rs`
- `sed -n ... codex-rs/app-server/src/request_user_input.rs`
- `sed -n ... codex-rs/app-server/src/request_user_input_fallback.rs`
- `sed -n ... codex-rs/app-server/src/transport.rs`
- `sed -n ... codex-rs/app-server/src/bespoke_event_handling.rs`
- `sed -n ... codex-rs/app-server-protocol/src/protocol/v2/item.rs`
- `sed -n ... codex-rs/app-server-protocol/src/protocol/v2/turn.rs`
- `sed -n ... codex-rs/app-server-protocol/src/protocol/v2/permissions.rs`

Validation command:

- `git diff --check -- .pipeline/reviews/C8/M2/test.md`

## Installed Extension Evidence

Manifest/package:

- Extension root: `/home/heyx/.vscode-server/extensions/openai.chatgpt-26.519.32039-linux-x64`
- `package.json`: `name=chatgpt`, `publisher=openai`, `version=26.519.32039`, display name `Codex - OpenAI's coding agent`.
- Activation: `onStartupFinished`, `onUri`.
- Entry point: `main: ./out/extension.js`.
- No `browser` entry found in the inspected manifest.
- Contributions include Codex commands, configuration key `chatgpt.cliExecutable`, webview sidebar views, conversation custom editor, command palette/menu entries, and `chatSessions` integration.
- Bundled binaries:
  - `bin/linux-x86_64/codex` exists, 219,517,248 bytes, ELF x86-64 static PIE, stripped.
  - `bin/linux-x86_64/codex-resources/bwrap`
  - `bin/linux-x86_64/rg`

Remote SSH logs:

- Logs exist under `/home/heyx/.vscode-server/data/logs/*/exthost*/openai.chatgpt/*.log`.
- Many logs contain `[CodexMcpConnection] Spawning codex app-server`, including a recent entry at `/home/heyx/.vscode-server/data/logs/20260528T204628/exthost6/openai.chatgpt/Codex.log:2` on `2026-05-30 12:29:25.063`.
- Logs mostly contain extension/app-server process lifecycle and stderr messages, not full JSON-RPC transcripts.
- Observed repeated CLI warnings for unknown local config feature keys: `plan_tool`, `rmcp_client`, `streamable_shell`, `view_image_tool`. Treat as environment/config compatibility evidence only, not plugin UI capability evidence.

Extension bundle protocol clues:

- `out/extension.js` sends app-server `initialize` with `clientInfo.name`, `clientInfo.title: "Codex Extension"`, current extension version, and `capabilities: { experimentalApi: true, requestAttestation: false }`.
- I did not find `nativeAskUi` in the extension initialize payload. This is important because app-server gates native Ask delivery separately from `experimentalApi`.
- Webview assets contain exact handling strings for:
  - `item/tool/requestUserInput`
  - `item/commandExecution/requestApproval`
  - `item/fileChange/requestApproval`
  - `item/permissions/requestApproval`
  - `turn/plan/updated`
  - `item/plan/delta`
  - `item/commandExecution/outputDelta`
  - `item/commandExecution/terminalInteraction`
  - `item/fileChange/patchUpdated`
  - `serverRequest/resolved`
  - `collabAgentToolCall` and rendered `multi-agent-action`
- The webview transforms `turn/plan/updated` into a `todo-list` UI item. The inspected bundle snippets showed `plan` and `explanation`; definitive evidence for structured `state` / nested subplan rendering was not found in the plugin bundle.
- The webview transforms `item/tool/requestUserInput` into a `userInput` item and later `user-input-response`, but because initialize omits `nativeAskUi`, this is best classified as latent/unknown for the installed extension rather than confirmed active native support.
- The broad bundle is minified and often single-line. I stopped at token/path evidence and did not attempt de-minification.

## App-Server Protocol Evidence

Request input:

- `InitializeCapabilities` includes distinct booleans: `experimental_api`, `request_attestation`, and `native_ask_ui` in `codex-rs/app-server-protocol/src/protocol/v1.rs`.
- `item/tool/requestUserInput` is declared as an experimental server request in `codex-rs/app-server-protocol/src/protocol/common.rs`.
- `ToolRequestUserInputParams` supports `threadId`, `turnId`, `itemId`, `summary`, `description`, and multiple questions.
- Question kinds include `singleChoice`, `multiChoice`, `freeText`, and `ranking`; question fields include `header`, `question`, optional `description`, `isOther`, `isSecret`, and options.
- `ToolRequestUserInputResponse` maps question ids to structured answer fields: `selectedOptionIds`, `orderedOptionIds`, `freeText`, `legacyAnswers`, and `answers`.
- `codex-rs/app-server/src/transport.rs` skips `ToolRequestUserInput` requests for a connection unless `native_ask_ui` is true.
- `codex-rs/app-server/src/bespoke_event_handling.rs` checks `first_native_ask_ui_connection_for_thread`; if none exists, it sends the request without delivery, records pending unsupported Ask state, and emits fallback prompt text.
- `codex-rs/app-server/src/request_user_input_fallback.rs` creates numbered Markdown-like fallback text and parses numbered replies back into structured answers.

Approvals and permissions:

- `ServerRequest.ts` includes native server requests for `item/commandExecution/requestApproval`, `item/fileChange/requestApproval`, `item/permissions/requestApproval`, `mcpServer/elicitation/request`, and `item/tool/call`.
- Command approval params support `threadId`, `turnId`, `itemId`, `startedAtMs`, optional `approvalId`, `reason`, `networkApprovalContext`, `command`, `cwd`, `commandActions`, `proposedExecpolicyAmendment`, and `proposedNetworkPolicyAmendments`.
- Experimental command approval fields `additionalPermissions` and `availableDecisions` exist in source, but stable generated TypeScript fixtures omit them; export tests confirm experimental schema generation retains them and stable schema generation removes `additionalPermissions`.
- `transport_tests.rs` verifies `additionalPermissions` is stripped without experimental capability and kept with capability.
- File change approvals support optional `grantRoot`.
- Permissions request approvals support `cwd`, `reason`, `permissions`, and response `permissions`, `scope`, and optional `strictAutoReview`.

Plans, items, tool output, and transcript surfaces:

- `ServerNotification.ts` includes `turn/plan/updated`, `item/started`, `item/completed`, `item/agentMessage/delta`, `item/plan/delta`, command output delta, file patch/update notifications, reasoning deltas, and collab/subagent item notifications.
- `ThreadItem` variants include `UserMessage`, `HookPrompt`, `AgentMessage`, `Plan`, `Reasoning`, `CommandExecution`, `FileChange`, `McpToolCall`, `DynamicToolCall`, `CollabAgentToolCall`, `WebSearch`, `ImageView`, `ImageGeneration`, review-mode items, and `ContextCompaction`.
- `TurnPlanState` in source supports structured plan state with root items and one-level `subplan`, but the inspected stable generated TypeScript `TurnPlanUpdatedNotification.ts` only included `plan` and `explanation`. Tests exercise the richer `state` payload, so classify nested/subplan state as app-server-supported but plugin support unknown.
- `ResponseItem` includes `function_call_output` and `custom_tool_call_output` with string or content item array outputs.
- `event_mapping.rs` maps dynamic tool call responses to `ThreadItem::DynamicToolCall` with output content items and collab agent lifecycle events to `ThreadItem::CollabAgentToolCall`.

## Preliminary Capability Evidence Table

| Capability | Classification | Evidence | Assertion for implement worker |
|---|---|---|---|
| App-server transport from Remote SSH extension | Native plugin-supported / evidence found | Manifest main `./out/extension.js`; logs show `[CodexMcpConnection] Spawning codex app-server`; extension initializes app-server in `out/extension.js`. | Use app-server protocol as the primary integration path. |
| Experimental app-server API capability | Native plugin-supported / evidence found | `out/extension.js` initialize payload includes `capabilities.experimentalApi: true`. | Experimental server requests/notifications may be delivered to this extension connection. |
| Native Request Input / Ask UI delivery | App-server-supported but plugin support unknown | Webview bundle handles `item/tool/requestUserInput`; app-server supports it; but extension initialize payload omits `nativeAskUi`, and app-server transport skips Ask requests unless `native_ask_ui` is true. | Do not assume native Ask will be active in installed extension `26.519.32039`; prefer fallback unless a live trace shows `nativeAskUi: true`. |
| Request Input fallback | Markdown fallback likely safe | `request_user_input_fallback.rs` emits numbered prompt text and parses numbered replies. `bespoke_event_handling.rs` uses fallback when no native Ask UI connection exists. | Server-side fallback is the safe default for M5 unless plugin negotiation changes. |
| Command execution approvals | Native plugin-supported / evidence found | Protocol request exists; webview handles `item/commandExecution/requestApproval`; snippets map it into exec approval UI with command/reason/network amendments. | Use native command approval request where possible. |
| File change approvals | Native plugin-supported / evidence found | Protocol request exists; webview handles `item/fileChange/requestApproval` and attaches approval id/grantRoot to patch item. | Use native file change approval request where possible. |
| Permissions request approvals | Native plugin-supported / evidence found | Protocol request exists; webview handles `item/permissions/requestApproval`; permissions request synthetic UI evidence found. | Use native permissions request path, but test exact response choices before relying on advanced scopes. |
| Command additional permissions / available decisions | App-server-supported but plugin support unknown | Source fields exist and are experimental; stable TS omits them; extension sets `experimentalApi: true`, but bundle evidence for rendering these exact fields was not definitive. | Include them only with conservative fallback or explicit live trace validation. |
| Plan updates flat list | Native plugin-supported / evidence found | `turn/plan/updated` in protocol and bundle; webview transforms to `todo-list`. | Use `turn/plan/updated` for flat plan/status updates. |
| Structured plan state / subplan | App-server-supported but plugin support unknown | Source `TurnPlanState` supports root items and one-level subplan; protocol tests exercise `state`; inspected stable TS and bundle evidence did not confirm rendering. | For M2 matrix, mark subplan as unknown/plugin-risk; Markdown summary fallback should accompany structured state if used. |
| Proposed plan text streaming | Native plugin-supported / evidence found | Protocol has `ThreadItem::Plan` and `item/plan/delta`; bundle handles `item/plan/delta` and renders `proposed-plan`. | Use for proposed-plan text, not as substitute for task checklist state. |
| Agent markdown/text messages | Markdown fallback likely safe | `ThreadItem::AgentMessage` text is rendered as assistant message; bundle imports `marked.esm` and external markdown link helpers. | Markdown-like workflow reports should be safe as assistant text, with normal Markdown limitations. |
| Tool output / transcript expansion | Native plugin-supported / evidence found | `CommandExecution` has `aggregatedOutput`; notifications include `item/commandExecution/outputDelta`; bundle maps exec output into exec item output and has previous-turn expansion UI evidence. | Use command/tool output protocol items for logs; avoid embedding huge output in assistant text unless summarizing. |
| Dynamic tool calls | Native plugin-supported / evidence found | `ServerRequest` includes `item/tool/call`; `ThreadItem::DynamicToolCall`; bundle handles `dynamic-tool-call` items and dynamic tool requests. | Native support appears present, but plugin-specific tool UX should still be validated per tool. |
| Subagent/collab lifecycle | Native plugin-supported / evidence found | `ThreadItem::CollabAgentToolCall` and event mapping for spawn/send/wait/close/resume; bundle renders `multi-agent-action` and hides `wait`. | Use native collab items for lifecycle, but do not assume full report display for all subagent outputs. |
| MCP elicitation | Native plugin-supported / evidence found | Protocol has `mcpServer/elicitation/request`; bundle handles mcp server elicitation synthetic item and response. | Native elicitation path can be part of final matrix. |
| Legacy approvals `applyPatchApproval` / `execCommandApproval` | Unsupported / legacy fallback only | Protocol still includes legacy requests; bundle snippets log/ignore legacy approval request methods. | Avoid relying on legacy approval methods for plugin UX. |
| Attestation | Unsupported / disabled in installed extension | Extension initialize sets `requestAttestation: false`; protocol supports `attestation/generate`. | Do not route required UX through attestation in this milestone. |
| Raw JSON-RPC trace from live session | Explicit unknown/unavailable | Logs do not contain full JSON-RPC request/notification transcripts; no live plugin session trace was captured to avoid side effects. | Final report should state no live trace was captured. |

## Unknowns And Caveats

- The installed extension is minified. Exact UI component behavior was inferred only from token/path snippets and protocol strings, not from readable source.
- Native Request Input remains the most important unknown: plugin bundle contains request handling, but the inspected initialize payload does not advertise `nativeAskUi`. The server therefore should not deliver native Ask requests to that connection under current app-server gating.
- Stable generated TypeScript fixtures omit some experimental fields even though source/tests cover them. The final matrix should specify whether it is using stable schema, experimental schema, or runtime `experimentalApi` behavior.
- Logs provide app-server lifecycle evidence, not a full protocol capture.
- No Remote SSH UI actions, dependency installs, network fetches, plugin edits, or production changes were performed.

## M2 Implement Worker Guidance

- Turn this report into the final capability matrix with four sections: native plugin-supported, app-server-supported/plugin-unknown, Markdown fallback safe, and unsupported/explicit unknown.
- Treat `nativeAskUi` negotiation as a hard gate. Until a live trace or plugin source confirms `nativeAskUi: true`, route Hypo-Workflow `request_user_input` through Markdown fallback.
- Prefer native requests for command approval, file approval, permissions approval, MCP elicitation, flat plan updates, command output, dynamic tools, and collab lifecycle.
- Pair structured plan/subplan payloads with Markdown summaries until plugin rendering is confirmed.
- Keep unsupported claims narrow: legacy approval methods and attestation are not suitable paths for this installed extension evidence.

## Validation

- `git diff --check -- .pipeline/reviews/C8/M2/test.md` from `/home/heyx/Codex-VSP`: passed.
