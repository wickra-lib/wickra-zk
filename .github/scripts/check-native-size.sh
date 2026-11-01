#!/usr/bin/env bash
# Size gate for the native libraries wickra-zk-go ships in git.
#
# The Go module carries the prebuilt C ABI library for each platform, because a
# Go module is served from its repository and cgo links the library at build
# time. That puts two hard limits on it:
#
#   * GitHub refuses to push a file larger than 100 MiB. The library holds the
#     whole risc0 prover (the Go binding proves), so it is ~91 MiB on Linux
#     x86_64, the largest of the four platforms.
#   * The Go module proxy refuses a module whose files add up to more than
#     500 MiB (golang.org/x/mod/zip.MaxZipFile).
#
# Crossing either fails the release's mirror push, after the other registries
# have already published. ci.yml runs this on every PR against the library it
# builds, so a dependency bump that grows it goes red on its own PR; release.yml
# runs it with --module against the assembled module before the push.
#
#   check-native-size.sh FILE...        each file against the per-file limit
#   check-native-size.sh --module DIR   every file in DIR, and their total
set -euo pipefail

MIB=$((1024 * 1024))
FILE_LIMIT=$((100 * MIB)) # GitHub: files larger than this are rejected
FILE_WARN=$((95 * MIB))   # warn while there is still room to act
MODULE_LIMIT=$((500 * MIB)) # Go module proxy: total extracted size
MODULE_WARN=$((450 * MIB))

mib() { awk -v b="$1" -v m="$MIB" 'BEGIN { printf "%.1f MiB", b / m }'; }

module=""
if [ "${1:-}" = "--module" ]; then
  module=${2:?usage: check-native-size.sh --module DIR}
  mapfile -t files < <(find "$module" -type f | sort)
else
  files=("$@")
fi
[ "${#files[@]}" -gt 0 ] || { echo "usage: check-native-size.sh FILE... | --module DIR" >&2; exit 2; }

failed=0
total=0
for f in "${files[@]}"; do
  [ -f "$f" ] || { echo "::error::$f does not exist"; exit 1; }
  size=$(wc -c < "$f" | tr -d ' ')
  total=$((total + size))
  if [ "$size" -gt "$FILE_LIMIT" ]; then
    echo "::error file=$f::$f is $(mib "$size"); GitHub rejects a pushed file over $(mib "$FILE_LIMIT"), so the wickra-zk-go mirror cannot carry it"
    failed=1
  elif [ "$size" -gt "$FILE_WARN" ]; then
    echo "::warning file=$f::$f is $(mib "$size"), within $(mib $((FILE_LIMIT - size))) of GitHub's $(mib "$FILE_LIMIT") file limit for the wickra-zk-go mirror"
  fi
  # The per-library lines are the trend worth reading; the module's small
  # sources would only bury them.
  if [ -z "$module" ] || [ "$size" -gt "$MIB" ]; then
    echo "$f: $(mib "$size")"
  fi
done

if [ -n "$module" ]; then
  echo "module total: $(mib "$total") of $(mib "$MODULE_LIMIT")"
  if [ "$total" -gt "$MODULE_LIMIT" ]; then
    echo "::error::the wickra-zk-go module is $(mib "$total"); the Go module proxy refuses a module over $(mib "$MODULE_LIMIT")"
    failed=1
  elif [ "$total" -gt "$MODULE_WARN" ]; then
    echo "::warning::the wickra-zk-go module is $(mib "$total"), within $(mib $((MODULE_LIMIT - total))) of the Go module proxy's $(mib "$MODULE_LIMIT") limit"
  fi
fi
exit "$failed"
