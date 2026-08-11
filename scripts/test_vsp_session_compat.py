#!/usr/bin/env python3

import json
import io
import os
from pathlib import Path
import sqlite3
import tempfile
import unittest
from contextlib import redirect_stdout
from unittest.mock import patch

from scripts.vsp_session_compat import Store
from scripts.vsp_session_compat import apply_neutralizations
from scripts.vsp_session_compat import backup_databases
from scripts.vsp_session_compat import ensure_shared_session_assets
from scripts.vsp_session_compat import main
from scripts.vsp_session_compat import read_migrations
from scripts.vsp_session_compat import reset_backfill
from scripts.vsp_session_compat import scan_vsp_rollouts
from scripts.vsp_session_compat import validate_stores


class VspSessionCompatTest(unittest.TestCase):
    def test_markerless_neutral_store_resets_backfill_once(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            official, vsp, _rollout = create_fixture(root, provider=None)
            backup_root = root / "first-backup"

            with (
                patch(
                    "sys.argv",
                    [
                        "vsp_session_compat.py",
                        "apply",
                        "--official-home",
                        str(official.home),
                        "--vsp-home",
                        str(vsp.home),
                        "--backup-root",
                        str(backup_root),
                    ],
                ),
                redirect_stdout(io.StringIO()),
            ):
                self.assertEqual(main(), 0)

            self.assertTrue((vsp.home / "session-compat-v1.json").is_file())
            self.assertTrue((backup_root / "official-state_5.sqlite").is_file())
            with sqlite3.connect(official.db_path) as connection:
                self.assertEqual(
                    connection.execute("SELECT status FROM backfill_state").fetchone()[
                        0
                    ],
                    "pending",
                )

            second_backup = root / "second-backup"
            with (
                patch(
                    "sys.argv",
                    [
                        "vsp_session_compat.py",
                        "apply",
                        "--official-home",
                        str(official.home),
                        "--vsp-home",
                        str(vsp.home),
                        "--backup-root",
                        str(second_backup),
                    ],
                ),
                redirect_stdout(io.StringIO()),
            ):
                self.assertEqual(main(), 0)
            self.assertFalse(second_backup.exists())

    def test_missing_vsp_session_assets_are_linked_non_destructively(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            official, vsp, _rollout = create_fixture(Path(temp_dir))
            (vsp.home / "sessions").unlink()
            (vsp.home / "archived_sessions").unlink()
            (official.home / "session_index.jsonl").write_text("{}\n")

            ensure_shared_session_assets(official, vsp)

            self.assertEqual(
                (vsp.home / "sessions").resolve(),
                (official.home / "sessions").resolve(),
            )
            self.assertEqual(
                (vsp.home / "archived_sessions").resolve(),
                (official.home / "archived_sessions").resolve(),
            )
            self.assertEqual(
                (vsp.home / "session_index.jsonl").resolve(),
                (official.home / "session_index.jsonl").resolve(),
            )

    def test_length_preserving_neutralization_keeps_rollout_suffix_unchanged(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            official, vsp, rollout = create_fixture(Path(temp_dir))
            original = rollout.read_bytes()

            validate_stores(official, vsp)
            scan = scan_vsp_rollouts(vsp)
            self.assertEqual(len(scan.candidates), 2)
            self.assertEqual(scan.already_neutral, 0)

            self.assertEqual(apply_neutralizations(scan.candidates), 2)
            migrated = rollout.read_bytes()
            self.assertEqual(len(migrated), len(original))
            self.assertEqual(
                migrated.replace(b'"model_provider":null ', b'"model_provider":"vsp"'),
                original,
            )
            self.assertEqual(
                json.loads(migrated.splitlines()[0])["payload"]["model_provider"],
                None,
            )
            self.assertEqual(migrated.count(b'{"type":"event_msg"}\n'), 1)

    def test_neutralized_rollout_is_idempotent(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            _official, vsp, _rollout = create_fixture(Path(temp_dir), provider=None)

            scan = scan_vsp_rollouts(vsp)

            self.assertEqual(scan.candidates, ())
            self.assertEqual(scan.already_neutral, 1)

    def test_openai_rollout_is_already_cross_client_compatible(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            _official, vsp, _rollout = create_fixture(
                Path(temp_dir), provider="OpenAI", db_provider="vsp"
            )

            scan = scan_vsp_rollouts(vsp)

            self.assertEqual(scan.candidates, ())
            self.assertEqual(scan.already_neutral, 1)

    def test_database_backup_and_backfill_reset_preserve_source_database(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            official, vsp, _rollout = create_fixture(root)
            backup_root = root / "backups"
            backup_root.mkdir()

            backup_databases((official, vsp), backup_root)
            reset_backfill((official, vsp))

            for store in (official, vsp):
                backup_path = backup_root / f"{store.name}-state_5.sqlite"
                self.assertTrue(backup_path.is_file())
                with sqlite3.connect(backup_path) as backup:
                    self.assertEqual(
                        backup.execute("SELECT status FROM backfill_state").fetchone()[
                            0
                        ],
                        "complete",
                    )
                with sqlite3.connect(store.db_path) as source:
                    self.assertEqual(
                        source.execute("SELECT status FROM backfill_state").fetchone()[
                            0
                        ],
                        "pending",
                    )

    def test_migration_mismatch_is_rejected_before_rollout_changes(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            official, vsp, rollout = create_fixture(Path(temp_dir))
            original = rollout.read_bytes()
            with sqlite3.connect(vsp.db_path) as connection:
                connection.execute(
                    "UPDATE _sqlx_migrations SET checksum = x'02' WHERE version = 1"
                )
                connection.commit()

            with self.assertRaisesRegex(RuntimeError, "migrations do not match"):
                validate_stores(official, vsp)
            self.assertEqual(rollout.read_bytes(), original)
            self.assertNotEqual(
                read_migrations(official.db_path), read_migrations(vsp.db_path)
            )

    def test_trailing_migration_is_allowed_when_required_schema_matches(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            official, vsp, _rollout = create_fixture(Path(temp_dir))
            with sqlite3.connect(official.db_path) as connection:
                connection.execute(
                    "INSERT INTO _sqlx_migrations VALUES (2, 'newer official', x'02')"
                )
                connection.commit()

            validate_stores(official, vsp)

    def test_missing_required_column_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            official, vsp, _rollout = create_fixture(Path(temp_dir))
            with sqlite3.connect(official.db_path) as connection:
                connection.execute("DROP TABLE backfill_state")
                connection.commit()

            with self.assertRaisesRegex(
                RuntimeError, "missing required backfill_state columns"
            ):
                validate_stores(official, vsp)


def create_fixture(
    root: Path,
    provider: str | None = "vsp",
    db_provider: str | None = None,
) -> tuple[Store, Store, Path]:
    official_home = root / "official"
    vsp_home = root / "vsp"
    sessions = official_home / "sessions"
    archived_sessions = official_home / "archived_sessions"
    sessions.mkdir(parents=True)
    archived_sessions.mkdir()
    vsp_home.mkdir()
    os.symlink(sessions, vsp_home / "sessions")
    os.symlink(archived_sessions, vsp_home / "archived_sessions")

    thread_id = "00000000-0000-0000-0000-000000000001"
    rollout = sessions / "rollout-test.jsonl"
    session_meta = {
        "type": "session_meta",
        "payload": {
            "id": thread_id,
            "model_provider": provider,
            "cwd": "/workspace",
        },
    }
    rollout.write_bytes(
        json.dumps(session_meta, separators=(",", ":")).encode()
        + b"\n"
        + b'{"type":"event_msg"}\n'
        + json.dumps(session_meta, separators=(",", ":")).encode()
        + b"\n"
    )

    official = Store("official", official_home)
    vsp = Store("vsp", vsp_home)
    for store in (official, vsp):
        create_database(
            store.db_path,
            thread_id,
            rollout,
            db_provider or provider or "vsp",
        )
    return official, vsp, rollout


def create_database(
    path: Path, thread_id: str, rollout_path: Path, provider: str
) -> None:
    with sqlite3.connect(path) as connection:
        connection.executescript(
            """
            CREATE TABLE _sqlx_migrations (
                version INTEGER PRIMARY KEY,
                description TEXT NOT NULL,
                checksum BLOB NOT NULL
            );
            INSERT INTO _sqlx_migrations VALUES (1, 'fixture', x'01');
            CREATE TABLE threads (
                id TEXT PRIMARY KEY,
                rollout_path TEXT NOT NULL,
                model_provider TEXT NOT NULL
            );
            CREATE TABLE backfill_state (
                id INTEGER PRIMARY KEY,
                status TEXT NOT NULL,
                last_watermark TEXT,
                last_success_at INTEGER,
                updated_at INTEGER NOT NULL
            );
            INSERT INTO backfill_state VALUES (1, 'complete', 'done', 1, 1);
            """
        )
        connection.execute(
            "INSERT INTO threads VALUES (?, ?, ?)",
            (thread_id, str(rollout_path), provider),
        )
        connection.commit()


if __name__ == "__main__":
    unittest.main()
