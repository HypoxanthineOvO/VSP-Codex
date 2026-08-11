#!/usr/bin/env python3

import argparse
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = REPO_ROOT / "scripts" / "vsp_test_groups.json"
RUST_TEST_TARGET_DIR = REPO_ROOT / "codex-rs" / "target-vsp-test-groups"
BASE_TEST_ENV = {
    "CI": "1",
    "INSTA_UPDATE": "no",
    "RUST_MIN_STACK": "8388608",
}
ALLOWED_PLACEHOLDERS = {"binary_suffix", "filter", "python", "repo"}
SHELL_EXECUTABLES = {
    "bash",
    "cmd",
    "cmd.exe",
    "fish",
    "nu",
    "powershell",
    "pwsh",
    "sh",
    "zsh",
}
SECRET_ENV_MARKERS = (
    "API_KEY",
    "CREDENTIAL",
    "PASSWORD",
    "PRIVATE_KEY",
    "SECRET",
    "TOKEN",
)
REQUIRED_GROUP_FIELDS = {
    "tier",
    "ciBlocking",
    "purpose",
    "value",
    "useWhen",
    "prerequisites",
    "commands",
}


def load_manifest() -> dict[str, Any]:
    with MANIFEST_PATH.open(encoding="utf-8") as file:
        manifest = json.load(file)
    validate_manifest(manifest)
    return manifest


def validate_manifest(manifest: dict[str, Any]) -> None:
    if manifest.get("schemaVersion") != 1:
        raise ValueError("unsupported test-group schemaVersion")

    policy = manifest.get("policy")
    if not isinstance(policy, dict) or set(policy) != {
        "selection",
        "failure",
        "fullSuite",
    }:
        raise ValueError("policy must define selection, failure, and fullSuite")

    groups = manifest.get("groups")
    if not isinstance(groups, dict) or not groups:
        raise ValueError("manifest must define at least one group")

    for name, group in groups.items():
        if not isinstance(group, dict):
            raise ValueError(f"group {name!r} must be an object")
        missing = REQUIRED_GROUP_FIELDS - set(group)
        if missing:
            raise ValueError(f"group {name!r} is missing {sorted(missing)}")
        if not isinstance(group["ciBlocking"], bool):
            raise ValueError(f"group {name!r} ciBlocking must be a boolean")
        for field in ("tier", "purpose", "value"):
            if not isinstance(group[field], str) or not group[field].strip():
                raise ValueError(f"group {name!r} {field} must be a non-empty string")
        for field in ("useWhen", "prerequisites"):
            value = group[field]
            if (
                not isinstance(value, list)
                or not value
                or not all(isinstance(item, str) and item.strip() for item in value)
            ):
                raise ValueError(
                    f"group {name!r} {field} must contain non-empty strings"
                )
        if not isinstance(group["commands"], list) or not group["commands"]:
            raise ValueError(f"group {name!r} must define a command")
        filter_placeholders = 0
        for command in group["commands"]:
            if not isinstance(command, dict):
                raise ValueError(f"group {name!r} has a non-object command")
            cwd_value = command.get("cwd")
            if not isinstance(cwd_value, str) or not cwd_value:
                raise ValueError(f"group {name!r} has an invalid cwd")
            cwd = (REPO_ROOT / cwd_value).resolve()
            if not cwd.is_relative_to(REPO_ROOT.resolve()):
                raise ValueError(f"group {name!r} cwd escapes the repository: {cwd}")
            if not cwd.is_dir():
                raise ValueError(f"group {name!r} has missing cwd {cwd}")
            argv = command.get("argv")
            if (
                not isinstance(argv, list)
                or not argv
                or not all(isinstance(part, str) and part for part in argv)
            ):
                raise ValueError(f"group {name!r} has an invalid argv")
            executable = Path(argv[0]).name.lower()
            if executable in SHELL_EXECUTABLES:
                raise ValueError(
                    f"group {name!r} must use direct argv, not shell {argv[0]!r}"
                )
            _validate_placeholders(name, argv)
            filter_placeholders += sum("{filter}" in part for part in argv)
            command_env = command.get("env", {})
            if not isinstance(command_env, dict) or not all(
                isinstance(key, str) and key and isinstance(value, str)
                for key, value in command_env.items()
            ):
                raise ValueError(f"group {name!r} has an invalid env mapping")
            _validate_placeholders(name, command_env.values())
            for key, value in command_env.items():
                if any(marker in key.upper() for marker in SECRET_ENV_MARKERS):
                    raise ValueError(
                        f"group {name!r} must not declare secret environment {key!r}"
                    )
                if key in BASE_TEST_ENV and value != BASE_TEST_ENV[key]:
                    raise ValueError(
                        f"group {name!r} must keep {key}={BASE_TEST_ENV[key]!r}"
                    )
        if group.get("requiresFilter") and filter_placeholders == 0:
            raise ValueError(f"group {name!r} requires a filter but never uses it")
        if not group.get("requiresFilter") and filter_placeholders:
            raise ValueError(f"group {name!r} uses a filter without requiring it")
        required_env = group.get("requiredEnv", [])
        if not isinstance(required_env, list) or not all(
            isinstance(key, str) and key for key in required_env
        ):
            raise ValueError(f"group {name!r} has invalid requiredEnv")


