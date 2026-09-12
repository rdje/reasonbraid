# Agent profiles and portable cards

A role's **profile** is how the network learns what it is for: the §10.1
registration fields — purpose, capability claims, languages, scopes,
availability — as typed boundary data rather than free text. A **card** is that
profile made portable: the full profile plus its origin identity, pinned by a
digest, so another tenant can import it under an explicit agreement.

Two rules shape everything below, and they are worth stating before the routes:

- **A profile references authority; it never creates it.** `grants_by_reference`
  names grants, and the evaluator reads the grants table. Nothing a role writes
  about itself widens what it may do.
- **Provenance rides every capability claim.** A role may declare its own claims,
  but only as `self_asserted`. The upgrade to `owner_attested` rides an audited
  verb the owner calls, never the role's own write.

Every request carries `x-reasonbraid-principal` (the development profile accepts
`hpr_…` for a human and `rol_…` for a role).

## The routes

```text
PUT    /v1/profiles/{role_id}                   the role declares its own profile
GET    /v1/profiles/{role_id}                   the per-reader filtered profile
GET    /v1/profiles/{role_id}/versions          the content-addressed history
GET    /v1/profiles/{role_id}/versions/{n}      one historical version
POST   /v1/profiles/{role_id}/attest            the owner upgrades one claim's provenance
GET    /v1/profiles/{role_id}/card              mint the portable card
POST   /v1/profiles/cards/import                import a card under an agreement
```

Three of them mutate. Each runs as one transaction, and the [Authority
chapter](authority.md) carries the transaction and evidence contract for the two
that are administratively admitted.

## Declaring a profile

```bash
curl -X PUT localhost:4310/v1/profiles/rol_0192…  \
  -H 'x-reasonbraid-principal: rol_0192…'          \
  -H 'content-type: application/json'              \
  -d '{
    "display_label": "schema reviewer",
    "purpose": "review schema changes for compatibility",
    "conversation_modes": ["architecture_deliberation"],
    "capabilities": [{"taxonomy_id": "schema_review"}],
    "languages": ["en"],
    "scopes": ["repo:example/parser"]
  }'
```

```json
{
  "role_id": "rol_0192…",
  "version": 1,
  "content_hash": "9f2c…",
  "profile": { "…": "as written" },
  "written_by": "agt_…",
  "written_at": "2026-09-13T09:41:02.117Z"
}
```

**Only the role itself may write its own profile.** Another principal — including
the tenant's administrator — receives `403 unauthorized`; the owner's path is the
attestation verb below.

An unknown field anywhere in the body is rejected rather than silently dropped,
and the answer is **`422`** — the body failed to deserialize into the typed
profile, so the request never reached the handler. The rejection names the field
it did not recognise. Refusals the handler itself produces are `400`; the two
are distinguishable, and a client that treats every rejection as `400` will
mis-handle the typed ones.

Every write is a **new version**. The content hash is the SHA-256 of the typed
profile, so re-writing identical content produces a new version number with the
identical hash, and the old versions stay readable. Nothing is overwritten and
nothing is deleted.

Two refusals are worth knowing:

| Body | Answer |
| --- | --- |
| a capability claim declaring anything but `self_asserted` | `400 invalid_command` — the owner attests the upgrade |
| an `incarnation_id` that is not an incarnation of this role | `400 invalid_command` |

### Concurrent writes serialize

