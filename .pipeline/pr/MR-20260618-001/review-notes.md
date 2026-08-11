# Review Notes: MR !1

## Findings

- Assessment: `still_relevant_superseded_by_mr3_plus_extra_guard`
- Recommendation: `do_not_merge_as_is`
- Evidence files: `evidence/metadata.json`, `evidence/changes.json`, `evidence/commits.json`, `evidence/notes.json`, `evidence/pipelines.json`

## Risk Notes

核心 installer staging bug 仍存在于当前 main，但该 MR 基于较旧 main；补丁包含两个点：把函数内 release_dir 改成 metadata_release_dir，以及在已有 vspRelease 时跳过追加。建议在当前 main 上重新应用必要修复，而不是直接合并旧分支。

## Remote Write Boundary

No remote write was performed. Merge, close, branch push, labels, reviewers, and target-branch changes require explicit confirmation.

## Hotfix Resolution

Absorbed metadata_release_dir fix and duplicate vspRelease guard into current main hotfix; direct old-branch merge no longer recommended.
