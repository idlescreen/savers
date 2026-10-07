#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright 2026 IdleScreen
#
# Regenerate `perf-baseline.json` from a fresh `cargo bench` run.
#
# Usage:
#   scripts/refresh-baseline.sh              # capture every saver
#   scripts/refresh-baseline.sh aurora       # capture one saver
#
# Numbers are read from criterion's own JSON — `target/criterion/*/new/
# estimates.json` — never from scraped stdout. Only files written after
# this script started are collected, so a stale directory left behind by
# a bench that was renamed cannot leak into the baseline.
#
# All twelve savers are T1: each holds the `Screensaver` impl that the
# `tick` bench drives, so each is gated.
set -euo pipefail

cd "$(dirname "$0")/.."

SAVERS=(ascii aurora beams bursts chaos cosmos glyphs gnats hearth radar ripple storm)

GROUP="${1:-}"
if [ -n "$GROUP" ] && ! printf '%s\n' "${SAVERS[@]}" | grep -qx "$GROUP"; then
    echo "Usage: $0 [${SAVERS[*]}]" >&2
    exit 2
fi

CRITERION_DIR="target/criterion"
# Stamped before the run; anything criterion wrote after this instant
# belongs to this run and nothing older does.
MARKER=$(mktemp)
OUT=$(mktemp)
trap 'rm -f "$MARKER" "$OUT"' EXIT

COMMIT=$(git rev-parse --short=7 HEAD)
CAPTURED_AT=$(date -u +%Y-%m-%d)

run_one() {
    local saver="$1"
    echo "Running T1 bench target: ${saver}/tick"
    # criterion exits non-zero when a benchmark records a change; the
    # JSON is still written, so the failure is logged, not fatal.
    #
    # No `-q` here: criterion 0.5 dropped the flag, and passing it made
    # every run die with "unexpected argument found".
    if ! cargo bench -p "$saver" --bench tick \
            -- --warm-up-time 1 --measurement-time 3 >"$OUT" 2>&1; then
        echo "WARNING: cargo bench reported a failure for $saver; reading JSON anyway" >&2
        tail -20 "$OUT" >&2
    fi
}

if [ -n "$GROUP" ]; then
    run_one "$GROUP"
else
    for saver in "${SAVERS[@]}"; do
        run_one "$saver"
    done
fi

python3 - "$CRITERION_DIR" "$MARKER" "$COMMIT" "$CAPTURED_AT" <<'PY'
import json
import os
import sys
from pathlib import Path

crit_dir, marker, commit, captured_at = sys.argv[1:5]
mark = os.path.getmtime(marker)

benches = {}
for path in sorted(Path(crit_dir).rglob("estimates.json")):
    if path.parent.name != "new":
        continue  # `base/` is the prior run, `change/` is the diff
    try:
        if os.path.getmtime(path) < mark - 1.0:
            continue  # left over from an earlier run
        data = json.loads(path.read_text())
        median = data["median"]["point_estimate"]
        mad = data.get("median_abs_dev", {}).get("point_estimate")
    except (OSError, ValueError, KeyError) as e:
        print(f"WARNING: skipping {path}: {e}", file=sys.stderr)
        continue
    name = str(path.parent.parent.relative_to(crit_dir))
    benches[name] = {"median_ns": median, "median_abs_dev_ns": mad}

if not benches:
    print(f"ERROR: no fresh estimates.json under {crit_dir} after the run", file=sys.stderr)
    print("       refusing to write an empty baseline; check that cargo bench ran",
          file=sys.stderr)
    sys.exit(2)

out = {
    "version": 2,
    "captured_at": captured_at,
    "commit": commit,
    "note": "Captured by scripts/refresh-baseline.sh from criterion estimates.json",
    "benches": benches,
}
Path("perf-baseline.json.tmp").write_text(json.dumps(out, indent=2, sort_keys=True) + "\n")
print(f"Wrote {len(benches)} bench entries to perf-baseline.json.tmp")
PY

# Show the delta against the existing baseline before overwriting it.
if [ -f perf-baseline.json ]; then
    echo
    echo "Delta vs current perf-baseline.json (informational, not enforced):"
    python3 scripts/compare-bench.py perf-baseline.json "$CRITERION_DIR" || true
fi

mv perf-baseline.json.tmp perf-baseline.json

echo
echo "Wrote perf-baseline.json. Review the diff, then:"
echo "  git add perf-baseline.json && git commit -m 'savers: refresh perf baseline'"
