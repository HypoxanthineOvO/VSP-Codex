# M3 Resume Metadata Cache + Progressive Verification

完成时间：2026-05-28T22:00:16+08:00

## 结果

M3 已完成。Resume/Fork 历史列表现在先走轻量 SQLite metadata cache 首屏，再用 canonical app-server 扫描结果做后台校验；如果 cache 中的名称、摘要、cwd、时间等字段过期，列表会用同一行更新，不会重复插入 session。

## 具体产出

- 新增 picker page load kind：`CachedFirstPaint` 和 `Canonical`。
- 首次打开 resume picker 时请求 `use_state_db_only=true`：
  - 先从 state DB/SQLite metadata 返回可用的 thread id、canonical name、preview、cwd、branch、created/updated 时间。
  - 不读取完整 rollout JSONL，不提前加载 transcript preview。
- 首屏返回后自动发同 token canonical verification：
  - 第二轮 `use_state_db_only=false` 走完整 app-server thread list。
  - stale cache 行按 `path` 或 `thread_id` 替换，保持选中项位置。
  - 新行会补入，verification 失败时静默保留首屏结果。
- 后续分页、搜索补页、排序/过滤重载继续走 canonical 路径，避免 stale cache 成为最终事实来源。
- 保留 lazy preview/transcript 行为：
  - `/resume` 首屏不 eager load recent transcript。
  - 用户 `Ctrl+E` 展开时才加载 recent preview。
  - 用户 `Ctrl+T` 打开 transcript 时才加载完整 transcript。
- 底层缓存事实来源复用已有 state DB metadata，而不是新增一个包含 transcript 正文的 TUI 私有 cache 文件；这样避免存储敏感正文，也让 Rename 后的 canonical name 继续由 thread metadata 统一驱动。

## 主要文件

- `codex-rs/tui/src/resume_picker.rs`
- 底层能力依赖并验证：
  - `codex-rs/app-server/src/request_processors/thread_processor.rs`
  - `codex-rs/thread-store/src/local/list_threads.rs`
  - `codex-rs/thread-store/src/local/read_thread.rs`
  - `codex-rs/rollout/src/recorder_tests.rs`

## 验证

- `just test -p codex-tui resume_picker`：98/98 通过。
- `just test -p codex-rollout state_db_only`：1/1 通过。
- `just test -p codex-app-server thread_list_state_db_only_returns_sqlite_without_jsonl_repair`：1/1 通过。
- `just test -p codex-thread-store read_thread_applies_sqlite_thread_name read_thread_uses_sqlite_metadata_for_rollout_without_user_preview list_threads_preserves_sqlite_title_search_results`：3/3 通过。
- `git diff --check`：通过。

## 性能结论

- 首屏路径不再必须扫描/修复 JSONL rollout：`thread/list` 首轮明确使用 `use_state_db_only=true`。
- canonical verification 在首屏之后运行，保证 stale metadata 会被修正，但不阻塞初始可见列表。
- 本轮没有新增独立性能脚本；用行为测试和 app-server state-db-only 回归测试证明快路径存在且不会触发 JSONL repair。

## Worker Separation 说明

项目配置为 strict worker separation，且 `/hw:resume` 已记录授权；但当前 Codex 子代理工具的运行约束要求用户显式请求 sub-agent/parallel agent work 才能 spawn。M3 由主代理执行，并保留命令级验证证据。

## 风险和后续

- SQLite metadata cache 是快路径，不是 canonical truth；如果 state DB 未初始化或无记录，首屏会正常降级为空/较少结果，然后 canonical verification 补齐。
- 当前 recent preview 仍是按需加载，不做持久 transcript preview cache；这是有意选择，避免把更多对话正文复制到额外缓存文件。
