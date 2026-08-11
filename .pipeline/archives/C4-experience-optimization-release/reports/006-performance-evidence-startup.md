# C4/M6 Performance Evidence + Low-Risk Startup Fixes

Updated: 2026-05-28T13:04:23+08:00

## Summary

M6 收敛为最小可运行的 release performance evidence runner。没有实现 resume metadata cache、recent preview cache、progressive stale-cache verification，也没有做大型启动架构优化。

新增脚本：

- `scripts/collect_performance_evidence.py`

默认输出：

- `.pipeline/evidence/performance-startup.json`

## Covered Scenarios

Runner 输出结构化 JSON，每个场景包含 `scenario`、`command`、`status`、`durationMs`、`reason` 和 `details`。

- `cold_tui_start`
  - 通过伪终端启动 `vsp-codex --no-alt-screen`，剥离 ANSI/control sequences 后等待 meaningful visible text。
  - 如果平台没有伪终端、超时前无输出，或只观察到终端控制字节，记录 `skipped`，不伪造 duration。
- `resume_list_first_page`
  - 通过 app-server stdio JSON-RPC 执行 `initialize` 后请求 `thread/list` 第一页。
- `open_historical_session`
  - 需要真实历史线程时，通过 `CODEX_PERF_THREAD_ID` 和 `CODEX_PERF_EXISTING_CODEX_HOME` 指定。
  - 没有历史线程时记录 `skipped` 和原因。
- `vscode_app_server_initialize`
  - 通过 app-server stdio JSON-RPC 发送 VSCode-style `initialize` payload，记录响应耗时。

## Smoke Evidence

Command:

```sh
python3 scripts/collect_performance_evidence.py --timeout 3
```

Result:

- Passed; wrote `.pipeline/evidence/performance-startup.json`.
- `cold_tui_start`: `skipped`; only terminal control output was observed before timeout, so no duration was claimed.
- `resume_list_first_page`: `passed`, 7 ms in the smoke run after initialize.
- `open_historical_session`: `skipped`; no thread id was supplied and isolated evidence home had no historical session.
- `vscode_app_server_initialize`: `passed`, 286 ms in the smoke run.

Additional check:

```sh
python3 -m py_compile scripts/collect_performance_evidence.py
python3 -m json.tool .pipeline/evidence/performance-startup.json
```

Result: passed.

## Notes For M7

M7 can call the runner directly and archive `.pipeline/evidence/performance-startup.json` with preview release artifacts. For a real historical transcript measurement, run with:

```sh
CODEX_PERF_EXISTING_CODEX_HOME=/path/to/codex-home \
python3 scripts/collect_performance_evidence.py --thread-id <thread-id>
```
