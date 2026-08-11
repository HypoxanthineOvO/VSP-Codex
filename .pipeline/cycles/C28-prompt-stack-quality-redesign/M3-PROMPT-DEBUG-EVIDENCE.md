# M3 Prompt Debug Evidence

## Conclusion

The Stage A prompt stack is present in a real request emitted by the locally built `vsp-codex` binary. The request uses the accepted VSP base instructions byte-for-byte, emits the runtime policy as one ordered developer message, defaults Multi-Agent V2 to proactive delegation, and contains no unsolicited recommended-plugin catalog.

## Execution

- Binary: `codex-rs/target/debug/vsp-codex`
- Version: `0.145.0-vsp.11.1-alpha.5`
- Command shape: `vsp-codex exec --ephemeral --ignore-user-config --ignore-rules --dangerously-bypass-approvals-and-sandbox --enable multi_agent_v2 -m gpt-5.6-sol ...`
- Prompt: `只回复 ok，不调用工具。`
- Transport: an isolated local Responses API mock configured through a temporary `model_providers.mock` entry with WebSockets disabled
- User configuration: isolated temporary `CODEX_HOME`; no user config was loaded
- Result: the CLI completed normally and printed `ok`

The first attempt set `OPENAI_BASE_URL`, but the built-in OpenAI provider still attempted its fixed WebSocket endpoint and received HTTP 401 before falling back to HTTPS. That attempt produced no local capture. The successful attempt used an explicit custom provider with `supports_websockets = false`, zero retries, and the local HTTP endpoint. No product source was changed to accommodate the capture.

## Captured Request

| Evidence | Observed value |
| --- | --- |
| Model | `gpt-5.6-sol` |
| Base-instruction bytes | `11,696` |
| Base-instruction SHA-256 | `33202ab56daa59f39644a36f82935532b910a78f313fbc11147b5e450f39e5b6` |
| Accepted candidate equality | exact byte equality with `S1-BASE-INSTRUCTIONS-CANDIDATE.md` |
| Message roles | 2 developer messages, 2 user messages |
| Base-instruction message | 1 content part |
| Runtime policy message | 5 ordered content parts |
| Unsolicited recommended-plugin entries | 0 |
| Apps feature catalog blocks | 0 |

The custom provider serializes base instructions as the first developer `message` rather than a top-level `instructions` field. The second developer `message` is the coherent runtime policy assembled by `Session::build_initial_context`.

Runtime policy parts, in order:

1. Compact unrestricted/never permissions contract: 273 bytes.
2. Root Multi-Agent ownership, delegation boundary, routing classes, shared-filesystem note, and concurrency: 1,559 bytes.
3. One `<multi_agent_mode>` fragment declaring proactive delegation: 170 bytes.
4. Available skills instructions: 6,222 bytes.
5. Hypo-Workflow host projection: 347 bytes.

The dynamic policy contains exactly one `<multi_agent_mode>` opening tag. It contains no `<recommended_plugins>` fragment. Default collaboration mode contributes no extra prose because its developer instructions are intentionally empty.

## Tool Surface

The request exposed four top-level entries through the Responses API `additional_tools` item:

- `exec`
- `wait`
- `request_user_input`
- `collaboration`

No plugin recommendation or install tool was exposed in this isolated run because no explicit discoverable-tool list was configured. Focused integration tests separately cover the explicit `list_available_plugins_to_install` to `request_plugin_install` flow.

## Interpretation

The evidence confirms the Stage A hierarchy:

1. VSP identity, communication, autonomy, continuity, and shared-workspace safety live in the Sol base instructions.
2. Runtime permissions, Multi-Agent ownership/routing/mode, skills, and Workflow host context are grouped into one developer policy message.
3. Project instructions and the actual user prompt remain user-role inputs.
4. Optional plugin discovery is absent until the explicit discovery workflow makes candidates available.

## Limits

- This is request-construction evidence, not an inference-quality evaluation against a live model.
- The local mock validates the exact outbound payload and a successful CLI response lifecycle; later Cycle stages still need scenario-level prompt quality evaluation.
- The raw captured JSON remains a temporary local file and is not committed because it includes the full project instruction payload and tool schemas.
