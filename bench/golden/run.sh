#!/bin/bash
# The golden test (docs/GOLDEN.md): the reference PDFs converted with their stored model answers
# and layout (--replay-only: no model runs, a request Sift makes differently shows up as a miss);
# the Markdown and reports must equal tests/golden/expected/<variant> byte for byte.
#
#   bash bench/golden/run.sh [full|nollm|both]      (default: both)
#
#   SIFT_GOLDEN_PDFS  folder holding the reference PDFs (paths in tests/golden/pdfs.txt; default:
#                     ../SLAB/DATASHEET_TRUTH_SOURCE_HARDWARE next to the repository, the
#                     maintainer's layout). The PDFs are not in the repository: docs/GOLDEN.md
#                     lists them with their sha256.
#   SIFT_BIN          the sift binary to test (default target/release/sift.exe)
#   SIFT_GOLDEN_OUT   where the outputs go (default target/golden)
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
G="$ROOT/tests/golden"
PDFS="${SIFT_GOLDEN_PDFS:-$ROOT/../SLAB/DATASHEET_TRUTH_SOURCE_HARDWARE}"
if [ ! -d "$PDFS" ]; then
  echo "golden: the reference PDFs folder $PDFS does not exist; set SIFT_GOLDEN_PDFS (docs/GOLDEN.md)"
  exit 2
fi
BIN="${SIFT_BIN:-$ROOT/target/release/sift.exe}"
OUT="${SIFT_GOLDEN_OUT:-$ROOT/target/golden}"
# the defaults only: the saved settings must not change the golden output
export SIFT_SETTINGS="$OUT/no-settings.toml"
unset SIFT_ANSWERS
case "${1:-both}" in
  full) VARIANTS=(full) ;;
  nollm) VARIANTS=(nollm) ;;
  both) VARIANTS=(full nollm) ;;
  *) echo "usage: run.sh [full|nollm|both]"; exit 2 ;;
esac
rc=0
for v in "${VARIANTS[@]}"; do
  extra=()
  [ "$v" = nollm ] && extra=(--no-llm)
  rm -rf "$OUT/$v"
  echo "== golden: $v"
  while IFS='|' read -r stem rel; do
    { [ -z "$stem" ] || [[ "$stem" == \#* ]]; } && continue
    "$BIN" convert "$PDFS/$rel" -o "$OUT/$v" --replay "$G/answers" --replay-only --layout-from "$G/layout" "${extra[@]}" 2>&1 \
      | grep -E "FAILED|panicked|not in the answer|rror" | head -3
  done < "$G/pdfs.txt"
  uv run --no-project python -I "$HERE/compare.py" "$G/expected/$v" "$OUT/$v" || rc=1
done
exit $rc
