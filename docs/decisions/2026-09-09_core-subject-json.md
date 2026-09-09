---
answers:
  - What JSON representation does GrantSubject use?
  - Why does the command envelope still carry a string on_behalf_of field?
  - How are ambiguous or mismatched authority subjects rejected?
---
# Core subject objects and established envelope compatibility

- Owner: `SIGNOFF-REPAIR.3.3.1`.
- Status: implemented; 49 core unit and 3 subject integration tests pass. All 40 live authority/command API/site-receipt compatibility controls and strict core/server lint pass.
- Companions: `docs/decisions/2026-09-09_site-operator-authority.md` and `docs/adr/009-delegated-authority-representation.md`.

The old core GrantSubject enum put an internal kind tag around ID newtypes that
serialize as strings. Serializing a human or a containing grant therefore failed;
json! callers could panic. The dedicated direct/enclosing-type controls reproduce
that failure independently of the earlier site-service run.

GrantSubject now serializes as an explicit object with kind and id. The kinds
are human and role, and their IDs retain the existing prefix-checked core types.
Serialization uses Serde's adjacent tag/content form. Deserialization requires a
map and uses derived field parsing to reject duplicates, missing or unknown
fields; it then parses the ID in the declared kind. Arrays and bare strings are
not alternative subject encodings. Field order carries no authority.

Serde documents the distinction between internal and adjacent tagging in its
[enum representation guide](https://serde.rs/enum-representations.html) and unknown
field handling in its [container attributes](https://serde.rs/container-attrs.html).
The pinned adjacent-enum derive also generates a sequence visitor; the explicit
map entrypoint makes this project's object-only contract narrower. It does not
first turn JSON into Value, which would discard duplicate-key evidence.

This repairs core AuthorityGrant, AuthorizationDecisionRecord with a delegated
subject, and DelegationConstraints payloads. It does not change the public
CommandEnvelope AuthorityContext.on_behalf_of string field, HTTP/MCP principal
strings, database subject_kind/subject_id columns or manually shaped inspection
responses. Existing schema/fixture, actor derivation and digest checks remain
required. Correcting the envelope rustdoc also updates its schema description;
a structural comparison confirms no validation field changes. Site issuance
already uses the same kind/id object.

The former delegation-size assertion compared hand-built JSON length with that
same length plus 64. It did not benchmark a token representation or depths 1–3.
The corrected core control serializes and round-trips the actual public
AuthorityContext. ADR-009 now distinguishes the chosen development envelope from
an unperformed comparative benchmark, owned by SIGNOFF-REPAIR.3.4.

Exact failing and corrected results belong to the task leaf. This wire correction
adds no authority and does not close actual-parent selection, delegation-policy
or tenant transaction/audit repairs.
