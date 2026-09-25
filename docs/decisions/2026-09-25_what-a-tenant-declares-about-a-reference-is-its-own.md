---
answers:
  - When two tenants cite the same URL, whose scheme, hints and purpose does each one's resolution and read use?
  - What do submitted_by and created_at mean on GET /v1/resources/{resource_id}?
---
# What a tenant declares about a reference is its own

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.7.1.4`
- **Date:** 2026-09-25
- **Work unit:** `REASONBRAID-REPAIR-0497`
- **Source:** `REASONBRAID-DOC-0161`'s census of `.7.1` measured the defect; `REASONBRAID-DOC-0162` classed it blocking (class 1, cross-tenant); the director delegated the corrective decisions (2026-09-25).

## The fact / decision

A §12.1 reference is one shared CONTENT row per `(original_locator, expected_digest)` and one STATEMENT per registering tenant. The eight declared attributes (`scheme`, `media_type_hint`, `fragment_or_selector`, `owning_node_or_capability`, `visibility_scope`, `purpose`, `retention_class`, `risk_class`) live on the tenant's `reference_registrations` row, beside its credential binding, and are written on the first registration and on every replay as the tenant's complete statement (`migrations/0111`). The tenant-bound read and the resolution read only the reading tenant's statement; the read's `submitted_by` and `created_at` are that tenant's `registered_by` and `registered_at`.

## Why

- **The first citer chose for everyone.** The shared row carried the declared attributes, written once by the first tenant to cite a URL; a later tenant's replay discarded its own. Resolution ranks on `scheme`, so tenant A declaring `ftp` made tenant B's `https` citation unresolvable, and B's read showed A's purpose, hints, risk class and actor handle (measured live before the change).
- **It is the shape the credential binding already took** (`SIGNOFF-REPAIR.11.14.3.10`, `migrations/0069`): the shared row keeps the content, the tenant-bound row keeps the decision, and the moved columns are dropped so an inheriting read cannot be written again.
- **Part of the row's identity was the rejected alternative.** Keying the row on `scheme` too would split one URL into one row per scheme and still leave the other seven attributes first-writer.

## How to apply

- Anything a caller STATES about a shared, content-addressed row belongs on its tenant-bound registration, never on the shared row.
- Existing registrations were backfilled with the shared values, which is what each tenant had been reading; a later citer's discarded values were never stored and cannot be recovered, and its next submission replaces them.
- Not decided here: `replayed: true` still confirmed that another tenant cited the pair, the limit `migrations/0067` records. With the read no longer showing the first citer's handle and time, it was the remaining signal, and `SIGNOFF-REPAIR.7.1.4.1` closed it the same day (`REASONBRAID-REPAIR-0498`): `replayed` reports this tenant's own history with the pair.
