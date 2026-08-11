---
kind: cycle-summary
cycle: C26-tui-heading-ci-review
status: closed
updated: 2026-08-10T01:17:05+08:00
version: 0.145.0-vsp.11.1-alpha.5
tag: v0.145.0-vsp.11.1-alpha.5
commit: 07b1fff4bc
---

# C26 TUI标题、最终回复完整性与CI真实性总结

## 结论

C26已完成并关闭。最终交付是不可变版本`0.145.0-vsp.11.1-alpha.5`：TUI标题层级与长文本
宽度安全、最终回复缺尾恢复、thread event FIFO、真实shell parallel声明、Cargo/Bazel/SDK测试合同、
以及Official/VSP state DB migration尾部兼容均已实现和验证。alpha.5已发布到GitLab alpha channel，
从公开Release asset安装到本机并通过入口、smoke与session compatibility复核。

## 目的与边界

- 审计全部测试、snapshot和CI是否存在硬编码、同源oracle、mock-only、弱断言、环境漂移或假绿。
- 以Markdown标题与C23最终回复完整性作为高风险真实链路，从protocol到VT100终端验证。
- 修复审计确认的产品、build/load、test target parity、SDK current-artifact和CI问题。
- 用户恢复C26后扩大到验证通过后的commit、push、不可变发布和本机安装；未重启service。

## 技术方案与主要修改

- Protocol/app-server：`turn/completed`保留canonical final text，connected TUI可恢复完整最终回复。
- TUI：H1/H2/H3拆入独立heading模块；显示列宽、长token、CJK、quote/list、Setext streaming、
  OSC 8链接与窄终端均使用宽度安全策略；active stream/final completion避免缺尾和重复。
- Event ordering：thread events统一走单一FIFO relay，移除channel-full spawned-send竞态。
- Shell：仅真实并行Classic backend声明parallel-safe，ZshFork维持串行。
- 测试/CI：17个行为组、Cargo/Bazel binary-root build contracts、Linux sandbox/permission/
  notification/parallel真实oracles、Python/TypeScript current-checkout合同和10分钟`vsp-smoke`。
- Installer：session compatibility由全量migration相等改为共同前缀逐项同checksum，并验证脚本实际
  使用的`threads`/`backfill_state`列；兼容Official 46 / VSP 40等尾部新migration。
- Workflow：激活semantic Cycle workspace，保存exhaustive inventory、审计、验证、PTY证据与发布记录。

## 测试设计与结果

- Exhaustive inventory：131 packages、217 Cargo harnesses、12,526+ Rust cases、614 snapshots、
  29 GitHub workflows / 94 jobs全部对账，源码语义未检查项为0。
- Protocol 273/273；rollout 107/107；四个Bazel binary-root suites 337/337。
- Python默认128 pass / 38个真实provider显式skip；current-checkout Python 44/44；TypeScript 5/5。
- 相关core/TUI/app-server/CLI/permission/parallel/network-denial/remote-notification测试与reviewed
  snapshots通过；pending snapshot为0。
- Strict Clippy、rustfmt、diff hygiene、Cargo/Bazel build contracts与三个Bazel native sandbox target通过。
- `just vsp-release-check` 48/48；session compatibility 9/9；Ruff check/format通过。
- 本地与GitLab pipelines 354-358的`vsp-smoke`通过：build、version/help、真实PTY首帧、
  app-server initialize与thread/list共7项。
- 真实100x34 PTY验证H1/H2/H3、长token、代码块、`End marker.`、composer分隔和状态0退出。

## 发布与安装证据

- Release source commit/tag：`07b1fff4bca39a9587c8fa9a4d0fcf09c3e53170`；main随后仅追加本Cycle关闭记录。
- Annotated tag：`v0.145.0-vsp.11.1-alpha.5`。
- Release：`https://gitlab.vsplab.cn/heyx/Codex/-/releases/v0.145.0-vsp.11.1-alpha.5`。
- Archive SHA-256：`55945d913660978bbfd177abcde04eac90925bfba7ee53affc4fe77b2f1d3a30`；
  local checksum、远端整包下载和stable manifest一致。
- Package/source binary hash均为`80245047cf85f457c83934f1fb07d1b33746dd54ea2e861b5b77528a63ee69e8`；
  package内migration脚本与source hash一致。
- 公开installer把本机从alpha.4更新到alpha.5；`~/.local/bin/vsp-codex`和兼容`codex`均报告
  alpha.5，current symlink指向alpha.5 standalone release，installed smoke 7/7。
- Bare alpha update返回“No newer VSP release matched this install”，确认stable manifest无更高alpha。

## 遇到的问题与处置

- 旧M5只比较失败集合且没有逐项审计，用户拒绝；重做exhaustive inventory和源码语义复核后再修复。
- Cargo native bwrap因主机缺`libcap.pc`不能运行；等价Bazel native targets全部通过，保留为显式host gap。
- alpha.4真实安装暴露migration exact-equality误拒；alpha.4 tag不移动，修复后发布新alpha.5。
- Live status发现63个历史rollout需要neutralize；先备份两个DB到
  `~/.vsp-codex/session-compat-backups/20260810-004550`，等长修改后复核0 pending。
- 一次candidate smoke因错误workdir未执行且被后续shell命令掩盖；该结果作废，随后单独重跑7/7。

## 预期用户行为

- H1居中、H2带双侧rule、H3左对齐；窄终端和长token不会溢出或遮挡composer。
- 最终回复在item completion缺失、image boundary或active stream情况下仍完整且不重复。
- CI绿色表示current VSP可构建并完成最小CLI/TUI/app-server交互；功能改动仍需选择对应行为组。
- Official/VSP共享session资产时，尾部migration版本不同不再阻断兼容迁移；共同prefix或所需schema
  真正不一致仍硬失败。

## 剩余风险与后续候选

- Cargo native bwrap仍依赖本机`libcap.pc`；当前由Bazel native sandbox验证覆盖。
- alpha channel仍需常规soak；未来stable promotion必须重新构建新不可变正式版本，不能移动alpha tag。
- macOS/Windows、真实第三方provider和跨主机remote场景按各自前置条件继续使用对应manual group。
- C22是独立遗留active Cycle，不属于C26关闭范围。
