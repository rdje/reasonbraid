answers: does Option make a serde field required; why did my missing JSON field decode instead of erroring; how do I require a field that may be null; what is the difference between a missing member and a null member; how do I prove a stored record was written by a build that understood it?

# A missing field is not an absent value

- **Type:** `knowledge`
- **Date:** `2026-09-12`
- **Owner / source:** leaf `SIGNOFF-REPAIR.3.3.4.7.1`

## The question

A stored record has an optional member. A writer drops it — a truncated write, a
partially-applied edit, an older build that never learned about it. What does the
reader conclude, and is that what you meant?

## The answer

**Two different facts arrive through the same door, and by default the language
merges them.** "The writer said there is no value" and "the writer said nothing"
are not the same claim, and for evidence they are very far apart: one is a
record of an absence, the other is a record that is incomplete.

### The specific trap: `Option<T>` is not a required field

In serde, a struct field with no `#[serde(default)]` is required — *except* that
the generated missing-field path builds a deserializer which answers
`visit_none`, so an `Option<T>` succeeds as `None`. This is not documented at the
field; it is behaviour of the derive. The result is that

```rust
submitted_reason: Option<AdministrativeReason>,
```

decodes a JSON object with **no** `submitted_reason` member into "no reason was
submitted" — silently, and with the same value an explicit `null` produces.

The fix is to give the field a `deserialize_with`, which has no missing-field
fallback:

```rust
fn explicit_reason<'de, D: Deserializer<'de>>(d: D) -> Result<Option<R>, D::Error> {
    Option::deserialize(d)
}

#[serde(deserialize_with = "explicit_reason")]
submitted_reason: Option<R>,
```

Now the member must be **present**, and `null` is how a writer says the value is
absent. The type is unchanged; only the requirement moved.

### When to require presence, and when not to

The deciding question is whether the record has HISTORY.

- **No history** — a new table, a new message, a record nothing has written yet.
  Require presence. There is no rolling writer that legitimately omits the
  member, so a missing one means the row is incomplete, and saying so costs
  nothing.
- **History** — an existing corpus written before the field existed. Then a
  missing member genuinely means "this writer predates the field", and that
  deserves its own named value rather than either an error or a guess. This
  project already has the shape: historical authorization records decode their
  absent provenance as `legacy_unspecified`, which is neither "ordinary
  evaluation" nor a failure.

Both answers are honest. The one to avoid is the default, where a missing
member quietly becomes the same value as a stated absence and nothing can tell
the two apart afterwards.

### The same principle, one level up

A closed enum decoded from storage faces the identical choice about values it
does not recognise. A record naming an outcome or a reason code outside this
build's registry can be treated three ways: guessed at, mapped to a catch-all, or
refused. For **evidence**, refuse — the build that wrote it knew something this
one does not, and a guess would put that gap inside a record an operator trusts.
For a **wire protocol** with independent implementations, preserve the unknown
verbatim instead, which is why `ReasonCode` has an `Unknown(String)` arm while
the stored administrative outcome does not.

## How this was found

The leaf's own control found it. The doc comment claimed the field was explicit,
the control asserted that a record without the member fails to decode, and the
first draft **passed** the decode. The claim was written before the behaviour was
checked; running the negative case is what separated them.

## Related

- `docs/knowledge/one-field-cannot-carry-two-facts.md` — the same disease at the
  value level: one field written from two independent facts.
- `docs/knowledge/proving-a-race-is-closed.md` — the general form of "assert the
  property, then watch the assertion fail against the build that lacks it".
