#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
handoff_dir="$repo_root/handoffs"

if [[ ! -d "$handoff_dir" ]]; then
  echo "No handoffs directory found." >&2
  exit 1
fi

latest="$(
  find "$handoff_dir" -type f -name '*.md' -print0 |
    xargs -0 ls -t 2>/dev/null |
    head -n 1
)"

if [[ -z "$latest" ]]; then
  echo "No handoff markdown files found." >&2
  exit 1
fi

printf '%s\n' "${latest#"$repo_root/"}"
