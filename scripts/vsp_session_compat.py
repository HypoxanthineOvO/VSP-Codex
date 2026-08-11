#!/usr/bin/env python3

import argparse
from dataclasses import dataclass
from datetime import datetime
import fcntl
import json
import os
from pathlib import Path
import re
import sqlite3
import sys
from typing import Iterable


STATE_DB_NAME = "state_5.sqlite"
MARKER_NAME = "session-compat-v1.json"
VSP_PROVIDER = "vsp"
MAX_SESSION_META_BYTES = 16 * 1024 * 1024
PROVIDER_VALUE_PATTERN = re.compile(
    rb'"model_provider"\s*:\s*(?P<value>"vsp")',
    re.IGNORECASE,
)


@dataclass(frozen=True)
class Store:
    name: str
    home: Path

    @property
    def db_path(self) -> Path:
        return self.home / STATE_DB_NAME


@dataclass(frozen=True)
class Neutralization:
    thread_id: str
    rollout_path: Path
    offset: int


@dataclass(frozen=True)
class ScanResult:
    candidates: tuple[Neutralization, ...]
    already_neutral: int


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Make shared VSP and official Codex rollouts provider-neutral."
    )
    parser.add_argument("command", choices=("status", "apply"))
    parser.add_argument("--official-home", type=Path, default=Path.home() / ".codex")
    parser.add_argument("--vsp-home", type=Path, default=Path.home() / ".vsp-codex")
    parser.add_argument(
        "--backup-root",
        type=Path,
        help="Backup directory. Defaults to <vsp-home>/session-compat-backups/<timestamp>.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    official = Store("official", args.official_home.expanduser().resolve())
    vsp = Store("vsp", args.vsp_home.expanduser().resolve())

    vsp.home.mkdir(parents=True, exist_ok=True)
    with (vsp.home / ".session-compat.lock").open("a", encoding="utf-8") as lock_handle:
        fcntl.flock(lock_handle.fileno(), fcntl.LOCK_EX)
        return run_locked(args, official, vsp)


def run_locked(args: argparse.Namespace, official: Store, vsp: Store) -> int:
    ensure_shared_session_assets(official, vsp)
    validate_stores(official, vsp)
    scan = scan_vsp_rollouts(vsp)
    summary = {
        "official_db": str(official.db_path),
        "vsp_db": str(vsp.db_path),
        "to_neutralize": len(scan.candidates),
        "already_neutral": scan.already_neutral,
    }
    if args.command == "status":
        print(json.dumps(summary, indent=2))
        return 0
    if not scan.candidates:
        marker_path = vsp.home / MARKER_NAME
        backup_root = None
        backfill_status = "unchanged"
        if not marker_path.is_file():
            backup_root = create_backup_root(args.backup_root, vsp)
            backup_databases((official, vsp), backup_root)
            reset_backfill((official, vsp))
            backfill_status = "pending"
        marker_path = write_marker(vsp, 0, backup_root)
        summary.update(
            {
                "neutralized": 0,
                "already_compatible": True,
                "backfill_status": backfill_status,
                "marker": str(marker_path),
            }
        )
        if backup_root is not None:
            summary["backup_root"] = str(backup_root)
        print(json.dumps(summary, indent=2))
        return 0

    backup_root = create_backup_root(args.backup_root, vsp)

    backup_databases((official, vsp), backup_root)
    applied = apply_neutralizations(scan.candidates)
    reset_backfill((official, vsp))
    manifest_path = write_manifest(
        backup_root,
        official,
        vsp,
        scan,
        applied,
    )
    marker_path = write_marker(vsp, applied, backup_root)
    summary.update(
        {
            "neutralized": applied,
            "backup_root": str(backup_root),
            "manifest": str(manifest_path),
            "marker": str(marker_path),
            "backfill_status": "pending",
        }
    )
    print(json.dumps(summary, indent=2))
    return 0


def validate_stores(official: Store, vsp: Store) -> None:
    for store in (official, vsp):
        if not store.db_path.is_file():
            raise RuntimeError(f"{store.name} state DB not found: {store.db_path}")

    official_migrations = read_migrations(official.db_path)
    vsp_migrations = read_migrations(vsp.db_path)
    shared_migration_count = min(len(official_migrations), len(vsp_migrations))
    if (
        shared_migration_count == 0
        or official_migrations[:shared_migration_count]
        != vsp_migrations[:shared_migration_count]
    ):
        raise RuntimeError(
            "official and VSP state DB migrations do not match in their shared prefix"
        )

    required_columns = {
        "threads": {"id", "rollout_path", "model_provider"},
        "backfill_state": {
            "id",
            "status",
            "last_watermark",
            "last_success_at",
            "updated_at",
        },
    }
    for store in (official, vsp):
        with open_database(store.db_path) as connection:
            for table, required in required_columns.items():
                columns = {
                    str(row[1])
                    for row in connection.execute(f"PRAGMA table_info({table})")
                }
                missing = sorted(required - columns)
                if missing:
                    raise RuntimeError(
                        f"{store.name} state DB is missing required {table} columns: "
                        f"{', '.join(missing)}"
                    )

    official_sessions = (official.home / "sessions").resolve()
    vsp_sessions = (vsp.home / "sessions").resolve()
    official_archived = (official.home / "archived_sessions").resolve()
    vsp_archived = (vsp.home / "archived_sessions").resolve()
    if official_sessions != vsp_sessions or official_archived != vsp_archived:
        raise RuntimeError(
            "official and VSP homes must share sessions and archived_sessions before migration"
        )


def ensure_shared_session_assets(official: Store, vsp: Store) -> None:
    vsp.home.mkdir(parents=True, exist_ok=True)
    for asset in ("sessions", "archived_sessions", "session_index.jsonl"):
        source = official.home / asset
        target = vsp.home / asset
        if os.path.lexists(target) or not source.exists():
            continue
        target.symlink_to(source, target_is_directory=source.is_dir())


def read_migrations(db_path: Path) -> tuple[tuple[int, str, str], ...]:
    with open_database(db_path) as connection:
        rows = connection.execute(
            "SELECT version, description, hex(checksum) "
            "FROM _sqlx_migrations ORDER BY version"
        ).fetchall()
    return tuple(
        (int(version), str(description), str(checksum))
        for version, description, checksum in rows
    )


def scan_vsp_rollouts(vsp: Store) -> ScanResult:
    allowed_roots = (
        (vsp.home / "sessions").resolve(),
        (vsp.home / "archived_sessions").resolve(),
    )
    with open_database(vsp.db_path) as connection:
        rows = connection.execute(
            "SELECT id, rollout_path FROM threads "
            "WHERE lower(trim(model_provider)) = ? ORDER BY id",
            (VSP_PROVIDER,),
        ).fetchall()

    candidates = []
    already_neutral = 0
    seen_paths: set[Path] = set()
    for thread_id, raw_path in rows:
        rollout_path = Path(str(raw_path))
        resolved_path = rollout_path.resolve(strict=True)
        if not any(resolved_path.is_relative_to(root) for root in allowed_roots):
            raise RuntimeError(
                f"rollout is outside shared session roots: {rollout_path}"
            )
        if resolved_path in seen_paths:
            continue
        seen_paths.add(resolved_path)

        rollout_candidates = scan_rollout_provider_values(resolved_path, str(thread_id))
        if rollout_candidates:
            candidates.extend(rollout_candidates)
        else:
            already_neutral += 1

    return ScanResult(tuple(candidates), already_neutral)


def scan_rollout_provider_values(
    rollout_path: Path, thread_id: str
) -> tuple[Neutralization, ...]:
    candidates = []
    with rollout_path.open("rb") as rollout:
        first_line = rollout.readline(MAX_SESSION_META_BYTES + 1)
        validate_canonical_session_meta(first_line, rollout_path, thread_id)
        rollout.seek(0)
        while True:
            line_offset = rollout.tell()
            line = rollout.readline()
            if not line:
                break
            if b'"session_meta"' not in line or b'"model_provider"' not in line:
                continue
            value = json.loads(line)
            payload = (
                value.get("payload") if value.get("type") == "session_meta" else None
            )
            if not isinstance(payload, dict) or payload.get("id") != thread_id:
                continue

            provider = payload.get("model_provider")
            if provider is None or (
                isinstance(provider, str)
                and (not provider.strip() or provider.strip().lower() == "openai")
            ):
                continue
            if not isinstance(provider, str) or provider != VSP_PROVIDER:
                raise RuntimeError(f"unexpected rollout model provider: {rollout_path}")

            matches = list(PROVIDER_VALUE_PATTERN.finditer(line))
            if len(matches) != 1:
                raise RuntimeError(
                    f"could not locate one VSP provider value: {rollout_path}"
                )
            start, end = matches[0].span("value")
            if end - start != len(b"null "):
                raise RuntimeError(
                    f"provider replacement is not length preserving: {rollout_path}"
                )
            candidates.append(
                Neutralization(
                    thread_id=thread_id,
                    rollout_path=rollout_path,
                    offset=line_offset + start,
                )
            )
    return tuple(candidates)


def validate_canonical_session_meta(
    first_line: bytes, rollout_path: Path, thread_id: str
) -> None:
    if len(first_line) > MAX_SESSION_META_BYTES or not first_line.endswith(b"\n"):
        raise RuntimeError(f"session_meta line is missing or too large: {rollout_path}")
    value = json.loads(first_line)
    payload = value.get("payload") if value.get("type") == "session_meta" else None
    if not isinstance(payload, dict):
        raise RuntimeError(f"rollout does not start with session_meta: {rollout_path}")
    if payload.get("id") != thread_id:
        raise RuntimeError(f"thread id does not match rollout metadata: {rollout_path}")


def backup_databases(stores: Iterable[Store], backup_root: Path) -> None:
    for store in stores:
        destination_path = backup_root / f"{store.name}-{STATE_DB_NAME}"
        with open_database(store.db_path) as source:
            with sqlite3.connect(destination_path) as destination:
                source.backup(destination)


def create_backup_root(requested: Path | None, vsp: Store) -> Path:
    backup_root = requested
    if backup_root is None:
        timestamp = datetime.now().astimezone().strftime("%Y%m%d-%H%M%S")
        backup_root = vsp.home / "session-compat-backups" / timestamp
    backup_root = backup_root.expanduser().resolve()
    backup_root.mkdir(parents=True, exist_ok=False)
    return backup_root


def apply_neutralizations(candidates: Iterable[Neutralization]) -> int:
    applied = 0
    for candidate in candidates:
        descriptor = os.open(candidate.rollout_path, os.O_RDWR)
        try:
            current = os.pread(descriptor, len(b'"vsp"'), candidate.offset)
            if current != b'"vsp"':
                raise RuntimeError(
                    f"rollout provider changed after preflight: {candidate.rollout_path}"
                )
            written = os.pwrite(descriptor, b"null ", candidate.offset)
            if written != len(b"null "):
                raise RuntimeError(f"short rollout write: {candidate.rollout_path}")
            os.fsync(descriptor)
            applied += 1
        finally:
            os.close(descriptor)
    return applied


def reset_backfill(stores: Iterable[Store]) -> None:
    now = int(datetime.now().timestamp())
    for store in stores:
        with open_database(store.db_path) as connection:
            result = connection.execute(
                "UPDATE backfill_state "
                "SET status = 'pending', last_watermark = NULL, "
                "last_success_at = NULL, updated_at = ? WHERE id = 1",
                (now,),
            )
            if result.rowcount != 1:
                raise RuntimeError(f"missing backfill state row in {store.db_path}")
            connection.commit()


def write_manifest(
    backup_root: Path,
    official: Store,
    vsp: Store,
    scan: ScanResult,
    applied: int,
) -> Path:
    manifest_path = backup_root / "manifest.json"
    manifest = {
        "version": 1,
        "created_at": datetime.now().astimezone().isoformat(timespec="seconds"),
        "strategy": "length_preserving_vsp_provider_neutralization",
        "official_home": str(official.home),
        "vsp_home": str(vsp.home),
        "neutralized": applied,
        "already_neutral": scan.already_neutral,
        "rollouts": [
            {
                "thread_id": candidate.thread_id,
                "path": str(candidate.rollout_path),
                "offset": candidate.offset,
                "before": '"vsp"',
                "after": "null ",
            }
            for candidate in scan.candidates
        ],
    }
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest_path


def write_marker(vsp: Store, neutralized: int, backup_root: Path | None) -> Path:
    marker_path = vsp.home / MARKER_NAME
    temporary_path = marker_path.with_name(f".{MARKER_NAME}.{os.getpid()}.tmp")
    marker = {
        "version": 1,
        "completed_at": datetime.now().astimezone().isoformat(timespec="seconds"),
        "neutralized": neutralized,
        "backup_root": str(backup_root) if backup_root is not None else None,
    }
    temporary_path.write_text(json.dumps(marker, indent=2) + "\n", encoding="utf-8")
    os.replace(temporary_path, marker_path)
    return marker_path


def open_database(path: Path) -> sqlite3.Connection:
    connection = sqlite3.connect(path, timeout=30)
    connection.execute("PRAGMA busy_timeout = 30000")
    return connection


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, sqlite3.Error, json.JSONDecodeError) as error:
        print(f"session compatibility migration failed: {error}", file=sys.stderr)
        raise SystemExit(1)
