---
kind: execution
cycle: C29-alpha6-vsplab-502-hotfix
status: in_progress
updated: 2026-08-11T20:19:00+08:00
---

# C29 Execution

## Diagnosis

- Alpha 5 and Alpha 6 were run sequentially with the same `CODEX_HOME`, provider, model,
  reasoning, service tier, cwd, prompt, and HTTP transport.
- Alpha 5 completed with `ok` and usage `17688/3840/5`.
- Alpha 6 failed after six distinct VSPLab 502 request ids.
- Alpha 6 with the Sol catalog temporarily set to no collaboration completed with `ok`.
- Alpha 6 with the Sol catalog temporarily set to V1 collaboration completed with `ok`.
- Alpha 6 V2 with usage/mode prompt text removed still failed after six 502 responses.
- Local request capture showed identical transport and scalar request fields. The Alpha 6 body
  was smaller than Alpha 5. The remaining V2 wire regression was the VSP-private
  `routing_class` parameter added to the provider-facing `spawn_agent` schema. The nested output
  schema was not present in the code-mode request, but routing diagnostics would still alter
  follow-up tool output and were removed for contract parity.

## Decision

Keep routing resolution and observability internal. Restore the Alpha 5/official model-facing
`spawn_agent` shape and simple output instead of exposing private routing parameters or nested
diagnostics to the provider. With no safe official field available under the active hidden-metadata
configuration, spawned agents use the trusted default `standard` routing class. Explicit six-class
selection is deferred until a client-internal intent channel is designed.

## Implementation

- Removed the private `routing_class` parameter from the provider-facing V2 schema.
- Removed the nested `routing` object from the V2 `spawn_agent` output schema/result.
- Removed routing diagnostics from serialized tool results while retaining internal selection.
- Kept internal `WorkerRoutingClass`, default standard policy resolution, child config application,
  telemetry, and tracing.
- Reframed tests around complete child configuration and simple output equality.

## Focused Validation

- `core --filter spawn_agent_tool`: 6/6 passed, including full visible/hidden schema equality.
- `core --filter multi_agent_v2_spawn`: 19/19 passed, including routing, fallback, overrides,
  role conflicts, fork behavior, and encrypted child delivery.
- `cargo fmt --check` and `git diff --check` passed.

## Release Authorization

The user added authorization to publish a Linux-first
`v0.145.0-vsp.11.1-alpha.7` hotfix after the live V2 smoke passes. macOS and Windows are not
release blockers for C29. Installation, deployment, and restart remain out of scope.

## Live Validation Correction

- Removing only nested routing output diagnostics did not fix the initial request; the live V2
  smoke still returned six 502 responses.
- Downgrading `routing_class` from enum to string also returned six 502 responses.
- Removing the private `routing_class` parameter entirely restored the official tool shape and the
  same real V2 request completed with `ok` (`15309` input, `3840` cached, `5` output).
- The user correctly rejected the earlier “upstream incompatibility” wording. The fault is a VSP
  wire-contract violation, not an OpenAI/VSPLab compatibility defect.

## Final Validation

- `spawn_agent` schema tests: 6/6 passed.
- Multi-Agent V2 spawn tests: 19/19 passed, including semantic routing plus an explicit
  `fork_turns=all` request.
- V2 root/subagent prompt assembly tests: 2/2 passed.
- Release/package/installer/session compatibility group: 52/52 passed.
- Real VSPLab root smoke returned `ok` with no 502.
- Real Subagent smoke logged internal resolution of `standard-luna-max` to
  `vsp / gpt-5.6-luna / max / priority`, initialized the Luna child session, completed without a
  spawn error or 502, and returned `root-ok`.

## Linux Alpha 7 Candidate

- Version: `0.145.0-vsp.11.1-alpha.7`.
- Target: `x86_64-unknown-linux-gnu` only; macOS/Windows Alpha 7 are intentionally not built.
- Archive SHA-256: `77764b9fecfbe6e156757bb07755213e32fdb6f3570d6e7c60449e62cad424d8`.
- The package layout, ELF formats, package metadata, sidecar, and extracted
  `vsp-codex --version` output were independently verified.
- The local machine lacks `libcap.pc`; the build used the repository-supported
  `CODEX_SKIP_BWRAP_BUILD=1` path and reused the already validated Alpha 6 Linux bwrap asset.
  The Alpha 7 entrypoint and code-mode host were built from current source in release profile.
