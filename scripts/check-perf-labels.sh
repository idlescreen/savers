#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright 2026 IdleScreen
#
# Check that the performance claims written into `// perf:` page labels
# are backed by something a machine can verify. Tier definitions live in
# .github/RULES.md §4.
#
# The check is deliberately narrow. It polices T1 *claims*, not T1
# coverage: a page with no label is simply not gated, while a page that
# claims T1 had better be telling the truth about its bench. Requiring
# every page to carry a label would be the slogan §4 replaces.
#
# For a T1 label we require:
#   1. `bench:` names a real `[[bench]]` target, not `none`.
#   2. That target's source actually references the page's symbol.
#   3. `gate:` names a baseline file that exists in the repo root.
#
# T2 is advisory: a named bench target must still exist so the claim
# cannot rot, but the symbol reference and the gate are not required.
# T3 is QA-only and carries no bench claim at all.
#
# Note on `sym:` — it is not an optional convenience here, it is load
# bearing. Every T1 page in this repo holds a `Screensaver` trait impl
# that must stay co-located with the trait, so the filename
# (`screensaver_impl.rs`, `mod.rs`) never matches a callable symbol. The
# label names the trait instead, and the `tick` target imports it.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

# `runtime/` is the engine, checked out here as a path dependency. It has
# its own linter and its own baseline; its labels must not be judged
# against this repo's.
EXCLUDES=(--exclude-dir=target --exclude-dir=runtime)

errors=0

fail() {
    printf 'perf-label: %s\n' "$1" >&2
    errors=$((errors + 1))
}

# Labels separate fields with U+00B7 MIDDLE DOT. Rewrite to `|` so the
# field parsing below stays pure ASCII and needs no locale games.
DOT=$(printf '\xc2\xb7')

# Pull `key: value` out of a rewritten label. awk exits on the first
# match, so the whole field is drained and no stage of this pipeline can
# die on SIGPIPE under `set -o pipefail`.
field() {
    printf '%s' "$1" | tr '|' '\n' |
        awk -v key="$2:" 'index($0, key) {
            sub("^.*" key, "")
            gsub(/^[ \t]+|[ \t]+$/, "")
            print
            exit
        }'
}

# Map bench target name -> source file. Workspace-wide, not per-crate:
# a saver's `tick` bench can reach into a sibling for shared helpers.
# `find` has no --exclude-dir, so the prune is written out longhand.
declare -A BENCH
while IFS= read -r f; do
    name="$(basename "$f" .rs)"
    [ -n "${BENCH[$name]:-}" ] || BENCH[$name]="$f"
done < <(find . -type d \( -name target -o -name runtime \) -prune \
             -o -type f -path '*/benches/*.rs' -print)

checked=0

while IFS= read -r hit; do
    file="${hit%%:*}"
    rest="${hit#*:}"
    lineno="${rest%%:*}"
    label="${rest#*:}"

    fields="${label//$DOT/|}"
    tier="$(field "$fields" perf)"
    bench="$(field "$fields" bench)"
    sym="$(field "$fields" sym)"
    gate="$(field "$fields" gate)"

    if [ -z "$sym" ]; then
        sym="$(basename "$file" .rs)"
    fi

    checked=$((checked + 1))
    where="$file:$lineno"

    if [ -z "$tier" ]; then
        fail "$where: label has no tier (expected T1, T2 or T3)"
        continue
    fi

    if [ -n "$bench" ] && [ "$bench" != "none" ] && [ -z "${BENCH[$bench]:-}" ]; then
        fail "$where: $tier names bench '$bench', which is not a bench target in this repo"
        continue
    fi

    case "$tier" in
        T1)
            if [ "$bench" = "none" ] || [ -z "$bench" ]; then
                fail "$where: T1 must name a bench target; '$sym' is never gated if it has none"
                continue
            fi
            if ! grep -q -- "$sym" "${BENCH[$bench]}"; then
                fail "$where: T1 claims bench '$bench', but that source never references '$sym'"
                continue
            fi
            if [ -z "$gate" ]; then
                fail "$where: T1 must name a gate: (regressions are checked against a baseline)"
                continue
            fi
            if [ ! -f "$ROOT/$gate" ]; then
                fail "$where: T1 gate '$gate' does not exist at the repo root"
                continue
            fi
            ;;
        T2)
            # Target existence already checked above; nothing else is owed.
            ;;
        T3)
            if [ -n "$bench" ] && [ "$bench" != "none" ]; then
                fail "$where: T3 is QA-only and must not claim a bench"
            fi
            ;;
        *)
            fail "$where: unknown tier '$tier' (expected T1, T2 or T3)"
            ;;
    esac
done < <(
    grep -rn --include='*.rs' "${EXCLUDES[@]}" \
        -E '^[[:space:]]*//[[:space:]]*perf:[[:space:]]' . |
        sed 's|^\./||'
)

if [ "$checked" -eq 0 ]; then
    echo "perf-label: no labels found — is the grep pattern stale?" >&2
    exit 1
fi

if [ "$errors" -gt 0 ]; then
    echo "perf-label: $errors bad label(s) across $checked checked" >&2
    exit 1
fi

echo "perf-label: $checked label(s) OK"
