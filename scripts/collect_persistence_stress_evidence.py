#!/usr/bin/env python3
"""Collect isolated multi-process SQLite and memory stress evidence for VSP Codex."""

from __future__ import annotations

import argparse
import concurrent.futures
import json
import os
import queue
import re
import sqlite3
import subprocess
import tempfile
import threading
import time
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[1]
LOCK_ERROR_RE = re.compile(
    r"database is (?:locked|busy)|SQLITE_(?:BUSY|LOCKED)", re.IGNORECASE
)


@dataclass
class WorkerResult:
    worker: int
    pid: int
    initialize_ms: int | None = None
    write_latencies_ms: list[int] = field(default_factory=list)
    thread_start_latencies_ms: list[int] = field(default_factory=list)
    metadata_write_latencies_ms: list[int] = field(default_factory=list)
    state_read_latencies_ms: list[int] = field(default_factory=list)
    compatibility_read_latencies_ms: list[int] = field(default_factory=list)
    rss_before_reads_kib: int | None = None
    rss_after_reads_kib: int | None = None
    error: str | None = None

    def to_json(self) -> dict[str, Any]:
        return {
            "worker": self.worker,
            "pid": self.pid,
            "initializeMs": self.initialize_ms,
            "writeLatencyMs": latency_summary(self.write_latencies_ms),
            "threadStartLatencyMs": latency_summary(self.thread_start_latencies_ms),
            "metadataWriteLatencyMs": latency_summary(self.metadata_write_latencies_ms),
            "stateReadLatencyMs": latency_summary(self.state_read_latencies_ms),
            "compatibilityReadLatencyMs": latency_summary(
                self.compatibility_read_latencies_ms
            ),
            "rssBeforeReadsKiB": self.rss_before_reads_kib,
            "rssAfterReadsKiB": self.rss_after_reads_kib,
            "rssDeltaKiB": rss_delta(self),
            "error": self.error,
        }


@dataclass
class WorkerProcess:
    index: int
    process: subprocess.Popen[str]
    stderr_path: Path
    stderr_file: Any
    output_queue: queue.Queue[dict[str, Any] | BaseException | None]
    stdout_thread: threading.Thread


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def elapsed_ms(start: float) -> int:
    return round((time.monotonic() - start) * 1000)


def percentile(values: list[int], fraction: float) -> int | None:
    if not values:
        return None
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, int((len(ordered) - 1) * fraction)))
    return ordered[index]


def latency_summary(values: list[int]) -> dict[str, int | None]:
    return {
        "count": len(values),
        "min": min(values) if values else None,
        "p50": percentile(values, 0.50),
        "p95": percentile(values, 0.95),
        "p99": percentile(values, 0.99),
        "max": max(values) if values else None,
    }


def rss_delta(result: WorkerResult) -> int | None:
    if result.rss_before_reads_kib is None or result.rss_after_reads_kib is None:
        return None
    return result.rss_after_reads_kib - result.rss_before_reads_kib


def write_config(codex_home: Path) -> None:
    codex_home.mkdir(parents=True, exist_ok=True)
    (codex_home / "config.toml").write_text(
        "\n".join(
            [
                'model = "gpt-5.4"',
                'model_provider = "stress"',
                'approval_policy = "never"',
                'sandbox_mode = "danger-full-access"',
                'preferred_auth_method = "apikey"',
                'cli_auth_credentials_store = "file"',
                'mcp_oauth_credentials_store = "file"',
                "",
                "[model_providers.stress]",
                'name = "Persistence Stress"',
                'base_url = "http://127.0.0.1:9/v1"',
                'env_key = "CODEX_STRESS_API_KEY"',
                'wire_api = "responses"',
                "request_max_retries = 0",
                "stream_max_retries = 0",
                "stream_idle_timeout_ms = 250",
                "",
                "[projects]",
                f'"{REPO_ROOT}" = {{ trust_level = "trusted" }}',
                "",
            ]
        ),
        encoding="utf-8",
    )


def app_server_env(codex_home: Path) -> dict[str, str]:
    env = os.environ.copy()
    env.update(
        {
            "CODEX_HOME": str(codex_home),
            "CODEX_SQLITE_HOME": str(codex_home),
            "CODEX_STRESS_API_KEY": "stress-placeholder",
            "NO_COLOR": "1",
            "RUST_LOG": "warn",
        }
    )
    return env


