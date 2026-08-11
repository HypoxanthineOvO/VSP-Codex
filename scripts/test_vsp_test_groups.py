import copy
import io
import json
import os
from pathlib import Path
import unittest
from unittest import mock

from scripts import vsp_test_groups


class VspTestGroupsTests(unittest.TestCase):
    def setUp(self) -> None:
        with vsp_test_groups.MANIFEST_PATH.open(encoding="utf-8") as file:
            self.manifest = json.load(file)

    def test_repository_manifest_is_valid(self) -> None:
        vsp_test_groups.validate_manifest(self.manifest)

        self.assertEqual(len(self.manifest["groups"]), 17)
        self.assertEqual(
            [
                name
                for name, group in self.manifest["groups"].items()
                if group["ciBlocking"]
            ],
            ["smoke"],
        )

    def test_sdk_group_uses_current_checkout_binary(self) -> None:
        group = self.manifest["groups"]["sdk-typescript"]
        commands = vsp_test_groups.resolve_commands(group, "abort.test.ts")

        self.assertEqual(
            commands[0][1],
            ["cargo", "build", "-p", "codex-cli", "--bin", "vsp-codex"],
        )
        expected_suffix = ".exe" if os.name == "nt" else ""
        self.assertEqual(
            commands[1][2],
            {
                "CODEX_PATH": str(
                    vsp_test_groups.REPO_ROOT
                    / "codex-rs"
                    / "target"
                    / "debug"
                    / f"vsp-codex{expected_suffix}"
                )
            },
        )
        self.assertIn("abort.test.ts", commands[1][1])

    def test_rust_test_commands_use_checkout_local_target_dir(self) -> None:
        group = self.manifest["groups"]["tui"]
        commands = vsp_test_groups.resolve_commands(group, "heading test")

        self.assertEqual(
            commands[0][2]["CARGO_TARGET_DIR"],
            str(vsp_test_groups.RUST_TEST_TARGET_DIR),
        )

    def test_validate_rejects_shell_commands(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        manifest["groups"]["smoke"]["commands"][0]["argv"] = [
            "sh",
            "-c",
            "true",
        ]

        with self.assertRaisesRegex(ValueError, "direct argv"):
            vsp_test_groups.validate_manifest(manifest)

    def test_validate_rejects_secret_environment(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        manifest["groups"]["smoke"]["commands"][0]["env"] = {
            "OPENAI_API_KEY": "not-a-real-key"
        }

        with self.assertRaisesRegex(ValueError, "secret environment"):
            vsp_test_groups.validate_manifest(manifest)

    def test_validate_rejects_cwd_escape(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        manifest["groups"]["smoke"]["commands"][0]["cwd"] = ".."

        with self.assertRaisesRegex(ValueError, "escapes the repository"):
            vsp_test_groups.validate_manifest(manifest)

    def test_focused_group_requires_filter(self) -> None:
        with self.assertRaisesRegex(ValueError, "requires --filter"):
            vsp_test_groups.run_group(
                self.manifest,
                "core",
                filter_value=None,
                reason=None,
                dry_run=True,
            )

    def test_full_suite_requires_reason(self) -> None:
        with self.assertRaisesRegex(ValueError, "requires --reason"):
            vsp_test_groups.run_group(
                self.manifest,
                "upstream-all",
                filter_value=None,
                reason=None,
                dry_run=True,
            )

    def test_required_external_environment_is_not_a_silent_pass(self) -> None:
        with mock.patch.dict(os.environ, {}, clear=True):
            with self.assertRaisesRegex(ValueError, "HYPO_WORKFLOW_ROOT"):
                vsp_test_groups.run_group(
                    self.manifest,
                    "hypo-external",
                    filter_value=None,
                    reason=None,
                    dry_run=True,
                )

    def test_list_json_reports_every_group(self) -> None:
        output = io.StringIO()
        with mock.patch("sys.stdout", output):
            result = vsp_test_groups.list_groups(self.manifest, as_json=True)

        self.assertEqual(result, 0)
        self.assertEqual(
            set(json.loads(output.getvalue())), set(self.manifest["groups"])
        )


if __name__ == "__main__":
    unittest.main()
