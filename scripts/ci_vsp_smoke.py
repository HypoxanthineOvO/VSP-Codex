#!/usr/bin/env python3
"""Validate the long-lived runtime contracts required by VSP GitLab CI."""

from __future__ import annotations

import argparse
import fcntl
import json
import os
import pty
import re
import select
import struct
import subprocess
import sys
import tempfile
import termios
import time
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
ANSI_CONTROL_RE = re.compile(
    r"\x1b\][^\x07]*(?:\x07|\x1b\\)"
    r"|\x1b\[[0-?]*[ -/]*[@-~]"
    r"|\x1b[@-Z\\-_]"
)
NON_VISIBLE_CONTROL_RE = re.compile(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f-\x9f]")
LOG_LINE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}T\S+\s+(?:TRACE|DEBUG|INFO|WARN|ERROR)\s+")
TERMINAL_PROBE_RESPONSES = (
    b"\x1b[1;1R"
    b"\x1b]11;rgb:1111/1111/1111\x07"
    b"\x1b[?64;1;2c"
    b"\x1b]10;rgb:eeee/eeee/eeee\x1b\\"
    b"\x1b[?7u"
)


def isolated_env(codex_home: Path) -> dict[str, str]:
    env = os.environ.copy()
    env.update(
        {
            "CODEX_HOME": str(codex_home),
            "CODEX_SQLITE_HOME": str(codex_home),
            "NO_COLOR": "1",
            "OPENAI_API_KEY": "ci-placeholder",
            "RUST_LOG": "warn",
        }
    )
    return env


def command_check(
    binary: Path,
    arguments: list[str],
    env: dict[str, str],
    timeout: float,
) -> None:
    command = [str(binary), *arguments]
    try:
        result = subprocess.run(
            command,
            cwd=REPO_ROOT,
            env=env,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise RuntimeError(f"command failed: {' '.join(command)}: {exc}") from exc

    output = result.stdout.strip() or result.stderr.strip()
    if result.returncode != 0 or not output:
        raise RuntimeError(
            f"command exited with {result.returncode} or produced no output: {' '.join(command)}"
        )
    print(f"ok: {' '.join(arguments)}")


def visible_terminal_text(output: bytes) -> str:
    decoded = output.decode("utf-8", errors="replace")
    without_ansi = ANSI_CONTROL_RE.sub("", decoded)
    without_controls = NON_VISIBLE_CONTROL_RE.sub("", without_ansi)
    lines = []
    for line in without_controls.replace("\r", "\n").splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith(
            "WARNING: proceeding, even though we could not create PATH aliases:"
        ):
            continue
        if LOG_LINE_RE.match(stripped):
            continue
        lines.append(stripped)
    return "\n".join(lines)


def tui_start_check(binary: Path, env: dict[str, str], timeout: float) -> None:
    command = [str(binary)]
    master_fd, slave_fd = pty.openpty()
    output = bytearray()
    probes_sent = False
    deadline = time.monotonic() + timeout
    try:
        fcntl.ioctl(slave_fd, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 120, 0, 0))
        attributes = termios.tcgetattr(slave_fd)
        attributes[3] &= ~termios.ECHO
        termios.tcsetattr(slave_fd, termios.TCSANOW, attributes)
        process = subprocess.Popen(
            command,
            cwd=REPO_ROOT,
            env=env,
            stdin=slave_fd,
            stdout=slave_fd,
            stderr=slave_fd,
            start_new_session=True,
        )
        os.close(slave_fd)
        slave_fd = -1

        while time.monotonic() < deadline:
            ready, _, _ = select.select([master_fd], [], [], 0.05)
            if ready:
                try:
                    chunk = os.read(master_fd, 4096)
                except OSError:
                    break
                if not chunk:
                    break
                output.extend(chunk)
                if visible_terminal_text(output):
                    print("ok: interactive TUI rendered visible output")
                    return
            if not probes_sent:
                try:
                    os.write(master_fd, TERMINAL_PROBE_RESPONSES)
                    probes_sent = True
                except OSError:
                    pass
            if process.poll() is not None:
                break
    finally:
        if "process" in locals() and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=2)
        if slave_fd >= 0:
            os.close(slave_fd)
        os.close(master_fd)

    raise RuntimeError(
        "the TUI did not render visible output before exiting or timing out"
    )