def start_worker(codex_bin: Path, codex_home: Path, index: int) -> WorkerProcess:
    stderr_path = codex_home / f"worker-{index}.stderr.log"
    stderr_file = stderr_path.open("w+", encoding="utf-8")
    process = subprocess.Popen(
        [str(codex_bin), "app-server"],
        cwd=REPO_ROOT,
        env=app_server_env(codex_home),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=stderr_file,
        text=True,
        encoding="utf-8",
        bufsize=1,
    )
    output_queue: queue.Queue[dict[str, Any] | BaseException | None] = queue.Queue()

    def read_stdout() -> None:
        try:
            if process.stdout is None:
                raise RuntimeError("app-server stdout is unavailable")
            for line in process.stdout:
                try:
                    output_queue.put(json.loads(line))
                except json.JSONDecodeError:
                    continue
        except BaseException as exc:
            output_queue.put(exc)
        finally:
            output_queue.put(None)

    stdout_thread = threading.Thread(
        target=read_stdout,
        name=f"vsp-persistence-stdout-{index}",
        daemon=True,
    )
    stdout_thread.start()
    return WorkerProcess(
        index,
        process,
        stderr_path,
        stderr_file,
        output_queue,
        stdout_thread,
    )


def send_request(
    worker: WorkerProcess,
    request_id: int,
    method: str,
    params: dict[str, Any] | None,
    timeout: float,
) -> tuple[dict[str, Any], int]:
    proc = worker.process
    if proc.stdin is None or proc.stdout is None:
        raise RuntimeError("app-server pipes are unavailable")
    start = time.monotonic()
    proc.stdin.write(
        json.dumps(
            {"id": request_id, "method": method, "params": params},
            separators=(",", ":"),
        )
        + "\n"
    )
    proc.stdin.flush()

    deadline = start + timeout
    while True:
        if proc.poll() is not None:
            raise RuntimeError(f"app-server exited with status {proc.returncode}")
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            break
        try:
            message = worker.output_queue.get(timeout=min(0.1, remaining))
        except queue.Empty:
            continue
        if message is None:
            raise RuntimeError("app-server stdout closed before the response arrived")
        if isinstance(message, BaseException):
            raise RuntimeError(
                f"app-server stdout reader failed: {message}"
            ) from message
        if message.get("id") != request_id:
            continue
        if "error" in message:
            raise RuntimeError(f"{method} failed: {message['error']}")
        return message, elapsed_ms(start)
    raise TimeoutError(f"timed out after {timeout}s waiting for {method}")


def initialize(worker: WorkerProcess, timeout: float) -> int:
    proc = worker.process
    _, duration = send_request(
        worker,
        1,
        "initialize",
        {
            "clientInfo": {
                "name": "vsp-persistence-stress",
                "title": "VSP Persistence Stress Evidence",
                "version": "1.0.0",
            },
            "capabilities": {
                "experimentalApi": True,
                "requestAttestation": False,
                "nativeAskUi": True,
            },
        },
        timeout,
    )
    if proc.stdin is None:
        raise RuntimeError("app-server stdin is unavailable")
    proc.stdin.write('{"method":"initialized","params":null}\n')
    proc.stdin.flush()
    return duration


def read_rss_kib(pid: int) -> int | None:
    try:
        for line in (
            Path(f"/proc/{pid}/status").read_text(encoding="utf-8").splitlines()
        ):
            if line.startswith("VmRSS:"):
                return int(line.split()[1])
    except (FileNotFoundError, PermissionError, ValueError):
        pass
    return None


