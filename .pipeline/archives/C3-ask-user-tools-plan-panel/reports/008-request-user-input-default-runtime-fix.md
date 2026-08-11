# M8 Repair Report: request_user_input 普通会话可用性修复

完成时间：2026-05-26T22:26:38+08:00

## 背景

用户拒绝 C3，原因是重启后的普通会话中，助手在需要提问时没有使用 Ask Tools。C3 之前只证明了协议、TUI、app-server 和部分 core 行为，没有证明普通 Default/Bypass 会话的 model-visible tools 中一定包含 `request_user_input`。

## 修复

- `codex-rs/tools/src/tool_config.rs`
  - `request_user_input_available_modes()` 现在返回所有 collaboration modes：`Default`、`Plan`、`PairProgramming`、`Execute`。
  - 不再受 `DefaultModeRequestUserInput` feature gate 影响。
- `codex-rs/protocol/src/config_types.rs`
  - `ModeKind::allows_request_user_input()` 扩展到所有 collaboration modes。
- `codex-rs/core/src/tools/handlers/request_user_input_spec.rs`
  - 工具描述从“只在某些模式可用”改为“every collaboration mode”。
- `codex-rs/core/tests/suite/request_user_input.rs`
  - 新增 `request_user_input_is_model_visible_in_default_turn`，直接检查普通 Default turn 发给 Responses API 的 `tools` 列表包含 `request_user_input`。
  - Default round-trip 不再启用旧 feature gate。
  - Execute/Pair Programming 从拒绝测试改为 round-trip 测试。

## 验证

- `cd codex-rs && just fmt`
- `cd codex-rs && just test -p codex-tools request_user_input_modes_include_all_collaboration_modes_by_default`
- `cd codex-rs && just test -p codex-core request_user_input`
- `cd codex-rs && just fix -p codex-core -p codex-tools`

结果：

- `codex-tools` focused test：1 passed。
- `codex-core request_user_input`：18 passed，2539 skipped。
- `just fix -p codex-core -p codex-tools`：通过。

## 剩余验收

这次修复证明了 core 普通 Default turn 的 model-visible tools 中包含 `request_user_input`，并且所有 collaboration modes 的 handler gate 可通过。

仍需用户在真实重启后的当前产品会话中复验：当助手需要询问选择/确认问题时，应实际弹出 Ask UI，而不是退回普通文本。

## 真实复验结果

2026-05-26T22:26:38+08:00 后，用户重启会话并要求进入规划时提问。助手实际调用 Ask 工具，运行时返回：

```text
request_user_input is unavailable in Default mode
```

结论：真实复验失败。仓库 core/tool 层修复还没有打通当前会话的 tool exposure / developer tool availability 层。C3 必须继续修订，不能验收。

## 失败原因复查

2026-05-26T22:52:01+08:00 复查当前运行环境：

- 当前交互进程是 `vsp-codex resume`。
- `vsp-codex` 指向 `/home/heyx/Codex-VSP/codex-rs/target/debug/vsp-codex`。
- 该二进制在真实复验失败时的时间戳是 `2026-05-26T16:43:28+08:00`，早于 M8 的源码修复和测试。

因此，真实复验失败不是因为当前源码仍保留 Default 门禁，而是因为用户第一次重启时仍在运行旧的本地 debug 入口。当前会话进程不会热加载新编译产物。

## 实际入口重建

已执行：

```text
cd codex-rs && cargo build --bin vsp-codex
```

结果：

- 构建通过。
- `/home/heyx/Codex-VSP/codex-rs/target/debug/vsp-codex` 更新时间为 `2026-05-26T22:51:47+08:00`。
- `vsp-codex --version` 输出 `vsp-codex 0.134.0-vsp.2`。

## 当前验收状态

M8 现在处于“已重建入口，等待新进程复验”。当前这条对话仍是旧进程启动后的会话，不能用来证明修复已生效。需要再次退出并重新启动 `vsp-codex resume`，然后让助手在普通 Default 会话中调用 `request_user_input`；只有实际弹出 Ask UI，M8 才能通过。

## 第二次真实复验结果

用户再次重启后，助手在普通 Default 会话中调用了 `request_user_input`。这一次没有再返回 `request_user_input is unavailable in Default mode`，说明 Default 工具门禁已经打通。

但 TUI 显示的是 unsupported-client fallback 文本：

```text
当前客户端不支持原生 Ask UI，请按编号回复
```

结论：M8 的 Default mode tool availability 修复只部分通过；原生 TUI Ask UI 仍失败。根因是 app-server client 初始化时 `native_ask_ui` capability 仍为 `false`。

## M9 修复：TUI native Ask UI capability

修复内容：

