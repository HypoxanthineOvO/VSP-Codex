#!/usr/bin/env python3
"""Collect preview-release startup timing evidence for VSP Codex.

The script records reproducible measurements when the local environment can run
the target scenario. If a binary, pseudo-terminal, or seed thread is missing, it
records a skipped scenario with the reason instead of fabricating data.
"""

from __future__ import annotations

import argparse
import fcntl
import json
import os
import pty
import re
import select
import shutil
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time
import uuid
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT = REPO_ROOT / ".pipeline" / "evidence" / "performance-startup.json"
ANSI_CONTROL_RE = re.compile(
    r"\x1b\][^\x07]*(?:\x07|\x1b\\)"
    r"|\x1b\[[0-?]*[ -/]*[@-~]"
    r"|\x1b[@-Z\\-_]"
)
NON_VISIBLE_CONTROL_RE = re.compile(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f-\x9f]")
TIMESTAMPED_LOG_LINE_RE = re.compile(
    r"^\d{4}-\d{2}-\d{2}T\S+\s+(?:TRACE|DEBUG|INFO|WARN|ERROR)\s+"
)
TERMINAL_PROBE_RESPONSES = (
    b"\x1b[1;1R"
    b"\x1b]11;rgb:1111/1111/1111\x07"
    b"\x1b[?64;1;2c"
    b"\x1b]10;rgb:eeee/eeee/eeee\x1b\\"
    b"\x1b[?7u"
)


@dataclass
class ScenarioResult:
    scenario: str
    command: list[str]
    status: str
    duration_ms: int | None
    reason: str | None = None
    details: dict[str, Any] | None = None

    def to_json(self) -> dict[str, Any]:
        return {
            "scenario": self.scenario,
            "command": self.command,
            "status": self.status,
            "durationMs": self.duration_ms,
            "reason": self.reason,
            "details": self.details or {},
        }


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def elapsed_ms(start: float) -> int:
    return round((time.monotonic() - start) * 1000)


def visible_terminal_text(output: str) -> str:
    without_ansi = ANSI_CONTROL_RE.sub("", output)
    without_controls = NON_VISIBLE_CONTROL_RE.sub("", without_ansi)
    lines = [
        line.strip()
        for line in without_controls.replace("\r", "\n").splitlines()
        if line.strip()
    ]
    return "\n".join(lines)


def user_visible_terminal_text(output: str) -> str:
    lines = []
    for line in visible_terminal_text(output).splitlines():
        if line.startswith(
            "WARNING: proceeding, even though we could not create PATH aliases:"
        ):
            continue
        if TIMESTAMPED_LOG_LINE_RE.match(line):
            continue
        lines.append(line)
    return "\n".join(lines)


def resolve_codex_bin(explicit: str | None) -> Path | None:
    candidates: list[Path] = []
    if explicit:
        candidates.append(Path(explicit))
    env_bin = os.environ.get("CODEX_PERF_CODEX_BIN")
    if env_bin:
        candidates.append(Path(env_bin))
    candidates.extend(
        [
            REPO_ROOT / "codex-rs" / "target" / "debug" / "vsp-codex",
            REPO_ROOT / "codex-rs" / "target" / "release" / "vsp-codex",
        ]
    )
    path_bin = shutil.which("vsp-codex") or shutil.which("codex")
    if path_bin:
        candidates.append(Path(path_bin))

    for candidate in candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return candidate.resolve()
    return None


def base_env(codex_home: Path, extra: dict[str, str] | None = None) -> dict[str, str]:
    env = os.environ.copy()
    env["CODEX_HOME"] = str(codex_home)
    env["CODEX_SQLITE_HOME"] = str(codex_home)
    env["NO_COLOR"] = "1"
    env.setdefault("RUST_LOG", "warn")
    if extra:
        env.update(extra)
    return env


