#!/bin/sh

set -eu

RELEASE="${CODEX_RELEASE:-latest}"
if [ -n "${CODEX_RELEASE:-}" ]; then
  RELEASE_EXPLICIT=1
else
  RELEASE_EXPLICIT=0
fi
ARCHIVE_URL="${VSP_CODEX_ARCHIVE_URL:-}"
ARCHIVE_SHA256="${VSP_CODEX_SHA256:-}"
ARCHIVE_TARGET="${VSP_CODEX_TARGET:-}"
ARCHIVE_VERSION="${VSP_CODEX_VERSION:-}"
ARCHIVE_MANIFEST_URL="${VSP_CODEX_RELEASE_MANIFEST_URL:-}"
ARCHIVE_CHANNEL="${VSP_CODEX_RELEASE_CHANNEL:-lab}"
ARCHIVE_SOURCE="${VSP_CODEX_RELEASE_SOURCE:-preview-release-builder}"
DEFAULT_MANIFEST_URL="https://gitlab.vsplab.cn/heyx/Codex/-/raw/main/.pipeline/release/vsp-release-manifest.json"
ASSUME_YES=0
CONFIGURE_VSCODE=0
REMOVE_EXISTING=0
LAUNCH_MODE="prompt"
NON_INTERACTIVE="${CODEX_NON_INTERACTIVE:-false}"

BIN_DIR="${CODEX_INSTALL_DIR:-$HOME/.local/bin}"
BIN_NAME="vsp-codex"
BIN_PATH="$BIN_DIR/$BIN_NAME"
CODEX_COMPAT_BIN_NAME="codex"
CODEX_COMPAT_BIN_PATH="$BIN_DIR/$CODEX_COMPAT_BIN_NAME"
CODEX_HOME_DIR="${CODEX_HOME:-$HOME/.vsp-codex}"
OFFICIAL_CODEX_HOME_DIR="${VSP_CODEX_OFFICIAL_HOME:-$HOME/.codex}"
CODE_MODE_HOST_BIN_PATH="$BIN_DIR/codex-code-mode-host"
STANDALONE_ROOT="$CODEX_HOME_DIR/packages/standalone"
RELEASES_DIR="$STANDALONE_ROOT/releases"
CURRENT_LINK="$STANDALONE_ROOT/current"
LOCK_FILE="$STANDALONE_ROOT/install.lock"
LOCK_DIR="$STANDALONE_ROOT/install.lock.d"
LOCK_STALE_AFTER_SECS=600

path_action="already"
path_profile=""
conflict_manager=""
conflict_path=""
lock_kind=""
tmp_dir=""

step() {
  printf '==> %s\n' "$1"
}

warn() {
  printf 'WARNING: %s\n' "$1" >&2
}

normalize_version() {
  case "$1" in
    "" | latest)
      printf 'latest\n'
      ;;
    rust-v*)
      printf '%s\n' "${1#rust-v}"
      ;;
    v*)
      printf '%s\n' "${1#v}"
      ;;
    *)
      printf '%s\n' "$1"
      ;;
  esac
}

validate_version() {
  version="$1"

  if [ "$version" = "latest" ]; then
    return
  fi

  if ! printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+(-(alpha|beta)(\.[0-9]+)*|-vsp(\.[0-9]+)*(-(alpha|beta)(\.[0-9]+)*)?)?$'; then
    echo "Invalid Codex release version: $version. Expected latest, x.y.z[-alpha[.N]|-beta[.N]|-vsp[.N]], or x.y.z-vsp[.N]-alpha[.N]." >&2
    exit 1
  fi
}

parse_args() {
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --release)
        if [ "$#" -lt 2 ]; then
          echo "--release requires a value." >&2
          exit 1
        fi
        RELEASE="$2"
        RELEASE_EXPLICIT=1
        shift
        ;;
      --version)
        if [ "$#" -lt 2 ]; then
          echo "--version requires a value." >&2
          exit 1
        fi
        ARCHIVE_VERSION="$2"
        shift
        ;;
      --archive-url)
        if [ "$#" -lt 2 ]; then
          echo "--archive-url requires a value." >&2
          exit 1
        fi
        ARCHIVE_URL="$2"
        shift
        ;;
      --sha256)
        if [ "$#" -lt 2 ]; then
          echo "--sha256 requires a value." >&2
          exit 1
        fi
        ARCHIVE_SHA256="$2"
        shift
        ;;
      --target)
        if [ "$#" -lt 2 ]; then
          echo "--target requires a value." >&2
          exit 1
        fi
        ARCHIVE_TARGET="$2"
        shift
        ;;
      --manifest-url)
        if [ "$#" -lt 2 ]; then
          echo "--manifest-url requires a value." >&2
          exit 1
        fi
        ARCHIVE_MANIFEST_URL="$2"
        shift
        ;;
      --channel)
        if [ "$#" -lt 2 ]; then
          echo "--channel requires a value." >&2
          exit 1
        fi
        ARCHIVE_CHANNEL="$2"
        shift
        ;;
      --yes | -y)
        ASSUME_YES=1
        ;;
      --vscode)
        CONFIGURE_VSCODE=1
        ;;
      --remove-existing)
        REMOVE_EXISTING=1
        ;;
      --launch)
        LAUNCH_MODE="always"
        ;;
      --no-launch)
        LAUNCH_MODE="never"
        ;;
      --help | -h)
        cat <<EOF
Usage: install.sh [--release VERSION]
       install.sh --archive-url URL --sha256 SHA256 --target TARGET --version VERSION --manifest-url URL [--channel CHANNEL] [--vscode] [--launch|--no-launch] [--yes]

Options:
  --release VERSION     Install an official GitHub release version.
  --archive-url URL     Install a VSP preview package archive directly.
  --sha256 SHA256       Expected SHA-256 for --archive-url.
  --target TARGET       Package target for --archive-url, e.g. x86_64-unknown-linux-gnu.
  --version VERSION     Version label for --archive-url installs.
  --manifest-url URL    VSP release manifest URL to persist for archive-url installs.
  --channel CHANNEL     VSP release channel to persist for archive-url installs.
  --vscode             Configure the official VSCode ChatGPT extension to use vsp-codex.
  --remove-existing    Offer to remove an existing npm/brew-managed Codex install.
  --launch             Start vsp-codex after installation.
  --no-launch          Do not prompt to start vsp-codex after installation.
  --yes, -y            Answer yes to installer confirmations.

