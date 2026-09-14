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

# The director's public-repository correction (`.11.4.3.1.2.3`): the superseded
# private instruction had already leaked past two hand-run censuses, so every
# private-visibility sentence is now reviewed mechanically.
if ! scripts/check_visibility_policy.sh >/dev/null 2>&1; then
    scripts/check_visibility_policy.sh >&2
    exit 1
fi

# The book may not hold a SECOND, unchecked copy of the task tree's frontier
# (`.11.4.3.1.2.13`): a stale pointer misleads the review surface itself.
if ! scripts/check_book_frontier.sh >/dev/null 2>&1; then
    scripts/check_book_frontier.sh >&2
    exit 1
fi

# Every tracked text file ends with exactly one newline (`.11.4.3.1.2.16`): a
# blank line at end of file reached a commit and forced a correction commit.
if ! scripts/check_file_termination.sh >/dev/null 2>&1; then
    scripts/check_file_termination.sh >&2
    exit 1
fi

# Every intra-book link resolves (`.11.4.3.1.2.18`): mdbook does not check them,
# so three dead links shipped to the surface the director reads.
if ! scripts/check_book_links.sh >/dev/null 2>&1; then
    scripts/check_book_links.sh >&2
    exit 1
fi

# Project data stays on the repository volume and names are proved by creation
# (`.11.4.3.1.2.21`): §13 was prose for the life of the project and was breached
# in 26 places — 8 ambient temporary directories and 18 clock-derived paths.
if ! scripts/check_storage_locality.sh >/dev/null 2>&1; then
    scripts/check_storage_locality.sh >&2
    exit 1
fi

# The SDK compatibility matrix (`.4.2`): the token + the evidence artifacts
# must agree with the code — the matrix is evidence-bound, never prose-bound.
if ! scripts/check_compatibility_matrix.sh >/dev/null 2>&1; then
    scripts/check_compatibility_matrix.sh >&2
    exit 1
fi

# Every reason code the SERVER emits is named in the book's table
# (`SIGNOFF-REPAIR.11.7`). §9.8 publishes a stable registry of 20 and the
# product emits 18, NINE of which postdate that list; `ReasonCode::Unknown`
# preserves them, so nothing broke — but nothing told a client author they
# existed either. ⭐ This gate fires on ZERO breaches today and would have fired
# on all nine, which is the shape a gate should have: it catches the NEXT
# drift rather than presenting a backlog.
if ! python3 -B scripts/census_reason_codes.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_reason_codes.py --check >&2
    exit 1
fi

# Every `attach` clause is NAMED by the leaf that owns it (`SIGNOFF-REPAIR.11.11`).
# `attach` is the one ledger state whose Next action is not "none": it needs a
# SENTENCE written into a leaf the classifier does not own, and every other
# property of a row is visible in the row itself. Tranche 1 recorded three and
# wrote none, so `SIGNOFF-REPAIR.11.9`'s own mechanism was live inside the
# instrument built to stop it. ⭐ Measured before registering: 3 of 3 breaching at
# tranche 1's close (`5862837`), and 0 of 7, 0 of 15 and 0 of 26 at every tranche
# close since — the `REASON-CODE-DOC` shape, not the backlog shape `.11.9`
# rejected. ⛔ The rule is the RECORD ID in the owner's own section, never a
# phrase: matching prose would have to guess at paraphrase.
if ! python3 -B scripts/census_record_reconciliation.py --classified >/dev/null 2>&1; then
    python3 -B scripts/census_record_reconciliation.py --classified >&2
    exit 1
fi

exit 0
