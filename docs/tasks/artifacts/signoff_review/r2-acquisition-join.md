# The R2 acquisition join: from a served document to its persisted evidence

Owner: `SIGNOFF-REPAIR.7.3.3.4.1`; REPAIR-0095. Predecessor: `300de41`
(REPAIR-0094, clean tree).

## The gap this closes

`.7.3.3.3.2` closed the production R2 input boundary and stated its own coverage
limit rather than implying it away: no live test drove a SUCCESSFUL acquisition
through to a snapshot and a derivation, because the live R2 test refuses at the
loopback destination gate and an outbound Internet fetch is not an acceptable
test dependency. That census is unchanged at this commit —
`git grep -n "R2_RESOLVER_ID\|r2-extract-worker\|extraction_version" --
'crates/**/tests/*.rs'` returns two hits, a resolver-ranking assertion and a
hand-submitted derivation, and `git grep -n 'acquisition"\]' --
'crates/**/tests/*.rs'` returns none.

That join is where `.7.3.3.1`'s misattribution consequence would have become a
persisted wrong record. Everything between the acquired bytes and the stored
evidence was covered only below the HTTP handler.

## The seam, and what it does not relax

`ApiState::with_acquisition(pool, enabled, broker, fetcher)` takes the R0
fetcher the acquisition legs use. `with_gate` now delegates to it with the same
`Fetcher::new(FetchLimits::default())` it built inline before, so `new`,
`with_gate`, `api_router` and `api_router_gated` are behaviourally unchanged and
every construction inside the server binary still receives the shipped
https-only public-destination policy. `api_router_with_acquisition` is the
matching router.

The shipped policy is not edited. A caller that wants another destination policy
has to construct one in its own source and owns what it admitted — which is the
same shape as the existing `with_gate` seam, already documented in the source as
"the test/deployment seam".

## The control

`crates/reasonbraid-server/tests/profiles.rs` —
`the_r2_acquisition_persists_the_served_document_s_own_evidence`.

A local origin serves one Atom document on two paths that differ only in the
type they are served under. THREE deployments then resolve the SAME reference,
so each production gate is isolated rather than bundled into one pass/fail:

| deployment | acquisition leg | measured outcome |
| --- | --- | --- |
| `api_router` | the shipped fetcher | `scheme_not_allowed`, no evidence row |
| `with_acquisition`, origin scheme, SHIPPED destination policy | `ssrf::evaluate` | `destination_refused`, the class `loopback` named |
| `with_acquisition`, origin scheme, loopback-admitting policy | loopback allowed, every other class still `ssrf::evaluate` | acquired, extracted, persisted |

The persisted evidence is then asserted against the bytes the origin served, not
against a status code:

- the receipt's `parent_digest` equals `sha256` over the served document;
- `evidence_snapshots.raw_digest` equals the same digest, `byte_length` equals
  the served length, `resolver_id` is `r2-extract-worker`, and `original_locator`
  is the requested locator;
- the `derivations` rows for that snapshot are exactly the three chunks
  `extract_feed` derives from that feed — digest AND content — stated in the
  control as literals rather than re-derived with the worker's own parser;
- the refused reference has zero `evidence_snapshots` rows.

## The registry row is a deployment fact, not a code path

The pre-established design did not account for `resolvers::resolve` filtering on
`schemes @> [reference.scheme]` over the registry row. Migration 0027 declares
`["https"]`, so an honest `http` reference ranks no R2 pack.

`resource_references.scheme` is caller-supplied and is not validated against the
locator, so submitting `scheme: "https"` for an `http://` locator would have made
the control pass on data that is simply false. The control instead asserts the
migration's `["https"]` baseline, widens the row for its three resolves, and
restores it BEFORE any assertion runs, so no later suite in the same database
inherits the change.

## Falsification — three injections, each reverted and re-run green

A control never observed failing is not known to work
(`docs/CLAIM_VERIFICATION.md` leg 2). Each injection was applied, run, reverted,
and the suite re-run green, so attribution is by revert-and-re-apply rather than
by reading.

| injection | where it failed | what it proves |
| --- | --- | --- |
| the origin serves ONE extra byte | the receipt digest assertion — `8f1d9f61…` observed against `561688a4…` expected | a successful acquisition that persisted the wrong bytes fails the control. The three chunk digests were IDENTICAL under this injection, so the derivation assertions alone would NOT have caught it: the raw-byte leg is load-bearing |
| `with_acquisition` discards the supplied fetcher | the destination-gate assertion — `scheme_not_allowed` observed where `destination_refused` was expected | the seam actually working is what makes the control pass |
| the handler persists a derivation whose content is not the worker's chunk | the derivations assertion, and nowhere earlier | the derivation leg is live and independent of the raw-byte leg |

## Verification

The profiles suite passes 32/32 live in the runner's owned disposable cluster,
including this control; the cluster was stopped and removed
(`pg-tests: stopped and removed target/pg-tests/run-hsk0hhd8`). 97 server
library tests, 16 completion controls, 7 owned-input integration controls,
strict `-D warnings` server lint over all targets, and workspace format pass.

Reproduce:

```bash
python3 -B scripts/project_env.py cargo build --locked -p reasonbraid-extract
RB_DEMO=0 bash scripts/run_pg_tests.sh profiles
```

The control refuses rather than skipping when the extraction worker is absent: a
control that reports neither way is worse than one that stops.

## The mismatch refusal, live (`.7.3.3.4.2`, REPAIR-0096)

The sibling join. The seven controls `.7.3.3.3.2` added all call
`extract_acquired_bytes` directly and never reach a database —
`git grep -n "PgPool\|sqlx" -- crates/reasonbraid-server/tests/extraction_input.rs`
returns nothing. They prove the refusal is RAISED. Nothing proved the HTTP
handler HONOURS it.

