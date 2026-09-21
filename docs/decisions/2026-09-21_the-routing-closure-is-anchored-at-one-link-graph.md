answers: why did a governing document have no registry row; what can the routing-pressure closure not see; how many tracked documents have no governing row; is a registry row a size bound; which second anchor should the routing closure get; how much would it cost to govern every tracked Markdown document; is DOCTRINE_ENFORCEMENT.md bounded by anything

# The routing closure is anchored at one link graph, and three rows were placed by hand

- **Type:** `decision`
- **Date:** `2026-09-21`
- **Owner:** leaf `SIGNOFF-REPAIR.11.4.2.7.3`, opened by `.11.4.2.7.2` on the
  second measured instance of the blind spot.
- **Status:** accepted as a measurement; the second anchor itself is owned by
  `SIGNOFF-REPAIR.11.4.2.7.3.1` and is not shipped here.

## The question

`scripts/check_readme_stability.sh` requires every routed destination to end at a
governed row in `.doctrine/readme_routes.txt`. It discovers destinations from
exactly three anchors: the landing page's link graph, the guard's own routing
hint, and paths named inside declared control sentences.

Twice, a document that *governs* this repository turned out to have no row, and
both times a person found it rather than the gate. `DEV_NOTES.md` had none for
the life of the project (`.11.4.2.5`) while `COMMIT.md` makes writing to it
mandatory every commit; `MEMORY_ARCHITECTURE.md` had none until `.11.4.2.7.2`,
while defining the layer-A contract that `MEMORY.md`, `CLAUDE.md` and `AGENTS.md`
are all governed against.

So: what else can the closure not see, and is a second anchor warranted?

## What was measured

Re-derive, never read from here:

```bash
python3 -B scripts/census_routing_closure.py
python3 -B scripts/census_routing_closure.py --as-of 386aa64^   # DEV_NOTES.md invisible
python3 -B scripts/census_routing_closure.py --as-of 8aadac3    # MEMORY_ARCHITECTURE.md invisible
python3 -B scripts/census_routing_closure.py --verify-closure
python3 -B scripts/census_routing_closure.py --probe-bounds
```

⭐ **The closure is RUN, not re-implemented.** The census lifts the guard's own
`extract_routes`, `routes_from_readme`, `routes_from_hint` and
`routes_from_controls` definitions out of its source and executes them under
`bash`, with one named substitution (`"$0"` becomes the guard's path, because the
hint leg reads the guard file and the driver is not that file). A Python
transcription would be a second copy of a gate's verdict.

| quantity at `d8df245` (the census reads the COMMITTED tree and prints its revision) | count |
| --- | --- |
| tracked Markdown documents | 432 |
| registry rows | 21 |
| closure tokens the three anchors produce | 21 |
| documents the closure would ever propose | 308 |
| documents with a row the closure NAMES | 340 |
| **rows the closure can never name** | **3** |
| **documents with no row at all** | **89** |
| **minimal covering terminals for those 89** | **14** |

## The findings

🔴 **Three rows exist that nothing would ever have asked for** — `DEV_NOTES.md`,
`MEMORY_ARCHITECTURE.md` and `knowledge-map/`. The first two are the known
instances. The third had not been noticed, and it is the same shape: the routing
hint names the generated index `KNOWLEDGE_MAP.md`, never the directory of sources
behind it.

🔴 **`DOCTRINE_ENFORCEMENT.md` — the document that defines how every doctrine in
this repository is enforced — has no row and is bounded by nothing.** Appending
200,000 bytes to it leaves the enforcer green. It is the third instance of the
class and the first still open.

⛔ **"No row" is NOT "ungoverned", and the probe keeps the two apart.**
`README.md` is the guard's own subject and refuses at its line and byte caps;
`MEMORY.md` refuses under `MEMORY-ARCH`. Both have no row and both are bounded.

⚠️ **A row is not a size bound either, and the first version of the probe assumed
it was.** `AGENTS.md` carries a row and is bounded by nothing; only a row
declaring `ceiling=<bytes>` refuses. The controls are now selected by what the
registry *declares* rather than by whether a row exists.

⭐ **The cost of the complete rule is 14 entries, not 432.** The registry governs
by prefix, so a wholly-undeclared directory needs one row: `docs/knowledge/`
carries 49 members, `docs/runbooks/` 13, `docs/evidence/` 11, `spec/` 6. The
owning leaf had declined that rule in writing as *"a different and much worse
rule"* on a population nobody had counted. Promoted to
`docs/knowledge/a-prefix-closed-rule-costs-terminals-not-members.md`.

## The candidate anchors, scored against both known instances

| anchor | new rows demanded | `DEV_NOTES.md` | `MEMORY_ARCHITECTURE.md` |
| --- | --- | --- | --- |
| A — tracked root-level Markdown | 3 | catches | catches |
| B — the documents `CLAUDE.md`'s bootstrap list names | 3 | **misses** | catches |
| C — every tracked Markdown document | 14 | catches | catches |

⛔ **Anchor B is refuted by measurement, not by preference.** `CLAUDE.md` never
names `DEV_NOTES.md` — `grep -c DEV_NOTES CLAUDE.md` is `0` — so the anchor the
opening leaf floated first would have left the first instance exactly as
invisible as it was. Both surviving anchors flag both instances when the census
is run `--as-of` the commits where each was genuinely ungoverned.

## The recommendation, which is not the decision

**Anchor C**, and the reason is a routing fact rather than the arithmetic:
`scripts/check_lesson_promotion.sh` *requires* a lesson to be promoted into
`docs/knowledge/` — pressure moved into a collection with no lifecycle, no owner
and no ceiling, by a gate, every time a lesson lands. The closure exists to
forbid exactly that, and anchor A would leave it standing along with `spec/`,
`docs/runbooks/`, `docs/evidence/` and five loose `docs/` files.

⛔ **The landing page is not the remedy and was not touched.** Director
instruction 2026-09-21: it is quasi-static, amended rarely and only for the
ramp-up sequence. A registry row requires no link; widening the closure's ANCHOR
is the repair, never widening the landing page.

⚠️ **Three of the fourteen terminals are collections nobody has decided should
carry a lifecycle.** Pricing the rule removes the cost objection; it does not
decide the rule, and `.11.6` keeps that decision with the leaf that ships it.

## How the derivation was proved against the gate

`--verify-closure` removes one row from the real registry, runs the real
enforcer, and restores byte-identically (SHA-256 compared):

- **Arm A** — drop `AGENTS.md`, which the closure names: `README-STABILITY`
  refuses with `unrouted destination: AGENTS.md`.
- **Arm B** — drop `DEV_NOTES.md`, which it cannot: `README-STABILITY` stays
  green. Other doctrines redden, because the row is load-bearing elsewhere, and
  they are reported by name rather than scored.

🔴 **Arm B failed on its first run and the control was wrong, not the finding.**
It asked *does any failing line name this path* and got a refusal from
`ROUTE-CONTROL`'s adjudication-coverage rule, whose table must match the registry
exactly. That is `census_live_documents.probe_verdict`'s lesson one layer up
(`.11.4.2.6.4`): a breach elsewhere makes every surface look refused, so a
control must match the **leg** it is about. The verdict now matches the guard's
own sentence, which nothing else in the enforcer emits.
