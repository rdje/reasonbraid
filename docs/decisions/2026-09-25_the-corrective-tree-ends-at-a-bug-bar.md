---
answers:
  - When does the corrective tree (SIGNOFF-REPAIR) end and the roadmap resume?
  - Which open corrective leaves must be fixed before the roadmap resumes, and which may wait?
  - How is a finding classified when it is discovered?
---
# The corrective tree ends at a bug bar, not at exhaustion

- **Type:** decision
- **Status:** active. Replaces `SIGNOFF-REPAIR.12`'s former exit wording (*close or refute every census finding*).
- **Owner:** `SIGNOFF-REPAIR.12` (the exit), decided under `SIGNOFF-REPAIR.12.1`
- **Date:** 2026-09-25
- **Work unit:** `REASONBRAID-DOC-0162`
- **Source:** the director delegated the triage and the exit bar (*"all are yours to decide, but better be sota and signoff"*, 2026-09-25) after asking whether the work was progressing or stuck.

## The fact / decision

The measured problem: between `fcb2a7c` and `ac90ee3` (2026-09-24) seventeen leaves were closed and seventeen opened, so the open count stayed at 65 (re-derived by a rule that reads both of the tree's state forms; the first count, which read only one, said 50). Every review of an area finds real defects, so *"close or refute every census finding"* has no fixed point: the bar moves with each census. A corrective programme needs a bar a finding is measured against, and the practice that has one is the **bug bar** (Microsoft SDL's security bug bar; the release-blocking severity classes most engineering organisations run): a finding's class is set by its impact on what the project CLAIMS, and only the blocking classes hold a release.

**The claimed profile** is what impact is measured against: a dev / trusted-LAN deployment. Internet exposure is not claimed (G6/G7 NOT MET, `LIVE_STATUS.md`); the R3/R5/RX acquisition packs are off unless `RB_ENABLE_R5R3RX=1`; `rb-node` constructs only the fake adapter; the Claude and Codex CLI adapters are a documented library surface.

**A finding BLOCKS the return to the roadmap** when, in the claimed profile, it is one of:

1. **Cross-tenant:** a principal can read, change, spend against, or deny service to another tenant's data, authority, budget or shared state.
2. **Integrity:** silent loss or corruption of durable state, money (the budget ledger), evidence, or publication, which are the guarantees the README's headline names (*"Rust enforces identity, authorization, ordering, budgets, and publication"*).
3. **A false claim:** the book, README or a qualification statement says something the code does not do. Narrowing the claim is a valid resolution when fixing the code is not warranted.
4. **A gate that lies:** a check the push or commit relies on fails spuriously, passes falsely, or cannot run.

**Otherwise it is DEFERRED WITH A TRIGGER**: owned by a leaf that names a readable trigger (a command or an observable condition, the form `DOC-0137`–`DOC-0141` and `DOC-0157` already use) and an honest limit visible in the book's qualification chapter. Typical cases: hardening that matters only once a profile the project does not claim is enabled (Internet exposure, a gated pack switched on), features the roadmap has not built, assessments.

**A leaf is CLOSED without code** when a closing census shows every goal clause held by committed work and a named control (the `.3.5`/`.4.1` form), or when it duplicates another leaf.

**The roadmap resumes** (`SIGNOFF-REPAIR.12`) when all of these hold:

1. every leaf in a blocking class is closed with reproducible evidence, fixed or refuted;
2. every deferred leaf names its trigger, and the qualification chapter states its limit;
3. the full CI checkpoint passes on a pushed commit (`SIGNOFF-REPAIR.11.4.3.1`);
4. the qualification chapter reconciles with measured behaviour (`SIGNOFF-REPAIR.11.4.3`).

**After the return**, every new finding is classified when it is found, by this bar. A blocking one preempts roadmap work; a deferred one gets its trigger and joins the backlog. The bar is also what a census uses to decide what it opens.

## Why

- **Convergence.** A bar that moves with every census cannot be met; a bar fixed to impact on the claimed profile can, and still refuses to ship any cross-tenant, integrity, honesty or gate defect.
- **It is the standard shape.** Severity-gated release criteria with explicit deferral are how security and reliability programmes ship: the SDL bug bar holds a release on its critical and important classes and tracks the rest; a deferral without an owner and trigger is what those programmes forbid, and this record forbids it too.
- **Nothing is dropped.** Deferred leaves stay in the tree with an owner, a readable trigger and a public limit, so a later profile change (enabling a gated pack, claiming Internet exposure) reopens them mechanically instead of by memory.

## The triage of the 65 open leaves (2026-09-25, at `ac90ee3` + REPAIR-0494)

⚠️ **The count is 65, not the 50 first reported.** The first count read only `- Status:` lines, and 16 open leaves record their state only as `- Opened: \`pending\``; ten more leaves say *opened and closed* on one line. Nothing counts open leaves by a rule that reads both forms, so `.12.2` owns that instrument, because the exit bar is a count.

Each census-first item is classified by its goal line against the bar, then re-classified clause by clause at its census; a census that finds a blocking clause opens a blocking leaf.