def write_minimal_config(codex_home: Path) -> None:
    codex_home.mkdir(parents=True, exist_ok=True)
    (codex_home / "config.toml").write_text(
        "\n".join(
            [
                'model = "perf-baseline"',
                'model_provider = "perf-baseline"',
                'approval_policy = "never"',
                'sandbox_mode = "workspace-write"',
                'preferred_auth_method = "apikey"',
                'cli_auth_credentials_store = "file"',
                'mcp_oauth_credentials_store = "file"',
                "",
                "[model_providers.perf-baseline]",
                'name = "Perf Baseline"',
                'base_url = "http://127.0.0.1:9/v1"',
                'env_key = "CODEX_PERF_API_KEY"',
                'wire_api = "responses"',
                "request_max_retries = 0",
                "stream_max_retries = 0",
                "stream_idle_timeout_ms = 250",
                "",
                "[projects]",
                f'"{str(REPO_ROOT)}" = {{ trust_level = "trusted" }}',
                "",
            ]
        ),
        encoding="utf-8",
    )


def run_pty_until_output(
    command: list[str],
    *,
    env: dict[str, str],
    cwd: Path,
    timeout: float,
    input_bytes: bytes = b"",
) -> tuple[str, int | None, str]:
    start = time.monotonic()
    pid, fd = pty.fork()
    if pid == 0:
        try:
            os.chdir(cwd)
            os.execvpe(command[0], command, env)
        except BaseException as exc:
            print(f"exec failed: {exc}", file=sys.stderr)
            os._exit(127)

    output = bytearray()
    status: int | None = None
    try:
        fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 120, 0, 0))
        tty_attributes = termios.tcgetattr(fd)
        tty_attributes[3] &= ~termios.ECHO
        termios.tcsetattr(fd, termios.TCSANOW, tty_attributes)
        while time.monotonic() - start < timeout:
            ready, _, _ = select.select([fd], [], [], 0.05)
            if ready:
                try:
                    chunk = os.read(fd, 4096)
                except OSError:
                    break
                if not chunk:
                    break
                output.extend(chunk)
                decoded = output.decode("utf-8", errors="replace")
                if user_visible_terminal_text(decoded):
                    break
            if input_bytes and (
                b"\x1b[6n" in output or time.monotonic() - start >= 0.05
            ):
                os.write(fd, input_bytes)
                input_bytes = b""
            finished_pid, raw_status = os.waitpid(pid, os.WNOHANG)
            if finished_pid == pid:
                status = raw_status
                break
        if status is None:
            try:
                os.kill(pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                _, status = os.waitpid(pid, 0)
            except ChildProcessError:
                status = None
    finally:
        try:
            os.close(fd)
        except OSError:
            pass

    return output.decode("utf-8", errors="replace"), status, "completed"


def measure_tui_first_output(
    scenario: str,
    command: list[str],
    *,
    env: dict[str, str],
    timeout: float,
    input_bytes: bytes = b"",
) -> ScenarioResult:
    if not hasattr(pty, "fork"):
        return ScenarioResult(
            scenario=scenario,
            command=command,
            status="skipped",
            duration_ms=None,
            reason="pty.fork is unavailable on this platform",
        )

    start = time.monotonic()
    try:
        output, raw_status, _ = run_pty_until_output(
            command,
            env=env,
            cwd=REPO_ROOT,
            timeout=timeout,
            input_bytes=input_bytes,
        )
    except Exception as exc:
        return ScenarioResult(
            scenario=scenario,
            command=command,
            status="failed",
            duration_ms=elapsed_ms(start),
            reason=str(exc),
        )

    visible_sample = user_visible_terminal_text(output)
    observed_visible_text = bool(visible_sample)
    status = "passed" if observed_visible_text else "skipped"
    reason = (
        None
        if observed_visible_text
        else "only terminal control output observed before timeout"
        if output.strip()
        else "no terminal output observed before timeout"
    )
    return ScenarioResult(
        scenario=scenario,
        command=command,
        status=status,
        duration_ms=elapsed_ms(start) if observed_visible_text else None,
        reason=reason,
        details={
            "exitStatus": raw_status,
            "observedBytes": len(output.encode("utf-8", errors="replace")),
            "sample": output[-500:],
            "visibleSample": visible_sample[-500:],
        },
    )


def send_jsonrpc(proc: subprocess.Popen[str], payload: dict[str, Any]) -> None:
    assert proc.stdin is not None
    proc.stdin.write(json.dumps(payload, separators=(",", ":")) + "\n")
    proc.stdin.flush()


def read_jsonrpc_until_id(
    proc: subprocess.Popen[str],
    request_id: int,
    *,
    timeout: float,
) -> dict[str, Any]:
    assert proc.stdout is not None
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        line = proc.stdout.readline()
        if not line:
            break
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("id") == request_id:
            return message
    raise TimeoutError(f"timed out waiting for JSON-RPC id {request_id}")


def start_app_server(
    codex_bin: Path,
    codex_home: Path,
) -> subprocess.Popen[str]:
    return subprocess.Popen(
        [str(codex_bin), "app-server"],
        cwd=REPO_ROOT,
        env=base_env(codex_home, {"CODEX_PERF_API_KEY": "perf-placeholder"}),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
    )


def initialize_app_server(
    proc: subprocess.Popen[str],
    *,
    client_name: str = "codex_vscode",
    request_id: int = 1,
    timeout: float,
) -> tuple[dict[str, Any], int]:
    request = {
        "method": "initialize",
        "id": request_id,
        "params": {
            "clientInfo": {
                "name": client_name,
                "title": "VSP Codex Performance Evidence",
                "version": "0.0.0",
            },
            "capabilities": {
                "experimentalApi": True,
                "requestAttestation": False,
                "nativeAskUi": True,
                "optOutNotificationMethods": [
                    "command/exec/outputDelta",
                    "item/agentMessage/delta",
                    "item/reasoning/summaryTextDelta",
                    "item/reasoning/textDelta",
                ],
            },
        },
    }
    start = time.monotonic()
    send_jsonrpc(proc, request)
    response = read_jsonrpc_until_id(proc, request_id, timeout=timeout)
    duration = elapsed_ms(start)
    send_jsonrpc(proc, {"method": "initialized", "params": None})
    return response, duration


def measure_initialize(codex_bin: Path, timeout: float) -> ScenarioResult:
    command = [str(codex_bin), "app-server"]
    with tempfile.TemporaryDirectory(prefix="codex-perf-home-") as tmp:
        codex_home = Path(tmp)
        write_minimal_config(codex_home)
        proc = start_app_server(codex_bin, codex_home)
        try:
            try:
                response, duration = initialize_app_server(proc, timeout=timeout)
            except Exception as exc:
                return ScenarioResult(
                    scenario="vscode_app_server_initialize",
                    command=command,
                    status="failed",
                    duration_ms=None,
                    reason=str(exc),
                    details={"stderr": read_stderr(proc)},
                )
            return ScenarioResult(
                scenario="vscode_app_server_initialize",
                command=command,
                status="passed" if "result" in response else "failed",
                duration_ms=duration,
                reason=response.get("error", {}).get("message"),
                details={"responseKeys": sorted(response.keys())},
            )
        finally:
            terminate_process(proc)


def measure_thread_list(
    codex_bin: Path, timeout: float
) -> tuple[ScenarioResult, str | None]:
    command = [str(codex_bin), "app-server", "thread/list"]
    with tempfile.TemporaryDirectory(prefix="codex-perf-home-") as tmp:
        codex_home = Path(tmp)
        write_minimal_config(codex_home)
        proc = start_app_server(codex_bin, codex_home)
        try:
            try:
                _, init_duration = initialize_app_server(proc, timeout=timeout)
                request_id = 2
                request = {
                    "method": "thread/list",
                    "id": request_id,
                    "params": {
                        "cursor": None,
                        "limit": 20,
                        "sortKey": None,
                        "sortDirection": None,
                        "modelProviders": None,
                        "sourceKinds": None,
                        "archived": None,
                        "cwd": None,
                        "useStateDbOnly": False,
                        "searchTerm": None,
                    },
                }
                start = time.monotonic()
                send_jsonrpc(proc, request)
                response = read_jsonrpc_until_id(proc, request_id, timeout=timeout)
                duration = elapsed_ms(start)
            except Exception as exc:
                return (
                    ScenarioResult(
                        scenario="resume_list_first_page",
                        command=command,
                        status="failed",
                        duration_ms=None,
                        reason=str(exc),
                        details={"stderr": read_stderr(proc)},
                    ),
                    None,
                )

            thread_id = first_thread_id(response)
            return (
                ScenarioResult(
                    scenario="resume_list_first_page",
                    command=command,
                    status="passed" if "result" in response else "failed",
                    duration_ms=duration,
                    reason=response.get("error", {}).get("message"),
                    details={
                        "initializeDurationMs": init_duration,
                        "threadCount": count_threads(response),
                        "hasNextCursor": bool(
                            response.get("result", {}).get("nextCursor")
                        ),
                    },
                ),
                thread_id,
            )
        finally:
            terminate_process(proc)


def measure_historical_open(
    codex_bin: Path,
    timeout: float,
    thread_id: str | None,
    explicit_thread_id: str | None,
) -> ScenarioResult:
    selected_thread_id = explicit_thread_id or thread_id
    command = [str(codex_bin), "app-server", "thread/resume"]
    if not selected_thread_id:
        return ScenarioResult(
            scenario="open_historical_session",
            command=command,
            status="skipped",
            duration_ms=None,
            reason=(
                "no thread id was supplied and isolated evidence CODEX_HOME did not contain "
                "a historical session"
            ),
        )

    codex_home_env = os.environ.get("CODEX_PERF_EXISTING_CODEX_HOME")
    if not codex_home_env:
        return ScenarioResult(
            scenario="open_historical_session",
            command=command,
            status="skipped",
            duration_ms=None,
            reason=(
                "set CODEX_PERF_THREAD_ID and CODEX_PERF_EXISTING_CODEX_HOME to measure "
                "a real historical transcript"
            ),
            details={"threadId": selected_thread_id},
        )

    codex_home = Path(codex_home_env)
    proc = start_app_server(codex_bin, codex_home)
    try:
        try:
            _, init_duration = initialize_app_server(proc, timeout=timeout)
            request_id = 2
            request = {
                "method": "thread/resume",
                "id": request_id,
                "params": {
                    "threadId": selected_thread_id,
                    "history": None,
                    "path": None,
                    "model": None,
                    "modelProvider": None,
                    "serviceTier": None,
                    "cwd": None,
                    "runtimeWorkspaceRoots": None,
                    "approvalPolicy": None,
                    "approvalsReviewer": None,
                    "sandbox": None,
                    "permissions": None,
                    "config": None,
                    "baseInstructions": None,
                    "developerInstructions": None,
                    "personality": None,
                    "excludeTurns": False,
                    "persistFullHistory": False,
                },
            }
            start = time.monotonic()
            send_jsonrpc(proc, request)
            response = read_jsonrpc_until_id(proc, request_id, timeout=timeout)
            duration = elapsed_ms(start)
        except Exception as exc:
            return ScenarioResult(
                scenario="open_historical_session",
                command=command,
                status="failed",
                duration_ms=None,
                reason=str(exc),
                details={"stderr": read_stderr(proc), "threadId": selected_thread_id},
            )

        return ScenarioResult(
            scenario="open_historical_session",
            command=command,
            status="passed" if "result" in response else "failed",
            duration_ms=duration,
            reason=response.get("error", {}).get("message"),
            details={
                "initializeDurationMs": init_duration,
                "threadId": selected_thread_id,
                "turnCount": len(
                    response.get("result", {}).get("thread", {}).get("turns", [])
                ),
            },
        )
    finally:
        terminate_process(proc)


def first_thread_id(response: dict[str, Any]) -> str | None:
    threads = response.get("result", {}).get("data")
    if isinstance(threads, list) and threads:
        first = threads[0]
        if isinstance(first, dict) and isinstance(first.get("id"), str):
            return first["id"]
    return None


def count_threads(response: dict[str, Any]) -> int | None:
    threads = response.get("result", {}).get("data")
    if isinstance(threads, list):
        return len(threads)
    return None


def read_stderr(proc: subprocess.Popen[str]) -> str:
    if proc.stderr is None:
        return ""
    if hasattr(select, "select"):
        try:
            ready, _, _ = select.select([proc.stderr], [], [], 0)
            if not ready:
                return ""
        except Exception:
            return ""
    try:
        return proc.stderr.read(2000)
    except Exception:
        return ""


def terminate_process(proc: subprocess.Popen[str]) -> None:
    if proc.poll() is not None:
        return
    proc.terminate()
    try:
        proc.wait(timeout=2)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=2)


