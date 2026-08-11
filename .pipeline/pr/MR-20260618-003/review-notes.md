# Review Notes: MR !3

## Findings

- Assessment: `still_relevant_minimal_fix`
- Recommendation: `rebase_or_cherry_pick_on_main`
- Evidence files: `evidence/metadata.json`, `evidence/changes.json`, `evidence/commits.json`, `evidence/notes.json`, `evidence/pipelines.json`

## Risk Notes

该 MR 是最小 installer 修复：避免 write_vsp_release_metadata 在 POSIX sh 下覆盖调用方 install_package_release 的 release_dir。当前 main 仍有同名局部变量问题，建议在当前 main 上应用该修复；可考虑同时吸收 !1 的 duplicate vspRelease guard。

## Remote Write Boundary

No remote write was performed. Merge, close, branch push, labels, reviewers, and target-branch changes require explicit confirmation.

## Hotfix Resolution

Absorbed core metadata_release_dir fix into current main hotfix.