**Closed by a closing census (8: every child done, parent never closed):** `.3.3.4.3.3.3.3.2.3`, `.7.2`, `.7.3.2`, `.7.4`, `.8.2`, `.9.2`, `.11.4.3.1.7`, `.11.4.5`. A goal clause no child held becomes its own leaf, classified by the bar.

**Closed as a duplicate (1):** `.11.2.7`, which `.11.26` records as its twin (same command, modules, error and hypothesis).

**Blocking (25 of the 65, plus `.12.2`, which this triage opens; must close before the roadmap resumes):**

| Leaf | Class | Why it blocks |
| --- | --- | --- |
| `.7.1.3.1` who may add a site-global resolver | 1 | any tenant administrator writes a row every tenant's resolution ranks |
| `.7.1.4` the first tenant to cite a URL fixes its scheme and hints | 1 | one tenant decides how another's citation resolves |
| `.7.1.5` ranking ties fall to row order | 3 | the goal line and the book promise deterministic ranking |
| `.9.1` policy registration and authority | 1, 2 | authorization of a shared control surface; census first |
| `.9.3.2` a publication can be reviewed once, for ever | 2 | the review lifecycle §15 requires cannot recur |
| `.9.3.3` a deployment's declared digest is bound to nothing | 2 | a deployment can claim content it does not deploy |
| `.8.1.1.4` two decision rules have no countable bar | 3 | a thread may declare a rule the close cannot apply; narrowing (refusing the rule) is a valid resolution |
| `.8.1.1.6` §13.3 rule obligations not implemented | 3 | as above, per obligation |
| `.11.1` console rendering | 1 | a value not rendered as inert text is script another tenant's data can run in an administrator's browser; census first |
| `.11.3` operational scripts | 2 | restore targets and secrets; census first |
| `.10.1` subprocess adapter supervision | 1, 2 | the Claude and Codex adapters are a documented library surface, with prompt-option injection and unbounded output; census first |
| `.6.3` A2A (its claims) | 3 | qualification must say *serialization only* until a peer exists; the build half is deferred at its census |
| `.11.4.7.2.1.1` the fuzz baseline | 4 | a deferral whose trigger FIRED at Phase 4 and was never acted on; the bar forbids a fired trigger left waiting |
| `.3.3.4.3.3.3.3.2.4`, `.3.3.4.3.3.3.3.3` the bootstrap client recovery | 2, 3 | finishes a half-landed protocol (obsolete recovery paths and incomplete book examples remain) |
| `.11.4.3.1.2.17`, `.19`, `.20`, `.22`, `.23` | 4 | the full CI checkpoint and the first remote CI runs fail |
| `.11.9.1.3.4`, `.11.9.1.3.5`, `.11.9.1.4`, `.11.9.1.5`, `.11.9.1.6` the remaining review-record tranches | 3 | the startup review's records not yet routed clause by clause: an unrouted finding is unclassified, so the exit bar cannot be evaluated until they are |
| `.12.2` the open-leaf instrument | 4 | the exit bar is a count, and the count was wrong by 15 |

**Deferred with a trigger (12; do not hold the roadmap):**

| Leaf | Trigger | Honest limit stated in the book |
| --- | --- | --- |
| `.7.3.1`, `.7.3.4.1` the browser worker's transport and retained workspaces | `RB_ENABLE_R5R3RX` is turned on in a claimed profile, or a gated pack is made default | the gated packs are not qualified |
| `.11.14.3.10.1` the credential broker's global binding namespace | as above (R5 is gated) | as above |
| `.10.2` certification and release verification | the first signed release (G9) | releases are not yet signed or certified |
| `.6.4` the introspection assessment | roadmap planning of introspection | a proposal, not a capability |
| `.11.26` the pre-push stall (0 in 299 runs since its instrument) | the next occurrence; the instrument captures it unattended | the script suite has a rare, uncaught stall |
| `.13.1` the external G6 preconditions, `.14` the exposure-profile candidate | a decision to claim Internet exposure; external judgment, never self-certified | Internet exposure is not claimed (G6/G7 NOT MET) |
| `.11.4.7.2.1.5.3.2.3.4`, `.3.2.4.1`, `.4.2`, `.4.4` | as each already states (`DOC-0137`–`DOC-0141`) | as each already states |

**Structural (19):** `.3.3`, `.3.3.4`, `.3.3.4.3`, `.3.3.4.3.3`, `.3.3.4.3.3.3`, `.3.3.4.3.3.3.3`, `.3.3.4.3.3.3.3.2`, `.7.1`, `.7.3`, `.7.3.4`, `.8.1`, `.9.3`, `.11.2`, `.11.4`, `.11.4.3`, `.11.4.3.1`, `.11.4.3.1.2`, `.11.9.1.3`, and `.12`. They close when their children do, and two carry a census obligation of their own: `.7.3`'s ungated R2 extraction clauses and `.11.2`'s gate clauses are classified at their closing census, and a blocking clause becomes a leaf.

## How to apply

- Before opening a leaf, classify it by this bar and write the class in its Status line; a deferred leaf states its trigger there.
- Work blocking leaves in order of exposure: class 1 and 2 first, then 3, with class 4 (the CI checkpoint) run as the last step before the push.
- Closing censuses are cheap and remove false backlog; run them early.
- A census that finds nothing blocking closes its leaf; it does not keep it open for completeness.
