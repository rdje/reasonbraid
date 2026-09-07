# Phase 2.6's adapter-conformance deferrals: five §19.4 items have no machinery to build against yet (`PHASE-2.6.3`)

- Date: 2026-09-07 · Leaf: `PHASE-2.6.3` · Decision record

## Context

§19.4's adapter conformance kit lists ten items. The `.6` census found the
dev profile already carries five of them in machinery (the capability
declaration + unsupported-operation behavior, the honest ambiguous-outcome
reporting, the usage accounting with confidence, the cancellation/streaming
semantics, the sanitized fixture corpus) — `.6.1` harnessed them and `.6.2`
pinned the corpus. Five items remain: they have either no machinery at all
or nothing to bind against in the dev profile (no policy projection surface,
no tool-capable adapter, no provider rate-limit signal ever observed).
Building placeholder machinery for absent surfaces would violate the
subtraction doctrine (§19.8) — the honest act is to name each deferral with
the exact trigger that re-opens it.

## Decision

- **Rate-limit/backoff normalization** — no adapter parses a provider
  rate-limit signal (no real 429-class response has been observed in the
  qualified CLI harnesses), and the node's retry policy (§14.6, the `.2.3`
  `retry_decision`) is decision-shaped without backoff timing. Trigger: the
  first provider response that carries a parseable rate-limit signal.
- **Output-size limits** — no bound on streamed output exists at the adapter
  boundary (chunks stream verbatim). Trigger: the first policy-set output
  limit (a named cap in the reservation/budget dimensions or a policy
  projection), or the first streamed run that crosses a limit the policy
  wants enforced.
- **Tool-call validation** — the `tool_support` capability flag exists; no
  validator does (no tool-capable adapter is qualified). Trigger: the first
  tool-capable adapter's qualification.
- **Provider error taxonomy** — the adapters map exit status + stderr tails
  into the contract's honest outcomes (`failed_known` with the tail, the
  definitive `is_error` result); there are no typed per-provider reason
  codes. Trigger: the first provider-specific error that a caller needs to
  distinguish by code rather than by text.
- **Prompt/policy projection fidelity** — no policy projection surface
  exists (the `policy_injection` capability is `None` everywhere; the
  projection machinery is Phase 6's territory). Trigger: the first policy
  projection.

## answers:

- **A deferral must name its trigger, not just disappear** — each item
  above names the exact observed surface whose arrival re-opens it, in the
  subtraction doctrine's shape.
- **The conformance kit's checkable half is ALREADY mechanical** — the
  `.6.1` harness + the `.6.2` pinned corpus prove five of the ten items on
  every test run; the deferrals are the half that needs an absent surface,
  not the half that needs discipline.
- **The qualification checklist is the manual half, deliberately so** —
  the §19.4 last item (the manual qualification checklist) is a HUMAN gate
  over the env-gated live runs; it lives in the book's adapter-boundary
  chapter where an operator qualifies an adapter, one artifact.
