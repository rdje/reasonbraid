answers: why did my function panic when it returns Result; how do I decide whether a parse failure should abort or return an error; where do trust boundaries actually live in my code; why did the error handling at the call site never run; how do I tell which copy of a duplicated helper is the dangerous one; what does it mean that only one copy of a function was written safely

# A signature is a promise the body must keep

- **Type:** `knowledge`
- **Date:** `2026-09-13`
- **Owner / source:** leaves `SIGNOFF-REPAIR.4.1.2.1`, `.4.2.7`

## The question

A function returns `Result` (or `Option`, or any fallible type) and its callers
handle the error properly. What has that established about what happens on bad
input?

## The answer

**Nothing, on its own.** A fallible return type is a claim the body has to
honour. When the body can abort instead — an index that panics, an arithmetic
overflow, an unwrap, a slice that is not on a character boundary — the type
says one thing and the code does another, and the caller's error handling is
dead code on exactly the input it was written for.

Two instances in this repository, found a week apart and in opposite directions
across the same trust boundary:

- A token-issuance route took `ttl_seconds` as an `i64` off the wire and passed
  it to `TimeDelta::seconds`. `i64::MAX` panicked and dropped the connection,
  **in front of the authority guard** — so an unauthorized caller could do it.
  The route's typed refusals existed; the panic happened before any of them.
- A node's hex decoder was `fn from_hex(&str) -> Result<Vec<u8>, String>` and
  indexed `&s[i..i + 2]`. `str` indexes by BYTE, and Rust panics on a slice that
  does not land on a UTF-8 character boundary. The call site read
  `from_hex(&parsed.cert_der).map_err(ChannelError::Malformed)?` — the typed
  refusal was written one line away from the abort.

## The test that separates them

Ask of every fallible function, at the boundary where untrusted or
partially-trusted data enters: **is there an input for which this returns
neither `Ok` nor `Err`?** Panics, aborts and infinite loops are all that third
answer. If one exists, the signature is wrong or the body is.

⚠️ The condition is usually NARROWER than the obvious description, and getting
it exact is what makes the control honest. "Non-ASCII input panics" was wrong:
`"éé"` decoded to a clean `Err` because the character happened to start at an
even offset. The real condition is an EVEN byte length — which clears the
length guard — whose chunk boundary falls INSIDE a character. A control that
asserts the loose description passes for the wrong reason and stops testing the
case it names. Assert the structural facts about the input (`len() == 4`,
`!is_char_boundary(2)`) before exercising the function.

## Severity is the trust boundary, and it is stated, not inflated

The same defect is a remote pre-auth denial of service in one direction and a
trust question in the other. Name which:

- data from an **unauthenticated caller** reaching a panic before the authority
  check is the severe case;
- data from **the control plane's own response** reaching one is a question
  about how much a client trusts its server — real, because the cost is the
  whole process rather than one request, and bounded, because it needs a
  malicious or faulty server rather than a network attacker.

State the bound in the same sentence as the finding. A severity claimed without
its reachability is the thing a reviewer has to go and check.

## ⭐ In a duplicated helper, the safe copy tells you where the attention went

Seven hand-rolled hex decoders existed in one repository. **Six** used the
panicking byte-index form. The **one** written safely, over
`as_bytes().chunks(2)`, was the one decoding a field from an unauthenticated
caller.

That is not a coincidence and it generalises: the copy on the path someone was
worried about got the careful treatment, and every copy nobody was worried
about kept the naive form. So when a census finds N copies of a helper and one
of them differs, **the odd one out is usually the correct one**, and the
difference is a map of where review attention has actually been spent.

The practical consequence: `git grep` for the shape, not for the name. The name
is the same in all seven; the indexing style is what separates them.

## Deleting beats repairing, when the copy is dead

One of the copies was `pub` and had no caller. Repairing it would have shipped
a corrected path no control can reach; deleting it removes the failure mode.

⛔ Prove "no caller" with an instrument that is not the one that found it. A
`git grep` returning nothing is a claim about a search pattern. Making the item
private and rebuilding turns it into `error: function ... is never used` under
`-D warnings` — the compiler, which has no loyalty to the grep. Then the build
across every target, tests included, is the gate that the deletion is safe.

Related: `docs/CLAIM_VERIFICATION.md` leg 2 (prefer an oracle you did not
build); `TOOLBOX.md`, "Check an instrument's first number against one obtained
a different way".