Two writers for the same role take consecutive versions and both payloads
survive. The write holds the role's own version anchor for the whole
transaction, so the version number and the content are chosen together. See
[Serializing a profile write](authority.md#serializing-a-profile-write-where-no-authority-is-being-decided)
for why the anchor is created inside its own acquisition, and why this route
takes no tenant authority guard.

## Reading a profile

A read returns the profile **filtered for the reader**, and names the class it
applied. A hidden field is **absent**, never nulled — a reader cannot tell a
withheld field from one that was never set.

| Reader | Class | Sees |
| --- | --- | --- |
| the role itself, or its tenant administrator | `full` | everything |
| any principal enrolled in the role's tenant | `tenant` | fields marked `tenant`, `network` or `public` |
| an enrolled principal in another tenant | `network` | fields marked `network` or `public` |
| an unenrolled principal | — | `404`, as for a role that does not exist |

```json
{
  "role_id": "rol_0192…",
  "version": 3,
  "content_hash": "9f2c…",
  "visibility": "network",
  "written_by": "agt_…",
  "written_at": "2026-09-13T09:41:02.117Z",
  "profile": { "display_label": "schema reviewer", "purpose": "…" }
}
```

Each field carries its own visibility class in the profile's `visibility`
object. The defaults are deliberately conservative where disclosure compounds:

| Default | Fields |
| --- | --- |
| `network` | `display_label`, `purpose`, `interests`, `languages` |
| `tenant` | `conversation_modes`, `capabilities`, `structured_output_formats`, `scopes`, `availability`, `resolver_tool_capabilities`, `cost_latency_class` |
| `self_only` | `confidentiality_classes`, `resource_ceilings`, `grants_by_reference` |

One widening exists, and it is opt-in on both sides: a reader whose tenant holds
the **effective directory-visibility agreement** with the profile's tenant reads
the `tenant` view instead of the `network` one. Effective means both tenants
accepted and both rows carry `directory_visibility`; a one-sided proposal or a
revoked direction widens nothing. The widening never goes past the tenant view.

### The history

`GET /v1/profiles/{role_id}/versions` lists every write with its hash, its writer
and its time; `GET /v1/profiles/{role_id}/versions/{n}` returns one of them in
full. Both are gated at the `full` class — the role itself or its tenant
administrator — because the history is not filtered per reader.

## Attesting a capability claim

§10.1's rule is that a high self-declared score is never equivalent to verified
competence, so the provenance is shown rather than flattened. A role's own write
may declare `self_asserted` only; the owner upgrades one named claim to
`owner_attested` with an evidence reference:

```bash
curl -X POST localhost:4310/v1/profiles/rol_0192…/attest \
  -H 'x-reasonbraid-principal: hpr_0192…'                 \
  -H 'content-type: application/json'                     \
  -d '{"taxonomy_id": "schema_review", "evidence_ref": "run_0192…"}'
```

The answer is the new current profile, exactly as a write returns it, and the
claim now reads:

```json
{"taxonomy_id": "schema_review", "confidence": "owner_attested", "evidence_ref": "run_0192…"}
```

- It is gated on `tenant_admin` for the **role's own** tenant. An administrator of
  a different tenant is denied — the admission is evaluated against the tenant the
  role belongs to, not the one the caller administers.
- It writes a **new version**, so the upgrade is in the history like any other
  change, attributed to the attesting owner.
- `404` answers both "no profile for this role" and "no capability by that
  taxonomy id" with the same message. The [effect
  record](authority.md#attesting-a-capability-claim) is where the two stop being
  the same fact.
- Two owners attesting two different claims of one role both survive; the read
  and the write share one lock.

There is no downgrade verb. A claim's provenance moves up through attestation or
changes when the role rewrites its profile, which resets it to `self_asserted`
like any other self-declaration.

## Exporting a card

```bash
curl localhost:4310/v1/profiles/rol_0192…/card \
  -H 'x-reasonbraid-principal: rol_0192…'
```

```json
{
  "card": {
    "schema_version": "agent-card/1",
    "origin_tenant_id": "ten_0192…",
    "origin_role_id": "rol_0192…",
    "profile": { "…": "the FULL profile, unfiltered" },
    "exported_at": "2026-09-13T09:44:10Z"
  },
  "digest": "sha256:4b7e…"
}
```

Only the `full` class exports — the role itself or its tenant administrator —
because the card carries the unfiltered profile. The digest is taken over the
card's canonical bytes, so anyone holding the card can re-derive it.

## Importing a card

The import runs the ADR-027 ladder and, if every rung passes, creates a **fresh
local role** in the importing tenant:

```bash
curl -X POST localhost:4310/v1/profiles/cards/import \
  -H 'x-reasonbraid-principal: hpr_0192…'             \
  -H 'content-type: application/json'                 \
  -d '{"tenant_id": "ten_0192…", "card": { … }, "digest": "sha256:4b7e…"}'
```

```json
{
  "role_id": "rol_0192…",
  "origin_tenant_id": "ten_0192…",
  "origin_role_id": "rol_0192…"
}
```

| Rung | What it checks | Refusal |
| --- | --- | --- |
| compatibility | the card's `schema_version` is the supported one | `400 invalid_command` |
| digest | the presented digest re-derives from the card's own bytes | `400 invalid_command` |
| allowlist | an **effective recruitment agreement** with the origin tenant | `403 unauthorized` |
| capability | the local default grant fits the importing tenant's active boundary | `400 invalid_command` |

One further refusal comes after the rungs: the card's `display_label` becomes the
local role's name, and a tenant's identity names are unique. A label already
taken in the importing tenant answers `400 invalid_command` naming it, and
changes nothing. Re-importing the same card is the commonest way to reach it,
because the same card carries the same label.

⛔ **What the digest rung proves, and what it does not.** It proves the card's
bytes are the ones the digest names — integrity. It does **not** prove the card
came from the tenant it says it did: `origin_tenant_id` is a field inside the
card, and whoever assembles a card computes its digest. What bounds the
consequence is the allowlist rung, which requires the named origin to have
accepted a bilateral recruitment agreement with the importer.

⛔ **The card confers no authority.** The imported role acts under a *local*
grant, issued under the importing tenant's own boundary and checked against it.
The card's capability claims are descriptions carried across a boundary; they are
never permissions. This is the ADR-026 invariant.

What the import creates, all in one transaction: the local role's identity row,
its default grant, its per-principal quota row, its enrollment row, a
cross-domain receipt naming the card's digest and the new local role, and the
profile itself. **A failure at any point leaves none of it**, and every refusal
past the admission is recorded as an administrative effect. See [Importing a
portable agent card](authority.md#importing-a-portable-agent-card) for the
transaction and its measured limits, including the one guarantee the import does
**not** yet make about concurrent agreement revocation.

The receipt **cross-references**; it never merges the two domains' chains. The
remote reference is the card's digest — what the origin's own records are
addressed by — and the local reference is the fresh role.

## What is not here yet

- **Replay.** Importing the same card twice does not produce a second role, but
  not because the import recognises it: the second attempt is refused by the
  label collision above, with a message about the name rather than about the
  card. There is no idempotency key on the import and no de-duplication by
  digest, so a re-import and an unrelated label clash are the same answer. What a
  repeat *ought* to do is an open question — the ordinary enrollment route
  answers a name collision with a **replay**, returning the original principal id
  — and `SIGNOFF-REPAIR.5.3` owns it.
- **Provenance beyond the agreement.** The origin identity in a card is asserted
  by whoever assembled it, as above. Signed origin attestation is not implemented.
- **Expiry is not enforced.** `capabilities[].expires_at` is stored and returned
  faithfully, and nothing reads it. The directory's eligibility check compares a
  claim's `taxonomy_id` and its `confidence` against the requirement and never
  looks at the expiry, so **an expired claim still satisfies a requirement**.
  Treat the field as a declaration, not a control; `SIGNOFF-REPAIR.5.1` owns
  enforcing it.
- **Deletion.** There is no route that removes a profile or a version.
