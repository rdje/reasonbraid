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
# product emits 18, NINE of which postdate that list — re-derive both with
# `python3 -B scripts/census_reason_codes.py`, never from this line, which has
# now moved TWICE: `SIGNOFF-REPAIR.9.2.1.1` took it from 18/NINE to 19/TEN by
# adding a code, and `SIGNOFF-REPAIR.11.14.3.2` took it back to 18/NINE by
# RETIRING one (`locator_digest_conflict` disclosed the cross-tenant existence
# §9.8 forbids). ⭐ The second move is why the "never from this line" clause is
# not boilerplate: a count can fall as well as rise. `ReasonCode::Unknown`
# preserves them, so nothing broke — but nothing told a client author they
# existed either. ⭐ This gate fires on ZERO breaches today and would have fired
# on all nine, which is the shape a gate should have: it catches the NEXT
# drift rather than presenting a backlog.
#
# ⛔⛔ THE SAME INVOCATION NOW CARRIES A SECOND ARM, `ACQUISITION-KIND-DOC`
# (`SIGNOFF-REPAIR.7.2.10`): an acquisition refusal travels in
# `acquisition_error.kind`, NOT in `code`, and the two vocabularies share not
# one string — measured, 0 overlap. So this census was RIGHT to key on `code:`
# and the gap it left was real: 41 kinds a client branches on, and the book
# documented none of them. ⭐ Derived from the PRODUCERS — each
# `AcquisitionError { kind: … }` construction, walked from `kind:` to
# `message:` — because two cheaper keys were both wrong at once: a marker of
# `kind: match &error {` missed the `GitError` site (written without the `&`),
# and a `kind:\s*"…"` regex also matched `derived_kind:` and `actor_kind:`.
# ⚠️ The set is OPEN at five sites that forward a worker-chosen kind, and the
# book says so rather than implying closure. ⭐ Calibrated at **0 of 200
# commits** introducing a new kind, with `.11.18.2`'s discriminator applied:
# this population was gated by NOTHING during the replay, so the zero is real
# rather than survivorship. Standing population discharged to 0 first.
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

# The declared licence and the granted texts may not drift apart
# (`SIGNOFF-REPAIR.13.2`). Blocker B5 was exactly this gap held open for the life
# of the project: all 13 tracked manifests declared `MIT OR Apache-2.0` and the
# repository contained no licence text at all, so a public repo offered readers a
# grant it had never made. ⭐ Measured before registering: this gate fires on 0
# breaches today and would have fired on every commit before the one that adds
# it — the REASON-CODE-DOC shape, catching the next drift rather than presenting
# a backlog. ⛔ It is two-directional on purpose: a licence FILE that no manifest
# declares is also a breach, because a file left behind after an expression
# changes still reads as an offer.
if ! scripts/check_licence_grant.sh >/dev/null 2>&1; then
    scripts/check_licence_grant.sh >&2
    exit 1
fi