Environment:
  CODEX_RELEASE          Version to install; overridden by --release.
  CODEX_NON_INTERACTIVE  Set to 1, true, or yes to skip prompts.
EOF
        exit 0
        ;;
      *)
        echo "Unknown argument: $1" >&2
        exit 1
        ;;
    esac
    shift
  done
}

download_file() {
  url="$1"
  output="$2"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$output"
    return
  fi

  if command -v wget >/dev/null 2>&1; then
    wget -q -O "$output" "$url"
    return
  fi

  echo "curl or wget is required to install Codex." >&2
  exit 1
}

download_text() {
  url="$1"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url"
    return
  fi

  if command -v wget >/dev/null 2>&1; then
    wget -q -O - "$url"
    return
  fi

  echo "curl or wget is required to install Codex." >&2
  exit 1
}

resolve_from_vsp_manifest() {
  target_filter="$1"
  channel_filter="$2"
  manifest_json="$(download_text "$DEFAULT_MANIFEST_URL")" || {
    echo "Could not fetch VSP release manifest from $DEFAULT_MANIFEST_URL." >&2
    exit 1
  }
  eval "$(printf '%s\n' "$manifest_json" | LC_ALL=C awk -v tf="$target_filter" -v cf="$channel_filter" '
    BEGIN {
      depth = 0; in_str = 0; esc = 0; tok = ""; key = ""
      arr_depth = 0; rel_depth = 0; in_releases = 0
      found = 0; pending_key = ""; expecting_val = 0
      r_version = ""; r_url = ""; r_sha = ""; r_target = ""; r_channel = ""
    }
    {
      for (i = 1; i <= length($0); i++) {
        c = substr($0, i, 1)
        if (in_str) {
          if (esc) { tok = tok "\\" c; esc = 0 }
          else if (c == "\\") { esc = 1 }
          else if (c == "\"") {
            in_str = 0
            if (expecting_val && rel_depth > 0) {
              if (key == "version") r_version = tok
              else if (key == "archive_url") r_url = tok
              else if (key == "sha256") r_sha = tok
              else if (key == "target") r_target = tok
              else if (key == "channel") r_channel = tok
            }
            if (!expecting_val) pending_key = tok
            expecting_val = 0; key = ""
          } else { tok = tok c }
          continue
        }
        if (c == "\"") { in_str = 1; tok = ""; esc = 0; next_is_val = expecting_val }
        else if (c == ":") {
          if (pending_key != "") { key = pending_key; pending_key = ""; expecting_val = 1 }
        }
        else if (c == "{") {
          depth++
          if (in_releases && arr_depth > 0 && rel_depth == 0) {
            rel_depth = depth
            r_version = ""; r_url = ""; r_sha = ""; r_target = ""; r_channel = ""
          }
          expecting_val = 0; key = ""; pending_key = ""
        }
        else if (c == "}") {
          if (depth == rel_depth && rel_depth > 0) {
            if (r_version != "" && r_url != "" && r_sha != "") {
              if ((tf == "" || r_target == tf) && (cf == "" || r_channel == cf)) {
                found = 1
                gsub(/'\''/, "'\''\\'\'''\''", r_version)
                gsub(/'\''/, "'\''\\'\'''\''", r_url)
                gsub(/'\''/, "'\''\\'\'''\''", r_sha)
                gsub(/'\''/, "'\''\\'\'''\''", r_target)
                print "ARCHIVE_VERSION='\''" r_version "'\''"
                print "ARCHIVE_URL='\''" r_url "'\''"
                print "ARCHIVE_SHA256='\''" r_sha "'\''"
                if (r_target != "") print "ARCHIVE_TARGET='\''" r_target "'\''"
                exit 0
              }
            }
            rel_depth = 0
          }
          depth--; expecting_val = 0; key = ""; pending_key = ""
        }
        else if (c == "[") {
          arr_depth++
          if (expecting_val && key == "releases" && depth == 1) in_releases = 1
          expecting_val = 0; key = ""
        }
        else if (c == "]") {
          if (in_releases) in_releases = 0
          arr_depth--; expecting_val = 0; key = ""; pending_key = ""
        }
        else if (c == ",") { expecting_val = 0; key = ""; pending_key = "" }
      }
    }
    END { if (!found) exit 1 }
  ')" || {
    echo "Could not find a matching release in the VSP manifest for target $target_filter." >&2
    exit 1
  }
  ARCHIVE_MANIFEST_URL="$DEFAULT_MANIFEST_URL"
}

parse_release_metadata() {
  # Bound awk's record size so compact, single-line JSON stays fast on every
  # supported awk implementation. JSON strings cannot contain literal newlines,
  # so the record boundaries inserted by fold do not change the document.
  LC_ALL=C fold -b -w 4096 | LC_ALL=C awk '
    function finish_string(value) {
      if (object_depth == 1 && key == "tag_name") {
        print "tag_name\t" value
      } else if (object_depth == asset_object_depth) {
        if (key == "name") {
          asset_name = value
        } else if (key == "digest") {
          asset_digest = value
        }
      }

      expecting_value = 0
      key = ""
    }

    {
      for (i = 1; i <= length($0); i++) {
        char = substr($0, i, 1)

        if (in_string) {
          if (escaped) {
            token = token "\\" char
            escaped = 0
          } else if (char == "\\") {
            escaped = 1
          } else if (char == "\"") {
            in_string = 0
            if (string_is_value) {
              finish_string(token)
            } else {
              pending_key = token
            }
          } else {
            token = token char
          }
          continue
        }

        if (char == "\"") {
          in_string = 1
          token = ""
          escaped = 0
          string_is_value = expecting_value
        } else if (char == ":" && pending_key != "") {
          key = pending_key
          pending_key = ""
          expecting_value = 1
        } else if (char == "{") {
          object_depth++
          if (assets_array_depth != 0 &&
              array_depth == assets_array_depth &&
              asset_object_depth == 0) {
            asset_object_depth = object_depth
            asset_name = ""
            asset_digest = ""
          }
          expecting_value = 0
          key = ""
        } else if (char == "}") {
          if (object_depth == asset_object_depth) {
            if (asset_name != "" && asset_digest != "") {
              print "asset\t" asset_name "\t" asset_digest
            }
            asset_object_depth = 0
            asset_name = ""
            asset_digest = ""
          }
          object_depth--
          expecting_value = 0
          key = ""
          pending_key = ""
        } else if (char == "[") {
          array_depth++
          if (expecting_value && key == "assets" && object_depth == 1) {
            assets_array_depth = array_depth
          }
          expecting_value = 0
          key = ""
        } else if (char == "]") {
          if (array_depth == assets_array_depth) {
            assets_array_depth = 0
          }
          array_depth--
          expecting_value = 0
          key = ""
          pending_key = ""
        } else if (char == ",") {
          expecting_value = 0
          key = ""
          pending_key = ""
        }
      }
    }

    END {
      if (in_string || object_depth != 0 || array_depth != 0) {
        exit 1
      }
    }
  '
}

