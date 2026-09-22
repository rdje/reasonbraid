# The governance charter and its decision rules

Every tenant is governed by a **charter**: the versioned document that says which
decision rules that tenant may take a decision under, and what bar each one
carries. ROADMAP §4.1 defines it; §13.3 names the rule families.

A charter is content-addressed. Its identity *is* its content, so changing the
allowed set does not edit a charter — it writes a new one, and the old one stays
readable forever under the digest a past decision was recorded against.

## The seven decision rules

§13.3 supports seven rule families, and these are their wire names.

| Wire name | ROADMAP §13.3 |
| --- | --- |
| `owner_decides` | owner decides after consultation |
| `majority_of_electorate` | simple or supermajority of a defined electorate |
| `unanimity` | unanimity of all non-recused electorate members, with explicit abstention semantics |
| `consensus` | consensus with no unresolved blocking objection |
| `role_weighted` | role-weighted or chambered approval defined by a governance charter |
| `human_committee` | human committee approval |
| `advisory_synthesis` | advisory synthesis with no binding decision |

Seven, not nine. *Simple or supermajority* is one family whose bar is a number,
and *role-weighted or chambered* is one family whose weighting the charter
defines. That is why §4.1 names *allowed decision rules **and** approval
thresholds* together: the thresholds parameterize the families rather than
multiplying them.

### Which rules take a threshold

`majority_of_electorate` is the only family that takes one, and it **must** have
one. The value is the fraction of the electorate that carries a decision, and it
lies in `(0.5, 1.0]` — `0.5` alone is not a majority, and above `1.0` is not a
fraction.

Every other family is refused a threshold. `unanimity` is `1.0` by definition, so
a threshold on it is a contradiction rather than a stricter rule;
`advisory_synthesis` produces *no binding decision*, so there is nothing for a
bar to gate.

## Registering a charter

Registration is a **site** act, not a tenant one, and the reason is structural.
The charter is the document that constrains a tenant, and the enrollment boundary
that names its digest is a root- or parent-granted ceiling (§4.4). A tenant that
could rewrite its own allowed decision rules would hold the ceiling it is bound
by. So `POST /v1/governance-charters` requires the `charter_register` site
capability, and is audited like every other site act — see
[Site authority](site-authority.md).

```http
POST /v1/governance-charters
```

```json
{
  "tenant_id": "tnt_<tenant-UUID>",
  "allowed_decision_rules": ["majority_of_electorate", "consensus"],
  "approval_thresholds": {"majority_of_electorate": 0.67},
  "reason": "the platform team's governance baseline"
}
```

The response carries the digest the server derived:

```json
{
  "charter_digest": "sha256:<64 hex>",
  "tenant_id": "tnt_<tenant-UUID>",
  "allowed_decision_rules": ["majority_of_electorate", "consensus"],
  "approval_thresholds": {"majority_of_electorate": 0.67}
}
```

### The digest is the server's, and the request never supplies one

The body may include `charter_digest`, but only to *assert* what registration
will derive. A value that disagrees is refused naming both, and nothing is
written. This is the same rule `POST /v1/policy-publications` follows for a
publication manifest.

### Registering the same content twice is one charter

The row's identity is its content, so a second registration of a byte-identical
charter returns the same digest and leaves one row. Order does not matter either:
`["consensus", "unanimity"]` and `["unanimity", "consensus"]` are the same
charter, because the digest is taken over the sorted set.

Two tenants that allow the same rules hold **two** charters. `tenant_id` is
inside the digested content, because a charter is a tenant's governance document
and not a shared template.

## Reading a charter back

```http
GET /v1/governance-charters/{charter_digest}
```

Tenant-bound: a principal of another tenant receives the same *no charter is
registered under this digest* answer an absent digest gets. The read surface is
not an existence oracle over other tenants' charters.

## Asking whether a rule is allowed

```http
GET /v1/decision-rules/{rule}
```

This answers §4.1's question — *may this tenant decide under this rule?* — for
the calling principal's own tenant.

```json
{
  "tenant_id": "tnt_<tenant-UUID>",
  "decision_rule": "majority_of_electorate",
  "allowed": true,
  "approval_threshold": 0.67
}
```

The threshold rides the answer, so a caller never has to ask twice.

### How the tenant is resolved

Through its **active enrollment boundary**, never by tenant id directly. The
boundary is the ceiling (§4.4), so the charter a tenant is bound by is the one
its issuer named — not the newest charter anybody happened to register for that
tenant id.

### It fails closed

| Situation | Answer |
| --- | --- |
| The rule is not one of the seven | refused, naming the rule and listing the seven |
| The tenant has no active boundary | refused — a tenant with no ceiling has no charter |
| The boundary names a digest nothing resolves | refused — *the question cannot be answered* |
| The charter does not allow the rule | refused, naming **only** that rule |

The third row matters in an existing deployment. Boundaries issued before the
charter store existed carry a label rather than a digest (`dev-charter-000`,
`fixture-charter`), and for one of those the honest answer is that the charter is
not readable. A rule is never allowed by default.

The fourth row is deliberate too. A refusal names the rule the caller asked
about and never enumerates the charter's set, because listing it would answer a
question the caller did not ask.

## How a thread uses the charter

A thread declares its `decision_rule` at creation, and the create checks that rule
against this charter. The thread then records the charter's digest, and its close
reports the result the server works out from that rule. See
[Deciding a thread](decision-rules.md). `role_weighted` and `human_committee` can
be allowed here but cannot yet be declared on a thread, because nothing can
evaluate them.

`role_weighted`'s weighting is charter-defined by §13.3 and has no schema here
yet; today the family can be allowed but its weights have no home.
