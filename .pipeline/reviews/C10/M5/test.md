# C10 M5 最终测试报告

Verdict: `green`

## 结论

最终候选 HEAD `ecf799926b` 上的 alpha.4 baseline、10 个 post-tag Cherry Pick、
Question ownership、权限、compaction、VSCode file links、GitLab updater 和 release
artifact 均通过。`b208579cc4` 审计发现的 listener-loss 两项缺口已修复并重测，
Ptolemy 对前 9 个提交与 VSP 修复的独立复审结论为 `pass`，没有 P0-P2。最终 ordinal
提交随后通过两个直接受影响 crate 的完整回归；按用户加速指令没有重复启动独立审计。

该结论对应最终 fetch `upstream/main@5c19155cbd`，其后提交数为 0。测试没有执行 tag、
push、上传、GitLab Release 或本地安装。

## 主要行为与测试设计

1. Question Tool ownership
   - Native 与 fallback 客户端都验证 A -> B Resume 后由 B 独占 pending Question。
   - A 的延迟回答被拒绝；B 回答完成原 turn。
   - B disconnect 或 unsubscribe 后，A 恢复 ownership 和原 request ID。
   - 已接受回答后 Resume 不会从 rollout history 复活旧 Question。
   - Cold cross-process Resume 在新进程创建 continuation。
   - 单元测试直接覆盖 connection-removal retarget/dismiss helper 与 missing-listener
     resolution cleanup；现有 websocket 测试覆盖 listener 存活时的 native/fallback
     replay。Listener-loss 分支复用同一完整 handoff 已由独立代码审查确认。
   - Native Ask 不输出 synthesized Question Context；fallback 保持可读并完成 turn。
2. 权限与 compaction
   - 无显式权限时 Plugin 默认 `never` + `danger-full-access`。
   - request config 中显式 `on-request` + `read-only` 不被 VSP 默认值覆盖。
   - Local compaction summary 不出现在任何 visible notification；ContextCompaction
     lifecycle 与最终真实回复仍可见。
3. 更新发现与提醒
   - 默认 GitLab stable manifest、`lab` channel 和 Linux GNU target 一致。
   - 显示版本与安装 action 来自同一 release。
   - upstream baseline 只来自精确匹配的已安装 release。
   - 当前 session 收到 VSP/upstream 刷新通知且不重复。
   - 有更高 VSP 版本时抑制 upstream 提醒。
4. Upstream 与发布
   - alpha.4 后 10 个提交按顺序纳入；7 个 exact patch-id，3 个 VSP context 适配有明确差异。
   - archive metadata、二进制版本、成员、SHA256 和 stable manifest 均结构化检查。

## 执行结果

| 范围 | 结果 |
| --- | --- |
| Ask integration | `codex-app-server --test all request_user_input`: 24 passed |
| Ask focused unit | 5 passed，覆盖 removal retarget/dismiss 与 missing-listener resolution |
| App-server lib | 282 passed，0 failed，串行 + `RUST_MIN_STACK=16777216` |
| Plugin permissions | defaults 2/2；request config override 1/1 |
| Local compaction | hidden summary + final reply 1/1；Complete/Interrupted cleanup unit included in 282 |
| TUI VSP updater | release profile 18 passed，0 failed |
| TUI cache/action | cache 2/2；update action 2/2 |
| TUI IDE/file/model | IDE IPC 10/10；VSCode file link 1/1；model availability 4/4 |
| Memories | `codex-memories-write`: 39 passed，0 failed |
| Extension API | 11 passed，0 failed |
| Exec-server | all package targets passed, including remote empty workspace roots |
| Core workspace roots | 6 unit + 2 integration passed |
| Core reasoning | 13 unit + 23 integration passed |
| Core review | intended integration module 10 passed |
| Rollout final delta | `codex-rollout --lib`: 99 passed，0 failed |
| Protocol final delta | `codex-app-server-protocol --lib`: 271 passed，0 failed |
| Release builder | Python unittest 4 passed |
| Release binary | source and archived binary: `vsp-codex 0.145.0-vsp.9` |
| Archive | 352415153 bytes；SHA256 check passed |
| Static | rustfmt、CI docs、diff、JSON、YAML、conflict markers、snapshots passed；strict Clippy 在定点 allow 2 条既有告警后通过 |

## 产物验证

- Archive: `.pipeline/release/v0.145.0-vsp.9/vsp-codex-preview-linux-x64-gnu.tar.gz`
- SHA256: `bbc6ecd5abbee8ea8273e57db80086bb05d3ba4dde6804937f1f5559db121a72`
- Entries: `bin/vsp-codex`, `bin/codex-code-mode-host`, `codex-path/rg`,
  `codex-resources/bwrap`, zsh resources, and `codex-package.json`.
- Metadata: version `0.145.0-vsp.9`, target `x86_64-unknown-linux-gnu`, channel `lab`,
  GitLab manifest URL, source `preview-release-builder`.
- Versioned and stable manifests identify `rust-v0.145.0-alpha.4`; stable manifest has
  exactly one vsp.9 Linux lab entry and preserves every historical release.

## 遇到的问题

- Release-only updater tests are excluded by `cfg(not(debug_assertions))`; a debug run
  only executed 8 tests. The final command used `--release` and executed the intended 18.
- IDE IPC fixtures inherited host `umask 002`, produced a 0775 socket parent, failed the
  production security check, then blocked a server thread on `accept()`. Test directories
  now use 0700 and accepts have a deadline; production validation remains strict.
- An over-broad core `review` filter ran 98 Guardian tests. 96 passed; two snapshots included
  this workspace's Hypo-Workflow runtime instruction because `.pipeline/` is intentionally
  present. The generated `.snap.new` files were rejected and removed. The intended
  `suite::review::` integration module passed 10/10.
- Core reasoning now matches 13 unit + 23 integration tests by name; the superseded alpha.2
  report listed 24 integration matches. No reasoning test failed on the final tree.
- The host is under high CPU load, so release ThinLTO and archive compression took several
  minutes but completed without timeout or linker failure.
- Strict Clippy initially reported two pre-existing, out-of-diff lints in
  `thread_processor.rs` (needless lifetime/borrow). Re-running with only those two lints
  allowed passed, leaving no lint finding in the remediation diff.
- 为响应用户加速发布要求，`5c19155cbd` 后没有重复旧的全矩阵或独立审计；直接受影响
  的 rollout/protocol crate 均全量通过，最终 release build、包内字节一致性、SHA 和
  manifest provenance 重新验证。
- 独立复审指出自动化没有直接强制 listener channel missing/closed 后再验证 native/fallback
  unsubscribe/disconnect replay，也没有直接覆盖 completion 丢失或 answer/disconnect 并发。
  报告已收窄，不再把 helper 与代码审查表述成该端到端测试。

## 剩余风险

- 没有 Windows/macOS archive 或 `pwsh` 运行时验证；候选仅声明 Linux x86_64 GNU。
- 没有两台真实电脑上的跨进程 UI acceptance；自动化覆盖了新进程 continuation，
  但不同 app-server 进程不能互相 dismiss UI。
- Updater 使用启动/缓存刷新节奏，不是常驻秒级轮询。
- Listener-loss native/fallback replay、closed-channel completion loss 和 compaction Error
  cleanup 仍缺直接的故障注入测试；当前结论由共享 helper、现有行为测试和独立代码审查支撑。
- 远端 URL、GitLab asset、三入口安装和 VSCode 实机更新必须在发布门后验证。