package_metadata_is_vsp() {
  package_dir="$1"
  expected_target="$2"
  expected_version="$3"
  package_metadata_path="$package_dir/codex-package.json"

  [ -f "$package_metadata_path" ] || return 1
  if ! command -v python3 >/dev/null 2>&1; then
    echo "python3 is required to validate VSP Codex package metadata." >&2
    return 1
  fi

  python3 - \
    "$package_metadata_path" \
    "bin/$BIN_NAME" \
    "$expected_target" \
    "$expected_version" \
    "$ARCHIVE_URL" \
    "$ARCHIVE_MANIFEST_URL" \
    "$ARCHIVE_CHANNEL" \
    "$ARCHIVE_SOURCE" <<'PY'
import json
import sys
from pathlib import Path


def reject_duplicate_keys(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError(f"duplicate JSON key: {key}")
        value[key] = item
    return value


(
    metadata_path,
    expected_entrypoint,
    expected_target,
    expected_version,
    archive_url,
    expected_manifest_url,
    expected_channel,
    expected_source,
) = sys.argv[1:]

try:
    metadata = json.loads(
        Path(metadata_path).read_text(encoding="utf-8"),
        object_pairs_hook=reject_duplicate_keys,
    )
    if not isinstance(metadata, dict):
        raise ValueError("package metadata root must be an object")
    if metadata.get("entrypoint") != expected_entrypoint:
        raise ValueError(f"entrypoint must be {expected_entrypoint}")
    if metadata.get("version") != expected_version:
        raise ValueError(f"version must be {expected_version}")

    release = metadata.get("vspRelease")
    if not isinstance(release, dict):
        raise ValueError("vspRelease must be an object")
    for field in ("manifestUrl", "channel", "source"):
        value = release.get(field)
        if not isinstance(value, str) or not value.strip():
            raise ValueError(f"vspRelease.{field} must be a non-empty string")
    if release.get("target") != expected_target:
        raise ValueError(f"vspRelease.target must be {expected_target}")

    if archive_url:
        expected = {
            "manifestUrl": expected_manifest_url,
            "channel": expected_channel,
            "source": expected_source,
        }
        for field, value in expected.items():
            if release.get(field) != value:
                raise ValueError(f"vspRelease.{field} must be {value}")
except (OSError, UnicodeError, ValueError, json.JSONDecodeError) as error:
    print(f"Invalid VSP Codex package metadata: {error}", file=sys.stderr)
    raise SystemExit(1)
PY
}

release_url_for_asset() {
  asset="$1"
  resolved_version="$2"

  printf 'https://github.com/openai/codex/releases/download/rust-v%s/%s\n' "$resolved_version" "$asset"
}

release_metadata_url() {
  resolved_version="$1"

  printf 'https://api.github.com/repos/openai/codex/releases/tags/rust-v%s\n' "$resolved_version"
}

resolve_release() {
  normalized_version="$(normalize_version "$RELEASE")"
  validate_version "$normalized_version"

  if [ "$normalized_version" = "latest" ]; then
    requested_release="latest"
    metadata_url="https://api.github.com/repos/openai/codex/releases/latest"
  else
    resolved_version="$normalized_version"
    requested_release="$resolved_version"
    metadata_url="$(release_metadata_url "$resolved_version")"
  fi

  if ! release_json="$(download_text "$metadata_url")"; then
    echo "Could not fetch GitHub release metadata for Codex $requested_release. GitHub API may be unavailable or rate limited." >&2
    exit 1
  fi

  if ! release_metadata="$(printf '%s\n' "$release_json" | parse_release_metadata)"; then
    echo "Could not parse GitHub release metadata for Codex $requested_release." >&2
    exit 1
  fi

  if [ "$normalized_version" = "latest" ]; then
    release_tag="$(printf '%s\n' "$release_metadata" | awk -F '\t' '$1 == "tag_name" { print $2; exit }')"
    case "$release_tag" in
      rust-v*) resolved_version="${release_tag#rust-v}" ;;
      *) resolved_version="" ;;
    esac
    if [ -z "$resolved_version" ]; then
      echo "Failed to resolve the latest Codex release version." >&2
      exit 1
    fi
    validate_version "$resolved_version"
  fi
}

release_asset_digest_or_empty() {
  asset="$1"

  digest="$(printf '%s\n' "$release_metadata" | awk -F '\t' -v asset="$asset" '
    $1 == "asset" && $2 == asset {
      print $3
      exit
    }
  ')"

  case "$digest" in
    sha256:????????????????????????????????????????????????????????????????)
      printf '%s\n' "${digest#sha256:}"
      ;;
    *)
      return 1
      ;;
  esac
}

release_asset_exists() {
  asset="$1"

  release_asset_digest_or_empty "$asset" >/dev/null 2>&1
}

release_asset_digest() {
  asset="$1"

  digest="$(release_asset_digest_or_empty "$asset" || true)"
  if [ -z "$digest" ]; then
    echo "Could not find SHA-256 digest for release asset $asset." >&2
    exit 1
  fi

  printf '%s\n' "$digest"
}

