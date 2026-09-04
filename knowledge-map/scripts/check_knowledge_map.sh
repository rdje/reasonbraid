#!/usr/bin/env bash
# knowledge-map/scripts/check_knowledge_map.sh — KNOWLEDGE-MAP doctrine.
# Verify the committed KNOWLEDGE_MAP.md equals a fresh render of its sources. The pre-commit
# hook regenerates+stages the map, so this always passes locally; it catches drift where the
# hook did not run (CI, a hand edit, a --no-verify commit).
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"
gen="$ROOT/knowledge-map/scripts/gen_knowledge_map.sh"
MAP="$("$gen" --print-map-path)"

if [ ! -f "$MAP" ]; then
  echo "KNOWLEDGE-MAP: $MAP is missing — run knowledge-map/scripts/gen_knowledge_map.sh > \"\$($gen --print-map-path)\"" >&2
  exit 1
fi
if ! diff -q <("$gen") "$MAP" >/dev/null 2>&1; then
  echo "KNOWLEDGE-MAP: KNOWLEDGE_MAP.md is out of sync with its sources — regenerate it:" >&2
  echo "  knowledge-map/scripts/gen_knowledge_map.sh > \"\$(knowledge-map/scripts/gen_knowledge_map.sh --print-map-path)\"" >&2
  exit 1
fi
exit 0