# The secret scan runs at COMMIT time (`SIGNOFF-REPAIR.11.4.7.2.3`). It lived only
# in `.github/workflows/supply-chain.yml` — remote CI, which has never run — so a
# `generic-api-key` finding sat red and invisible from 2026-09-13. ⭐ The reason it
# belongs HERE and `cargo deny` does not is what each is TRIGGERED BY: a commit can
# introduce a secret, so the secret scan is change-triggered; an advisory appears
# against code nobody touched, so `cargo deny` is TIME-triggered and lives in
# `.githooks/pre-push`. ⚠️ Priced before wiring (`SIGNOFF-REPAIR.11.5`: a gate
# people route around is a gate that lies): 1.07-1.15 s over three runs, scanning
# 488 commits / 15.11 MB, against a 6.65 s enforcer — about +17%.
#
# ⛔ THE RESIDUAL, STATED RATHER THAN HIDDEN: this SKIPS LOUDLY when `gitleaks` is
# absent, because failing closed would block every contributor who has not installed
# it. A skip is a weaker guarantee than a pass and must never read like one — hence
# the notice on stderr. CI installs the pinned 8.30.1 and is the real backstop, which
# is exactly the backstop blocker C1 says has never run.
# ⛔ TWO ARMS, AND THE SECOND EXISTS BECAUSE THE FIRST ALONE SHIPPED A DEFECT.
# `gitleaks git` scans HISTORY. In a pre-commit hook that means it validates the
# PREVIOUS state and cannot see the commit being made — so REPAIR-0200 verified
# itself green against an uncommitted working tree, and the finding its own change
# introduced surfaced one commit later (`SIGNOFF-REPAIR.13.1.2`). `--staged` is the
# arm that reads what is actually being committed, and it costs 0.03 s.
# ⚠️ `detect --no-git` was measured and REJECTED: it walks `target/` and
# `.project-data/` and did not finish in 120 s.
if command -v gitleaks >/dev/null 2>&1; then
    if ! gitleaks git --staged --redact --no-banner . >/dev/null 2>&1; then
        echo "SECRET-SCAN: the STAGED changes contain a finding — re-run to see it:" >&2
        echo "  gitleaks git --staged --redact ." >&2
        echo "  Fix the value rather than allowlisting it: nothing is in history yet," >&2
        echo "  so this is the one moment a fingerprint entry is NOT required." >&2
        exit 1
    fi
    if ! gitleaks git --redact --no-banner . >/dev/null 2>&1; then
        echo "SECRET-SCAN: gitleaks reported a finding in HISTORY — re-run to see it:" >&2
        echo "  gitleaks git --redact ." >&2
        echo "  A verified test literal is cleared by its EXACT historical fingerprint" >&2
        echo "  in .gitleaksignore — never by suppressing the path or the rule." >&2
        exit 1
    fi
else
    echo "SECRET-SCAN: SKIPPED — gitleaks is not installed, so this commit is NOT" >&2
    echo "  secret-scanned. Install it (CI pins 8.30.1) or accept the CI backstop." >&2
fi

# Blocker B3's deferral trigger, made evaluable (`SIGNOFF-REPAIR.13.1.2`). §16.6's
# prompt-injection action-boundary suite is DEFERRED on a measured ground: model
# output cannot reach an action. ⛔ The problem was never the deferral — it was that
# the revisit trigger was PROSE. `SIGNOFF-REPAIR.11.4.7.2` measured what that costs:
# Phase 1 deferred a fuzz baseline until "the first untrusted parser"; that parser
# shipped, the phase closed, and `git ls-files | grep -ic fuzz` still returns 0. The
# trigger fired and nothing noticed, because nothing was built to notice.
if ! scripts/check_action_boundary.sh >/dev/null 2>&1; then
    scripts/check_action_boundary.sh >&2
    exit 1
fi

# A cleanup plan that deletes a parent must delete the children that reference it
# (`SIGNOFF-REPAIR.11.14.1.2`). The shared runtime checker already refuses such a
# plan — and only when the suite RUNS. `migrations/0062` added a table with a key
# to `tenants` and swept only the plans naming its other parent; six suites could
# not start for roughly twenty commits because nothing executed them. Then
# `migrations/0067` did the same thing, in the session that wrote the rule down.
# This is the commit-time trigger the runtime checker lacks, and it needs no
# database: every foreign key in this corpus is inline in a `CREATE TABLE`.
if ! python3 -B scripts/check_fixture_plan_children.py >/dev/null 2>&1; then
    python3 -B scripts/check_fixture_plan_children.py >&2
    exit 1
fi

# A suite's cleanup plan must also name every table its own HTTP calls WRITE
# (`SIGNOFF-REPAIR.7.1.2.2.2`). The check above answers a foreign-key question;
# this one answers a pollution question no foreign key can express, and the two
# are different classes. `tests/routing.rs` has posted to `/v1/evaluations/trials`
# since REPAIR-0274 and never purged `evaluation_trials`, so `routing evaluation`
# failed `evaluation` 2/1 while `evaluation routing` passed both — a row left
# behind never hurts the suite that left it, only whichever later suite asserts
# over the same table, which is why it reads as a flake in someone else's file.
if ! python3 -B scripts/census_fixture_write_reach.py >/dev/null 2>&1; then
    python3 -B scripts/census_fixture_write_reach.py >&2
    exit 1
fi

