#!/usr/bin/env bash
#
# R0 inventory benchmark: materialize the pinned Audio Tensor Lab revision in
# an isolated temporary directory and verify Rootline's inventory oracle.
#
# A dirty live checkout must never serve as the golden snapshot, so this
# script always clones the pinned revision fresh instead of trusting a local
# checkout. Every result records the analyzed revision and scanner mode.
#
# Usage:
#   benchmarks/runners/r0-inventory.sh [--keep-workdir]
#
# Requirements: bash, git, cargo, sort, diff. Network access to fetch the
# corpus source. Run from anywhere; the Rootline checkout is derived from
# this script's location.
#
# Exit status: 0 when all oracle checks pass, 1 otherwise.

set -euo pipefail

KEEP_WORKDIR=0
for arg in "$@"; do
  case "$arg" in
    --keep-workdir) KEEP_WORKDIR=1 ;;
    -h|--help)
      sed -n '2,/^#$/p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "r0-inventory: unknown argument '$arg' (see --help)" >&2
      exit 2
      ;;
  esac
done

ROOTLINE="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CORPUS="$ROOTLINE/benchmarks/corpus.yaml"
EXPECTED="$ROOTLINE/benchmarks/expected/r0-inventory.txt"
RESULTS_DIR="$ROOTLINE/benchmarks/results"

# Read a scalar from the R0 block of corpus.yaml. The corpus file is kept
# flat and one-value-per-line so a clean checkout needs no YAML parser.
corpus_value() {
  local key="$1"
  local value
  value="$(grep -E "^[[:space:]]+$key:" "$CORPUS" | head -n 1 | sed -E "s/^[^:]+:[[:space:]]*//")"
  if [ -z "$value" ]; then
    echo "r0-inventory: cannot read '$key' from $CORPUS" >&2
    exit 2
  fi
  printf '%s' "$value"
}

SOURCE_URL="$(corpus_value source_url)"
PINNED_REV="$(corpus_value pinned_revision)"
EXPECTED_FILES="$(corpus_value files)"
EXPECTED_BYTES="$(corpus_value bytes)"
EXPECTED_SKIPPED="$(corpus_value skipped)"
EXPECTED_MODE="$(corpus_value scanner_mode)"

WORKDIR="$(mktemp -d "${TMPDIR:-/tmp}/rootline-r0-XXXXXX")"
cleanup() {
  if [ "$KEEP_WORKDIR" -eq 0 ]; then
    rm -rf "$WORKDIR"
  else
    echo "r0-inventory: workdir kept at $WORKDIR" >&2
  fi
}
trap cleanup EXIT

START_EPOCH="$(date +%s)"

echo "r0-inventory: cloning $SOURCE_URL at $PINNED_REV" >&2
git clone --quiet "$SOURCE_URL" "$WORKDIR/repo" >&2
git -C "$WORKDIR/repo" checkout --quiet "$PINNED_REV" >&2
ANALYZED_REV="$(git -C "$WORKDIR/repo" rev-parse HEAD)"
if [ "$ANALYZED_REV" != "$PINNED_REV" ]; then
  echo "r0-inventory: materialized revision $ANALYZED_REV != pinned $PINNED_REV" >&2
  exit 1
fi
if [ -n "$(git -C "$WORKDIR/repo" status --porcelain)" ]; then
  echo "r0-inventory: materialized checkout is not clean" >&2
  git -C "$WORKDIR/repo" status --porcelain >&2
  exit 1
fi

echo "r0-inventory: running inventory" >&2
OUTPUT="$WORKDIR/inventory.txt"
# --locked keeps the benchmark on the committed dependency set.
(cd "$ROOTLINE" && cargo run --quiet --locked --bin rootline -- index "$WORKDIR/repo" > "$OUTPUT")

field() {
  local name="$1"
  grep -E "^$name: " "$OUTPUT" | head -n 1 | sed -E "s/^$name: //"
}

ACTUAL_MODE="$(field 'listing')"
ACTUAL_REV="$(field 'git revision')"
ACTUAL_FILES="$(field 'files')"
ACTUAL_BYTES="$(field 'bytes')"
ACTUAL_SKIPPED="$(field 'skipped')"

