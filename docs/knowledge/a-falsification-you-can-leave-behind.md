answers: how do I falsify a repair whose predicate I cannot safely remove; what do I do when the harness refuses to let me weaken a security check; how do I prove a control is driven by the thing it claims to test; what is better than neutralizing the fix and re-running the suite; why does my one-off falsification prove nothing to the next reader; how do I falsify a repair that changed a type signature

# A falsification you can leave behind

- **Type:** `knowledge`
- **Date:** `2026-09-15`
- **Owner / source:** leaf `SIGNOFF-REPAIR.9.2.1.1`, REPAIR-0190 (which needed it); it extends `docs/CLAIM_VERIFICATION.md` §3 leg 2, which says to make a control go red and does not say how to do it durably

## The question

A repair lands and its control passes. The standard next move here is to
**neutralize** the fix — remove the predicate, re-run, watch exactly the right
legs go red, restore — and it is a good move: it separates "the control is
driven by the repair" from "the control passes for some other reason".

But it fails in three situations, and two of them are common:

1. The repair **changed a type signature**, so reverting the file does not
   compile and the control cannot run at all.
2. The predicate is a **security check**, and the tooling, the review process,
   or a harness classifier refuses the edit that removes it — correctly, because
   that edit is indistinguishable from the attack it guards against.
3. Nobody is watching. The neutralization is a transcript entry. It proves the
   claim **once**, to whoever was present, and the next reader has prose.

## The answer

> Falsify with a **matched pair inside the control**: the same request, one
> configuration knob moved, and the legs flip.

Instead of removing the predicate and showing the legs go red, find the *input*
that makes the predicate legitimately answer the other way, and assert both
answers side by side. If the legs were passing for any reason other than the
predicate, the second half of the pair stays refused and the control fails.

The pair has to differ in **exactly one** thing, and that thing has to be the
predicate's own input. A pair that differs in two is an illustration again.

## The instance

`SIGNOFF-REPAIR.9.2.1.1` bound a publish verb to a server-configured repository
root: the caller names a location, the server refuses anything that resolves
outside the root. Three legs asserted the refusal — a `..` walk, a symlink
planted inside the root, an absolute path elsewhere.

Neutralizing the containment test would have been the usual proof. The harness
refused the edit, on its face correctly: deleting a path-containment check is
exactly what an attacker would want written.

The matched pair proves more, and keeps proving it:

> The **same three locations**, against a server whose declared root is the
> directory **above**, are ACCEPTED — and reach the record exactly as a
> legitimate location does.

One knob moves: which root the deployment declares. Nothing about the request,
the fixture or the code changes. If those three refusals had come from a
malformed path, a missing publication, an authorization check or a typo in the
test, widening the root would leave them refused. They flip, so the refusal was
containment's verdict about *that root* and nothing else.

## Why it is strictly better, not merely available

- **It is durable.** `docs/CLAIM_VERIFICATION.md` leg 3 asks whether the reader
  can re-run the check. A neutralization cannot be re-run by anyone who was not
  there; the pair runs on every CI invocation for ever.
- **It survives the repair being edited.** A future change that quietly breaks
  containment turns the pair red from *both* directions.
- **It cannot be faked by a "refuse everything" repair**, which is the failure
  mode a negative-only control is blind to — and which a neutralization catches
  only indirectly.
- **It reads as documentation.** The pair states the contract: this location is
  outside *this* root, and inside *that* one.

## The honest limit of the instance above

The refusal was observed **once**, in one session, by one harness. It is not
re-derivable by a later reader, and it was deliberately **not** retested with
different phrasing — working out whether a denial can be talked around is not a
thing to spend a session on, and the answer would not change the rule.

⭐ That limit is also the argument. The rule does not rest on *why* the
neutralization was unavailable; it rests on the matched pair being the better
control once you have written one. The type-changing case (1) reaches the same
place with no harness involved at all.

## When it does not apply

Some repairs have no such knob — the fix is unconditional, and there is no input
for which the old behaviour is correct. A transaction boundary, a decoder that
must not panic, an ordering constraint: for those, neutralization remains the
instrument, and `MEMORY.md`'s rules still hold (falsify each part separately,
prove the neutralization landed by census, read `rc` directly).

⛔ The test is not "is there a knob" but "is the knob the predicate's own input".
Turning off a feature flag that skips the whole code path proves nothing: the
legs go green because the code never ran, which is the second hypothesis, not the
first.

## Related

- [[a-control-is-calibrated-against-the-renderer]] — the other half of the same
  worry: a control written from the same reading as the code cannot disagree
  with it. This one is about a control that cannot be *shown* to disagree.
- [[proving-a-race-is-closed]] — the same instinct applied to ordering, where the
  discriminating input is a lock holder rather than a configuration value.
