#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOCAL_FILE="$ROOT/codex-rs/models-manager/models.json"
UPSTREAM_URL="https://raw.githubusercontent.com/openai/codex/main/codex-rs/models-manager/models.json"
TMP_DIR="$(mktemp -d)"
UPSTREAM_FILE="$TMP_DIR/models.json"
trap 'rm -rf "$TMP_DIR"' EXIT

curl -fsSL "$UPSTREAM_URL" -o "$UPSTREAM_FILE"

if diff -u "$LOCAL_FILE" "$UPSTREAM_FILE" >"$TMP_DIR/models.diff"; then
  printf 'Model catalog matches %s\n' "$UPSTREAM_URL"
  exit 0
fi

printf 'Upstream model catalog differs from %s\n' "$LOCAL_FILE" >&2
python3 - "$LOCAL_FILE" "$UPSTREAM_FILE" <<'PY'
import json
import sys
from pathlib import Path

local = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
upstream = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
local_models = {model["slug"]: model for model in local.get("models", [])}
upstream_models = {model["slug"]: model for model in upstream.get("models", [])}
added = sorted(upstream_models.keys() - local_models.keys())
removed = sorted(local_models.keys() - upstream_models.keys())
changed = sorted(
    slug
    for slug in local_models.keys() & upstream_models.keys()
    if local_models[slug] != upstream_models[slug]
)
print("Added slugs:", ", ".join(added) or "(none)")
print("Removed slugs:", ", ".join(removed) or "(none)")
print("Changed slugs:", ", ".join(changed) or "(none)")
PY
printf '\nFull diff:\n' >&2
cat "$TMP_DIR/models.diff" >&2
exit 1
