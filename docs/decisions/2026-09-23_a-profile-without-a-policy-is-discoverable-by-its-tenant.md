---
answers:
  - What visibility does a profile written without a `visibility` policy get?
  - Why is the default not `self_only` for every field, as a doc comment once said?
  - Which reader receives `incarnation_id` and the policy itself?
---
# A profile without a policy is discoverable by its tenant

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.5.1.5`
- **Date:** 2026-09-23
- **Work unit:** `REASONBRAID-REPAIR-0445`
- **Cites:** ROADMAP §10.1 (*visibility policy for each profile field*), §10.2 (*expose only visibility-filtered directory information*), the Phase-3 exit gate (*recruit appropriate available participants without enumerating the entire network*); `docs/decisions/2026-09-23_the-directory-isolation-goal-line-three-items-met-three-live-defects.md` (DOC-0149, which attached the three clauses).

## The fact / decision

1. **The shipped tiered default is the default.** A profile written without a
   policy shows `display_label`, `purpose`, `interests`, `languages` to the
   network; `conversation_modes`, `capabilities`, `structured_output_formats`,
   `scopes`, `availability`, `resolver_tool_capabilities`, `cost_latency_class`
   to its tenant; `confidentiality_classes`, `resource_ceilings`,
   `grants_by_reference` to nobody but the full reader. The doc comment that
   said *every named field defaults to `self_only`* was wrong and is corrected;
   the book always documented the tiered default.
2. **`field_visible`'s doc example is corrected** — a `tenant` field reaches a
   `Tenant` reader and not a `Network` one. The code was right.
3. **The full reader gets the whole profile.** `filter_profile` emits
   `incarnation_id` and `visibility` to `ReaderClass::Full` only.

## Why

The clause asked for the discovery consequence to be MEASURED, not assumed.

- **The suite does not measure it.** With the default rewritten to
  `self_only` for all fourteen fields, `bash scripts/run_pg_tests.sh profiles`
  passed **86 / 86** (`target/s515/narrow.log`, 2026-09-23): every control that
  reads a profile as a non-owner writes an explicit policy. No census of the
  tests could have decided this.
- **The code decides it.** `matching::eligible` refuses a candidate whose
  filtered profile is empty (*the profile exposes nothing at the requested
  scope*), and a capability requirement reads the capability at the
  expression's scope. Under an all-`self_only` default a policy-less role is
  empty at the tenant and network views, so every match and call not run by
  its owner at the full scope passes it over. The new control
  `a_profile_without_a_policy_takes_the_default_and_the_full_reader_sees_all_of_it`
  is the measurement going forward: a tenant-mate's match must find the
  policy-less role. **Observed:** with the all-`self_only` default applied to
  the repaired code, that match answered `{"candidates":[],"scope":"tenant"}`
  (`target/s515/mutant_narrow2.log`, `FAILED. 86 passed; 1 failed`) — the
  role is invisible to its own tenant.
- **Who writes a policy-less profile.** No production writer builds a profile
  (`git grep -l display_label -- ':!*.md'` → the server's own profile module,
  card import, and tests); the book's own first example (*Declaring a profile*)
  writes none, so a reader following the book gets this default.
- **The narrow default buys little.** The fields it would additionally hide
  from the tenant are the ones recruitment reads, and every one of them can be
  narrowed per field by writing a policy. Confidentiality classes, ceilings and
  grants — what compounds when disclosed — are already `self_only`.

## Consequences

- A role that wants to be undiscoverable must write a policy; the book says so.
- The full read gains two keys. The top-level `visibility` string (the class
  served) and `profile.visibility` (the policy) are different things; the book
  names both.
- Card export, which carries the full profile, is unaffected: it never went
  through `filter_profile`.
