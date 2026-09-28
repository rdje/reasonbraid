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

# The shared-registry censuses hold their baselines as SETS
# (`SIGNOFF-REPAIR.7.1.6`): who writes a table with no tenant dimension, and by
# what admission; who reads one, and how the tenant is recovered. Both had a
# `--check` and nothing ran it, so the read census drifted from 29 tables to 31
# unguarded, readers without a tenant predicate appeared unjudged, and two book
# figures went stale; the write census held only because leaves ran it by hand.
# A drift is a design question, so the refusal names the adjudicating record
# rather than offering a refresh.
if ! python3 -B scripts/census_shared_registry_writes.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_shared_registry_writes.py --check >&2
    exit 1
fi
if ! python3 -B scripts/census_registry_read_reach.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_registry_read_reach.py --check >&2
    exit 1
fi

# Every Rust instant bound into a TIMESTAMPTZ column is judged
# (`SIGNOFF-REPAIR.11.31.2`): PostgreSQL keeps microseconds, so a Rust copy that
# escapes and is compared with the stored value disagrees by up to 999 ns on a
# nanosecond clock (`.11.30`: the budget ledger re-lent capacity that way). This
# host's clock is microsecond-granular, so no local run can reproduce the class;
# the census over the source is the only instrument that sees it from here.
# `.11.31.1` judged 43 sites and nothing re-ran the check, so six more arrived
# unjudged, two of them in the budget ledger.
if ! python3 -B scripts/census_bound_instants.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_bound_instants.py --check >&2
    exit 1
fi

# The corrective exit bar is a COUNT of open blocking leaves
# (`docs/decisions/2026-09-25_the-corrective-tree-ends-at-a-bug-bar.md`), and
# nothing derived it (`SIGNOFF-REPAIR.12.2`): a leaf records its state as
# `- Status:`, `- Opened:` or "Opened and closed", and a count reading only the
# first said 50 where 65 were open. It also produced this repository's claim that
# no class 1-3 leaf was open, while seven class-3 leaves were. Every open leaf
# must carry its `- ⚖️ Bar` line, and every leaf a readable state.
if ! python3 -B scripts/census_open_leaves.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_open_leaves.py --check >&2
    exit 1
fi

# The frozen plan states targets and names no task-tree leaf
# (`SIGNOFF-REPAIR.11.43`, the director's lockstep rule of 2026-09-28). A
# restated status goes false as soon as a leaf moves: `ROADMAP.md` carried a
# thirteen-reference narrative that was weeks stale when it was found.
if ! python3 -B scripts/check_plan_states_targets.py >/dev/null 2>&1; then
    python3 -B scripts/check_plan_states_targets.py >&2
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

# RELATIVE-LEAF-REF (`SIGNOFF-REPAIR.11.24.1.6`) — a relative leaf reference
# like `.2.3` resolves against its OWN tree, and the trees share a numbering
# shape, so one meant for another tree can land on a real leaf with a real
# status. The instance that opened the leaf: `PHASE-8.4.4` writes *the `.2.3`
# distribution-channel deferral* and means `PHASE-7.2.3`, while `PHASE-8.2.3`
# exists and is the A2A facade.
#
# ⛔ THE RATCHET IS NOT ON THAT CLASS, and the reason is that nothing mechanical
# could be. A reference that resolves in its own tree AND names a leaf in
# another is `shared`, and there are 2178 of those — only the surrounding words
# separate a correct one from an incorrect one. What IS mechanical is the two
# classes a reader cannot follow at all: `dangling` (resolves nowhere) and
# `internally-ambiguous` (resolves BOTH ways inside one tree, because two
# dialects coexist). Neither may RISE against HEAD.
#
# ⚠️ Priced over the 30 commits touching docs/tasks/ before registering:
# `dangling` rose in 1, `internally-ambiguous` in 0, and `foreign` in 3.
# `foreign` is DECLINED at ten times the cost and is the SAFE class besides — it
# cannot silently resolve to the wrong leaf, because it does not resolve in its
# own tree at all. The standing 76 + 126 are NOT a backlog to discharge: the
# dominant `dangling` cause is a lane referenced but never declared as a node,
# which is a different defect and is owned at `SIGNOFF-REPAIR.11.24.1.6.1`.
if ! python3 -B scripts/census_relative_leaf_refs.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_relative_leaf_refs.py --check >&2
    exit 1
fi