- `codex-rs/app-server-client/src/lib.rs`
  - `InProcessClientStartArgs` 增加 `native_ask_ui`。
  - `initialize_params()` 使用调用方传入的 `native_ask_ui`。
  - 新增 `in_process_initialize_params_preserve_native_ask_ui_capability` 测试。
- `codex-rs/app-server-client/src/remote.rs`
  - `RemoteAppServerConnectArgs` 增加 `native_ask_ui`。
  - remote initialize params 使用调用方传入的 `native_ask_ui`。
- `codex-rs/tui/src/lib.rs`
  - embedded 和 remote TUI app-server client 都传 `native_ask_ui: true`。
- `codex-rs/exec/src/lib.rs`、`codex-rs/tui/src/onboarding/auth.rs`
  - 非 TUI Ask UI 客户端保持 `native_ask_ui: false`。

验证：

```text
cd codex-rs && just fmt
cd codex-rs && just test -p codex-app-server-client in_process_initialize_params_preserve_native_ask_ui_capability
cd codex-rs && just test -p codex-app-server-client remote_server_request
cd codex-rs && just test -p codex-tui request_user_input
cd codex-rs && just fix -p codex-app-server-client -p codex-tui -p codex-exec
cd codex-rs && cargo build --bin vsp-codex
```

结果：

- app-server-client focused test：1 passed。
- app-server-client remote server request tests：2 passed。
- codex-tui request_user_input tests：78 passed，2596 skipped。
- `just fix -p codex-app-server-client -p codex-tui -p codex-exec`：通过。
- `cargo build --bin vsp-codex`：通过。
- `/home/heyx/Codex-VSP/codex-rs/target/debug/vsp-codex` 更新时间：`2026-05-26T23:18:00+08:00`。

当前状态：M9 已修复并重建入口，但仍需再次重启后真实复验原生 Ask UI。C3 不能验收。

## M9 追加验证：native Ask capability 真实链路覆盖

2026-05-26T23:38:44+08:00 继续修复中断后的收尾，补强了之前只验证 in-process 参数的覆盖缺口：

- 新增 `remote_initialize_preserves_native_ask_ui_capability`，模拟 remote app-server 并直接断言 initialize payload 中 `nativeAskUi=true`。
- 重新跑 app-server `request_user_input` 端到端集合，覆盖 native UI 投递、unsupported fallback、pending Ask resume replay。
- 更新 app-server turn_start 断言，从旧的 `Default or Plan mode` 改为新语义 `every collaboration mode`。

验证：

```text
cd codex-rs && just fmt
cd codex-rs && just test -p codex-app-server-client remote_initialize_preserves_native_ask_ui_capability
cd codex-rs && just test -p codex-app-server request_user_input
cd codex-rs && just test -p codex-app-server turn_start_uses_thread_feature_overrides_for_request_user_input_tool_description_v2
cd codex-rs && just test -p codex-tui request_user_input
cd codex-rs && just fix -p codex-app-server-client -p codex-app-server -p codex-tui -p codex-exec
cd codex-rs && cargo build --bin vsp-codex
```

结果：

- app-server-client remote initialize native Ask capability test：1 passed。
- app-server request_user_input tests：5 passed。
- app-server turn_start request_user_input description override test：1 passed。
- codex-tui request_user_input tests：78 passed，2596 skipped。
- scoped `just fix`：通过。
- `cargo build --bin vsp-codex`：通过。
- `/home/heyx/Codex-VSP/codex-rs/target/debug/vsp-codex` 更新时间：`2026-05-26T23:38:44+08:00`。

当前状态：源码、focused tests 和本地入口均已更新；仍需用户重启 `vsp-codex resume` 后进行真实 TUI native Ask UI 复验。若仍出现 fallback 文本，下一步应继续查 thread subscription/capability registration，而不是验收 C3。

## 第三次真实复验结果

2026-05-26T23:48:12+08:00，用户重启 `vsp-codex resume` 后，助手在普通 Default 会话中调用 `request_user_input` 做最小复验。

结果：

- TUI 显示原生 Ask UI。
- 用户在 Ask UI 中选择 `native_ask_ui`。
- `request_user_input` 返回结构化 answers，说明当前客户端已能接收并完成原生 Ask UI 请求。

结论：

- M8：普通 Default 会话可调用 `request_user_input`，通过。
- M9：TUI native Ask UI capability 修复通过。
- C3 仍不能最终验收，因为 M7 要求真实换电脑/跨设备 pending Ask resume 验收，用户此前明确说明在真实换电脑测试前不能验收通过。

新发现：

- Plan 在压缩或对话过长后可能被 LLM 忘记更新，需要追加提醒/恢复策略。
- Ask Tools UI 和交互逻辑还需要微调，需要进一步收集具体交互问题并追加修订。
