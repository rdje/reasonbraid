<!-- README-POLICY-LOCAL-ADOPTION:BEGIN -->
## Local adoption note — ReasonBraid

- Authority: ReasonBraid maintainers; accountable owner Richard DJE. Adopted
  and revised 2026-09-06 under decision
  `docs/decisions/2026-09-06_readme-policy-readoption.md` (leaf `PHASE-0-MAINT-1`).
- Authoritative copy: repository-root `README_POLICY.md`. Agent/harness
  bootstrap files may point here, but they are not the policy's authority.
- Independence: the originating template is not an upstream. There is no
  automatic synchronization; later changes require deliberate local review
  (§14 of the director's session directives).
- Reviewed landing page and derived ceilings: `README.md` measured 47 lines /
  1,772 bytes at the 2026-09-06 review; the inclusive enforcement ceilings are
  **60 lines / 2,400 bytes** (the reviewed survivor plus explicit bounded
  headroom for legitimate landing-page changes). The guard derives them from
  this note's decision record; env overrides (`README_LINE_CAP`,
  `README_BYTE_CAP`) exist only for falsification runs, never as a growth path.
- Routed destinations: the project-owned data-only registry
  `.doctrine/readme_routes.txt` maps every destination named by the README, this
  policy, and the guard's failure guidance to an owner, lifecycle class, and
  pressure control. `scripts/check_readme_stability.sh` fails when an emitted or
  linked destination has no governed route, and enforces the append-only-history
  rotation threshold for `CHANGELOG.md` (96,000 bytes over the measured
  48,495-byte baseline recorded as governed debt; rotation = git history).
- Landing-page identity: top-level `README.md` is the project landing page, so
  its purpose, minimal first-use path, architecture summary, and canonical
  navigation remain directly visible there. Containment routes changing detail
  and chronology; it does not off-load the landing function.
- Adoption checklist: all nine steps of the checklist below are satisfied and
  evidenced in the decision record.
<!-- README-POLICY-LOCAL-ADOPTION:END -->

---

# README Stability Policy

This project- and harness-neutral policy keeps a repository README useful as a
stable landing page instead of letting it grow into a changelog, roadmap, or
documentation catalog.

## Authority and provenance

After adoption, the project owns this policy. Cite its authority by project
owner and adoption or revision date, together with the project-owned
`<repository-root>/README_POLICY.md`. Never cite a vendor-, agent-, or
harness-specific bootstrap file as the authority. Bootstrap files may help
authors and tools discover the policy; they do not make the policy binding.

## Storage location

Store the adopting project's canonical copy as the git-tracked
`<repository-root>/README_POLICY.md`, alongside `README.md`. Keeping the policy
with the file it governs gives contributors, local hooks, and CI one
discoverable, versioned source of truth. A user-home, machine-global, or other
external copy may serve as a reusable template, but it must not replace the
project-owned repository copy. Once copied, the project-owned file is
authoritative and the origin is not an upstream. Do not automatically re-sync
from the origin; adopt later revisions only through deliberate local review.

Keep project-specific adoption metadata—owner, date, decisions, derived caps,
and local enforcement links—in a clearly fenced adoption note above the
neutral policy body. This keeps the reusable body free of project-specific and
harness-vendor-specific tokens without hiding local authority.

## Content contract

Keep only information a first-time visitor needs:

- purpose, audience, and top-level scope;
- prerequisites and one minimal verified quick start;
- stable architecture at a glance;
- links to canonical documentation, support, and contribution guidance;
- license and other essential repository-level notices.

Route changing detail elsewhere:

| Content | Canonical home |
| --- | --- |
| User-facing feature detail and examples | User guide or product manual |
| Current work, priorities, and roadmap status | Roadmap, issue tracker, or task system |
| Release history | Releases, changelog, or git history |
| Design rationale | Decision records or architecture docs |
| Exhaustive file/API/sample inventories | Generated indexes or dedicated references |
| Diagnostics and operational procedures | Troubleshooting or contributor docs |

Change the README only when its purpose, first-use path, top-level architecture,
or canonical navigation changes. Ordinary feature work should update the
canonical destination, not the README.

Before deleting or relocating apparent duplication, prove that it is genuinely
duplicated with a phrase, identity, or content probe against the intended
canonical home. If that home is already richer and maintained, delete the
README copy and retain one link. Relocate only information that is unique and
still belongs in maintained documentation.

## Routing pressure closure

