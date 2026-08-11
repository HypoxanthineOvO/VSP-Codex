# MR !3: fix: preserve installer release directory during metadata write

- URL: https://gitlab.vsplab.cn/heyx/Codex/-/merge_requests/3
- Author: lichf / 李超凡
- State: opened
- Source: `fix-installer-metadata-stage-dir` @ `be8e4676ea`
- Target: `main`
- Files: `scripts/install/install.sh`
- Merge status snapshot: `checking`, conflicts: `False`
- Notes: 0; Pipelines: 2

## Local Assessment

该 MR 是最小 installer 修复：避免 write_vsp_release_metadata 在 POSIX sh 下覆盖调用方 install_package_release 的 release_dir。当前 main 仍有同名局部变量问题，建议在当前 main 上应用该修复；可考虑同时吸收 !1 的 duplicate vspRelease guard。

## Recommendation

`rebase_or_cherry_pick_on_main`