def _validate_placeholders(group_name: str, values: Any) -> None:
    for value in values:
        placeholders = set(re.findall(r"\{([^{}]+)\}", value))
        unknown = placeholders - ALLOWED_PLACEHOLDERS
        if unknown:
            raise ValueError(
                f"group {group_name!r} uses unknown placeholders {sorted(unknown)}"
            )


def require_group(manifest: dict[str, Any], name: str) -> dict[str, Any]:
    try:
        return manifest["groups"][name]
    except KeyError:
        choices = ", ".join(manifest["groups"])
        raise ValueError(f"unknown group {name!r}; choose one of: {choices}") from None


def print_group(name: str, group: dict[str, Any]) -> None:
    ci_status = "CI required" if group["ciBlocking"] else "local/manual"
    print(f"{name} [{group['tier']}, {ci_status}]")
    print(f"  Purpose: {group['purpose']}")
    print(f"  Value: {group['value']}")
    print("  Use when:")
    for condition in group["useWhen"]:
        print(f"    - {condition}")
    print("  Prerequisites:")
    for prerequisite in group["prerequisites"]:
        print(f"    - {prerequisite}")


def list_groups(manifest: dict[str, Any], as_json: bool) -> int:
    if as_json:
        summary = {
            name: {
                "tier": group["tier"],
                "ciBlocking": group["ciBlocking"],
                "purpose": group["purpose"],
            }
            for name, group in manifest["groups"].items()
        }
        print(json.dumps(summary, indent=2))
        return 0

    for name, group in manifest["groups"].items():
        ci_status = "CI required" if group["ciBlocking"] else "local/manual"
        print(f"{name:<15} {group['tier']:<12} {ci_status:<12} {group['purpose']}")
    return 0


def explain_group(manifest: dict[str, Any], name: str, as_json: bool) -> int:
    group = require_group(manifest, name)
    if as_json:
        print(json.dumps(group, indent=2))
    else:
        print_group(name, group)
        print(f"  Failure policy: {manifest['policy']['failure']}")
    return 0