def run_worker(
    worker: WorkerProcess,
    *,
    write_iterations: int,
    read_iterations: int,
    post_start_delay_ms: int,
    timeout: float,
    start_barrier: threading.Barrier,
    read_barrier: threading.Barrier,
) -> WorkerResult:
    proc = worker.process
    result = WorkerResult(worker=worker.index, pid=proc.pid)
    try:
        result.initialize_ms = initialize(worker, timeout)
        start_barrier.wait(timeout=timeout)
        request_id = 2
        for iteration in range(write_iterations):
            response, start_duration = send_request(
                worker,
                request_id,
                "thread/start",
                {"model": "gpt-5.4", "cwd": str(REPO_ROOT)},
                timeout,
            )
            request_id += 1
            try:
                thread_id = response["result"]["thread"]["id"]
            except (KeyError, TypeError) as exc:
                raise RuntimeError(
                    "thread/start response did not contain a thread id"
                ) from exc
            if post_start_delay_ms:
                time.sleep(post_start_delay_ms / 1000)
            _, name_duration = send_request(
                worker,
                request_id,
                "thread/name/set",
                {
                    "threadId": thread_id,
                    "name": f"stress-{worker.index}-{iteration}",
                },
                timeout,
            )
            result.write_latencies_ms.append(start_duration + name_duration)
            result.thread_start_latencies_ms.append(start_duration)
            result.metadata_write_latencies_ms.append(name_duration)
            request_id += 1

        read_barrier.wait(timeout=timeout)
        result.rss_before_reads_kib = read_rss_kib(proc.pid)
        for _ in range(read_iterations):
            _, duration = send_request(
                worker,
                request_id,
                "thread/list",
                {
                    "cursor": None,
                    "limit": 20,
                    "archived": False,
                    "useStateDbOnly": True,
                },
                timeout,
            )
            result.state_read_latencies_ms.append(duration)
            request_id += 1
        for _ in range(read_iterations):
            _, duration = send_request(
                worker,
                request_id,
                "thread/list",
                {
                    "cursor": None,
                    "limit": 20,
                    "archived": False,
                    "useStateDbOnly": False,
                },
                timeout,
            )
            result.compatibility_read_latencies_ms.append(duration)
            request_id += 1
        result.rss_after_reads_kib = read_rss_kib(proc.pid)
    except Exception as exc:
        result.error = f"{type(exc).__name__}: {exc}"
        try:
            start_barrier.abort()
            read_barrier.abort()
        except threading.BrokenBarrierError:
            pass
    return result


def file_sizes(codex_home: Path) -> dict[str, int]:
    names = [
        "logs_2.sqlite",
        "logs_2.sqlite-wal",
        "logs_2.sqlite-shm",
        "state_5.sqlite",
        "state_5.sqlite-wal",
        "state_5.sqlite-shm",
    ]
    return {
        name: (codex_home / name).stat().st_size if (codex_home / name).exists() else 0
        for name in names
    }


def sqlite_evidence(path: Path, table: str) -> dict[str, Any]:
    if not path.exists():
        return {"exists": False}
    result: dict[str, Any] = {"exists": True, "bytes": path.stat().st_size}
    try:
        connection = sqlite3.connect(f"file:{path}?mode=ro", uri=True, timeout=5)
        try:
            result["quickCheck"] = connection.execute("PRAGMA quick_check").fetchone()[
                0
            ]
            result["journalMode"] = connection.execute(
                "PRAGMA journal_mode"
            ).fetchone()[0]
            result["pageCount"] = connection.execute("PRAGMA page_count").fetchone()[0]
            result["freelistCount"] = connection.execute(
                "PRAGMA freelist_count"
            ).fetchone()[0]
            result["rowCount"] = connection.execute(
                f"SELECT COUNT(*) FROM {table}"
            ).fetchone()[0]
        finally:
            connection.close()
    except (sqlite3.Error, TypeError) as exc:
        result["error"] = str(exc)
    return result


def terminate_worker(worker: WorkerProcess) -> None:
    proc = worker.process
    if proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=5)
    worker.stderr_file.close()
    worker.stdout_thread.join(timeout=1)


