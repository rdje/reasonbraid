# LAN completeness precedes Internet exposure

- Date: 2026-09-18
- Status: accepted
- Owner: director instruction, 2026-09-18
- Related: `SIGNOFF-REPAIR.13` (the blocker register), `SIGNOFF-REPAIR.14` (the
  frozen exposure candidate), `docs/book/src/blockers.md`,
  `docs/decisions/2026-09-16_internet-qualification-route.md` (the route this
  re-ranks but does not cancel), ROADMAP §16.12, Phase 7.

## The instruction, verbatim in substance

> The internet exposure is not high priority right now. It needs to fully work
> on the local network first.

## What it changes

**Priority, not ownership.** B1 (externally reviewed threat model), B2
(penetration test) and B3 (prompt-injection action-boundary suite) keep
`Owed here? = yes` — in-repo work genuinely remains for each. What changes is
that they no longer ride the frontier as current work.

`SIGNOFF-REPAIR.13`'s re-surfacing rule said a `yes` row "belongs in the
frontier and is worked like any other leaf". That rule was written when nothing
ranked the rows against each other. It now admits a third state, and the
register records it explicitly rather than leaving the rows to look abandoned:

| Column value | Meaning | How it is surfaced |
| --- | --- | --- |
| `yes` | in-repo work is owed AND is current | rides the frontier |
| `yes (deferred)` | in-repo work is owed and is NOT current, by a recorded ruling | surfaced once per session with its ruling and resumption trigger |
| `no` | parked on someone outside this repository | surfaced once per session with the external party and trigger |

⛔ **`yes (deferred)` is not `no`.** The distinction `.13` was built around is that
an owned-but-unsurfaced blocker is harder to notice than an unowned one. A
deferred row is still owed here, still has a leaf, and is still surfaced — it is
simply not what the next commit should be about.

## What it does NOT change

- `SIGNOFF-REPAIR.14`'s standing prohibition is untouched: nothing in that lane
  may deploy the profile or claim G6/G7. Deferring the work does not licence a
  claim about it.
- Phase 7's G6/G7 remains **NOT MET** and must keep being reported that way.
  This ruling changes what is worked on, never what may be claimed.
- B4 (name clearance) and C1/C2 are unaffected.
- `2026-09-16_internet-qualification-route.md` stands as the route; it is
  re-ranked, not superseded.

## The resumption trigger, defined

Clarified by the director on 2026-09-18, in two steps:

> *"ReasonBraid behavior shall match the objective as defined in the roadmap in
> the LAN not over the internet yet."*
>
> *"G7 should work inside the LAN before trying to make it work over the
> internet."*

⛔ **THE FIRST CANDIDATE THIS RECORD CARRIED IS WITHDRAWN, and it was wrong in
KIND rather than in detail.** It read "every corrective repair under Phases 1–6
closed, plus `.5.3`/`.5.4`" — phase arithmetic, which can diverge from behaviour
in both directions: every repair can close with an objective clause still unmet,
and an objective can be met while unrelated repairs stay open. The definition is
BEHAVIOURAL, and the roadmap already encodes it.

**The roadmap states it in its own header** — *Initial deployment: multiple
trusted hosts on a private LAN or private overlay* against *Target deployment:
authenticated agents and humans on arbitrary Internet-connected hosts* — and its
gate table (`ROADMAP.md:2076–2085`) carries an *unlocks* column that partitions
on exactly this line:

| Gate | What it unlocks | In the LAN bar? |
| --- | --- | --- |
| G0 Contract, G1 Component | boundary implementation, merge artifact | yes |
| G2 Vertical slice | LAN preview | yes |
| G3 Governance | binding policy use | yes |
| G4 Resource safety | arbitrary-reference feature | yes |
| G5 Quality | the "deliberation improves answers" claim | yes |
| G6 Internet security | **Internet exposure** | **NO — this is the deferral** |
| G7 Operations | production beta | **yes, on the LAN** (director, 2026-09-18) |
| G8 Compatibility, G9 Release | stable protocol, declared maturity | beyond the bar |

**So: the LAN bar is G0–G5 genuinely met, plus G7 earned on the LAN. G6 is out of
scope.** G6 is the only gate whose unlock is a transport, which is why the line
falls there and nowhere else.

### Why G7 belongs inside, and the one part of it that does not

G7's evidence is mostly **transport-independent**, so earning it on the LAN is
not merely the safer order — it is the order that does not waste the work.
Whether a restore restores, whether the system survives losing a node, whether it
is instrumented at all: none of that changes when the transport does. The cost
asymmetry points the same way — chaos and game-day work is cheap and repeatable
on a private network and expensive after exposure.

⛔ **But G7 splits, and the split must be recorded now or it becomes a false claim
later.** Load thresholds and SLO targets measured on a LAN say nothing about
Internet latency, loss or adversarial traffic:

- **Structural legs — earned once, on the LAN:** restore correctness, survival of
  induced failure, instrumentation coverage.
- **Numeric legs — re-derived under the Internet posture:** load thresholds and
  SLO targets. A LAN figure is anchored to a LAN, and `docs/CLAIM_VERIFICATION.md`
  forbids restating it as though it were not.

### What G7 costs, measured rather than estimated

Stated so the bar is entered with the number in front of the director. G7 is the
least-built of the LAN gates:

| G7 leg | State on 2026-09-18 |
| --- | --- |
| Backup / restore | **real** — `scripts/backup.sh`, `scripts/restore.sh`, and the `backup_restore` suite |
| SLO instrumentation | **partial** — `GET /v1/admin/metrics` exists (ADR-023); objectives and thresholds do not |
| Load | **dev scale only** — one storm control asserting fan-out caps and expiry refusal, explicitly "at the dev scale" |
| Chaos / game day | **absent** — `git grep -lni "chaos\|game.day" -- crates scripts docs/book/src` returns **0** |

### The practical rule, which needs no further definition

**Work the LAN bar — G0–G5 and G7-on-LAN — and do not spend a commit on
Internet-exposure work.** ⚠️ Note that G5 is currently *withdrawn* rather than
merely unverified (the subtraction gate retracted the quality-lift claim), and
G1–G5's shipped-line claims are mid-re-derivation under `.11.4.7.x`. Re-earning
them IS the LAN-completeness work, not a detour from it.
