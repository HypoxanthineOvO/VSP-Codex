# C8/M4 Test Worker 报告

## 修改文件

- `codex-rs/rollout/tests/official_codex_discovery.rs`
- `codex-rs/rollout/tests/fixtures/official_codex_discovery/vsp_rollout_with_private_metadata.jsonl`
- `codex-rs/rollout/tests/fixtures/official_codex_discovery/field_classification.json`
- `.pipeline/reviews/C8/M4/test.md`

## 测试命令与红灯结果

命令：

```sh
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-rollout --test official_codex_discovery
```

观察结果：按预期 compile-red，因为测试约定的 official-discovery API 尚未实现。

```text
error[E0432]: unresolved import `codex_rollout::official_codex_discovery`
 --> rollout/tests/official_codex_discovery.rs:5:20
  |
5 | use codex_rollout::official_codex_discovery::{
  |                    ^^^^^^^^^^^^^^^^^^^^^^^^ could not find `official_codex_discovery` in `codex_rollout`

error: could not compile `codex-rollout` (test "official_codex_discovery") due to 1 previous error
```

格式化与空白检查：

```sh
rustfmt --edition 2024 codex-rs/rollout/tests/official_codex_discovery.rs
```

该命令完成；输出了仓库现有稳定版工具链警告：`imports_granularity` 是 unstable rustfmt 选项。

```sh
git diff --check -- codex-rs/rollout/tests/official_codex_discovery.rs codex-rs/rollout/tests/fixtures/official_codex_discovery/field_classification.json codex-rs/rollout/tests/fixtures/official_codex_discovery/vsp_rollout_with_private_metadata.jsonl .pipeline/reviews/C8/M4/test.md
```

观察结果：无空白错误。

## 覆盖的隐私断言

- sanitized official export 必须剥离 provider/auth secrets：
  `model_provider`、私有 provider details、`base_url`、`env_key`、auth headers、API keys、refresh tokens，以及原始 secret-looking env values。
- sanitized official export 必须剥离 Workflow/runtime metadata：
  `.pipeline`、Workflow state/continuation fields、`/hw:resume`、私有实现步骤，以及 rollout trace internals。
- sanitized official export 必须保留 official discovery/resume/import 所需字段：
  session id、created timestamp、cwd、安全 title/name、source kind，以及 sanitized artifact path。
- official export 不得把 VSP home/sessions tree 以 symlink 或 raw-share 形式暴露到 official Codex home。
- field classification 使用 fixture 固定 safe/private/secret 预期，防止 provider/auth/workflow 字段被静默重分类为 safe。

## 预期测试 API

测试假设 rollout/session discovery 拥有公开模块：

```rust
codex_rollout::official_codex_discovery
```

并提供：

```rust
async fn export_sessions_for_official_codex(
    vsp_codex_home: &Path,
    official_codex_home: &Path,
) -> anyhow::Result<OfficialCodexExport>;

fn classify_field_for_official_discovery(field_path: &str) -> OfficialDiscoveryFieldClass;
```

预期 export shape：

- `OfficialCodexExport.manifest_path: PathBuf`
- `OfficialCodexExport.sessions: Vec<OfficialCodexSessionExport>`
- `OfficialCodexSessionExport.id: String`
- `OfficialCodexSessionExport.created_at: String`
- `OfficialCodexSessionExport.cwd: PathBuf`
- `OfficialCodexSessionExport.title: Option<String>`
- `OfficialCodexSessionExport.source: Option<String>`
- `OfficialCodexSessionExport.sanitized_artifact_path: PathBuf`

预期 classifier 的 debug string 转小写后匹配 fixture class：

- `safe`
- `private`
- `secret`

实现可以使用更丰富的内部类型，但需要暴露足够稳定的测试接口来验证 official Codex discovery/export。

## 假设

- `codex-rs/rollout` 是最小可信 owner，因为它已经拥有 session rollout persistence、listing、finding 和 session-index 行为。
- 默认策略应是 sanitized export/mirror，而不是 raw directory sharing。
- 代表性 fixture 中的 conversation text 可通过 sanitized artifact 保留；provider/auth/workflow/runtime 字段不可保留。

## Fix-round regression tests

### 修改文件

- `codex-rs/rollout/tests/official_codex_discovery.rs`
- `.pipeline/reviews/C8/M4/test.md`

### 覆盖的审计发现

- `session_index.jsonl.thread_name` suspicious markers must not appear in the manifest/exported title:
  added `session_index_thread_name_is_sanitized_before_manifest_title`.
- A symlinked directory under `vsp_home/sessions` pointing outside the sessions root must not be traversed/exported:
  added Unix-only `symlinked_sessions_directory_outside_root_is_not_traversed`.
- Sanitized artifact compatibility with the normal rollout reader:
  added `sanitized_artifact_is_readable_by_rollout_thread_reader`, which calls
  `codex_rollout::read_thread_item_from_rollout(session.sanitized_artifact_path.clone()).await`
  and asserts thread id, cwd, preview, and source.
- Duplicate source basename collisions must not produce the same sanitized artifact path or overwrite either export:
  added `duplicate_source_basenames_do_not_overwrite_sanitized_artifacts`.

### 命令与红灯结果

命令：

```sh
cd /home/heyx/Codex-VSP/codex-rs
cargo test -p codex-rollout --test official_codex_discovery
```

观察结果：red/failing as expected for the fix round. The integration test binary ran 8 tests; 6 passed and 2 failed.

```text
running 8 tests
test field_classification_fixture_locks_provider_auth_workflow_fields ... ok
test fixture_contains_expected_private_values_so_sanitizer_assertions_are_meaningful ... ok
test symlinked_sessions_directory_outside_root_is_not_traversed ... ok
test duplicate_source_basenames_do_not_overwrite_sanitized_artifacts ... FAILED
test export_for_official_codex_strips_private_runtime_and_preserves_resume_metadata ... ok
test export_for_official_codex_does_not_symlink_or_raw_share_vsp_home ... ok
test sanitized_artifact_is_readable_by_rollout_thread_reader ... ok
test session_index_thread_name_is_sanitized_before_manifest_title ... FAILED
```

Failing assertions:

```text
duplicate_source_basenames_do_not_overwrite_sanitized_artifacts:
assertion `left != right` failed: same-basename source rollouts must get distinct sanitized artifact paths
left:  ".../official_codex_discovery/sessions/rollout-2026-05-30T12-00-00-duplicate-basename.jsonl"
right: ".../official_codex_discovery/sessions/rollout-2026-05-30T12-00-00-duplicate-basename.jsonl"

session_index_thread_name_is_sanitized_before_manifest_title:
manifest title leaked unsafe session_index thread_name marker ".pipeline"
```

Formatting command:

```sh
rustfmt --edition 2024 codex-rs/rollout/tests/official_codex_discovery.rs
```

Result: completed; emitted the existing stable-toolchain warning that `imports_granularity = Item` is unstable.

### Notes

- The symlink regression test is present and passed in this checkout, indicating the current Unix traversal behavior did not follow the test symlinked directory.
- The sanitized artifact reader regression test is present and passed in this checkout, so the current sanitized fixture remains readable enough for id/cwd/preview/source discovery.
