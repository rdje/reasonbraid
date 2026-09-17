# A reference's `scheme` is a capability selector; its omitted fields take the schema's defaults; its fragment stays in the locator

- Date: 2026-09-17
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.14.3.5`
- Related: `2026-09-16_a-citation-registers-the-reference-it-names.md` (which
  derived the scheme on the other writer and left this one open),
  `2026-09-17_a-snapshot-names-the-locator-its-reference-names.md` (the same
  check-don't-overwrite ruling, one leaf earlier).

Three mechanisms sat in one goal line. Each is measured and decided separately,
because deciding three from two reproductions is the over-reporting
`SIGNOFF-REPAIR.11.15` exists to catch.

## (1) The declared `scheme` — REFUTED, not repaired

**The premise, and where it came from.** The leaf was opened on a test helper's
comment: *"`resource_references.scheme` is caller-supplied and is NOT validated
against the locator, so a control that wrote `https` here would be routed on data
that is simply false"* — beside the observation that §12.2 ranks resolvers on the
field and that `resources::scheme_of` already exists with exactly one caller.

**The repair that followed was written, taken through a RED/GREEN cycle, and then
refused by two of the product's own controls.**

🔴 **Measured instead of inferred.** The consumer is one predicate,
`resolvers::resolve`'s `WHERE schemes @> $1::jsonb` over
`resolver_capabilities`, and **two shipped packs pair a non-URI scheme with an
`https://*` locator pattern**:

| Pack | `schemes` | `locator_patterns` |
| --- | --- | --- |
| `r1-git-fetcher` (`migrations/0026`) | `["git"]` | `["https://*"]` |
| the R3 browser pack (`resolvers.rs`) | `["web+render"]` | `["https://*"]` |

A Git repository and a rendered page are both reached over HTTPS. **The field is
how a caller asks for a CAPABILITY**, and was never a claim about the locator.
The two controls that refused the repair — `the_r1_resolver_resolves_git_…` and
`the_gated_packs_resolve_only_while_the_gate_is_open` — are not about references
at all: they exercise the packs, and the packs are the consumer.

**Decision: no check, and the contract is STATED.** The field's meaning is
recorded at the type and in the manual, an arm of the control pins the
refutation (`git` and `web+render` register for `https://` locators), and the
helper comment that produced the wrong premise is corrected **at its source** —
because a sentence that produced one wrong repair will produce another.

⚠️ **The field is validated against nothing else either, and that is also
deliberate.** §3.7: *"Acceptance of a reference is not a promise that the core can
resolve it… Unsupported references remain durable and can later be resolved or
delegated."* So an unadvertised scheme is `resource_unresolvable_now` at
resolution rather than a refusal at registration, and a control arm pins that
too.

⛔ **A shape check on the scheme token was considered and rejected**, for
`SIGNOFF-REPAIR.11.6`'s standing rule: a rule may not be proposed before its
population is measured, and no census of malformed scheme tokens exists. It would
be a check invented because one was expected, which is how this section went
wrong the first time.

⭐ **The general lesson is promoted**
(`docs/knowledge/a-fields-name-is-not-its-contract.md`): a field's name is not
its contract — its consumer is. And the trap is that the wrong reading *also*
explains the evidence: "a caller-supplied routing field that nothing validates"
describes the defect and the design equally well, so only the consumer separates
them.

## (2) The omitted `visibility_scope` and `risk_class`

**The finding.** `migrations/0023` declares `NOT NULL DEFAULT 'network'` and
`DEFAULT 'low'`; `ResourceReference` declared `#[serde(default)]` on both, which
is `String::default()` — the **empty string** — and `submit` binds the field
explicitly, so the column default never applied. Every reference submitted
without those fields held `''`, and `''` is not a risk class. Meanwhile the
citation path writes the declared values, so **the two writers produced different
rows for the same omission**.

**Decision: the serde default IS the schema's declared default.** Both writers
now produce the same row, and the control asserts that by comparing them.

⚠️ **This is the permissive direction and it is adopted rather than chosen.**
`low` is the value the migration already recorded and the citation path already
writes; nothing here invents a policy. ⛔ §12.2's risk filter does not exist —
`grep` finds no site reading either column for a decision — and when it is built
it owns whether `low` may be a default at all. That is stated at the field.

## (3) The fragment in the locator

**The finding.** §12.1 lists `fragment_or_selector` beside `original_locator`,
which implies the locator excludes the fragment. Both writers store the locator
verbatim, so `…/page#a`, `…/page#b` and `…/page` are three references over
(possibly) identical bytes.

**Decision: leave it, and publish it.** ⛔ Splitting the fragment out **is**
canonicalization, and §12.1 says canonicalization is scheme-specific and "must
not erase security-relevant distinctions" — merging three locators onto one row
is exactly such an erasure. The conservative choice costs a shared row; the other
choice costs a distinction, which is the harder one to get back.

⭐ This is the same boundary `.11.14.3.13` drew from the other side: a **strict**
comparison normalises nothing and is not a canonicalization decision, while
**splitting** a locator is one. The two leaves agree, and the line between them
is stated rather than felt.

The control's third arm asserts the behaviour — three spellings, three
references — so the decision is pinned rather than left to a reader's inference.

## What it does NOT close

- ⚠️ `retention_class` and `purpose` are `Option<String>` and stay nullable; the
  migration declares no default for them, so there is nothing to adopt.
- ⚠️ The throwaway `ResourceReference` values in `fetcher.rs` and `snapshots.rs`
  carry `"tenant"`/`"standard"` — values that match neither writer. They are
  constructed only to reuse `digest_error()` and are never stored, which the
  census confirms; they are left alone rather than harmonised, because changing
  them would imply they mean something.

## Verification

- ⛔ **(1)** RED and GREEN were both reached for the repair that is now refuted —
  the `ftp://` locator declared as `https` registered (`200`), and the check
  refused it — and then `the_r1_resolver_resolves_git_…` and
  `the_gated_packs_resolve_only_while_the_gate_is_open` refused the check. The
  standing arm asserts the refutation instead: `git` and `web+render` register
  for `https://` locators, and an unadvertised scheme stays submitted per §3.7.
- **(2)** An omitted pair stores `network`/`low`, and the citation path's row
  **matches it** — asserted by comparing the two writers rather than a literal
  twice.
- **(3)** Three fragment spellings remain three references.
- ⭐ **What caught (1) is worth recording: the affected suites were run broadly
  rather than only this leaf's own control.** The narrow run was green.
