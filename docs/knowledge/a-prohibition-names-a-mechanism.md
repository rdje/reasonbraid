answers: a task leaf forbids the approach I think is right — what now; how do I tell a real design prohibition from a loosely worded one; why did obeying a leaf's warning lead me to the wrong answer; should I re-open a decision a leaf already closed; how do I record walking past a prohibition

# A prohibition names a mechanism, and the mechanism may not be the one you are proposing

- **Type:** `knowledge`
- **Date:** `2026-09-17`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.14.3.4`

## The question

A leaf you are executing carries a ⛔ that appears to rule out the answer the
evidence points to. Do you obey it, or do you overrule it?

## The answer

> A prohibition is written about a **named mechanism**. Read which mechanism, and
> check whether it is the one you are proposing. Very often it is not, and the ⛔
> is simply silent about your case rather than opposed to it.

The measured instance. A leaf said:

> ⛔ *Do NOT assume the answer is a citation table … a per-tenant reference set
> would break the dedupe the pair key exists for.*

The table's identity is `UNIQUE (original_locator, expected_digest)`, so two
tenants naming the same pair share one row. A `tenant_id` **column** really does
break that — either the constraint goes or the sharing does. A separate
**many-to-many table** breaks nothing: the row stays shared, and the tenant lives
in the disclosure decision rather than in the key. The two mechanisms differ
structurally, and the sentence is true of one and false of the other.

⭐ The giveaway was in the leaf's own goal line: it named the sibling repair's
migration — the one that IS a separate table — in the same breath as forbidding
the approach.

## The rule

> Before obeying a ⛔, restate the mechanism it names and the mechanism you are
> proposing, and check whether they are the same thing. If they differ, **say so
> in the leaf** — do not quietly proceed.

⛔ Quietly proceeding is the failure that matters. From outside it is
indistinguishable from ignoring the warning, and it leaves the next reader a
prohibition that has already been walked past once, with no record of why.

## The mirror, which is the more likely error

⚠️ A leaf's ⛔ is usually right. Talking yourself past one because the wording is
loose is how a decision already taken gets re-taken badly, and *"the warning
didn't quite mean my case"* is available for almost any case.

What makes an exception safe is that the difference is **structural and
checkable** — here, one mechanism alters a unique constraint and the other does
not. If you cannot state the difference as a fact someone else can verify, the
prohibition wins.

## A corollary about assertions

The same slice produced a smaller version of the same confusion, in a test. A
control asserted that two refusals were **byte-identical**, meaning to prove that
a refusal cannot confirm an identifier exists. It failed — both messages echo the
id the *caller* supplied, which discloses nothing.

> The property was *the answer depends only on the caller's own input*. Equality
> across two different requests is a stronger claim than that, and a stronger
> assertion fails for reasons that are not defects.

## Related

- [[a-compatibility-objection-is-a-claim]] — the same move on the cost side: a
  stated reason to hesitate is checkable, not a settled fact.
- [[where-an-invariant-lives]] — having chosen the mechanism, where it belongs.
- [[one-field-cannot-carry-two-facts]] — the structural test that separated a
  column from a set here is the same one that separates two facts in one column.