def collect(args: argparse.Namespace) -> dict[str, Any]:
    codex_bin = resolve_codex_bin(args.codex_bin)
    results: list[ScenarioResult] = []

    if codex_bin is None:
        skipped_command = [args.codex_bin or "vsp-codex"]
        for scenario in [
            "cold_tui_start",
            "resume_list_first_page",
            "open_historical_session",
            "vscode_app_server_initialize",
        ]:
            results.append(
                ScenarioResult(
                    scenario=scenario,
                    command=skipped_command,
                    status="skipped",
                    duration_ms=None,
                    reason="no executable vsp-codex/codex binary found",
                )
            )
        return build_artifact(args, codex_bin, results)

    with tempfile.TemporaryDirectory(prefix="codex-perf-home-") as tmp:
        codex_home = Path(tmp)
        write_minimal_config(codex_home)
        tui_env = base_env(codex_home, {"CODEX_PERF_API_KEY": "perf-placeholder"})
        results.append(
            measure_tui_first_output(
                "cold_tui_start",
                [
                    str(codex_bin),
                    "--no-alt-screen",
                    "--config",
                    "features={}",
                ],
                env=tui_env,
                timeout=args.timeout,
                input_bytes=TERMINAL_PROBE_RESPONSES,
            )
        )

    thread_list_result, discovered_thread_id = measure_thread_list(
        codex_bin, args.timeout
    )
    results.append(thread_list_result)
    results.append(
        measure_historical_open(
            codex_bin,
            args.timeout,
            discovered_thread_id,
            args.thread_id or os.environ.get("CODEX_PERF_THREAD_ID"),
        )
    )
    results.append(measure_initialize(codex_bin, args.timeout))

    return build_artifact(args, codex_bin, results)


def build_artifact(
    args: argparse.Namespace,
    codex_bin: Path | None,
    results: list[ScenarioResult],
) -> dict[str, Any]:
    return {
        "schemaVersion": 1,
        "generatedAt": now_iso(),
        "repoRoot": str(REPO_ROOT),
        "codexBin": str(codex_bin) if codex_bin else None,
        "timeoutSeconds": args.timeout,
        "runId": str(uuid.uuid4()),
        "scenarios": [result.to_json() for result in results],
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Collect VSP Codex startup performance evidence."
    )
    parser.add_argument(
        "--codex-bin",
        help="Path to a vsp-codex/codex binary. Defaults to target/debug, target/release, then PATH.",
    )
    parser.add_argument(
        "--output",
        default=str(DEFAULT_OUTPUT),
        help=f"Artifact path. Defaults to {DEFAULT_OUTPUT}",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=10.0,
        help="Per-scenario timeout in seconds.",
    )
    parser.add_argument(
        "--thread-id",
        help="Existing thread id to use for the historical transcript open scenario.",
    )
    args = parser.parse_args()

    artifact = collect(args)
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(artifact, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