Moving content out of the README is not sufficient if the destination can
become an unbounded neighboring sink. Inventory every destination named by the
README, this policy, or the guard's failure guidance. Give each route an owner,
lifecycle class, and pressure control, and follow routes transitively until
they end at a controlled terminal. An unclassified destination, routing cycle,
or chain that merely moves the same append pressure again is a failed adoption.

Classify routes as `reader_navigation` or `author_overflow`. The sets may
legitimately differ: readers may inspect immutable change history, while
authors must not be told to append new status prose there. Derive overflow
candidates from path-shaped destinations in the guard's actual emitted
guidance, not only from a hand-maintained table, and fail when an emitted hint
has no governed destination.

Use controls appropriate to the destination:

| Destination class | Required pressure control |
| --- | --- |
| Hot/live file | Derived line and byte ceilings plus overwrite, review, or staleness semantics |
| Partitioned manual or task collection | Bounded index plus per-part, file-count, and aggregate ceilings |
| Generated index | Size ceilings plus a reproducible freshness check against canonical sources |
| Append-only history | Query-first access plus a shard, rotation, or archival threshold; never a mandatory bootstrap read |
| External service | Named authority, retention/lifecycle owner, and a stable query/link contract |
| Frozen legacy record | Content identity or another write prohibition; never an overflow destination |

A legacy destination that is already too large is not exempt. Record its
current measured ceiling as debt, stop further growth there, and open a
separately owned partition/compaction task. Do not describe a measured legacy
ceiling as an ideal reusable default. Raising any destination threshold needs
the same explicit review as raising the README cap.

In one measured adoption, README status/history guidance routed overflow into
an otherwise unchecked neighboring status file. That file reached 1,547,057
bytes, and 94.7% of it was dated changelog content. The README cap had displaced
the pressure rather than removing it. A destination registry and unconditional
closure check make that failure visible before it becomes another megabyte-
scale bootstrap surface.

## Mechanical growth guard

Enforce both a line cap and a byte cap. Derive both from the landing page that
survives a deliberate review and trim, leaving only modest explicit headroom.
Do not copy example values from this policy. Never raise a cap merely to land
new content; move the detail to its canonical home. A cap increase requires an
explicit reviewed decision that the landing-page contract itself expanded.

A minimal deterministic check is:

```sh
line_cap=__DERIVED_LINE_CAP__
byte_cap=__DERIVED_BYTE_CAP__
lines=$(wc -l < README.md | tr -d ' ')
bytes=$(wc -c < README.md | tr -d ' ')
test "$lines" -le "$line_cap"
test "$bytes" -le "$byte_cap"
```

Replace both placeholders with the adopting project's reviewed values before
enabling the check. Keep it non-mutating, return nonzero with a routing hint on
failure, and run it unconditionally on every commit and CI build. Landing-page
size is a property of the resulting tree, so the guard must not short-circuit
merely because `README.md` is absent from a staged or changed-path set; this
also catches over-budget merge and revert results.

The same unconditional check must validate the routed-destination inventory
and each declared pressure control. A commit that does not touch the README can
still overgrow, unfreeze, remove, or silently retarget one of its destinations.

Line and byte checks are independent. In one real adoption, the retained README
was 141 lines yet already 10,297 bytes; a numbered prose list measured roughly
118 bytes per line while a path list measured roughly 57. A line budget alone
therefore cannot constrain prose density, and a byte budget alone cannot
constrain vertical sprawl.

## Adoption checklist

1. Add and commit `<repository-root>/README_POLICY.md` beside `README.md`.
2. Fence local owner/date, authoritative-copy, independence, decision, and cap
   metadata above the neutral policy body.
3. Prove apparent status, history, inventory, and deep-reference duplication
   against its canonical home; delete-with-link when that home is richer, and
   relocate only genuinely unique maintained content.
4. Verify the retained quick start and links.
5. Record where each excluded content class belongs, then inventory every
   actual route through a controlled terminal; reject cycles and unclassified
   neighboring sinks.
6. Give hot/live files line and byte caps; give partitioned, generated,
   historical, external, and frozen terminals the class-specific controls
   above. Treat measured legacy ceilings as debt, not examples.
7. Derive reviewed line and byte caps from the trimmed survivor with modest
   explicit headroom; do not copy illustrative values.
8. Commit the deterministic README and routing-closure check and wire it
   unconditionally into every local commit and CI build, independent of
   changed-path scope.
9. Require an explicit decision before the README cap or any routed-destination
   threshold can increase.
