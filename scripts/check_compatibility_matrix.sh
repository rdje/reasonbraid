#!/usr/bin/env bash
# scripts/check_compatibility_matrix.sh — the `.4.2` mechanical re-derivation
# (the schema's "the matrix is evidence-bound" rule, enforced): the matrix's
# sdk_version column carries the contract's CURRENT token; the cited evidence
# artifacts exist (the conformance suite's three tests + the corpus manifest);
# the corpus is the replay oracle the matrix cites. A drift fails here —
# never as a prose note.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MATRIX="$ROOT/docs/compatibility-matrix.md"
fails=0

# 1. The token: the matrix's sdk_version cell == the contract's SDK_VERSION.
TOKEN="$(grep -o 'pub const SDK_VERSION: &str = "[0-9]*"' "$ROOT/crates/reasonbraid-adapter/src/contract.rs" | grep -o '[0-9]*')"
if [ -z "$TOKEN" ]; then
    echo "compatibility-matrix: SDK_VERSION not found in contract.rs" >&2
    fails=1
elif ! grep -q "| \`$TOKEN\` |" "$MATRIX"; then
    echo "compatibility-matrix: the sdk_version column must carry \`$TOKEN\` (the contract's token)" >&2
    fails=1
fi

# 2. The corpus manifest (the replay oracle) exists + the matrix cites it.
if [ ! -f "$ROOT/crates/reasonbraid-adapter/fixtures/MANIFEST.json" ]; then
    echo "compatibility-matrix: the corpus manifest is missing" >&2
    fails=1
fi
grep -q "fixtures/MANIFEST.json" "$MATRIX" || {
    echo "compatibility-matrix: the corpus evidence must cite fixtures/MANIFEST.json" >&2
    fails=1
}

# 3. The conformance-suite evidence cites the REAL test names (the sweep's
#    measured runs — the suite file carries them).
for t in fake_adapter_passes_the_conformance_suite \
         codex_adapter_passes_the_conformance_suite \
         claude_adapter_passes_the_conformance_suite; do
    grep -q "$t" "$ROOT/crates/reasonbraid-adapter/tests/adapter_conformance.rs" || {
        echo "compatibility-matrix: the conformance test $t is missing from the suite" >&2
        fails=1
    }
    grep -q "$t" "$MATRIX" || {
        echo "compatibility-matrix: the evidence column must cite $t" >&2
        fails=1
    }
done

if [ "$fails" = 0 ]; then
    echo "compatibility-matrix: OK (the token + the evidence artifacts agree)"
fi
exit "$fails"