package_archive_digest() {
  asset="$1"
  manifest_path="$2"

  digest="$(awk -v asset="$asset" '
    $2 == asset && length($1) == 64 && $1 !~ /[^0-9a-fA-F]/ {
      print tolower($1)
      found = 1
      exit
    }
    END {
      if (!found) {
        exit 1
      }
    }
  ' "$manifest_path" 2>/dev/null || true)"

  if [ -z "$digest" ]; then
    echo "Could not find SHA-256 digest for $asset in codex-package_SHA256SUMS." >&2
    exit 1
  fi

  printf '%s\n' "$digest"
}

file_sha256() {
  path="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$path" | awk '{print $1}'
    return
  fi

  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$path" | awk '{print $1}'
    return
  fi

  if command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "$path" | sed 's/^.*= //'
    return
  fi

  echo "sha256sum, shasum, or openssl is required to verify the Codex download." >&2
  exit 1
}

verify_archive_digest() {
  archive_path="$1"
  expected_digest="$2"
  actual_digest="$(file_sha256 "$archive_path")"

  if [ "$actual_digest" != "$expected_digest" ]; then
    echo "Downloaded Codex archive checksum did not match expected digest." >&2
    echo "expected: $expected_digest" >&2
    echo "actual:   $actual_digest" >&2
    exit 1
  fi
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 is required to install Codex." >&2
    exit 1
  fi
}

validate_direct_archive_args() {
  if [ -z "$ARCHIVE_URL" ]; then
    return
  fi

  if [ -z "$ARCHIVE_SHA256" ]; then
    echo "--sha256 is required when --archive-url is used." >&2
    exit 1
  fi
  if [ -z "$ARCHIVE_MANIFEST_URL" ]; then
    echo "--manifest-url is required when --archive-url is used so the installed package can find VSP updates." >&2
    exit 1
  fi

  case "$ARCHIVE_SHA256" in
    ????????????????????????????????????????????????????????????????)
      ;;
    *)
      echo "--sha256 must be a 64-character hexadecimal digest." >&2
      exit 1
      ;;
  esac

  case "$ARCHIVE_SHA256" in
    *[!0-9a-fA-F]*)
      echo "--sha256 must be a 64-character hexadecimal digest." >&2
      exit 1
      ;;
  esac
}
pick_profile() {
  # Use the same shell-specific split Homebrew documents because there is no
  # universal startup file across macOS/Linux login and interactive shells.
  case "$os:${SHELL:-}" in
    darwin:*/zsh)
      printf '%s\n' "$HOME/.zprofile"
      ;;
    darwin:*/bash)
      printf '%s\n' "$HOME/.bash_profile"
      ;;
    linux:*/zsh)
      printf '%s\n' "$HOME/.zshrc"
      ;;
    linux:*/bash)
      printf '%s\n' "$HOME/.bashrc"
      ;;
    *)
      printf '%s\n' "$HOME/.profile"
      ;;
  esac
}

add_to_path() {
  path_action="already"
  path_profile=""

  case ":$PATH:" in
    *":$BIN_DIR:"*)
      if [ -z "$conflict_manager" ]; then
        return
      fi
      ;;
  esac

  profile="$(pick_profile)"
  path_profile="$profile"
  begin_marker="# >>> Codex installer >>>"
  end_marker="# <<< Codex installer <<<"
  path_line="export PATH=\"$BIN_DIR:\$PATH\""

  if [ -f "$profile" ] && grep -F "$begin_marker" "$profile" >/dev/null 2>&1; then
    if grep -F "$path_line" "$profile" >/dev/null 2>&1; then
      path_action="configured"
      return
    fi

    if grep -F "$end_marker" "$profile" >/dev/null 2>&1; then
      rewrite_path_block "$profile" "$begin_marker" "$end_marker" "$path_line"
      path_action="updated"
      return
    fi
  fi

  append_path_block "$profile" "$begin_marker" "$end_marker" "$path_line"
  path_action="added"
}

append_path_block() {
  profile="$1"
  begin_marker="$2"
  end_marker="$3"
  path_line="$4"

  {
    printf '\n%s\n' "$begin_marker"
    printf '%s\n' "$path_line"
    printf '%s\n' "$end_marker"
  } >>"$profile"
}

rewrite_path_block() {
  profile="$1"
  begin_marker="$2"
  end_marker="$3"
  path_line="$4"
  tmp_profile="$tmp_dir/profile.$$.tmp"

  awk -v begin="$begin_marker" -v end="$end_marker" -v line="$path_line" '
    BEGIN {
      in_block = 0
      replaced = 0
    }
    $0 == begin {
      if (!replaced) {
        print begin
        print line
        print end
        replaced = 1
      }
      in_block = 1
      next
    }
    in_block {
      if ($0 == end) {
        in_block = 0
      }
      next
    }
    {
      print
    }
    END {
      if (in_block != 0) {
        exit 1
      }
    }
  ' "$profile" >"$tmp_profile"
  mv "$tmp_profile" "$profile"
}

mkdir_lock_is_stale() {
  [ -d "$LOCK_DIR" ] || return 1

  pid="$(cat "$LOCK_DIR/pid" 2>/dev/null || true)"
  started_at="$(cat "$LOCK_DIR/started_at" 2>/dev/null || true)"
  now="$(date +%s 2>/dev/null || printf '0')"

  case "$started_at" in
    ''|*[!0-9]*)
      started_at=0
      ;;
  esac

  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    return 1
  fi

  if [ "$started_at" -eq 0 ] || [ "$now" -eq 0 ]; then
    return 0
  fi

  [ $((now - started_at)) -ge "$LOCK_STALE_AFTER_SECS" ]
}

