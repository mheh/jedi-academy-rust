#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

oracle_keys="$tmp_dir/oracle-keys.tsv"
oracle_candidates="$tmp_dir/oracle-candidates.tsv"
rust_keys="$tmp_dir/rust-keys.tsv"
oracle_missing="$tmp_dir/oracle-missing.tsv"
rust_missing="$tmp_dir/rust-missing.tsv"

normalize_key() {
  printf '%s' "$1" | tr '[:upper:]' '[:lower:]'
}

oracle_to_rows() {
  local file rel dir name ext key direct_key header_key expected

  find "$repo_root/oracle/code" "$repo_root/oracle/codemp" \
    -type f \( -name '*.c' -o -name '*.cpp' -o -name '*.h' \) | sort |
    while IFS= read -r file; do
      rel="${file#"$repo_root/oracle/"}"
      dir="${rel%/*}"
      name="$(basename "$rel")"
      ext=".${name##*.}"
      name="${name%.*}"

      case "$ext" in
        .c|.cpp)
          key="$(normalize_key "$dir/$name")"
          expected="src/$dir/$name.rs"
          printf '%s\t%s\t%s\n' "$key" "oracle/$rel" "$expected" >> "$oracle_keys"
          printf '%s\t%s\t%s\t%s\n' "$key" "" "oracle/$rel" "$expected" >> "$oracle_candidates"
          ;;
        .h)
          header_key="$(normalize_key "$dir/${name}_h")"
          direct_key="$(normalize_key "$dir/$name")"
          expected="src/$dir/${name}_h.rs"
          printf '%s\t%s\t%s\n' "$header_key" "oracle/$rel" "$expected" >> "$oracle_keys"
          # Some existing ports fold a header into the direct module name
          # (`g_local.h` -> `g_local.rs`) rather than a separate `_h` module,
          # so a header is paired when either key exists.
          printf '%s\t%s\t%s\t%s\n' "$header_key" "$direct_key" "oracle/$rel" "$expected" >> "$oracle_candidates"
          ;;
      esac
    done
}

rust_to_key_rows() {
  local file rel stem key

  find "$repo_root/src/code" "$repo_root/src/codemp" -type f -name '*.rs' | sort |
    while IFS= read -r file; do
      rel="${file#"$repo_root/src/"}"
      if [[ "$(basename "$rel")" == "mod.rs" ]]; then
        continue
      fi
      stem="${rel%.rs}"
      key="$(normalize_key "$stem")"
      printf '%s\t%s\n' "$key" "src/$rel"
    done
}

: > "$oracle_keys"
: > "$oracle_candidates"
oracle_to_rows
rust_to_key_rows > "$rust_keys"

awk -F '\t' '
  NR == FNR { rust[$1] = 1; next }
  {
    primary = $1
    alternate = $2
    if (!rust[primary] && (alternate == "" || !rust[alternate])) {
      print $3 "\t" $4
    }
  }
' "$rust_keys" "$oracle_candidates" |
  sort -u > "$oracle_missing"

awk -F '\t' '
  NR == FNR { oracle[$1] = 1; next }
  !oracle[$1] { print $2 }
' "$oracle_keys" "$rust_keys" |
  sort -u > "$rust_missing"

oracle_missing_count="$(wc -l < "$oracle_missing" | tr -d ' ')"
rust_missing_count="$(wc -l < "$rust_missing" | tr -d ' ')"

cat <<EOF
# src/oracle comparison

Oracle files without a paired Rust file: $oracle_missing_count
Rust files without an obvious oracle source file: $rust_missing_count

## Oracle missing Rust

EOF

if [[ "$oracle_missing_count" -eq 0 ]]; then
  printf 'None\n'
else
  awk -F '\t' '{ print "- " $1 " -> " $2 }' "$oracle_missing"
fi

cat <<EOF

## Rust missing Oracle

EOF

if [[ "$rust_missing_count" -eq 0 ]]; then
  printf 'None\n'
else
  awk '{ print "- " $0 }' "$rust_missing"
fi
