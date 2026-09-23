---
answers:
  - With the call's remote form built, which items of `SIGNOFF-REPAIR.5.3`'s goal line are met, and by what?
  - Do the origin-delivery receipts satisfy "bind receipts to actual digest references"?
  - Can a revocation of the recruitment agreement still widen visibility or effects through the origin binding?
---
# The federation goal line after the remote form: four items met, one live race

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.5.3`
- **Date:** 2026-09-23
- **Work unit:** `REASONBRAID-DOC-0151`
- **Cites:** ADR-026 (*the remote domain vouches for its own records, the local domain acts on its own grants*); ROADMAP §20.10, ROADMAP:596 (federation between deployments, signed bundles — post-v1); `docs/decisions/2026-09-23_the-federation-goal-line-two-items-met-two-live-defects-and-the-calls-remote-form-unbuilt.md` (DOC-0143, the first census of this line); `docs/decisions/2026-09-23_an-imported-identity-runs-on-a-bound-machine-local-now-origin-designed.md` (DOC-0148).

## The fact / decision

`.5.3`'s goal line, read against the code at `8f383c5`:

| Item | Verdict | Where / by what |
| --- | --- | --- |
| Complete remote recruitment under explicit local grants | ✅ MET | `.5.3.5` closed: the advertisement (`.5.3.5.1`, REPAIR-0434/0435/0437), the join request (`.5.3.5.2`, REPAIR-0436), the imported identity's execution — `local` (REPAIR-0441) and `origin` end to end (REPAIR-0446, 0447, 0449, 0450, 0451, 0453). The imported identity acts only under the grant the import issued under the importing boundary (`profile_admin::import_after_admission`), and its origin node's result folds as that identity under that grant (`apply_node_result_in_tx`, REPAIR-0451). |
| Bind receipts to actual digest references | ✅ MET, with a scoped decision | `agreement` → the counterparty's terms digest (REPAIR-0432); `card_import` → the card digest. The two delivery kinds (REPAIR-0453) reference RECORDS by id — `ack:{node}:{cursor}` and the `authz_ref` — not by digest. Decided: within one deployment both ids name immutable rows of the one ledger, which is a real reference and not the bare tenant id the item was opened against; a digest becomes owed when the two domains stop sharing a database — the post-v1 trigger (ROADMAP:596) DOC-0143 already records for signed bundles. |
| Transact identity, profile, quota and receipt import together | ✅ MET | REPAIR-0122; the `origin` binding's `executes_on` rides the same transaction (REPAIR-0446). |
| Isolate replay and provenance | ✅ MET | REPAIR-0431 (`card_imports`, the replay key); a repeat import cannot change the binding (REPAIR-0446). |
| Prove revocation races cannot widen visibility or effects | 🔴 LIVE → `.5.3.6` | The direction verbs and the import are fenced (REPAIR-0125/0126), and every NEW act resolves the binding at its own instant: the respond, the close, the directory, the dispatch (`409`) and the result fold (REPAIR-0446…0451). But work ALREADY queued for an origin-bound identity is not: `node_channel::replay`'s tail filters on the row's grant and the node's certificate, never on the binding, so after either side revokes the recruitment direction the origin node is still OFFERED the importing tenant's queued rows — the thread's subject and objective in the payload — and a row it already holds still DISPATCHES, because the node's per-tenant epoch for the importing tenant does not move (`revoke_direction_in_one_transaction` bumps no epoch and touches no inbox row). The fold refuses the result, so no local EFFECT widens; the VISIBILITY does, and the origin machine spends its provider on work its operator withdrew consent for. Source reading; `.5.3.6` reproduces it RED first. |

The attached clauses: clause 1 (re-consent) was adjudicated by DOC-0143 and its record built by the terms digest (REPAIR-0432); clause 2's version (the digest) and expiry (REPAIR-0433) are built, its signature deferred to the post-v1 signed bundles with the trigger recorded.

## Why

A goal line closes item by item against code (DOC-0142, DOC-0143, DOC-0149). The origin binding added three new cross-domain surfaces in one session; the revocation item is the one that has to be re-asked of each, and the one surface that was built to resolve the binding at a single instant — the NEW act — left the work already in flight to the old rule.

## How to apply

- Build `.5.3.6` next: the replay stops offering a row whose role no longer executes on the reading node, and the handshake/poll's epoch map becomes the COMPLETE set of tenants the node may act for now (its own, and each tenant with an identity whose binding resolves to it), which the node adopts wholesale — so a held command of a tenant that left the set has no epoch reference and is refused at the dispatch gate. RED first.
- `.5.3` closes with `.5.3.6`.
