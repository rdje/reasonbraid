---
answers:
  - Which of SIGNOFF-REPAIR.5.3's five goal items are met by earlier repairs, and by which?
  - What does the cross-domain receipt of an agreement acceptance actually name, and is it a digest?
  - What decides whether a card import is a replay today, and where is an imported role's origin recorded?
  - Who is the issuer of an imported role's grant?
  - What does "remote recruitment" mean in one deployment, and how much of it exists?
  - Is a terms change that the counterparty never re-consented to a widening?
  - In what order are the federation children built?
---
# The federation goal line: two items met, two carry live defects, and the call machinery has no remote form

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.5.3`
- **Date:** 2026-09-23
- **Cites:** `docs/adr/026-federation-trust-agreement.md`; ROADMAP §20.10 (*explicit federation
  trust agreements, tenant-to-tenant visibility, remote recruitment, portable agent cards/profiles,
  and cross-domain audit receipts*) and ROADMAP:596 (*Federation — independent ReasonBraid domains —
  federated trust policy and signed bundles — Post-v1*); `migrations/0048_federation_agreements.sql`,
  `migrations/0049_cross_domain_receipts.sql`; `docs/tasks/artifacts/signoff_review/RECONCILIATION.md`
  (`R-43-4` clauses 3–5); `docs/decisions/2026-09-23_no-call-transition-is-one-transaction-and-the-minimum-is-checked-before-the-filter.md`
  (the census-then-split shape this record repeats)

## Context

`SIGNOFF-REPAIR.5.3` inherited a five-item goal line, two attached clauses and a routing
follow-up, and six repairs under `.3.3.4.11`–`.12` have since rebuilt most of its surfaces.
It is censused before anything is built, as `.5.2` was: which items the repairs met, which are
live defects with a reproduction, and which name work that does not exist yet.

## The census

**1. Remote recruitment under explicit local grants.** ADR-026: cross-tenant recruitment
accepts remote participants only under the agreement's scope, and every local effect rides a
LOCAL grant. What exists is the card import: the remote role lands as a NEW local role under the
importing boundary's default grant (`ThreadContribute`, `ThreadInvitationRespond`). The call
machinery has no remote form — `offer_to_subscribers` is bound to `r.tenant_id = $2`
(`crates/reasonbraid-server/src/recruitment.rs`), `respond_to_call_core` refuses a respondent
enrolled elsewhere, and the match query clamps `network` to the reader's class
(`crates/reasonbraid-server/src/api.rs:6676`). Census:
`grep -rn "has_effective_recruitment_agreement" crates/reasonbraid-server/src --include=*.rs | grep -v src/federation.rs`
→ one line, `crates/reasonbraid-server/src/authority/profile_admin.rs:470`: the import is the
agreement's only consumer. 🔴 And the "explicit local grant" is issued by nobody:
`import_after_admission` mints it with `HumanPrincipalId::new()` (`crates/reasonbraid-server/src/authority/profile_admin.rs:490`) — a
fresh id that names no principal — while the admitting administrator is passed in as
`_principal` and never read. Enrolment does the same for a role's grant (`crates/reasonbraid-server/src/api.rs:1147`) as a
recorded dev limitation, but enrolment is un-admitted by design (DOC-0141) and the import is
admitted, so the import can name a real issuer and does not. ROADMAP:596 places federation
between independent domains post-v1; within one deployment the remote form of a CALL is an offer
reaching a federated tenant's subscribers under the directory agreement and a join that requires
the import — nothing of which is built.

**2. Receipts bound to actual digest references.** `cross_domain_receipts.remote_ref` is
declared *the remote record's digest-pinned reference* (migration 0049). Two writers. The card
import writes the presented card digest, verified against the card's bytes in the same
transaction (`crates/reasonbraid-server/src/authority/profile_admin.rs:572`–`580`; pinned by `crates/reasonbraid-server/tests/cards.rs:517`–`532`) ✅.
The acceptance writes the COUNTERPARTY'S TENANT ID (`crates/reasonbraid-server/src/authority/federation_admin.rs:318`–`326`,
`remote_ref = remote_tenant_id`) — an id, of no record, and no digest ❌; nothing pins it
(`grep -c remote_ref crates/reasonbraid-server/tests/federation.rs` → 0). There is nothing to
digest: a direction row has no version (clause 4).

**3. Identity, profile, quota and receipt imported together** ✅ `REASONBRAID-REPAIR-0122`
(`SIGNOFF-REPAIR.3.3.4.11.3`); control `a_profile_write_failure_leaves_no_imported_identity_behind`.

**4. Replay and provenance isolated** ❌. Replay: the import's identity is the card's
`display_label`, through `agent_roles (tenant_id, name)`. So the same card twice is
`LabelTaken` by accident (the book says so under *What is not here yet*), the same origin role
under a new label imports AGAIN as a second local identity, and an unrelated card carrying a
taken label is refused as if it were a replay. Provenance: the origin ROLE id is stored nowhere —
`grep -rn origin_role_id crates/reasonbraid-server/src migrations` → the card struct
(`crates/reasonbraid-server/src/cards.rs:25`) and the response (`crates/reasonbraid-server/src/api.rs:7194`, `:7280`) only. The receipt carries the origin
tenant and the card digest, so which origin role a local role came from is derivable only by
whoever still holds the card.

**5. Revocation races cannot widen visibility or effects.** Effects ✅: the three direction verbs
run under the tenant's exclusive guard (`REASONBRAID-REPAIR-0125`, control
`authority_that_ends_while_a_direction_revocation_waits_refuses_it`), the import declares both
tenants' guards (`REASONBRAID-REPAIR-0126`, control
`an_import_is_fenced_by_the_origin_tenants_own_guard`), and the epoch clause was declined with
its reason (`R-43-4` clause 5: no federation fact enters a cached decision). Visibility:
`classify_reader` asks `has_effective_directory_agreement` on the pool at every read and nothing
caches the answer, so a read cannot widen; applying visibility per candidate tenant in the
directory and match surfaces is `SIGNOFF-REPAIR.5.1`'s.

**Clause 3 — a terms change is not re-consented by the counterparty. Adjudicated: not a
widening, under ADR-026's model.** A direction row is one side's own declaration and the
effective agreement is the INTERSECTION of the two accepted declarations
(`has_effective_*`: both `accepted` and both flags). A re-proposal changes only the proposer's
declaration, so the counterparty's row bounds every effect exactly as before; asking the
counterparty to re-consent would be consent to a declaration it never made. What is missing is
the RECORD — which terms the counterparty's row carried when this side accepted — and that is
item 2's receipt, once a direction has a digest.

**Clause 4 — no version, expiry or signature.** Version: the terms digest (with item 2).
Expiry: none; a direction lapses only by revocation. Signature: post-v1's signed bundles
(ROADMAP:596), the trigger DOC-0141 already records for the issuance signature; recorded here,
not built.

**The routing follow-up** ✅ discharged by `REASONBRAID-REPAIR-0172` (`SIGNOFF-REPAIR.11.10`):
`RouteError::Storage` and the control
`a_storage_failure_is_not_a_policy_verdict_and_a_real_refusal_still_is`
(`crates/reasonbraid-server/tests/regions.rs:352`).

## Decision — five children, smallest first

1. **`SIGNOFF-REPAIR.5.3.3` — an imported role's grant names its admitting administrator as
   its issuer.** When the admitting principal is a role, the enrolment limitation stands and the
   reason is written beside it. RED: read the grant after an import; its issuer is no enrolled
   principal.
2. **`SIGNOFF-REPAIR.5.3.2` — an import is identified by its origin.** A provenance record
   (importing tenant, local role, origin tenant, origin role, card digest, the admission), unique
   per origin role per importing tenant, written in the import's transaction. A repeat of the
   same origin role answers a REPLAY naming the original local role — the enrolment route's
   shape, which the book already points at — and names the digest on file when the card differs.
   The label collision stays a label collision. RED: the same origin role under two labels lands
   two local roles today.
3. **`SIGNOFF-REPAIR.5.3.1` — a direction carries a terms digest, and the acceptance receipt
   pins the counterparty's.** The server computes `terms_digest` over the canonical terms on
   every proposal; the acceptance reads the counterparty's row, records its digest as the
   receipt's `remote_ref` and as `accepted_against` on the accepting row. An acceptance therefore
   requires the counterparty's row to exist — the one wire change, a `409` naming it, decided
   with its RED. The effective predicates are unchanged (clause 3's adjudication).
4. **`SIGNOFF-REPAIR.5.3.4` — a direction may carry `expires_at`**, proposer-set, honoured by
   the effective predicates and shown by the reads.
5. **`SIGNOFF-REPAIR.5.3.5` — the call machinery's remote form.** A `network`-scope call's
   offers reach a federated tenant's subscribers under the DIRECTORY agreement, and a foreign
   role's join is a request to import under the RECRUITMENT agreement. A feature lane: it opens
   with a design census (who imports, what the offer carries across, what the origin's operator
   sees), and it is last.

⛔ Not decided here: whether a repeat import whose card DIFFERS from the one on file should refresh
the local profile. The leaf answers a replay first and records the question.
