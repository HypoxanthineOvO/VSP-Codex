# MR !1: Fix VSP archive installer staging move

- URL: https://gitlab.vsplab.cn/heyx/Codex/-/merge_requests/1
- Author: lichf / 李超凡
- State: opened
- Source: `fix/install-archive-metadata-dir` @ `fc36c272fd`
- Target: `main`
- Files: `scripts/install/install.sh`
- Merge status snapshot: `checking`, conflicts: `False`
- Notes: 0; Pipelines: 2

## Local Assessment

核心 installer staging bug 仍存在于当前 main，但该 MR 基于较旧 main；补丁包含两个点：把函数内 release_dir 改成 metadata_release_dir，以及在已有 vspRelease 时跳过追加。建议在当前 main 上重新应用必要修复，而不是直接合并旧分支。

## Recommendation

`do_not_merge_as_is`