def send_request(
    process: subprocess.Popen[bytes],
    buffer: bytearray,
    request_id: int,
    method: str,
    params: dict[str, Any],
    timeout: float,
) -> None:
    if process.stdin is None or process.stdout is None:
        raise RuntimeError("app-server stdin is unavailable")
    process.stdin.write(
        (
            json.dumps(
                {"id": request_id, "method": method, "params": params},
                separators=(",", ":"),
            )
            + "\n"
        ).encode()
    )
    process.stdin.flush()

    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        while b"\n" in buffer:
            raw_message, _, remainder = buffer.partition(b"\n")
            buffer[:] = remainder
            try:
                message = json.loads(raw_message)
            except json.JSONDecodeError:
                continue
            if message.get("id") != request_id:
                continue
            if "error" in message:
                raise RuntimeError(f"{method} failed: {message['error']}")
            if "result" not in message:
                raise RuntimeError(f"{method} response has no result")
            return
        if process.poll() is not None:
            raise RuntimeError(f"app-server exited with status {process.returncode}")
        remaining = deadline - time.monotonic()
        ready, _, _ = select.select(
            [process.stdout.fileno()], [], [], min(0.1, remaining)
        )
        if not ready:
            continue
        chunk = os.read(process.stdout.fileno(), 4096)
        if not chunk:
            raise RuntimeError("app-server stdout closed before the response arrived")
        buffer.extend(chunk)
    raise TimeoutError(f"timed out waiting for {method}")


def app_server_checks(binary: Path, timeout: float) -> None:
    with tempfile.TemporaryDirectory(prefix="vsp-ci-app-server-") as tmp:
        process = subprocess.Popen(
            [str(binary), "app-server"],
            cwd=REPO_ROOT,
            env=isolated_env(Path(tmp)),
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
        buffer = bytearray()
        try:
            send_request(
                process,
                buffer,
                1,
                "initialize",
                {
                    "clientInfo": {
                        "name": "vsp-ci-smoke",
                        "title": "VSP CI Smoke",
                        "version": "ci-smoke",
                    },
                    "capabilities": {
                        "experimentalApi": True,
                        "requestAttestation": False,
                        "nativeAskUi": True,
                    },
                },
                timeout,
            )
            if process.stdin is None:
                raise RuntimeError("app-server stdin is unavailable")
            process.stdin.write(b'{"method":"initialized","params":null}\n')
            process.stdin.flush()
            print("ok: app-server initialize")

            send_request(
                process,
                buffer,
                2,
                "thread/list",
                {
                    "cursor": None,
                    "limit": 1,
                    "sortKey": None,
                    "sortDirection": None,
                    "modelProviders": None,
                    "sourceKinds": None,
                    "archived": None,
                    "cwd": None,
                    "useStateDbOnly": False,
                    "searchTerm": None,
                },
                timeout,
            )
            print("ok: app-server thread/list")
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=2)


def run_checks(binary: Path, timeout: float) -> None:
    with tempfile.TemporaryDirectory(prefix="vsp-ci-home-") as tmp:
        env = isolated_env(Path(tmp))
        for arguments in (
            ["--version"],
            ["--help"],
            ["exec", "--help"],
            ["resume", "--help"],
        ):
            command_check(binary, arguments, env, timeout)
        tui_start_check(binary, env, timeout)
    app_server_checks(binary, timeout)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--codex-bin", required=True, type=Path)
    parser.add_argument("--timeout", type=float, default=10.0)
    args = parser.parse_args()

    binary = args.codex_bin.resolve()
    if not binary.is_file():
        parser.error(f"binary does not exist: {binary}")

    try:
        run_checks(binary, args.timeout)
    except Exception as exc:
        print(f"VSP CI smoke failed: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
