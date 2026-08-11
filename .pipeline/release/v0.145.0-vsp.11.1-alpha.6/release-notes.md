# VSP Codex 0.145.0-vsp.11.1-alpha.6

This alpha improves the model-visible prompt stack, adds semantic Worker Routing for Subagents,
and introduces native preview packages for Linux, macOS, and Windows.

## Highlights

- The base instructions and default runtime policy are shorter and more explicit about Chinese
  communication, task steering, compaction recovery, evidence-backed reporting, authorization,
  and root-agent ownership.
- Bundled System Skills use a smaller effective-name catalog with aliases, a 2,000-token metadata
  ceiling, and active Skill persistence through compaction and resume. A new `vsp-provider` Skill
  owns VSP-specific provider setup and diagnosis.
- Multi-Agent V2 accepts semantic Worker Routing classes for mechanical, long-context, standard,
  exploratory, critical, and escalation work. Trusted machine policy selects compatible
  provider/model/reasoning candidates and reports the effective choice and fallback reasons.
- Release manifests preserve multiple targets for the same version. The updater can select native
  Linux x86-64 GNU, macOS ARM64, or Windows x86-64 MSVC packages.
- Windows packages include the command runner and sandbox setup helpers; Linux retains the packaged
  bwrap asset.

## Install Or Update

```shell
VSP_CODEX_RELEASE_CHANNEL=alpha vsp-codex update
```

For a fresh alpha installation on Linux or macOS:

```shell
curl -fsSL https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/scripts/install/install.sh | VSP_CODEX_RELEASE_CHANNEL=alpha sh
```

Windows users should use the PowerShell installer after the Release is published.

## Validation

- Accepted prompt, Skill lifecycle, and Worker Routing regressions passed, including Core Skills
  121/121, compact/lifecycle 19/19, Multi-Agent V2 spawn 19/19, required smoke 7/7, and release
  checks 52/52.
- Native GitHub Actions builds passed for Linux x86-64 GNU, macOS ARM64, and Windows x86-64 MSVC.
  Each runner executed the packaged entrypoint version check before uploading its archive.
- All three downloaded archives match their SHA-256 sidecars and aggregate manifest. Archive
  layouts, target metadata, executable formats, and aggregate manifest fields were independently
  checked.

## Known Limitations

- A live Subagent inference smoke was blocked by repeated VSPLab 502 responses before spawn;
  offline routing and real child-configuration handler tests passed.
- The deferred AGENTS/runtime split means the original prompt-size targets were not fully met.
- macOS and Windows alpha binaries are unsigned and not notarized, so operating-system trust
  prompts may appear.