# RELEASE-BINARY (`SIGNOFF-REPAIR.6.8.1`) — every binary the workspace builds is
# adjudicated into or out of the release manifest, with a reason.
#
# 🔴 THE DEFECT IT CLOSES: `make release` named FOUR binaries with `--bin` while
# the line immediately above it, `cargo build --release --bins`, built TEN.
# ADR-027 makes the manifest "the single verification unit" — "binaries verify
# THROUGH it, never individually" — so the six it omitted could not be verified
# at all. ⛔ Two of them are the acquisition workers `rb-server` SPAWNS, and
# `extraction::worker_path` resolves those from `current_exe().parent()`: the
# release directory itself. An executable sitting beside the server that no
# manifest names is a code-execution path the ladder cannot reach.
#
# ⭐ The flags are now DERIVED — `make release` asks this same script for them —
# so the Makefile and the ledger cannot disagree about which binaries are signed.
# What the gate checks is the wiring and the coverage: a workspace binary the
# ledger does not cover, a ledger row naming no binary, a disposition outside the
# vocabulary, a row with no reason, and a release target that has gone back to a
# hardcoded list.
#
# ⚠️ It costs well under a second: the binary targets are read from the crate
# manifests and the filesystem, never from `cargo metadata`, which would resolve
# the whole dependency graph for a question that does not need it. Both routes
# were run against each other and returned the same ten names before this was
# registered.
if ! python3 -B scripts/census_release_binaries.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_release_binaries.py --check >&2
    exit 1
fi

# EXTERNAL-LEDGER (`SIGNOFF-REPAIR.6.8.2`) — every entry in the ROADMAP §7.4
# dependency ledger carries the fields §7.4 names.
#
# 🔴 THE DEFECT IT CLOSES: `grep -rln "external-ledger" scripts/ .github/ Makefile
# deploy/` returned NOTHING. §7.4 says the ledger records fourteen named fields,
# that "CI warns on expired checks", and that "release gates require fresh
# records for exposed compatibility profiles" — and the file's own header
# described both mechanisms as though they ran. Nothing read it.
#
# ⛔ THIS GATES SHAPE AND NEVER AGE. §7.4 says CI WARNS on an expired check and
# names no horizon, so a staleness threshold here would be a number nobody
# derived, enforced forever (`SIGNOFF-REPAIR.11.6`). The warning is a
# non-blocking CI step; the shape is a property of the file and is gated.
#
# ⚠️ An EMPTY list is a legitimate "not yet pinned" — the ledger's own header
# defines it that way — so the gate refuses an ABSENT field, never an empty one.
# Demanding a value would demand invention, which is the opposite of the rule.
if ! python3 -B scripts/census_external_ledger.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_external_ledger.py --check >&2
    exit 1
fi

# ROUTE-CONTROL (`SIGNOFF-REPAIR.11.4.2.6.7`) — every pressure control the
# routed-destination registry DECLARES is evaluated, and a false one refuses.
#
# 🔴 THE DEFECT IT CLOSES: `.doctrine/readme_routes.txt` gave `LIVE_STATUS.md`
# the control *overwritten rather than appended* while its own 632 versions
# showed 614 growing against 17 shrinking with the tip at its all-time high, and
# `check_readme_stability.sh` passed throughout — it validates that a row has
# four non-empty fields, which is ARITY, not truth.
#
# ⛔ THE OPERAND IS DECLARED, NEVER EXTRACTED FROM THE PROSE. A census of all
# twenty controls measured the obvious extractor at **5 wrong operands in 18**
# — two task-tree names read as doctrine ids, three HISTORICAL byte figures read
# as ceilings — with **6 of the 19 real claims invisible to it**
# (`SIGNOFF-REPAIR.11.4.2.6.7.1`). A paraphrase matcher is `.11.6`'s measured
# failure mode, so a row carries machine-readable terms BESIDE its human
# sentence and the sentence is never deleted to fit them.
#
# ⭐ IT IS DOUBLE-ENTRY, WHICH IS WHAT STOPS IT BEING THEATRE. The check refuses a
# term the prose reading found and the row does not declare (the field cannot be
# emptied to go green) AND a term the row declares that no reading of its prose
# supports (a term cannot be invented). The hand reading and the declaration are
# produced by different acts; their agreement is the assertion.
#
# ⚠️ Falsified against the REAL historical defect with no edit at all:
# `--check --as-of 9221467` puts the row's growth claim back against the tree
# that refuted it and returns rc=1 naming `LIVE_STATUS.md` — *623 grew, 17
# shrank, tip 620448 against a peak of 620448 -> at_all_time_high* — while the
# two sibling ledgers' identical claims still hold, so the control comes apart
# from its subject rather than failing wholesale at an old commit. Each of the
# seven evaluators is separately two-sided.
#
# ⚠️ Priced before registering (`SIGNOFF-REPAIR.11.5`): 0.18 s over three runs
# against a 28.74 s enforcer, about 0.6%. The growth leg reads 1,756 ledger
# versions and is affordable only because `.11.4.2.6.7.2` made that one
# `git cat-file` call instead of 1,756.
if ! python3 -B scripts/census_route_controls.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_route_controls.py --check >&2
    exit 1