FAILURES=0
check() {
  local label="$1" actual="$2" expected="$3"
  if [ "$actual" != "$expected" ]; then
    echo "r0-inventory: FAIL $label: got '$actual', want '$expected'" >&2
    FAILURES=$((FAILURES + 1))
  else
    echo "r0-inventory: ok $label = $actual" >&2
  fi
}

check "scanner mode" "$ACTUAL_MODE" "$EXPECTED_MODE"
check "git revision" "$ACTUAL_REV" "$PINNED_REV"
check "files" "$ACTUAL_FILES" "$EXPECTED_FILES"
check "bytes" "$ACTUAL_BYTES" "$EXPECTED_BYTES"
check "skipped" "$ACTUAL_SKIPPED" "$EXPECTED_SKIPPED"

# Artifact rows are the `category<TAB>bytes<TAB>path` lines; header and warning
# lines never contain two tabs, so select rows structurally rather than by
# line number (the `git revision` header is absent outside Git checkouts).
ARTIFACTS="$WORKDIR/artifacts.txt"
awk -F'\t' 'NF == 3' "$OUTPUT" > "$ARTIFACTS"
ACTUAL_PATHS="$WORKDIR/paths.txt"
awk -F'\t' '{print $3}' "$ARTIFACTS" > "$ACTUAL_PATHS"

if ! LC_ALL=C sort -c "$ACTUAL_PATHS" 2>/dev/null; then
  echo "r0-inventory: FAIL inventory paths are not in deterministic order" >&2
  FAILURES=$((FAILURES + 1))
else
  echo "r0-inventory: ok deterministic path ordering" >&2
fi

if ! diff -u "$EXPECTED" "$ACTUAL_PATHS" > "$WORKDIR/paths.diff"; then
  echo "r0-inventory: FAIL inventory paths differ from benchmarks/expected/r0-inventory.txt" >&2
  cat "$WORKDIR/paths.diff" >&2
  FAILURES=$((FAILURES + 1))
else
  echo "r0-inventory: ok exact inventory paths" >&2
fi

for category in code config documentation data other infrastructure; do
  count="$(awk -F'\t' -v category="$category" '$1 == category' "$ARTIFACTS" | wc -l | tr -d ' ')"
  echo "r0-inventory: category $category = $count" >&2
done

END_EPOCH="$(date +%s)"
DURATION_S=$((END_EPOCH - START_EPOCH))
ROOTLINE_REV="$(git -C "$ROOTLINE" rev-parse HEAD)"
ROOTLINE_VERSION="$(grep -E '^version = ' "$ROOTLINE/crates/rootline-cli/Cargo.toml" | head -n 1 | sed -E 's/^version = "(.*)"$/\1/')"
HOST_INFO="$(uname -srm)"

mkdir -p "$RESULTS_DIR"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RESULT="$RESULTS_DIR/r0-inventory-$STAMP.json"
if [ "$FAILURES" -eq 0 ]; then RESULT_PASS=true; else RESULT_PASS=false; fi
cat > "$RESULT" <<EOF
{
  "benchmark": "R0-inventory",
  "pass": $RESULT_PASS,
  "source_url": "$SOURCE_URL",
  "pinned_revision": "$PINNED_REV",
  "analyzed_revision": "$ANALYZED_REV",
  "scanner_mode": "$ACTUAL_MODE",
  "files": $ACTUAL_FILES,
  "bytes": $ACTUAL_BYTES,
  "skipped": $ACTUAL_SKIPPED,
  "duration_s": $DURATION_S,
  "rootline_revision": "$ROOTLINE_REV",
  "rootline_cli_version": "$ROOTLINE_VERSION",
  "host": "$HOST_INFO",
  "failures": $FAILURES
}
EOF

if [ "$FAILURES" -ne 0 ]; then
  echo "r0-inventory: $FAILURES check(s) failed; result kept at $RESULT" >&2
  exit 1
fi
echo "r0-inventory: all checks passed; result kept at $RESULT" >&2
