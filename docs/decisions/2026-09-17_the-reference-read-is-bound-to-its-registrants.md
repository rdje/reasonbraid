# The §12.1 reference detail read is bound to its registrants; the pair replay is not

- Date: 2026-09-17
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.14.3.4`
- Corrects: `2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`'s per-table
  row for `resource_references` — *site-wide by design* — which answered the
  COLUMN question and is left byte-unchanged; this record supersedes its READ
  disposition for that one table.
- Related: `2026-09-16_a-citation-registers-the-reference-it-names.md` (the pair
  key), `2026-09-17_the-standalone-assessment-is-citation-bound.md` (the same
  shape on the write side, ruled one commit earlier).

## The finding, reproduced before it was classified

`GET /v1/resources/{resource_id}` gated on enrolment alone. Driven with a second
tenant holding a `res_…` id it never registered, the answer was the whole §12.1
row:

```json
{"created_at":"2026-09-17T07:34:54.018015+00:00",
 "reference":{"original_locator":"https://internal.example.org/q3-reserve-review",
   "expected_digest":"sha256:aaaa…","credential_binding_ref":"the-owner-binding",
   "purpose":"the reserve review the owner is running","visibility_scope":"tenant",
   "risk_class":"high","scheme":"https", …},
 "resource_id":"res_a01e402a-…","submitted_by":"agt_c06e5644-…"}
```

⭐ **Read the `visibility_scope` field in that payload.** The row declares
`"tenant"` and the read ignored it. §12.1 lists the field; nothing consumed it.

`POST /v1/resources/{id}/resolve` shares the same lookup and the same gate, so a
caller refused nothing could also drive an acquisition off another tenant's
reference.

## The census, in both directions

```bash
grep -n '"/v1/resources' crates/reasonbraid-server/src/api.rs   # -> 3 routes
grep -rn "resources::get\b" crates/ --include=*.rs              # -> 2 callers
```

Three routes: `POST /v1/resources` (register/replay), `GET /v1/resources/{id}`
(detail), `POST /v1/resources/{id}/resolve` (acquire). There is **no list verb**,
so a caller must already hold the id for the latter two — an ORACLE, not an
enumeration, and materially weaker than `.11.14.1`'s `GET /v1/snapshots/stale`,
which returned every row.

## The decision, and it is TWO answers rather than one

**The detail read and the resolve verb are bound to the tenants that registered
the reference. The pair replay is kept and published.**

`migrations/0067` adds `reference_registrations` — the `0062` shape applied to
the other content-addressed table: a SET of tenants per reference, written on
every registration **including the replay**. No tenant column is added to
`resource_references`; the pair key is untouched, and one shared row serves every
tenant that names it.

⭐ **The two halves get different answers, and the difference is nameable** —
which is the test `docs/CLAIM_VERIFICATION.md` §3 leg 2 sets when a case of the
same shape has already been adjudicated:

| | Snapshot (`.11.14.1`) | Reference (here) |
| --- | --- | --- |
| to confirm existence you must present… | **the BYTES** — the digest is verified against them | **a LOCATOR** — a string anyone can type |
| so the existence oracle is… | closed by construction; a caller that can replay already holds the content | **structural** — closing it means the same pair stops returning the same id, which §12.1 and §12.6 require |
| the detail read is… | bound | **bound** |

So `POST /v1/resources` still answers `replayed: true` for a pair that exists.
That is published rather than implied, with its exact width: the caller must
already know the locator **and** the digest, and what it learns is that the pair
is registered.

## Why binding the detail read costs nothing, measured

The rule this project earned one commit earlier applies directly: *before paying
for a compatibility break, enumerate what the caller you would be breaking can
currently do other than the thing you are removing.*

Every route that yields a `res_…` id goes through the pair replay — and the
replay now **records the caller's registration**. A tenant that legitimately
reaches a reference reaches it by naming its pair, which registers it, which
restores the read. ⭐ What changes is the **price of admission**: a bare opaque
handle used to be the whole predicate, and the locator is now required — the very
thing the row would have disclosed. The read stops disclosing anything the caller
did not already hold.

## The alternatives, and why each was rejected

1. **A `tenant_id` column on `resource_references`** — ❌ rejected by
   `2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md` for all twelve
   tables and rejected again here for the same structural reason: it either
   breaks `UNIQUE (original_locator, expected_digest)` or duplicates identical
   references per tenant. This is the mechanism the leaf's own warning was about.
2. **Leave it site-wide and publish the exposure** — ❌ the leaf's warning said a
   locator a contributor *chose* to cite is weaker than an acquisition receipt,
   and that is true of the LOCATOR. It is not true of the row, which also carries
   `purpose` (free text a tenant wrote), `credential_binding_ref`,
   `owning_node_or_capability` and `risk_class`. And the row's own
   `visibility_scope` field says otherwise in the reproduction above.
3. **A read predicate over `submitted_by` instead of a registration set** — ❌
   measured impossible. `submitted_by` is a one-way `Uuid::new_v5` that joins to
   no identity table, and the pair replay leaves it naming the FIRST registrant
   whatever happens afterwards. It is an audit breadcrumb, never an
   authorization input. This is the same measurement `0062` recorded.
4. **Binding the replay too** — ❌ it is the pair key. §12.6 requires a changed
   page to be a second reference and §12.1 forbids erasing security-relevant
   distinctions, so the same pair must return the same id. `.11.14.3.2` retired
   `locator_digest_conflict` precisely to stop one principal's pin making a
   locator uncitable by everyone else; refusing the replay would reintroduce
   that one layer down.

## What it does NOT close

- ⚠️ **The pair replay stays an existence confirmation.** Published above with
  its width.
- ⛔ **`credential_binding_ref` remains an unauthenticated caller field that
  SELECTS a credential**, and binding the read does not touch it: the second
  tenant can register the same pair, inherit the shared row, and resolve it. That
  is `SIGNOFF-REPAIR.11.14.3.10`'s, opened with its measurement. ⚠️ **Its width is
  measured, not assumed**: the R5 pack is off by default (`RB_ENABLE_R5R3RX`), and
  `grep -rn "\.register(" crates/ --include=*.rs | grep -i broker` finds **no
  production caller** — every shipped deployment runs an empty broker store, so
  there is no credential to reach today. It is a latent design defect, not a live
  disclosure, and saying otherwise would be a claim derived by reading.
- ⚠️ **`visibility_scope` is still read by nothing.** The binding is over
  registration, not over the field the row declares. Whether that field should
  govern anything is `SIGNOFF-REPAIR.11.14.3.5`'s, which already owns the
  unvalidated §12.1 caller fields.
- ⚠️ **No backfill.** A reference registered before `0067` is read by no tenant
  until it is registered again, at which point the replay records it. That is the
  fourth time this family has taken that route and the reason is the same one
  `0062` measured: the attribution is not recoverable even in principle.

## Verification

- RED: a second tenant received the full §12.1 row (the payload above);
  `45 passed; 1 failed`.
- GREEN: the second tenant's detail read and resolve both answer `404` with the
  same body an absent id gets; the registering tenant still reads its own row;
  registering the same pair replays to the SAME `resource_id`, records the second
  registration and restores the read; **2 registrations on one shared row**.
- `RB_DEMO=0 bash scripts/run_pg_tests.sh profiles`, plus the suites whose
  fixture plans gained the new table.
