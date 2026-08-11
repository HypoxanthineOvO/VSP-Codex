# C10 M5 Upstream alpha.4 同步与 Cherry Pick 报告

## 结论

VSP-Codex 已完整合并官方 `rust-v0.145.0-alpha.4`，并把最终 fetch 时
`upstream/main@5c19155cbd` 相对本地 alpha.4 审查点的 10 个提交按官方顺序纳入。
最终候选版本为 `0.145.0-vsp.9`，VSP suffix 延续 `vsp.8.x` 后的功能序列，
不会因官方 baseline 变化重置。

同步、语义冲突处理、Cherry Pick、行为回归、release build 和本地 package candidate
均已完成。用户已授权发布；本报告写入时尚未执行 tag、push、GitLab asset/upload/release
或本地安装。

## Baseline Merge

- Official tag: `rust-v0.145.0-alpha.4`
- Official tag commit: `8a6381616f`
- Local merge commit: `820f9b14b8`
- Merge conflicts: 7
- Resolution policy:
  - Cargo metadata 保留 `0.145.0-vsp.9`。
  - 保留 VSP app-server/TUI/test 模块，同时接受 alpha.4 personality 与官方依赖变化。
  - Cargo.lock 删除已废弃 execpolicy 漂移并保持 VSP workspace version 一致。
  - justfile 同时保留 VSP Bazel 说明与官方 e2e benches。
- Merge validation: locked Cargo metadata、`codex-models-manager` 和 `codex-core`
  checks passed before post-tag selection.

## Post-alpha.4 Cherry Picks

| 顺序 | Upstream | 本地 | 内容 | Patch 结果 |
| --- | --- | --- | --- | --- |
| 1 | `610f09abcf` | `d9b6346cd1` | GPT-5.6 Sol Bedrock default | exact |
| 2 | `bfe31598c7` | `d85602243d` | paginated local thread persistence | VSP adaptation |
| 3 | `dffe1f02a3` | `455fe9fcab` | reasoning-summary capability | exact |
| 4 | `076a110eb8` | `c5b745a2f3` | materialized workspace plugin hook trust | VSP adaptation |
| 5 | `dfefd8aa8b` | `60a18cbbb9` | prefer CODEX_HOME IDE socket | exact |
| 6 | `08ba14b03d` | `98863d6e38` | file upload diagnostics | exact |
| 7 | `c9d52de5ca` | `24600fdefc` | outbound response-item ID prefixes | exact |
| 8 | `203d530cc6` | `d10f954ed7` | model announcement priority | exact |
| 9 | `f93c18ed0f` | `b208579cc4` | canonical moved-config links | exact |
| 10 | `5c19155cbd` | `47bb38a787` | paginated rollout ordinals and history projection | context adaptation |

Exact patch IDs: 7/10. The three non-identical patch IDs are:

- Pagination keeps VSP cleanup for an empty model-provider field.
- Plugin-hook trust keeps VSP model-cache coverage and remote-plugin feature guards.
- Rollout ordinals retain the pre-existing VSP discovery module and provider tests at the two
  conflict sites. Upstream/local numstats are identical, and `range-diff` shows only VSP context.

All three adapted commits retain the complete upstream production behavior. `git cherry` reports
all seven exact commits as equivalent and only these three documented adaptations as different.
The final fetch after package generation remained at `5c19155cbd`; latest official release tag
remained `rust-v0.145.0-alpha.4`.

## VSP Behaviors Preserved

1. Question Tool routes pending cards and response authority to the latest resumed connection.
2. Native Ask suppresses synthesized context; fallback clients receive readable numbered answers.
3. Plugin defaults to Full Access while explicit request permissions override the defaults.
4. Local compaction hides internal summaries and clears suppression on terminal turn paths.
5. VSCode file paths use native Ctrl+click instead of external file OSC8 handling.
6. GitLab stable manifest drives VSP update discovery and exact installed baseline lookup.
7. GPT-5.6 Sol/Terra Ultra reasoning and V1/V2 role reasoning locks remain present.

## Modified Areas

- `codex-rs/app-server`: Question handoff, permissions, compaction, alpha.4 APIs/tests.
- `codex-rs/tui`: updater, file links, IDE IPC, model announcements, upstream TUI commits.
- `codex-rs/thread-store`, `codex-rs/rollout`, `codex-rs/app-server-protocol`: paginated
  persistence, durable ordinals, history projection, and VSP provider compatibility.
- `codex-rs/core`, `codex-rs/model-provider`, `codex-rs/models-manager`: reasoning,
  Bedrock model, plugin hooks, response IDs, upload diagnostics.
- `codex-rs/config.md`: canonical official configuration links.
- `.pipeline/release`: alpha.4-provenance candidate package and manifests.

## Test Design And Results

- Merge/compile: `cargo metadata --locked`, core/models-manager checks, TUI check passed.
- Post-tag focused tests: pagination 4/4, history mode 1/1, Bedrock 3/3 + 2/2,
  reasoning capability 1/1 + spawned-agent 2/2, response IDs 6/6, plugin hooks
  unit 2/2 + integration trust/fail-closed 1/1 each, upload diagnostics 2/2.
- VSP surfaces: Ask focused 5/5 + integration 24/24; app-server lib 282/282;
  permissions 2+1; compaction 1;
  updater 18/18; cache/action 2/2 each; IDE IPC 10/10; file link 1/1.
- Broad upstream matrix: memories 39/39; extension API 11/11; exec-server all targets;
  core workspace-roots 6+2, reasoning 13+23, review 10/10.
- Final two commits: model availability 4/4; TUI check, rustfmt, CI docs and diff passed.
- Final ordinal commit: `codex-rollout` 99/99 and `codex-app-server-protocol` 271/271.
- Release: builder 4/4; source and archived binary version, members, metadata, SHA256,
  alpha.4 provenance and stable manifest uniqueness passed.

## Package Candidate

- Version: `0.145.0-vsp.9`
- Archive size: `352415153` bytes
- SHA256: `bbc6ecd5abbee8ea8273e57db80086bb05d3ba4dde6804937f1f5559db121a72`
- Manifest baseline: `rust-v0.145.0-alpha.4`
- Upstream cutoff: `5c19155cbd`

## Problems Encountered

- alpha.2 dry-run evidence became invalid when alpha.3/alpha.4 appeared; M5 was reopened and
  the alpha.4 merge plus test matrix replaced it.
- Three later fetches advanced official main first by seven commits, then by two commits, and
  finally by the rollout ordinal commit.
  Each advancement reopened GREEN before packaging or reporting.
- IDE IPC tests hung under host `umask 002`; secure 0700 test directories and deadline-bound
  accepts fixed the fixture without weakening production validation.
- Release-only updater tests require `--release`; debug profile compiles only parser tests.
- A broad Guardian filter captured local `.pipeline/` instructions in two snapshots. The
  generated snapshots were rejected; the intended review integration suite passed 10/10.

## Risks And Follow-up

1. Final cutoff is a point-in-time `main` commit, not a promise that official main will stop moving.
   Future official updates are surfaced by the new upstream reminder and require a new review cycle.
2. The three adapted patch IDs must remain documented during future range-diff audits.
3. Windows/macOS packaging and real cross-machine UI acceptance remain outside this Linux candidate.
4. GitLab publication is explicitly authorized; local installation remains a separate operation.
5. Runtime `.pipeline/chat`, inbox and mini-cycle files are intentionally excluded from commits.
