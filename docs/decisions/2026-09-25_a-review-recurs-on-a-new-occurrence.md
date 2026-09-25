---
answers:
  - When does a policy publication get a new review, and when does a schedule add nothing?
  - What counts as a repeated waiver, and over what window?
  - Why does a review have its own id instead of one built from the publication and trigger?
---
# A review recurs on a new occurrence, and a repeated waiver is two in force within 90 days

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.9.3.2`
- **Date:** 2026-09-25
- **Work unit:** `REASONBRAID-REPAIR-0523`
- **Cites:** ROADMAP §15.11 (*review triggers include … repeated waiver, drift …*);
  `docs/book/src/policy-lifecycle.md` (*Reviews*)

## The fact / decision

1. **Occurrence rule.** A `(publication, trigger)` gets a new review when the
   records name an occurrence of that trigger recorded after the pair's latest
   review, and no review of the pair is `due`. An occurrence is an outcome row
   naming the trigger, a drift row (`drift`), or the repeated-waiver condition
   becoming true (`repeated_waiver`, dated by its latest waiver). Times are the
   database's.
2. **One due review per pair**, as the partial unique index
   `policy_reviews_one_due` (`migrations/0113`); the scheduler's insert names it
   in `ON CONFLICT … DO NOTHING`. An occurrence arriving while a review is due
   folds into that review.
3. **A review has its own id** (`rev_<uuid v7>`), not `rev_{publication}_{trigger}`.
4. **A repeated waiver** is at least `REPEATED_WAIVER_THRESHOLD = 2` waivers of
   one publication in force (`expires_at` absent or in the future) and recorded
   within `REPEATED_WAIVER_WINDOW_DAYS = 90` days.
5. **An insert failure is an error**, returned to the caller as `500`.

## Why

- **The deterministic id made the lifecycle one-shot.** It was the primary key,
  so after the first review of a pair was done, every later insert collided, and
  `is_ok()` discarded the collision. §15.11 describes review as recurring.
- **The partial index is the rule the id stood in for.** "One due review per
  pair" was enforced by a read before the insert, which two concurrent schedules
  can both pass; an index cannot be raced.
- **Two, not one, and in force.** One waiver is the mechanism working as designed
  (a bounded exception); §15.11's trigger is the *repeated* waiver. A lapsed waiver
  is no longer an exception anyone relies on. 90 days is a quarter: long enough
  that two related exceptions granted weeks apart are seen together, short
  enough that an old, forgotten one does not revive the trigger. It is a named
  constant, stated in the book, so changing it is a visible decision.
- **"Nothing was due" and "the store failed" must not share an answer.**

## How to apply

- A new trigger source names its occurrence time; the scheduler compares it with
  the pair's latest review.
- Tests set waiver expiries RELATIVE to now; a hard-coded date expires under them.
