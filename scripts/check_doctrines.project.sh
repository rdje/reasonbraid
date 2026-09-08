#!/usr/bin/env bash
# scripts/check_doctrines.project.sh — THE PROJECT-SPECIFIC DOCTRINE SLOT.
#
# This is where a project instantiated from the template adds ITS OWN mechanizable
# doctrine checks — the equivalent of PGEN's "EBNF is the single source of truth",
# "regex self-hosts", "cert-coverage / shape-contract gates", etc.
#
# It runs LAST in scripts/check_doctrines.sh. Exit 0 = all project doctrines pass;
# exit nonzero (with a message on stderr) = a breach that blocks the commit.
#
# The template ships this as a passing no-op. Add checks below as your project grows;
# keep each one cheap, deterministic, and self-describing. For anything heavier than a
# few seconds, gate it in CI instead and keep this hook fast.
set -uo pipefail

# --- add project-specific checks here ---
# Example:
#   ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"
#   if ! cargo fmt --all -- --check >/dev/null 2>&1; then
#     echo "PROJECT: rustfmt drift — run 'cargo fmt --all'" >&2; exit 1
#   fi

# The SDK compatibility matrix (`.4.2`): the token + the evidence artifacts
# must agree with the code — the matrix is evidence-bound, never prose-bound.
if ! scripts/check_compatibility_matrix.sh >/dev/null 2>&1; then
    scripts/check_compatibility_matrix.sh >&2
    exit 1
fi

exit 0
