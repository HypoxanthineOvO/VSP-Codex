# C22 Progress

| Time (Asia/Shanghai) | State | Detail |
| --- | --- | --- |
| 2026-08-05 22:45 | started | Created isolated branch and worktree from `d7f9ef84b0`; main-worktree compact changes remain untouched. |
| 2026-08-05 22:47 | discovered | Confirmed ten-command manifest, current Host status fields, legacy VSP validators, and stale documentation claims. |
| 2026-08-05 22:54 | red | Host contract rejected new fields; semantic routing missed both new cases; TUI rejected the ten-command release and dynamic command fixture. |
| 2026-08-05 22:58 | implemented | Added bounded manifest-declared routing, safe semantic workspace detection, Host status compatibility, and current ownership documentation. |
| 2026-08-05 23:05 | green | Host Contract 7/7, semantic routing 7/7, release routing/palette 11/11, and command palette unit 1/1 passed. |
| 2026-08-05 23:06 | snapshot | Reviewed and accepted the ten-command Workflow palette snapshot with `/hw:experiment`. |
| 2026-08-05 23:13 | validated | Rustfmt, affected-crate check, focused Clippy with known unrelated lint allowances, diff hygiene, documentation scan, and snapshot pending checks completed. |
| 2026-08-05 23:14 | complete | C22 semantic interface adaptation is ready for review; no commit, push, release, or install was performed. |
| 2026-08-05 23:50 | packaged | Rebuilt the compact-fix `0.145.0-vsp.11.1-alpha.1` archive directly from the matching installed package without another Cargo build. |
| 2026-08-06 00:00 | versioned | Combined the compact fix and semantic Host UX on `0.145.0-vsp.11.1-alpha.2`; the single release build succeeded. |
| 2026-08-06 00:03 | installed | Built and installed the local alpha.2 package; `vsp-codex --version` reports the expected version. |
| 2026-08-06 00:06 | cleaned | Removed the shared Cargo target and temporary package directory, reclaiming 83.1 GiB plus 1.4 GiB. |
| 2026-08-06 00:15 | published | Published alpha.1 compact hotfix assets and the new alpha.2 release to GitLab; main and the alpha channel now select alpha.2. |
