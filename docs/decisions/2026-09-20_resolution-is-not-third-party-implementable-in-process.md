---
answers:
  - Is there a third-party acquisition trait in the resolver SDK?
  - Why can a third party advertise a resolver but not implement one in process?
  - What can a third party actually build against the resolver SDK today?
  - What does a resolver's advertised sandbox_level describe?
---
# Resolution is not third-party-implementable in process, and the SDK says so

- **Type:** decision
- **Status:** accepted; the SDK's crate root, the `resolver` module header and
  the book state it
- **Owner:** `SIGNOFF-REPAIR.11.24.1.5`
- **Date:** 2026-09-20
- **Raised by:** `SIGNOFF-REPAIR.11.24.1`, adjudicating `PHASE-8.4.1`'s goal line
- **Related:** ADR-018 (resolver sandbox/runtime isolation), ADR-027 (signing
  and distribution),
  `docs/decisions/2026-09-19_an-advertised-line-carries-an-adjudicated-verdict.md`,
  `docs/decisions/2026-09-19_the-egress-claim-is-a-ceiling-the-sandbox-claim-is-a-floor.md`

## The question

`PHASE-8.4.1`'s goal reads: *the resolver SURFACE extracted from the internal
registry (the advertise type as the third-party shape — **the acquisition trait
rides the `.4.4` load side**)*. Measured:

- `git grep -n "pub trait" -- crates/reasonbraid-adapter/src` returns **two**,
  `AttemptStream` and `Adapter`, both in `contract.rs` and both the harness
  contract. There is no resolver trait.
- `PHASE-8.4.4` is `done`. What it delivered is ADR-027's five-rung allowlist
  ladder, the ledger migration and the operator verbs. Its own closing line
  defers again — *the load-path WIRING is the named follow-on* — to `.2.3`.

🔴 **So the deferral pointed at a leaf that closed without owning it**, which is
the family `SIGNOFF-REPAIR.11.24` opened over: *a deferral that names a leaf is
not owned by it*. The promise had been sitting in `resolver.rs`'s module header,
readable by any third party, for the whole time.

## The decision

**There is no in-process acquisition trait, and there will not be one.** A
third-party resolver contributes a `ResolverAdvertise` row and an
out-of-process worker binary.

This is not a scheduling answer. It follows from ADR-018's own vocabulary:

1. **`sandbox_level` states what the resolver's CODE provides.** That reading is
   not new here — `SIGNOFF-REPAIR.7.3.6.1` established it by repairing R3, which
   advertised `vm_container` while being an ordinary child process, and left
   `security_evidence.container_required` beside it to carry what the DEPLOYMENT
   must add. The level is the product's property; the evidence key is the
   operator's obligation.
2. **The server filters on it.** `resolvers::resolve` compares the declared
   sandbox level against the reference's required one as a floor, so a caller can
   demand isolation and a weaker pack is ineligible.
3. **In-process code provides `none`**, and no arrangement makes it provide more.

⛔ Put together: an in-process trait would let a third-party pack advertise
`constrained_process` or `vm_container` while structurally being `none`, and the
registry would admit it on the claim. **That is worse than an absent surface** —
it turns the one advertised field the registry actually consults into a field it
cannot back. The other four policy fields are already consumed by nothing
(`2026-09-19_an-advertised-line-carries-an-adjudicated-verdict.md`); spending the
two that are consumed would leave the advertise contract decorative.

⭐ **And the execution surface already exists — it is a child process.** The
built-in packs' declared levels track their execution shape exactly, measured
rather than asserted: the only child-process spawns on any resolver execution
path are `browse.rs` (R3) and `extraction.rs` (R2), and those are precisely the
two packs declaring `process`. The other four run in the server process and
declare `none`.

| Pack | Runs as | `sandbox_level` |
| --- | --- | --- |
| `r0-https-fetcher` | the server process | `none` |
| `r1-git-fetcher` | the server process | `none` |
| `r2-extract-worker` | a child process per extraction | `process` |
| `r3-browser-worker` | a child process per render | `process` |
| `r5-credential-broker` | the server process | `none` |
| `rx-agent-mediated` | the server process (the acquisition is the node's) | `none` |

⭐ **ADR-027 verifies that artefact and not any other.** Its ladder is allowlist
→ digest → signature → API compatibility → capability manifest. Every rung is a
property of a distributed binary; none of them has a meaning for a Rust trait
implementation compiled into the server, and there is no dynamic-load mechanism
in the workspace for one to arrive through.

## What the SDK now says, and where

The finding was not only that the trait is absent — it is that **the SDK's front
door never mentioned the resolver surface at all.** `lib.rs` declares
`pub mod resolver;` and re-exports its types, while the crate-root documentation
lists `contract`, `fake`, `fixtures` and `bench` and stops. A third party reading
the SDK's first page learned nothing about resolvers, and the one place that did
describe them carried the stale promise.

Corrected in three places, because a boundary a reader must infer from which
traits happen to exist is not a documented boundary:

- **`lib.rs`** — the crate root names the resolver surface and states the
  asymmetry: a harness is implemented in process, a resolver is not.
- **`resolver.rs`** — the module header carries the argument, replacing the
  `.4.4` deferral.
- **`docs/book/src/adapter-boundary.md`** — a *The other boundary in the same
  SDK* section with the comparison table, the reasoning, and the pack table
  above. That page had **zero** occurrences of the word *resolver* before this.

⚪ **Five broken intra-doc links were repaired in the same change**, two of them
in the crate-root paragraph this decision rewrites: the root linked `[`contract`]`
and `[`fake`]`, which are private modules, so rustdoc dropped both links on the
SDK's own front page. `cargo doc -p reasonbraid-adapter` is now warning-free.

## What is NOT decided, and is stated as a limit rather than left implicit

⚠️ **The worker wire protocol is not published as a stable SDK surface.** R2's
and R3's request/response shapes live in `reasonbraid-server`'s `extraction` and
`browse` modules. Promoting one into the SDK means taking on its compatibility
obligations — a version token, a deprecation story, a conformance oracle — and
that is a separate decision with its own leaf. Until then a third party can
publish an advertise the registry will rank and cannot yet write a worker against
a pinned contract. The book says this in those words.

⚠️ **This decision does not make the advertised level verifiable.** The registry
still accepts whatever a row claims; what it now has is a boundary under which
the claim can be true. Verifying it is the province of ADR-027's capability-
manifest rung, on the load path that does not exist yet.