# A positional source reference must name a file a reader can find
# (`SIGNOFF-REPAIR.11.17`). `CLAIM_VERIFICATION.md` §4.1 grades a NAMED INSTANCE
# as exact with no tolerance band, and a BARE BASENAME is exact only when it
# names one tracked file: `profiles.rs:5696` names a 606-line source and an
# 11,154-line suite, and the prose does not say which. ⭐ `DOCPATH` already wants
# repo-root-relative references and does not reach this — a bare basename
# satisfies it while naming nothing, which is `BOOK-LINKS`' founding shape.
# Calibrated across 200 commits before it was proposed (`.11.6`): of 415
# positional references ADDED, 48 were ambiguous (11.6%) and the gate would have
# blocked 19 commits (9.5%) — far below the 87% and 93% that got `.11.9`'s and
# `.11.15`'s candidate gates rejected for teaching bypass. The standing
# population was discharged to 0 first, so it fires only on the next one.
if ! python3 -B scripts/census_positional_refs.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_positional_refs.py --check >&2
    exit 1
fi

# A blank line ENDS a GFM table (`.11.19`), so every row after it renders as
# literal pipe-text rather than a row — and the source looks perfectly fine,
# which is why this survived every review that read the file instead of the
# page. It was live in this tree's OWN Current Frontier table: 46 rows,
# including row 1, the cell INDEX-FRONTIER and BOOK-FRONTIER both exist to
# police. ⛔ `TABLE-ARITY-RATCHET` governs these same files and cannot see it:
# it compares a row's cell count against its header's, and an ORPHANED ROW HAS
# NO HEADER to disagree with — `BOOK-LINKS`' founding shape a third time.
# BOTH DIRECTIONS of that boundary are gated (`.11.19.2`): a blank line where
# none belongs SPLITS a table, and no blank line where one belongs means the
# table SWALLOWS the block after it — `PHASE-3`'s eleven-line closing paragraph
# was rendering as eleven padded table rows.
# Calibrated over the FULL history before the second arm was proposed (`.11.6`):
# 19 of 551 commits (3.4%) would have been blocked, and every one of the 19
# introduced a defect this work has since repaired — zero false positives across
# the project's whole life. Over the last 200 commits it is 1 (0.5%). The
# standing population was discharged to 0 first, so it ships green and fires
# only on the next one.
if ! python3 -B scripts/census_broken_tables.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_broken_tables.py --check >&2
    exit 1
fi

# Every policy line a resolver pack ADVERTISES carries an adjudicated verdict
# (`SIGNOFF-REPAIR.7.3.6.1`). A pack publishes six of them to every caller that
# reads the §12.2 registry, and a caller CHOOSING a pack reads all six.
# `SIGNOFF-REPAIR.7.3.5` found the R3 pack advertising `redirect_policy: "deny"`
# and `subresource_policy: "deny"` and enforcing NEITHER — a claim in a
# REGISTRY, which is worse than one in a module header, because a module header
# is read by maintainers and an advertisement is read by callers.
#
# ⭐ The gate is the JOIN, not the count: the census enumerates the lines from
# their two producers (the Rust literals in `resolvers.rs::gated_advertises` and
# the `INSERT INTO resolver_capabilities` rows in `migrations/`) and refuses any
# line `.doctrine/advertised_policy_verdicts.tsv` does not cover AT ITS CURRENT
# VALUE. A new pack arrives unadjudicated; a changed word invalidates the
# verdict earned for the old one.
#
# 🔴 THE VALUE IS IN THE KEY BECAUSE THE INSTRUMENT SHIPPED WITHOUT IT AND
# CARRIED THE EXACT DEFECT IT WAS BUILT TO FIND. Keyed by pack and field alone,
# flipping R3's `subresource_policy` from `deny` to `allow` in the producer left
# the census GREEN, still reporting `enforced — .7.3.5` for a line advertising
# the opposite of what that leaf repaired. Measured in situ, restored
# byte-identical, then replayed against the repair.
#
# ⚠️ Calibrated over the FULL history before registering, because the instance
# is older than any 300-commit window (`docs/knowledge/calibrate-over-the-
# history-that-contains-the-instance.md`): 4 of 594 commits (0.7%) would have
# been blocked, and all four are the commits that ADDED a pack — exactly the
# moment the adjudication is owed. No commit has ever MOVED a value, so the
# stale-verdict arm guards the next one rather than presenting a backlog. The
# standing population was discharged to 0 first, so it ships green.
if ! python3 -B scripts/census_advertised_policies.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_advertised_policies.py --check >&2
    exit 1
fi

exit 0
