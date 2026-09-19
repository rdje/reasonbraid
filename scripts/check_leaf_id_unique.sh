#!/usr/bin/env bash
# LEAF-ID-UNIQUE — no two task-tree leaves share one id.
#
# A leaf id IS its address. The Current Frontier, the task-tree index, every
# `Owned by` routing and every cross-reference in this repository resolve work
# by it, so two leaves sharing one means a reader — or a script — reaches
# whichever comes first.
#
# ⛔ NOTHING ELSE DETECTS THIS, and the one near-miss this project has had shows
# why that matters. `FRONTIER-STATUS` refused it, but by accident: its resolver
# maps an id to the FIRST heading carrying it, so a new `pending` row was
# compared against the OLDER leaf's `done` and the message said a row disagreed
# with its leaf. Had the colliding leaf been `pending`, the gate would have
# passed and two leaves would have shared one address in a tree that addresses
# work by id. (`SIGNOFF-REPAIR.11.24`.)
#
# ⭐ PRICED, not asserted: 0.36 s against a 20.5 s enforcer (1.8%). Replayed over
# the last 200 commits touching `docs/tasks`, it would have blocked **0** — the
# defect has never reached `main`. It is registered anyway because the exposure
# is per COMMIT here: this project adds leaves in almost every commit, which is
# not true of the class `census_shared_registry_writes.py --check` guards (a
# route, added rarely) and priced its way OUT of the enforcer for that reason.
#
# The predicate lives in the census instrument so that the gate and the census
# cannot disagree about what a leaf is:
#     python3 -B scripts/census_goal_receipt_gap.py --gate
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 -B "$ROOT/scripts/census_goal_receipt_gap.py" --gate
