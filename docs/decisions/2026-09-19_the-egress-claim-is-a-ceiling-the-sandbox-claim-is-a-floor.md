# The egress claim is a ceiling, the sandbox claim is a floor, and one test cannot serve both

- **Type:** `decision`
- **Date:** `2026-09-19`
- **Status:** `active`
- **Owner / source:** `SIGNOFF-REPAIR.7.3.6.4` (REASONBRAID-REPAIR-0269) — measured

## The fact / decision

`resolvers::resolve` filters the two ADR-018 isolation classes in **opposite
directions**:

- `sandbox_level`: `declared >= required`. The ladder
  `none < process < constrained_process < vm_container` goes up towards more
  isolation, so the requirement is a **floor**.
- `egress_class`: `declared <= required`. The ladder
  `none < loopback < listed < any` goes up towards more **reach**, and a pack's
  declared class is its maximum, so the requirement is a **ceiling**.

`POST /v1/resources/{id}/resolve`'s `required_egress` default becomes `any`
(no bound requested), which is the honest spelling of the behaviour that
already shipped. A required class outside the ADR-018 vocabulary is refused by
name with `invalid_command`, after the tenant binding.

**ADR-018 did not move. The code did.**

## Why

ADR-018 states the egress class as *the allowed destinations … the claim is the
**MAXIMUM**, never the minimum*. Both classes were filtered with the same test,
`declared >= required`, and the comment above it quoted that rule and drew the
opposite conclusion from it in the same sentence:

> the resolver's declared classes must MEET the required ones (the ADR-018
> ladder order — the claim is the maximum, so a resolver claiming LESS than
> required is ineligible)

Under a maximum claim, a resolver claiming *less* is claiming to be **more**
constrained. That is safer, so it should be more eligible, not less. The rule is
correct for the sandbox ladder and inverted for the egress one, and the root
cause is one test written for two ladders that run in opposite safety
directions.

**Measured over the whole 4 × 6 matrix before the repair**: the egress filter
refused exactly **one** combination of twenty-four, and it was the wrong one.
A caller requiring `listed` was served `rx-agent-mediated`, which declares
`any`; asking for `any` — the widest class — was the only way to narrow the
field, and it narrowed it to the single pack that declares no bound at all.
**No value of `required_egress` meant *do not give me a pack that can dial
anywhere*,** which is the one thing ADR-018 says the class is for.

### Why the code moved and not the ADR

The shipped default `required_egress: "loopback"` only makes sense under the
capability reading — under the ceiling reading no pack would ever be eligible,
since every pack declares `listed` or `any`. So the code was internally
consistent, and the repair is a decision rather than an obvious inversion.

It moved because a capability floor over a maximum claim is incoherent. *"I
need a pack that promises to reach at least this far"* asks a promise-not-to-
exceed to behave like a promise-to-reach. The classes exist so that a reference
can require isolation and get the explicit failure rather than a silent
downgrade (ADR-018's own exit clause); a floor on egress cannot express any
isolation requirement at all.

## How to apply

- **A ladder's comparison direction is a property of the ladder, not of the
  filter.** When adding a third ADR-018 class, ask which end is safer before
  reusing either test. Two ladders in one loop with one comparison is the
  defect this record exists for.
- **`required_egress` is a ceiling.** A caller that wants a bound states the
  widest class it will accept. `any` requests no bound; `none` admits only a
  pack that promises no egress.
- **A requirement no pack meets is `unresolvable_now`**, never a downgrade to a
  wider pack. A requirement outside the vocabulary is `invalid_command`,
  validated **after** the tenant binding so a foreign or absent resource id
  keeps giving one answer (`.11.14.3.4`'s existence oracle).
- **The default's permissiveness is not settled here.** `required_egress`
  defaults to `any` and `required_sandbox` to `process`, which are different
  strictnesses; tightening either changes what every existing caller receives
  and is its own decision.
- Re-derive the matrix with a caller that registers two resolvers differing only
  in `egress_class` — `an_egress_bound_excludes_a_pack_that_declares_a_wider_one`
  in `crates/reasonbraid-server/tests/profiles.rs` is that control.

⚠️ **Open, and owned elsewhere:** `rx-agent-mediated` declares `egress_class:
"any"` while performing no egress at all — it emits a capability call and
returns. `[[2026-09-19_an-advertised-line-carries-an-adjudicated-verdict]]`
grades that line `misdescribed`, and `SIGNOFF-REPAIR.7.3.6.5` owns it. If that
advertisement is corrected, RX stops being the pack a caller must permit `any`
to reach — for a different reason than this record's.
