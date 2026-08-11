# MR !2: docs: fix VSP 7.2 install checksum

- URL: https://gitlab.vsplab.cn/heyx/Codex/-/merge_requests/2
- Author: lichf / 李超凡
- State: opened
- Source: `fix-vsp-7-2-install-sha` @ `dec4d98756`
- Target: `main`
- Files: `docs/install.md`, `docs/vsp-codex.md`, `README.md`
- Merge status snapshot: `checking`, conflicts: `False`
- Notes: 0; Pipelines: 2

## Local Assessment

该 MR 只把 0.134.0-vsp.7.2 文档 checksum 从旧值改为 844b...；当前 main 已发布 0.140.0-vsp.0，README/docs 均使用 90086f...，直接合并会回退文档到旧 release。

## Recommendation

`close_as_obsolete`