def collect(args: argparse.Namespace) -> dict[str, Any]:
    codex_bin = Path(args.codex_bin).resolve()
    if not codex_bin.is_file() or not os.access(codex_bin, os.X_OK):
        raise RuntimeError(f"not an executable file: {codex_bin}")

    with tempfile.TemporaryDirectory(prefix="vsp-persistence-stress-") as tmp:
        codex_home = Path(tmp)
        write_config(codex_home)
        workers = [
            start_worker(codex_bin, codex_home, index)
            for index in range(args.processes)
        ]
        start_barrier = threading.Barrier(args.processes)
        read_barrier = threading.Barrier(args.processes)
        try:
            with concurrent.futures.ThreadPoolExecutor(
                max_workers=args.processes
            ) as pool:
                futures = [
                    pool.submit(
                        run_worker,
                        worker,
                        write_iterations=args.write_iterations,
                        read_iterations=args.read_iterations,
                        post_start_delay_ms=args.post_start_delay_ms,
                        timeout=args.timeout,
                        start_barrier=start_barrier,
                        read_barrier=read_barrier,
                    )
                    for worker in workers
                ]
                results = [future.result() for future in futures]

            before_idle = file_sizes(codex_home)
            time.sleep(args.idle_seconds)
            after_idle = file_sizes(codex_home)
        finally:
            for worker in workers:
                terminate_worker(worker)

        stderr_by_worker = {
            str(worker.index): worker.stderr_path.read_text(
                encoding="utf-8", errors="replace"
            )
            for worker in workers
        }
        lock_errors = [
            {"worker": int(index), "matches": LOCK_ERROR_RE.findall(stderr)}
            for index, stderr in stderr_by_worker.items()
            if LOCK_ERROR_RE.search(stderr)
        ]
        state_db = sqlite_evidence(codex_home / "state_5.sqlite", "threads")
        logs_db = sqlite_evidence(codex_home / "logs_2.sqlite", "logs")
        errors = [result.error for result in results if result.error]
        integrity_ok = all(
            evidence.get("quickCheck") in (None, "ok") and "error" not in evidence
            for evidence in (state_db, logs_db)
        )
        status = (
            "passed" if not errors and not lock_errors and integrity_ok else "failed"
        )

        all_write_latencies = [
            latency for result in results for latency in result.write_latencies_ms
        ]
        all_state_read_latencies = [
            latency for result in results for latency in result.state_read_latencies_ms
        ]
        all_compatibility_read_latencies = [
            latency
            for result in results
            for latency in result.compatibility_read_latencies_ms
        ]
        all_thread_start_latencies = [
            latency
            for result in results
            for latency in result.thread_start_latencies_ms
        ]
        all_metadata_write_latencies = [
            latency
            for result in results
            for latency in result.metadata_write_latencies_ms
        ]
        return {
            "schemaVersion": 1,
            "generatedAt": now_iso(),
            "runId": str(uuid.uuid4()),
            "status": status,
            "codexBin": str(codex_bin),
            "parameters": {
                "processes": args.processes,
                "writeIterationsPerProcess": args.write_iterations,
                "readIterationsPerProcess": args.read_iterations,
                "postStartDelayMs": args.post_start_delay_ms,
                "timeoutSeconds": args.timeout,
                "idleSeconds": args.idle_seconds,
                "sharedCodexHome": True,
            },
            "aggregate": {
                "writeLatencyMs": latency_summary(all_write_latencies),
                "threadStartLatencyMs": latency_summary(all_thread_start_latencies),
                "metadataWriteLatencyMs": latency_summary(all_metadata_write_latencies),
                "stateReadLatencyMs": latency_summary(all_state_read_latencies),
                "compatibilityReadLatencyMs": latency_summary(
                    all_compatibility_read_latencies
                ),
                "maxRssDeltaKiB": max(
                    (
                        delta
                        for result in results
                        if (delta := rss_delta(result)) is not None
                    ),
                    default=None,
                ),
            },
            "workers": [result.to_json() for result in results],
            "sqlite": {"state_5": state_db, "logs_2": logs_db},
            "idleFileSizes": {
                "before": before_idle,
                "after": after_idle,
                "growth": {
                    name: after_idle[name] - before_idle[name] for name in before_idle
                },
            },
            "lockErrors": lock_errors,
            "workerErrors": errors,
            "stderrTailByWorker": {
                index: stderr[-2000:]
                for index, stderr in stderr_by_worker.items()
                if stderr
            },
        }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Stress multiple VSP app-server processes sharing one isolated CODEX_HOME."
    )
    parser.add_argument("--codex-bin", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--processes", type=int, default=6)
    parser.add_argument("--write-iterations", type=int, default=40)
    parser.add_argument("--read-iterations", type=int, default=400)
    parser.add_argument("--post-start-delay-ms", type=int, default=0)
    parser.add_argument("--timeout", type=float, default=15.0)
    parser.add_argument("--idle-seconds", type=float, default=5.0)
    args = parser.parse_args()

    if args.processes < 1:
        parser.error("--processes must be at least 1")
    if args.write_iterations < 1 or args.read_iterations < 1:
        parser.error("iteration counts must be positive")
    if args.post_start_delay_ms < 0:
        parser.error("--post-start-delay-ms must be non-negative")

    artifact = collect(args)
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(artifact, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(output)
    return 0 if artifact["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
