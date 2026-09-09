---
answers:
  - How does an authorization record distinguish a frozen inspection from ordinary evaluation?
  - What does missing provenance mean in historical audit records?
  - Does an admission record prove a write, an effect or delivery of a read response?
---
# Record the evaluation path explicitly without inventing historical intent

- Owner: `SIGNOFF-REPAIR.3.3.3.2.2`, with three bounded implementation children.
- Status: `.2.2.1` complete in 305ed26 and qualification closure dec4d3c (51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint). `.2.2.2` implements the seven HTTP receipt producers with 45 live authority/API tests, ten pure controls and strict lint passed. All results and owned-cluster shutdown are consumed; scoped readback remains `.2.2.3`.
- Predecessor: `docs/decisions/2026-09-09_frozen-tenant-read-eligibility.md`.

Add a closed `evaluation` object to authorization records. `legacy_unspecified`
means the producer did not record the evaluation path. `boundary_checked` means
the ordinary evaluator required an active actual boundary; it does not identify
a write because this evaluator also gates ordinary reads. `tenant_admin_inspection`
identifies the approved frozen-own-tenant exception and carries the direct
principal, named inspection, actual boundary status and selected grant scope.
Existing record fields still carry the actual grant/boundary references.

Missing authority sources have absent references and absent status/scope evidence.
Malformed evidence is a storage error, never a guessed decision or default intent.
The inspection name is closed: nodes presence, grants, boundaries, incarnations,
runs, breakers, usage, or the explicitly planned lookup of one authorization
record by its typed ID. The last surface is a separate implementation child;
declaring its metadata does not make it an available HTTP endpoint.

Migration 0055 gives existing rows and old writers an explicit legacy_unspecified
default. New ordinary writers supply boundary_checked themselves. Core JSON
without evaluation remains readable as legacy_unspecified. New serialized records
include the field, so old strict readers that reject unknown fields must upgrade;
forward-reader compatibility is not claimed. No historical row is relabeled based
on its action or time. The existing policy digest format does not bind this new
field or every source field; this change does not introduce a cryptographic
commitment to complete evidence.

Stored-record decoding validates typed IDs, decision/target discriminants,
nullable field pairs and evaluation metadata. Unknown values, contradictory
fields and malformed source evidence fail explicitly without panic or silently
dropped subjects. The existing public loader keeps its function signature.

Pinned Serde 1.0.229 internally tagged unit variants discard extra fields through
InternallyTaggedUnitVisitor even with enum-level deny_unknown_fields. A matched
control reproduced this on boundary_checked. Marker variants therefore use empty
struct forms, which select the strict struct decoder while keeping the same wire
JSON. Retain the unknown-field and alternative-shape controls when changing these
types; deriving a marker enum is not itself evidence of strict decoding. The
internal-tag and struct visitors also accept sequence alternatives. Evaluation,
inspection and shared TargetSelector decoding therefore enter through
deserialize_map, passing MapAccess directly to private derived wire enums so
duplicate keys remain detectable. TargetSelector's public variants, valid JSON
and schema stay the same; tenant-wide inputs with discarded thread fields and
sequence alternatives are refused. Remaining tagged authority codecs are separately
owned by `.3.4`, without a blanket strictness claim for every exported type.

The seven HTTP inspection routes now commit admissions and return record IDs in
`x-reasonbraid-authorization`, including authority denials. The common record writer
takes explicit evaluation from the normal or inspection entrypoint. Inspection
samples database clock_timestamp after acquiring its connection and retains the
selected parent's original status and grant selector. It commits before response
queries; successful body shapes remain unchanged. Extraction and authority/audit
storage failures have no confirmed receipt. A later response-query failure retains
its already committed receipt with a safe storage-error response. The owned HTTP
controls force both failure boundaries and verify recovery. `.2.2.3` will add one
tenant-scoped receipt lookup, gated and audited as an explicit eighth inspection
purpose. There is no unbounded audit-list expansion in this work.

A record describes admission under a named evaluator. It does not prove a domain
effect, delivery of every response byte, or transaction/revocation serialization.
Effect and ordering guarantees remain `.3.3.4`. The owning children record matched
controls and completion; this design is not a claim that future children ship.
