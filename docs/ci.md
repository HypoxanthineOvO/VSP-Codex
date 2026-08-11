# VSP CI Contract

VSP CI is a small merge gate for long-lived product contracts. It is not a mirror of every local test, every upstream check, or the current implementation layout.

GitLab CI in `.gitlab-ci.yml` is the VSP branch and release gate. GitHub Actions are inherited primarily for upstream Codex development; a downstream repository may run them according to its own merge policy.

The inherited GitHub and Bazel workflows are not part of the VSP merge decision and consume no time in the GitLab pipeline. They remain in the tree to keep upstream synchronization reviewable; do not report their status as VSP CI status.

## Required Gate

The required `vsp-smoke` job performs two operations:

1. Build the `vsp-codex` binary with the repository lockfile and repository Rust toolchain.
2. Run `scripts/ci_vsp_smoke.py` against that binary in isolated temporary homes.

The job has a 10-minute hard timeout. Normal successful runs should stay comfortably below that budget; a longer gate indicates toolchain or scope drift rather than an acceptable increase in routine validation.

The job bootstraps rustup when a shell runner does not provide Cargo, caches that project-local toolchain, and lets `rust-toolchain.toml` select the Rust version. Runner provisioning is therefore not an undocumented prerequisite and the CI file does not duplicate the toolchain version. Cargo uses the Git CLI and redirects the runner-inaccessible libyuv origin to a GitHub mirror; the locked submodule commit remains the integrity boundary.

The smoke check verifies only these stable contracts:

- the binary starts and exposes the top-level CLI;
- the core `exec` and `resume` command surfaces can be parsed;
- an interactive launch renders a visible TUI frame;
- app-server accepts `initialize` and a read-only `thread/list` request.

The checks use exit status and protocol behavior. They do not compare a product version, exact help text, UI snapshot, test count, snapshot count, file layout, or command list.

## Admission Rule

A new mandatory CI assertion must satisfy all of the following:

- It should remain valid across reasonable feature upgrades and refactors.
- Its failure means a core user-facing or compatibility contract is broken.
- It is deterministic and does not require an external service.
- Its maintenance cost is small relative to the regression it prevents.

Adding a test does not automatically add a CI requirement. A test that describes the current implementation but not a durable contract stays outside the merge gate.

## Scoped Validation

Formatting, linting, crate suites, UI snapshots, benchmarks, performance runs, persistence stress tests, packaging, and full upstream regression remain available as scoped local or release validation. Run the checks that match the change and report what was run.

Use these entry points:

- `just vsp-ci`: build the current `vsp-codex` binary and run the required runtime smoke.
- `just test-groups`: list behavior-oriented local test groups with their purpose.
- `just test-group explain <group>`: show when a group is valuable, its prerequisites, and its failure policy.
- `just test-group run <group> [--filter <name-or-file>]`: run the smallest relevant group from `scripts/vsp_test_groups.json`.
- `just test -p <crate> <filter>`: escape hatch for a focused selection not yet represented in the manifest. No selection fails immediately.
- `just test-all --reason "<owned investigation>"`: deliberately run the 12,000+ case upstream suite. It refuses to start without a reason and is never a VSP merge gate.
- `just vsp-release-check`: validate package, installer, session-compatibility, and preview-release logic without publishing or installing anything.

The manifest groups tests by the behavior they protect:

| Group            | Contract                                                                   |
| ---------------- | -------------------------------------------------------------------------- |
| `smoke`          | Current binary build plus minimal CLI, TUI, and app-server operation       |
| `build-contracts` | Cargo/Bazel target parity, compile fixtures, doctests, and package loading |
| `cli`            | One named CLI parsing, doctor, update, or routing behavior                 |
| `core`           | One named core request, tool, config, goal, conversation, or host behavior |
| `app-server`     | One named public JSON-RPC implementation behavior                          |
| `exec`           | One named headless event, final-output, JSONL, or resume behavior          |
| `protocol`       | Complete Rust, TypeScript, and JSON protocol shape contract                |
| `tui`            | One named rendering or interaction behavior plus snapshot review           |
| `persistence`    | Rollout durability and VSP session compatibility                           |
| `release`        | Package, installer, compatibility, and preview release assembly            |
| `sdk-typescript` | One SDK Jest file against the current VSP binary                           |
| `sdk-python`     | Python resolver and app-server handshake against the current VSP binary    |
| `security`       | Apply-patch parsing and filesystem mutation boundaries                     |
| `linux-sandbox`  | Provisioned Linux filesystem, socket denial, and bubblewrap build           |
| `v8-runtime`     | Real embedded V8 evaluation, CRDTP, and sandbox linkage                    |
| `hypo-external`  | Manual cross-repository Workflow compatibility                             |
| `upstream-all`   | Explicitly owned full upstream diagnostic, never routine validation        |

AI agents must choose groups before running tests. A group failure caused by the current change must be fixed and rerun. An unrelated failure must be isolated with a focused reproduction and treated as a real blocker only when it prevents the scoped result; a raw full-suite failure inventory is not a completed test report.

Snapshot changes should be reviewed when user-visible output changes, but snapshot equality is not a normal merge requirement. Performance, stress, and packaging checks belong in release preparation or focused investigations unless a deterministic long-lived contract is identified.

Tests that require a sibling checkout, real provider, remote executor, Docker, Wine, or a particular operating system are manual tests. Keep their prerequisites visible and do not turn a missing prerequisite into a passing result.

## Test Portfolio Rule

Keep a test when it protects a current fork behavior with an independent assertion. Move it out of routine runs when it is useful but slow, platform-specific, or dependent on external infrastructure.

Delete a test or snapshot when its product behavior no longer exists, its producer has been removed, it exactly duplicates a stronger assertion, or it can pass without observing the behavior named by the test. A failing or slow test is not, by itself, a deletion reason.

Do not pin duplicate tool versions in `.gitlab-ci.yml`. Rust and dependency versions come from repository-owned toolchain and lock files.

## Changing CI

When proposing a new required job or assertion, document the product contract it protects and why ordinary future upgrades should continue to satisfy it. If that case cannot be made, keep the check local or release-only.

Other repositories that merge VSP work remain responsible for their own CI contracts. VSP CI should not duplicate those policies preemptively.