def resolve_commands(
    group: dict[str, Any], filter_value: str | None
) -> list[tuple[Path, list[str], dict[str, str]]]:
    replacements = {
        "{binary_suffix}": ".exe" if os.name == "nt" else "",
        "{filter}": filter_value or "",
        "{python}": sys.executable,
        "{repo}": str(REPO_ROOT),
    }

    def replace_placeholders(value: str) -> str:
        for placeholder, replacement in replacements.items():
            value = value.replace(placeholder, replacement)
        return value

    commands = []
    for command in group["commands"]:
        argv = [replace_placeholders(part) for part in command["argv"]]
        env = {
            key: replace_placeholders(value)
            for key, value in command.get("env", {}).items()
        }
        if (
            Path(argv[0]).name == "cargo"
            and len(argv) > 1
            and argv[1] in {"nextest", "test"}
        ):
            env["CARGO_TARGET_DIR"] = str(RUST_TEST_TARGET_DIR)
        commands.append((REPO_ROOT / command["cwd"], argv, env))
    return commands


def run_group(
    manifest: dict[str, Any],
    name: str,
    filter_value: str | None,
    reason: str | None,
    dry_run: bool,
) -> int:
    group = require_group(manifest, name)
    if group.get("requiresFilter") and not (filter_value and filter_value.strip()):
        raise ValueError(
            f"group {name!r} requires --filter tied to the changed behavior"
        )
    if group.get("requiresReason") and not (reason and reason.strip()):
        raise ValueError(
            f"group {name!r} requires --reason; full-suite output must have an owner"
        )

    missing_env = [key for key in group.get("requiredEnv", []) if not os.getenv(key)]
    if missing_env:
        raise ValueError(
            f"group {name!r} requires environment: {', '.join(missing_env)}"
        )

    print_group(name, group)
    if reason:
        print(f"  Reason: {reason}")
    print(f"  Failure policy: {manifest['policy']['failure']}")

    for cwd, argv, command_env in resolve_commands(group, filter_value):
        print(f"  Run ({cwd.relative_to(REPO_ROOT) or '.'}): {shlex.join(argv)}")
        if dry_run:
            continue
        env = os.environ.copy()
        env.update(BASE_TEST_ENV)
        env.update(command_env)
        sys.stdout.flush()
        result = subprocess.run(argv, cwd=cwd, env=env, check=False)
        if result.returncode:
            print(
                f"Test group {name!r} failed. Isolate and fix a related failure, or stop "
                "on a proved blocker; do not hand off a raw failure list.",
                file=sys.stderr,
            )
            return result.returncode

    if not dry_run and "noPendingSnapshots" in group.get("postChecks", []):
        pending = sorted(REPO_ROOT.rglob("*.snap.new"))
        if pending:
            print("TUI group produced snapshots that require review:", file=sys.stderr)
            for path in pending:
                print(f"  {path.relative_to(REPO_ROOT)}", file=sys.stderr)
            return 1

    if not dry_run:
        print(f"Test group {name!r} passed.")
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="List, explain, validate, or run VSP test groups."
    )
    subparsers = parser.add_subparsers(dest="action", required=True)

    list_parser = subparsers.add_parser("list", help="List available groups")
    list_parser.add_argument("--json", action="store_true")

    explain_parser = subparsers.add_parser("explain", help="Explain one group")
    explain_parser.add_argument("group")
    explain_parser.add_argument("--json", action="store_true")

    subparsers.add_parser("validate", help="Validate the group manifest")

    run_parser = subparsers.add_parser("run", help="Run one group")
    run_parser.add_argument("group")
    run_parser.add_argument("--filter")
    run_parser.add_argument("--reason")
    run_parser.add_argument("--dry-run", action="store_true")
    return parser.parse_args()


def main() -> int:
    try:
        manifest = load_manifest()
        args = parse_args()
        if args.action == "list":
            return list_groups(manifest, args.json)
        if args.action == "explain":
            return explain_group(manifest, args.group, args.json)
        if args.action == "validate":
            print(f"Validated {len(manifest['groups'])} VSP test groups.")
            return 0
        if args.action == "run":
            return run_group(
                manifest,
                args.group,
                args.filter,
                args.reason,
                args.dry_run,
            )
        raise AssertionError(f"unhandled action: {args.action}")
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