fi

# Every shipped surface carries a per-member JUDGEMENT, and the judgement is
# still about the facts it was made from (`SIGNOFF-REPAIR.11.4.6.2`).
#
# ⛔ The population is derived from the producers — every `.route(` outside a
# `#[cfg(test)]` block, and `cargo metadata --no-deps` for the binaries, because
# only 4 of the 10 carry a `[[bin]]` stanza. A member nothing judged FAILS.
#
# ⭐ WHY A TABLE RATHER THAN PROSE: `docs/knowledge/a-sample-is-not-a-traversal.md`
# — a statement about every member does not fit in a sentence; carry it keyed to
# each member and refuse when a member is unjudged, when a judgement outlives its
# member, or when a member's INPUTS moved since it was judged. The third is the
# one usually missing and the one that makes a stale judgement look current.
#
# ⭐ AND THE COARSE SIGNAL IS WRONG IN BOTH DIRECTIONS, which is the argument for
# adjudicating rather than counting: asking *does any chapter mention this family*
# calls 21 of 30 covered where only 14 have every route documented, and it reads
# `/` as a bare mention while `web-ui.md` is a whole chapter about it.
#
# ⛔ A `gap` row must name an owning leaf (`CLAUDE.md` §15: a finding nobody owns
# is a complaint), so the table cannot decay into an unheld backlog.
#
# ⚠️ Priced before registering (`SIGNOFF-REPAIR.11.5`): 0.27-0.35 s over three
# runs against a ~30 s enforcer, about 1%. The `cargo metadata` call is 0.024 s.
# ⚠️ First published as 0.16 s and re-measured by `.11.26.2`; that reading was
# taken on a quieter machine, and the re-measurement ran with another project's
# release build resident. The decision is unchanged at either figure, which is
# why the range is published rather than the friendlier number.
if ! python3 -B scripts/census_surface_judgement.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_surface_judgement.py --check >&2
    exit 1
fi

# OPERATOR-SURFACES (`SIGNOFF-REPAIR.4.6.1.6`): ROADMAP §18.5's nine operator
# surfaces stay mapped to routes the server still registers. `.4.6` graded the
# nine by hand and `.4.6.1` built the missing five; a route renamed or removed
# since would leave that grade standing while the surface is gone. The mapping is
# a judgement written once in the census; whether each route EXISTS is derived
# from the routers every run, through the same parser `SURFACE-JUDGEMENT` uses.
# ⛔ A blocked part (bullet 8's checkpoint age, ADR-022) is reported `partial`,
# never counted exposed.
# ⚠️ Priced before registering: 0.06 s over three runs.
if ! python3 -B scripts/census_operator_surfaces.py --check >/dev/null 2>&1; then
    python3 -B scripts/census_operator_surfaces.py --check >&2
    exit 1
fi

# QUALIFICATION-CURRENCY (`SIGNOFF-REPAIR.11.37`): a limitation row on the
# qualification review may not stay open once every leaf it names is done. One
# row stated a defect repaired at REPAIR-0184 until REPAIR-0528, and the census
# that followed found 24 such rows; the page is where a reader looks for what is
# still wrong, so a stale row there is a false claim in the pessimistic
# direction, which is why nothing noticed. It judges ownership, not wording: a
# closed row's text and an open row owned by a live leaf still need reading.
# ⚠️ Priced before registering: 0.03 s over three runs.
if ! python3 -B scripts/check_qualification_currency.py --check >/dev/null 2>&1; then
    python3 -B scripts/check_qualification_currency.py --check >&2
    exit 1
fi

exit 0
