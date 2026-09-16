answers: my negative control passes — how do I know it refused for the reason I think; why did a test that asserts 401 tell me nothing about the gate; how should I write an unauthorized/forbidden control; what happens to my security control when the repair makes the attack unexpressible; how do I stop a fixture literal from silently being the wrong shape

# A refusal must be attributed, not just observed

- **Type:** `knowledge`
- **Date:** `2026-09-16`
- **Owner / source:** leaf `SIGNOFF-REPAIR.7.4.3`, which found the defect in `SIGNOFF-REPAIR.11.14.1`'s control one commit after it shipped

## The question

A control presents a principal that should not be allowed through, and asserts
that the response is a refusal. It passes. What has actually been proved?

## The answer

> A status code proves that *something* refused. It does not name **which
> gate**. A negative control that does not assert the refusal's identity is
> compatible with the gate under test never having run.

`SIGNOFF-REPAIR.11.14.1` shipped this leg, and it was green:

```rust
let (status, body) = get(&client, &base, "/v1/snapshots/stale",
                         "hum_00000000000000000000000000000000").await;
assert_eq!(status, 401, "an unenrolled principal reads no staleness: {body}");
```

The principal shapes in this system are `hpr_…` and `rol_…`. `hum_…` is not one,
so the header parser refused it before any authorization ran. The leg measured
`resolve_principal`. The gate it named answers **403** `unauthorized`, and the
`401` in the assertion was the parser's `unauthenticated`. The wrong number then
travelled into the book, in a table of what each caller receives.

Three things follow, and each is cheap.

**Assert the `code`, not only the status.** A status is a class; the code is the
gate. `assert_eq!(body["code"], json!("unauthorized"))` would have failed on the
day the leg was written, because the parser answers `unauthenticated`. Where a
refusal carries an audit id, assert that it is present too — a denial that
records nothing is a different defect wearing the same status.

**A fixture identifier must be the right shape and wrong in exactly one way.**
The stranger has to be well-formed, or it never reaches what is being tested.
Give it a name and a sentence, so the next control inherits the reasoning rather
than the literal:

```rust
/// A principal of the right SHAPE that no enrolment ever mints.
const STRANGER: &str = "hpr_00000000-0000-7000-8000-00000000dead";
```

**A repair that makes the attack unexpressible retires its own control.** When
the same leaf removed the exploited field, the original payload stopped reaching
the gate at all — it became a body-validation `400`, and the assertion that had
read `403` was rewritten to `400` without anyone noticing that the authority gate
had lost its coverage. The control has to be *re-pointed*: keep the leg proving
the attack is unexpressible, and add a second, well-formed leg that still
exercises the gate. Otherwise the commit that adds a gate is the commit that
stops testing it.

## Where it applies

Any control whose subject is a refusal: authorization, authentication, admission,
schema validation, rate limits. It is the negative-control counterpart of
[[a-falsification-you-can-leave-behind]] — that note is about making a passing
claim falsifiable; this one is about making a failing one attributable. See also
[[a-repair-owns-every-sentence-that-states-its-limit]]: the 401 here was not only
in a test, and a number that is wrong in a control is usually wrong in the prose
written from it.
