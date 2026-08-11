---
kind: milestone-report
cycle: C24-responses-multi-provider-config-model-switching
milestone: M4
status: completed
updated: 2026-08-08T21:22:00+08:00
---

# M4 Final Hardening Report

## 结论

C24 已完成并达到 release-ready。新版 VSP/DEX 可以从独立的
`$CODEX_HOME/model-providers.toml` 读取 Responses addon providers，通过统一 `/model`
面板按 `(provider_id, model_id, reasoning_effort)` 切换，并在 resume、fork 与 manual
compact 中保持 provider-local endpoint、credential reference 和 model metadata。官方
`config.toml` 不会收到第三方 provider 定义或默认选择。

本报告只确认源码和自动化验证完成；未执行 commit、push、打包、安装、服务重启或远端
release。

## 技术实现

- Core 独立解析严格 versioned addon 文件，将 provider-scoped model catalog 合并到有效
  registry，并只原子更新 addon default pair。
- App-server v2 新增 `modelProvider/list` 与 `modelProvider/write`。服务端拥有有效
  `CODEX_HOME` 和 sibling file 的读写；连接式/remote TUI 不直接访问远端文件系统。
- TUI 从 app-server 获取 provider catalog、credential availability 与 current provider，
  使用 provider tabs 展示模型；切换失败时按 runtime、addon default、official config
  的反向顺序回滚。
- Per-turn catalog 覆盖 models-manager 的 global catalog，避免跨 provider 同名 model
  复用错误 context/reasoning metadata。
- Manual compact 显式传入当前 turn provider，使用该 provider 的 `/responses/compact`
  endpoint；resume/fork 继续携带已持久化 provider/model/effort。
- 同轮修复了 resize transcript replay 贴底和 streamed/canonical CJK whitespace
  reconciliation，两项均有 focused regression。

## 修改模块

- `codex-rs/core/src/config/model_providers_file.rs` 与 sibling tests：addon contract、merge、
  collision、default persistence。
- `codex-rs/core/src/session/turn_context.rs`、`thread_manager.rs`、
  `models-manager/src/manager.rs`：provider-scoped runtime metadata。
- `codex-rs/app-server-protocol/src/protocol/{common.rs,v2/model.rs}` 与生成 schema：v2
  provider list/write API。
- `codex-rs/app-server/src/request_processors/model_provider.rs`、message routing 与公共
  JSON-RPC tests：server-owned catalog/persistence。
- `codex-rs/tui/src/app/provider_{catalog,model_selection}.rs`、
  `chatwidget/provider_model_popups.rs` 与 sibling tests/snapshots：remote catalog、事务切换、
  grouped picker。
- `codex-rs/core/tests/suite/provider_compact.rs`：provider-specific compact endpoint 回归。
- TUI resize/streaming/markdown modules：同轮两个已记录的 UI 回归修复。

## 测试设计与验证结果

- App-server protocol：271 unit tests + 2 schema fixture tests，全通过；stable/experimental
  JSON 与 TypeScript fixtures 均与生成结果一致。
- Core addon contract：13/13，通过 missing/blank、strict schema、URL/env/default、
  duplicate/collision、merge 与 comment-preserving persistence。
- Runtime metadata：provider-local `get_model_info` 1/1，通过。
- Compact：manual compact provider catalog/endpoint 1/1，通过。
- App-server 公共 JSON-RPC：remote list + addon/official write ownership 1/1，通过。
- TUI C24 focused：server catalog 1/1、transaction rollback 1/1、provider popup 3/3、
  4 个 provider snapshots 全通过；resize 与 CJK spacing 回归各 1/1，通过。
- 额外复跑：默认线程栈发生 overflow 的既有 agent-picker test 使用
  `RUST_MIN_STACK=33554432` 后 1/1 通过。
- Hygiene：`cargo fmt --all -- --check`、`git diff --check`、pending snapshot check 全通过；
  本轮生成的无关 `.snap.new` 已全部 reject，未混入 C24。

## 全量验证例外

- `cargo test -p codex-tui` 运行 3161-test binary 时，C24 provider tests 均已通过，但被
  9 个与 C24 无关的既有 snapshot drifts 和一个默认线程栈 overflow 中止。未接受这些
  snapshots；overflow 用例增大线程栈后单独通过。
- Workspace Clippy 在到达完整目标前被既有 deny lint 阻断：
  `core/src/session/mod.rs` 的 mutex guard 跨 await 与 redundant clone，以及
  `app-server/src/request_processors/thread_processor.rs` 的 needless borrow。C24 touched
  modules 未产生新的 Clippy diagnostic；未越界修改这些无关代码。
- Rust 构建仍打印仓库既有 dead-code warnings；不影响本轮 focused tests。

## 预期用户行为

- addon 文件缺失或为空时维持官方单 provider 行为。
- addon 存在时 `/model` 展示官方 base provider 和 addon tabs；窄屏布局稳定。
- 缺 credential 的 provider 可发现但不可选择，只显示 env key 名，不泄漏 value。
- provider/model/effort 同时生效；失败不会留下 runtime 与持久化选择不一致的半状态。
- connected/remote TUI 通过 app-server 读写服务端配置根，不会错误写客户端本机文件。
- resume、fork、compact 和同名模型跨 provider 场景均使用正确 provider-local metadata 与
  endpoint。

## 问题、风险与后续

- 未使用真实 DeepSeek/Kimi/GLM credential；provider-neutral Responses 合同由本地
  deterministic mock 验证，不承诺供应商特有工具/reasoning 兼容性。
- 变更跨 core、protocol、app-server 和 TUI，人工逻辑面超过单个 800-line review 建议；
  源码已按独立模块拆分，generated schema 属机械输出。发布提交可按 contract/runtime、
  server API、TUI 三个审阅单元组织，但必须保持同一兼容发布批次。
- 发布仍需明确版本号、目标分支/remote、产物范围，以及是否包含安装和重启；这些动作
  均不由 C24 完成状态隐式授权。