acquire_install_lock() {
  mkdir -p "$STANDALONE_ROOT"

  if [ "$os" = "darwin" ] && command -v lockf >/dev/null 2>&1; then
    : >>"$LOCK_FILE"
    exec 9<>"$LOCK_FILE"
    lockf 9
    lock_kind="lockf"
    return
  fi

  if command -v flock >/dev/null 2>&1; then
    exec 9>"$LOCK_FILE"
    flock 9
    lock_kind="flock"
    return
  fi

  while ! mkdir "$LOCK_DIR" 2>/dev/null; do
    if mkdir_lock_is_stale; then
      warn "Removing stale installer lock at $LOCK_DIR"
      rm -rf "$LOCK_DIR"
      continue
    fi
    sleep 1
  done

  printf '%s\n' "$$" >"$LOCK_DIR/pid"
  date +%s >"$LOCK_DIR/started_at" 2>/dev/null || true
  lock_kind="mkdir"
}

release_install_lock() {
  if [ "$lock_kind" = "mkdir" ]; then
    rm -rf "$LOCK_DIR" 2>/dev/null || true
  elif [ "$lock_kind" = "flock" ] || [ "$lock_kind" = "lockf" ]; then
    exec 9>&- 2>/dev/null || true
  fi
  lock_kind=""
}

cleanup_stale_install_artifacts() {
  mkdir -p "$RELEASES_DIR" "$STANDALONE_ROOT"

  find "$RELEASES_DIR" -mindepth 1 -maxdepth 1 -name '.staging.*' -exec rm -rf {} +
  find "$STANDALONE_ROOT" -mindepth 1 -maxdepth 1 -name '.current.*' -exec rm -f {} +

  if [ -d "$BIN_DIR" ]; then
    find "$BIN_DIR" -mindepth 1 -maxdepth 1 -name '.codex.*' -exec rm -f {} +
    find "$BIN_DIR" -mindepth 1 -maxdepth 1 -name '.vsp-codex.*' -exec rm -f {} +
  fi
}

replace_path_with_symlink() {
  link_path="$1"
  link_target="$2"
  tmp_link="$3"

  rm -f "$tmp_link"
  ln -s "$link_target" "$tmp_link"

  if mv -Tf "$tmp_link" "$link_path" 2>/dev/null; then
    return
  fi

  if mv -hf "$tmp_link" "$link_path" 2>/dev/null; then
    return
  fi

  rm -f "$link_path"
  mv -f "$tmp_link" "$link_path"
}

shell_single_quote() {
  printf "'%s'" "$(printf '%s' "$1" | sed "s/'/'\\\\''/g")"
}

replace_path_with_codex_compat_wrapper() {
  wrapper_path="$1"
  wrapper_target="$2"
  tmp_wrapper="$3"

  rm -f "$tmp_wrapper"
  quoted_target="$(shell_single_quote "$wrapper_target")"
  cat >"$tmp_wrapper" <<EOF
#!/bin/sh
CODEX_HOME="\${VSP_CODEX_HOME:-\${HOME}/.vsp-codex}"
export CODEX_HOME
exec $quoted_target "\$@"
EOF
  chmod 0755 "$tmp_wrapper"

  if mv -Tf "$tmp_wrapper" "$wrapper_path" 2>/dev/null; then
    return
  fi

  if mv -hf "$tmp_wrapper" "$wrapper_path" 2>/dev/null; then
    return
  fi

  rm -f "$wrapper_path"
  mv -f "$tmp_wrapper" "$wrapper_path"
}

version_from_binary() {
  codex_path="$1"

  if [ ! -x "$codex_path" ]; then
    return 1
  fi

  "$codex_path" --version 2>/dev/null | sed -n 's/.* \([0-9][0-9A-Za-z.+-]*\)$/\1/p' | head -n 1
}

current_installed_version() {
  version="$(version_from_binary "$CURRENT_LINK/bin/$BIN_NAME" || true)"
  if [ -n "$version" ]; then
    printf '%s\n' "$version"
    return 0
  fi

  version="$(version_from_binary "$CURRENT_LINK/bin/codex" || true)"
  if [ -n "$version" ]; then
    printf '%s\n' "$version"
    return 0
  fi

  version="$(version_from_binary "$CURRENT_LINK/codex" || true)"
  if [ -n "$version" ]; then
    printf '%s\n' "$version"
    return 0
  fi

  version="$(version_from_binary "$CURRENT_LINK/$BIN_NAME" || true)"
  if [ -n "$version" ]; then
    printf '%s\n' "$version"
    return 0
  fi

  return 0
}

resolve_existing_codex() {
  command -v "$BIN_NAME" 2>/dev/null || true
}