`the_r2_mismatch_refusal_persists_neither_snapshot_nor_derivation` acquires the
same served document through the same admitting deployment, with a dishonest
worker injected through the `R2_WORKER_BIN` override the spawner already reads —
so production is untouched. The reply is deliberately WELL-FORMED: a malformed
one is refused by the parser and never exercises the digest binding at all. The
stub is written mode 0700 under a runtime-discovered `target/r2-join-controls`
and removed by the control itself.

The absence is asserted three ways, because each permits a different defect:

| assertion | what it still permits on its own |
| --- | --- |
| zero `evidence_snapshots` for the exact reference | a row written under another reference id |
| zero `derivations` joined to that reference through `parent_snapshot_id` | the same |
| whole-table snapshot and derivation counts unchanged across the request | nothing in this purged database |

Falsification — two injections, each reverted and the suite re-run green:

| injection | result | what it proves |
| --- | --- | --- |
| the digest binding removed from `extract_acquired_bytes` | fails naming the stub's own `another document` chunk in the receipt | the handler would otherwise persist a foreign document |
| the refusal KEPT, but a snapshot written before reporting it | the `extraction_source_mismatch` assertion **passes**, and the count fails with `left: 1` | the control measures the ABSENCE, not the error kind — which is the whole point of the leaf |

`git diff --quiet -- crates/reasonbraid-server/src/` confirms production source is
byte-identical to REPAIR-0095 after both injections were reverted. The suite
passes 33/33 live; the cluster was stopped and removed.

`.4.1`'s control is refactored onto the helpers this child needed
(`admitting_fetcher`, `widen_r2_to_http`/`restore_r2_schemes`, `submit_hinted`,
`require_extraction_worker`), with its assertions unchanged.

## The advertised-format repair (`.7.3.3.5`, REPAIR-0097 + 0098)

The finding this work surfaced got a census before it got a repair, and the
census changed the repair.

**Measured (`.5.1`, no production change).** A DECLARED content type is accepted
only from `{text/html, application/xhtml+xml, text/*}` — three arms, enumerated
in both directions — and none of migration 0027's five advertised types is in
it. UNTYPED, the verdict is a property of the BYTES: an all-printable body is
accepted whatever format it belongs to, and the same body with one non-text byte
is refused; ZIP and tar cannot reach that branch by construction. The controls
pass on unchanged production, as a census must, and were falsified by adding
`application/pdf` to the accept set.

**So the first statement of the finding was the wrong SHAPE.** The leg's rule is
not about formats, which means no subset of the advertisement satisfies it —
narrowing the registry row was rejected on that measured ground, not on
preference (`docs/decisions/2026-09-12_r2-acquisition-accept-set.md`).

**Repaired (`.5.2`).** The acquisition leg admits the RANKED pack's own
advertised media types, read from its registry row per resolution. `sniff_kind`
gains that parameter and the additive `SniffedKind::DeclaredType`;
`Fetcher::fetch_admitting` supplies it while `fetch`, `fetch_head` and
`fetch_authenticated` pass an empty slice, so R0's shipped accept set is unchanged.
A missing or malformed row yields an EMPTY set — a bad advertisement must not
widen a gate.

Bound census: `git grep -n "fetch_admitting" -- 'crates/**/*.rs'` returns exactly
one call site, `api.rs:2032`. The R0 arm, the R5 authenticated arm and the R3
preflight pass no admitted set, and no destination policy, scheme list, SSRF
control or ceiling changed.

| injection | result | what it proves |
| --- | --- | --- |
| the admitted set is not the ranked row's | the advertised-type acquisition fails | the set really is that row's |
| an empty admitted set at the R2 call site | fails identically | the repair is load-bearing |
| ONE unadvertised type added to the set | the unadvertised document acquires and the negative control fails | the decision's bound is live, not asserted |

One measured fact is deliberately FLIPPED: this record's own earlier statement
that `application/atom+xml` is refused `media_type_refused`. That refusal was the
defect. It stays recorded here and in `.7.3.3.4.1` because it is the evidence the
repair rests on, and the live control now carries an `application/json` path for
the negative case.

## What this does NOT establish

Both joins are qualified: the success path persists the served document's own
evidence, and the mismatch refusal persists nothing. Pipe bounds, descendant
containment and aggregate retained storage remain `.7.3.4`. This is one document,
one format, one loopback origin: it is the join that was missing, not a
qualification of the R2 pack's reach — and the finding below is exactly why that
distinction matters.

## The finding this surfaced

The R2 pack advertises five media types and its acquisition leg refuses them.
The R0 sniff accepts a DECLARED content type only when it is `text/html`,
`application/xhtml+xml` or `text/*` (`fetcher.rs:767`), so the same feed that
succeeds served as `text/xml` is refused `media_type_refused` when served under
its own `application/atom+xml` — measured live by this control, on the same
bytes.

The census is decisive and runs both directions: `git grep -n "R2_RESOLVER_ID"
-- 'crates/**/*.rs'` returns three hits — the constant, the single dispatch arm
`api.rs:2012`, and the `resolver_id` written into the snapshot — and that arm
makes exactly one acquisition call, `state.fetcher.fetch`. None of migration
0027's five advertised types is in the accepted set.

What an UNTYPED response of each format sniffs to from its bytes is NOT measured
and is therefore not claimed here. `SIGNOFF-REPAIR.7.3.3.5` owns that per-format
census and the decision that follows it: narrow the advertised types, or teach
the acquisition leg the R2 format set under an explicit bounded rule. The second
is a production content-policy change with its own hostile-content surface and
does not happen as a side effect of a test.
