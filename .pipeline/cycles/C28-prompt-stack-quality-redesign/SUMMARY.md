---
kind: cycle-summary
cycle: C28-prompt-stack-quality-redesign
status: closed
accepted: 2026-08-11T19:10:44+08:00
result: M4-RELEASE-CANDIDATE.md
release: v0.145.0-vsp.11.1-alpha.6
---

# C28 Summary

## Accepted Result

The user accepted immediate publication of the verified alpha.6 artifacts without another native
build. The GitLab Release is live at:

`https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.11.1-alpha.6`

The release contains Linux x86-64 GNU, macOS ARM64, and Windows x86-64 MSVC archives, three
SHA-256 sidecars, and the aggregate manifest. Public re-downloads match all sidecars; the public
manifest contains all three targets and exact digests.

## Delivered Changes

- Accepted Sol base prompt and compact runtime policy with proactive Multi-Agent assembly.
- Revised bundled System Skills, `vsp-provider`, effective-name precedence, aliases, 2,000-token
  metadata cap, and Skill persistence through compaction/resume.
- Trusted semantic Worker Routing for Multi-Agent V2 with six classes, offline candidate
  validation, fallback reasons, role/explicit conflict handling, fork selection, and telemetry.
- Target-specific archive naming and strict multi-target release manifest aggregation.
- Native GitHub Actions build matrix for Linux, macOS, and Windows, including MSVC rusty_v8
  artifact selection and CRLF-safe checksum verification.

## Evidence

- Native run `31476110232`: Linux 46m06s, Windows 1h23m06s, macOS 1h29m58s, aggregate 32s;
  all jobs successful.
- Core Skills 121/121, compact/lifecycle 19/19, Worker Routing/V2 focused tests, required smoke
  7/7, release checks 52/52, TUI platform regression 1/1, CLI check, formatting, diff and YAML
  checks passed.
- Archive SHA-256 values:
  - Linux: `ced4c72705006186cc9c91a13ad6a1fff47afb91028d21967941e8399938d7dd`
  - macOS: `5824b199315a154f3fe377ed5b8813533b351a15a632db5adad1f7a77bab1b07`
  - Windows: `1cefdba90a3f29d29b36c436482aa96985a4e321aec25da2a727453e86c02333`

## Scope and Risks

- The user deferred Stage C AGENTS/runtime splitting and Stage D image capability. Prompt reduction
  measured 22.6% for input tokens and 28.4% for non-tool characters, below the original targets.
- Live Subagent inference was blocked by repeated VSPLab 502 responses before spawn; offline
  routing and child-configuration tests passed.
- macOS and Windows artifacts are unsigned and not notarized; operating-system trust warnings may
  appear.
- The first public manifest upload accidentally contained one target after a second merge of an
  already-aggregated manifest. Public verification caught this; the package file/link was replaced
  with the original three-target manifest. Archive bytes and digests were unchanged. The immutable
  tag remains on the original release metadata commit; canonical `main` contains the correction.

No local installation, deployment, or service restart was performed.