classify_existing_codex() {
  existing_path="$1"

  if [ -z "$existing_path" ] || [ "$existing_path" = "$BIN_PATH" ]; then
    return 1
  fi

  case "$existing_path" in
    /opt/homebrew/* | /usr/local/*)
      if [ "$os" = "darwin" ]; then
        printf 'brew\n'
        return 0
      fi
      ;;
  esac

  if [ -f "$existing_path" ] && grep -F "#!/usr/bin/env node" "$existing_path" >/dev/null 2>&1; then
    case "$existing_path" in
      *".bun"*)
        printf 'bun\n'
        ;;
      *)
        printf 'npm\n'
        ;;
    esac
    return 0
  fi

  return 1
}

prompt_yes_no() {
  prompt="$1"

  if [ "$ASSUME_YES" -eq 1 ]; then
    return 0
  fi

  case "$NON_INTERACTIVE" in
    1 | [Tt][Rr][Uu][Ee] | [Yy][Ee][Ss])
      return 1
      ;;
  esac

  if ( : </dev/tty ) 2>/dev/null; then
    printf '%s [y/N] ' "$prompt" >/dev/tty
    if ! IFS= read -r answer </dev/tty; then
      return 1
    fi
  elif [ -t 0 ]; then
    printf '%s [y/N] ' "$prompt"
    if ! IFS= read -r answer; then
      return 1
    fi
  else
    return 1
  fi

  case "$answer" in
    y | Y | yes | YES)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

print_launch_instructions() {
  case "$path_action" in
    added)
      step "Current terminal: export PATH=\"$BIN_DIR:\$PATH\" && $BIN_NAME"
      step "Future terminals: open a new terminal and run: $BIN_NAME"
      step "PATH was added to $path_profile"
      ;;
    updated)
      step "Current terminal: export PATH=\"$BIN_DIR:\$PATH\" && $BIN_NAME"
      step "Future terminals: open a new terminal and run: $BIN_NAME"
      step "PATH was updated in $path_profile"
      ;;
    configured)
      step "Current terminal: export PATH=\"$BIN_DIR:\$PATH\" && $BIN_NAME"
      step "Future terminals: open a new terminal and run: $BIN_NAME"
      step "PATH is already configured in $path_profile"
      ;;
    *)
      step "Current terminal: $BIN_NAME"
      step "Future terminals: open a new terminal and run: $BIN_NAME"
      ;;
  esac
}

maybe_launch_codex_now() {
  case "$LAUNCH_MODE" in
    always)
      step "Launching Codex"
      "$BIN_PATH"
      ;;
    never)
      return
      ;;
    *)
      if prompt_yes_no "Start Codex now?"; then
        step "Launching Codex"
        "$BIN_PATH"
      fi
      ;;
  esac
}

configure_vscode_extension() {
  if [ "$CONFIGURE_VSCODE" -ne 1 ]; then
    return
  fi

  step "Configuring VSCode ChatGPT extension"
  "$BIN_PATH" vscode install --yes --cli "$BIN_PATH"
}

detect_conflicting_install() {
  existing_path="$(resolve_existing_codex)"
  manager="$(classify_existing_codex "$existing_path" || true)"

  if [ -z "$manager" ]; then
    return
  fi

  conflict_manager="$manager"
  conflict_path="$existing_path"
  step "Detected existing $manager-managed Codex at $existing_path"
  warn "Multiple managed Codex installs can be ambiguous because PATH order decides which one runs."
}

handle_conflicting_install() {
  if [ -z "$conflict_manager" ]; then
    return
  fi

  if [ "$REMOVE_EXISTING" -ne 1 ]; then
    warn "Leaving the existing $conflict_manager-managed Codex installed. PATH order will determine which $BIN_NAME runs."
    warn "Run again with --remove-existing to remove it during installation."
    return
  fi

  case "$conflict_manager" in
    brew)
      uninstall_cmd="brew uninstall --cask codex"
      ;;
    bun)
      uninstall_cmd="bun remove -g @openai/codex"
      ;;
    *)
      uninstall_cmd="npm uninstall -g @openai/codex"
      ;;
  esac

  if prompt_yes_no "Uninstall the existing $conflict_manager-managed Codex now?"; then
    step "Running: $uninstall_cmd"
    if ! sh -c "$uninstall_cmd"; then
      warn "Failed to uninstall the existing $conflict_manager-managed Codex. Continuing with the standalone install."
    fi
  else
    warn "Leaving the existing $conflict_manager-managed Codex installed. PATH order will determine which $BIN_NAME runs."
  fi
}

install_package_release() {
  release_dir="$1"
  archive_path="$2"
  expected_version="$3"
  stage_release="$RELEASES_DIR/.staging.$(basename "$release_dir").$$"

  mkdir -p "$RELEASES_DIR"
  rm -rf "$stage_release"
  mkdir -p "$stage_release"
  tar -xzf "$archive_path" -C "$stage_release"

  if [ ! -f "$stage_release/bin/$BIN_NAME" ]; then
    if [ -f "$stage_release/bin/codex" ]; then
      echo "Refusing to install an official Codex package as VSP Codex; package is missing bin/$BIN_NAME." >&2
    else
      echo "VSP Codex package is missing bin/$BIN_NAME." >&2
    fi
    exit 1
  fi
  if ! package_metadata_is_vsp "$stage_release" "$vendor_target" "$expected_version"; then
    echo "VSP Codex package metadata is invalid or does not target bin/$BIN_NAME for $vendor_target." >&2
    exit 1
  fi
  if [ ! -f "$stage_release/codex-resources/vsp-session-compat.py" ]; then
    echo "VSP Codex package is missing codex-resources/vsp-session-compat.py." >&2
    exit 1
  fi

  chmod 0755 "$stage_release/bin/$BIN_NAME"
  actual_version="$(version_from_binary "$stage_release/bin/$BIN_NAME" || true)"
  if [ "$actual_version" != "$expected_version" ]; then
    echo "VSP Codex package binary version mismatch: expected $expected_version, got ${actual_version:-unknown}." >&2
    exit 1
  fi

  chmod 0755 \
    "$stage_release/bin/$BIN_NAME" \
    "$stage_release/bin/codex-code-mode-host" \
    "$stage_release/codex-path/rg" \
    "$stage_release/codex-resources/vsp-session-compat.py"
  if [ -f "$stage_release/codex-resources/bwrap" ]; then
    chmod 0755 "$stage_release/codex-resources/bwrap"
  fi
  ln -sf "bin/$BIN_NAME" "$stage_release/$BIN_NAME"

  if [ -e "$release_dir" ] || [ -L "$release_dir" ]; then
    rm -rf "$release_dir"
  fi
  mv "$stage_release" "$release_dir"
}

install_legacy_platform_npm_release() {
  release_dir="$1"
  archive_path="$2"
  target="$3"
  stage_release="$RELEASES_DIR/.staging.$(basename "$release_dir").$$"
  extract_dir="$tmp_dir/extract"
  vendor_root="$extract_dir/package/vendor/$target"

  mkdir -p "$RELEASES_DIR"
  rm -rf "$stage_release" "$extract_dir"
  mkdir -p "$stage_release/codex-resources" "$extract_dir"
  tar -xzf "$archive_path" -C "$extract_dir"

  if [ ! -f "$vendor_root/codex/$BIN_NAME" ]; then
    if [ -f "$vendor_root/codex/codex" ]; then
      echo "Refusing to install an official Codex npm package as VSP Codex; package is missing codex/$BIN_NAME." >&2
    else
      echo "VSP Codex npm package is missing codex/$BIN_NAME." >&2
    fi
    exit 1
  fi
  cp "$vendor_root/codex/$BIN_NAME" "$stage_release/$BIN_NAME"
  cp "$vendor_root/path/rg" "$stage_release/codex-resources/rg"
  chmod 0755 "$stage_release/$BIN_NAME" "$stage_release/codex-resources/rg"
  if [ -f "$vendor_root/codex-resources/bwrap" ]; then
    cp "$vendor_root/codex-resources/bwrap" "$stage_release/codex-resources/bwrap"
    chmod 0755 "$stage_release/codex-resources/bwrap"
  fi

  if [ -e "$release_dir" ] || [ -L "$release_dir" ]; then
    rm -rf "$release_dir"
  fi
  mv "$stage_release" "$release_dir"
}

release_dir_is_complete() {
  release_dir="$1"
  expected_version="$2"
  expected_target="$3"
  layout="$4"

  [ -d "$release_dir" ] &&
    [ "$(basename "$release_dir")" = "$expected_version-$expected_target" ] ||
    return 1

  case "$layout" in
    package)
      [ -f "$release_dir/codex-package.json" ] &&
        [ -x "$release_dir/bin/$BIN_NAME" ] &&
        [ -x "$release_dir/bin/codex-code-mode-host" ] &&
        [ -x "$release_dir/$BIN_NAME" ] &&
        [ -x "$release_dir/codex-path/rg" ] &&
        [ -x "$release_dir/codex-resources/vsp-session-compat.py" ] ||
        return 1
      package_metadata_is_vsp "$release_dir" "$expected_target" "$expected_version" || return 1
      [ "$(version_from_binary "$release_dir/bin/$BIN_NAME" || true)" = "$expected_version" ] ||
        return 1
      ;;
    legacy-platform-npm)
      [ -x "$release_dir/$BIN_NAME" ] &&
        [ -x "$release_dir/codex-resources/rg" ] ||
        return 1
      [ "$(version_from_binary "$release_dir/$BIN_NAME" || true)" = "$expected_version" ] ||
        return 1
      ;;
    *)
      return 1
      ;;
  esac

  case "$layout:$expected_target" in
    package:*linux* | legacy-platform-npm:*linux*) [ -x "$release_dir/codex-resources/bwrap" ] ;;
    *) true ;;
  esac
}

update_current_link() {
  release_dir="$1"
  tmp_link="$STANDALONE_ROOT/.current.$$"

  replace_path_with_symlink "$CURRENT_LINK" "$release_dir" "$tmp_link"
}

release_codex_relative_path() {
  release_dir="$1"

  if [ -x "$release_dir/bin/$BIN_NAME" ]; then
    printf 'bin/%s\n' "$BIN_NAME"
  elif [ -x "$release_dir/$BIN_NAME" ]; then
    printf '%s\n' "$BIN_NAME"
  else
    printf 'codex\n'
  fi
}

update_visible_command() {
  release_dir="$1"
  mkdir -p "$BIN_DIR"
  tmp_link="$BIN_DIR/.vsp-codex.$$"
  tmp_compat_link="$BIN_DIR/.codex.$$"
  tmp_code_mode_link="$BIN_DIR/.codex-code-mode-host.$$"
  codex_relative_path="$(release_codex_relative_path "$release_dir")"

  replace_path_with_symlink "$BIN_PATH" "$CURRENT_LINK/$codex_relative_path" "$tmp_link"
  replace_path_with_codex_compat_wrapper \
    "$CODEX_COMPAT_BIN_PATH" \
    "$CURRENT_LINK/$codex_relative_path" \
    "$tmp_compat_link"

  if [ "$os" = "darwin" ] && [ -x "$release_dir/bin/codex-code-mode-host" ]; then
    replace_path_with_symlink \
      "$CODE_MODE_HOST_BIN_PATH" \
      "$CURRENT_LINK/bin/codex-code-mode-host" \
      "$tmp_code_mode_link"
  elif [ "$(readlink "$CODE_MODE_HOST_BIN_PATH" 2>/dev/null || true)" = \
    "$CURRENT_LINK/bin/codex-code-mode-host" ]; then
    rm -f "$CODE_MODE_HOST_BIN_PATH"
  fi
}

verify_visible_command() {
  expected_version="$1"
  vsp_version="$(version_from_binary "$BIN_PATH" || true)"
  compat_version="$(version_from_binary "$CODEX_COMPAT_BIN_PATH" || true)"
  if [ "$vsp_version" != "$expected_version" ] || [ "$compat_version" != "$expected_version" ]; then
    echo "Installed Codex version mismatch: expected $expected_version, got vsp-codex=${vsp_version:-unknown}, codex=${compat_version:-unknown}." >&2
    return 1
  fi
  if [ "$os" = "darwin" ] && [ "$install_layout" = "package" ]; then
    [ -x "$CODE_MODE_HOST_BIN_PATH" ]
  fi
}

run_session_compat_migration() {
  release_dir="$1"
  script="$release_dir/codex-resources/vsp-session-compat.py"

  if [ ! -f "$script" ] ||
    [ ! -f "$OFFICIAL_CODEX_HOME_DIR/state_5.sqlite" ] ||
    [ ! -f "$CODEX_HOME_DIR/state_5.sqlite" ]; then
    return
  fi
  if ! command -v python3 >/dev/null 2>&1; then
    warn "python3 is unavailable; run 'vsp-codex session-compat apply' after installing it."
    return
  fi

  step "Synchronizing saved sessions with Codex Official"
  if ! python3 "$script" apply \
    --official-home "$OFFICIAL_CODEX_HOME_DIR" \
    --vsp-home "$CODEX_HOME_DIR"; then
    warn "Session migration did not complete. The install is usable; retry with 'vsp-codex session-compat apply'."
  fi
}

parse_args "$@"
validate_direct_archive_args

require_command mktemp
require_command tar

case "$(uname -s)" in
  Darwin)
    os="darwin"
    ;;
  Linux)
    os="linux"
    ;;
  *)
    echo "install.sh supports macOS and Linux. Use install.ps1 on Windows." >&2
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64 | amd64)
    arch="x86_64"
    ;;
  arm64 | aarch64)
    arch="aarch64"
    ;;
  *)
    echo "Unsupported architecture: $(uname -m)" >&2
    exit 1
    ;;
esac

if [ "$os" = "darwin" ] && [ "$arch" = "x86_64" ]; then
  if [ "$(sysctl -n sysctl.proc_translated 2>/dev/null || true)" = "1" ]; then
    arch="aarch64"
  fi
fi

if [ "$os" = "darwin" ]; then
  if [ "$arch" = "aarch64" ]; then
    npm_tag="darwin-arm64"
    vendor_target="aarch64-apple-darwin"
    platform_label="macOS (Apple Silicon)"
  else
    npm_tag="darwin-x64"
    vendor_target="x86_64-apple-darwin"
    platform_label="macOS (Intel)"
  fi
else
  if [ "$arch" = "aarch64" ]; then
    npm_tag="linux-arm64"
    vendor_target="aarch64-unknown-linux-musl"
    platform_label="Linux (ARM64)"
  else
    npm_tag="linux-x64"
    vendor_target="x86_64-unknown-linux-musl"
    platform_label="Linux (x64)"
  fi
fi

if [ -n "$ARCHIVE_URL" ]; then
  if [ -n "$ARCHIVE_TARGET" ]; then
    vendor_target="$ARCHIVE_TARGET"
  elif [ "$os" = "linux" ] && [ "$arch" = "x86_64" ]; then
    vendor_target="x86_64-unknown-linux-gnu"
  else
    echo "--target is required for this platform when --archive-url is used." >&2
    exit 1
  fi

  if [ -n "$ARCHIVE_VERSION" ]; then
    resolved_version="$(normalize_version "$ARCHIVE_VERSION")"
  elif [ "$(normalize_version "$RELEASE")" != "latest" ]; then
    resolved_version="$(normalize_version "$RELEASE")"
  else
    resolved_version="preview"
  fi

  install_layout="package"
  asset="vsp-codex-preview.tar.gz"
  checksum_asset=""
  download_url="$ARCHIVE_URL"
  checksum_url=""
elif [ "$RELEASE_EXPLICIT" -eq 0 ] && [ "$(normalize_version "$RELEASE")" = "latest" ]; then
  step "Resolving latest VSP release from manifest..."
  manifest_target="$vendor_target"
  if [ "$os" = "linux" ] && [ "$arch" = "x86_64" ]; then
    manifest_target="x86_64-unknown-linux-gnu"
  fi
  resolve_from_vsp_manifest "$manifest_target" "$ARCHIVE_CHANNEL"

  if [ -n "$ARCHIVE_TARGET" ]; then
    vendor_target="$ARCHIVE_TARGET"
  elif [ "$os" = "linux" ] && [ "$arch" = "x86_64" ]; then
    vendor_target="x86_64-unknown-linux-gnu"
  fi

  resolved_version="$(normalize_version "$ARCHIVE_VERSION")"
  install_layout="package"
  asset="vsp-codex-preview.tar.gz"
  checksum_asset=""
  download_url="$ARCHIVE_URL"
  checksum_url=""
else
  resolve_release
  package_asset="codex-package-$vendor_target.tar.gz"
  checksum_asset="codex-package_SHA256SUMS"
  if release_asset_exists "$package_asset" &&
    release_asset_exists "$checksum_asset"; then
    install_layout="package"
    asset="$package_asset"
  elif release_asset_exists "codex-npm-$npm_tag-$resolved_version.tgz"; then
    install_layout="legacy-platform-npm"
    asset="codex-npm-$npm_tag-$resolved_version.tgz"
  else
    echo "Could not find Codex package or platform npm release assets for Codex $resolved_version." >&2
    exit 1
  fi
  download_url="$(release_url_for_asset "$asset" "$resolved_version")"
  checksum_url="$(release_url_for_asset "$checksum_asset" "$resolved_version")"
fi
release_name="$resolved_version-$vendor_target"
release_dir="$RELEASES_DIR/$release_name"
current_version="$(current_installed_version)"

if [ -n "$current_version" ] && [ "$current_version" != "$resolved_version" ]; then
  step "Updating Codex CLI from $current_version to $resolved_version"
elif [ -n "$current_version" ]; then
  step "Updating Codex CLI"
else
  step "Installing Codex CLI"
fi
step "Detected platform: $platform_label"
step "Resolved version: $resolved_version"

detect_conflicting_install

tmp_dir="$(mktemp -d)"
cleanup() {
  release_install_lock
  if [ -n "$tmp_dir" ]; then
    rm -rf "$tmp_dir"
  fi
}
trap cleanup EXIT INT TERM

acquire_install_lock
cleanup_stale_install_artifacts

if ! release_dir_is_complete "$release_dir" "$resolved_version" "$vendor_target" "$install_layout"; then
  if [ -e "$release_dir" ] || [ -L "$release_dir" ]; then
    warn "Found incomplete existing release at $release_dir; reinstalling."
  fi

  archive_path="$tmp_dir/$asset"
  checksum_path="$tmp_dir/${checksum_asset:-checksum}"

  step "Downloading Codex CLI"
  if [ -n "$ARCHIVE_URL" ]; then
    expected_digest="$ARCHIVE_SHA256"
  elif [ "$install_layout" = "package" ]; then
    checksum_digest="$(release_asset_digest "$checksum_asset")"
    download_file "$checksum_url" "$checksum_path"
    verify_archive_digest "$checksum_path" "$checksum_digest"
    expected_digest="$(package_archive_digest "$asset" "$checksum_path")"
  else
    expected_digest="$(release_asset_digest "$asset")"
  fi
  download_file "$download_url" "$archive_path"
  verify_archive_digest "$archive_path" "$expected_digest"

  step "Installing standalone package to $release_dir"
  if [ "$install_layout" = "package" ]; then
    install_package_release "$release_dir" "$archive_path" "$resolved_version"
  else
    install_legacy_platform_npm_release "$release_dir" "$archive_path" "$vendor_target"
  fi
fi
update_current_link "$release_dir"
update_visible_command "$release_dir"
add_to_path
verify_visible_command "$resolved_version"
run_session_compat_migration "$release_dir"
configure_vscode_extension
release_install_lock
handle_conflicting_install

case "$path_action" in
  added)
    print_launch_instructions
    ;;
  updated)
    print_launch_instructions
    ;;
  configured)
    print_launch_instructions
    ;;
  *)
    step "$BIN_DIR is already on PATH"
    print_launch_instructions
    ;;
esac

printf 'Codex CLI %s installed successfully.\n' "$resolved_version"
maybe_launch_codex_now
