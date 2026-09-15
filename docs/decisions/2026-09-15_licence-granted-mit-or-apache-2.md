---
answers:
  - Under what licence is ReasonBraid released, and where is the grant?
  - Who is the copyright holder?
  - Why was the licence text not simply written from memory?
  - What stops the manifests and the licence files from drifting apart?
  - Does granting the licence clear the public-name blocker?
---
# The licence is granted: `MIT OR Apache-2.0`, with the texts corroborated on disk

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.13.2`
- **Date:** 2026-09-15
- **Supersedes:** nothing. It **completes** a declaration that had stood since the
  first manifest, and closes blocker **B5**.

## Context

Every tracked manifest in this workspace declared `MIT OR Apache-2.0`, and the
repository contained **no licence text at all**. Measured: 13 tracked
`Cargo.toml` files — the workspace root plus 12 crates — of which **5 state the
expression literally and 8 inherit it** with `license.workspace = true`; one
distinct expression tree-wide; zero `license-file` keys.

⛔ **A licence expression in a manifest is metadata, not a grant.** It tells
`cargo` and a package index what the author intends. It conveys nothing to a
reader, because nothing in it is addressed to them and it is not signed by
anyone. Absent `LICENSE-MIT` and `LICENSE-APACHE`, the operative default for this
**public** repository was ordinary copyright: all rights reserved, and a reader
holding none of the rights the expression appeared to offer.

That gap was open for the life of the project: **487 commits passed under every
gate set this project has ever had, and not one of those sets contained a licence
check** — `git log --oneline -- 'scripts/check_licence*'` returns nothing before
this commit, and the current 18-doctrine set has been in force for only 67 of
them. Gates governed documents, code, tables and claims; **none governed the grant**.

## Decision

**`MIT OR Apache-2.0`, dual, at the licensee's option. The copyright holder is
Richard DJE.** `LICENSE-MIT` and `LICENSE-APACHE` now sit at the repository root.

⛔ The **expression is untouched**. The choice was made when the first manifest
was written; narrowing or broadening it here would have been a relicensing act
wearing the clothes of a completion. What was missing was the grant, and only the
grant was added.

⚠️ **One fact genuinely required the director and nothing else did.** The
copyright holder's name is not derivable from the tree — `git log` shows the
committer, which is evidence of authorship and not a statement of ownership, and
inferring one from the other is exactly the kind of guess a legal document must
not contain. Everything else followed mechanically from declarations already in
the tree.

## Why the texts were corroborated rather than written

⭐ **Neither licence was typed from memory, and this is the part of the decision
worth keeping.** A licence is a legal instrument whose operative content is its
exact words. A paraphrase is not the licence; worse, a *plausible* paraphrase is
undetectable, because nothing in a fluent reconstruction signals which clause
drifted. The failure mode is silent and the blast radius is every downstream user.

So both came off disk, from this workspace's own dependency graph:

| File | Source of truth | Corroboration |
| --- | --- | --- |
| `LICENSE-APACHE` | the copy shipped by `arc-swap-1.9.2` | `cmp -s` byte-identical; **104 crates** in the local registry ship that exact sha256; 201 lines, 10,847 bytes |
| `LICENSE-MIT` | the same crate's MIT file, copyright line replaced | body whitespace-normalised-identical to the copy **127 crates** ship; 25 lines, 1,055 bytes |

⚠️ Three distinct Apache-2.0 variants exist in the registry. `diff` shows they
differ **only in indentation**. The 201-line form carrying the APPENDIX is the
complete apache.org document, and that is what shipped.

⛔ The appendix keeps its `Copyright [yyyy] [name of copyright owner]`
placeholder. That placeholder is *part of the canonical document* — it is the
template Apache offers for a source-file header, not a blank for the licensor to
complete. Filling it would modify the licence text.

## What stops this from drifting

`scripts/check_licence_grant.sh`, registered in the project doctrine slot as
**LICENCE-GRANT**. It is two-directional on purpose:

1. every SPDX identifier any manifest declares has its text at the root;
2. every `LICENSE-*` file at the root is **named by the declared expression** — a
   file left behind after an expression changes still reads as an offer;
3. every literal expression in the tree is the *same* expression;
4. each text **is** the licence it claims, by sentinels spanning the whole
   document plus a length floor, and the MIT copyright line names a real holder
   rather than a placeholder.

⭐ It fires on **zero** breaches today and would have fired on every commit before
the one that added it — the shape a gate should have: it catches the next drift
rather than presenting a backlog.

⚠️ **Two defects were found in the gate by falsifying it, and both are recorded
because the second is the instructive one.** (a) Its first sentinels were literal
substrings, and the real MIT text is hard-wrapped at ~55 columns, so
`WITHOUT WARRANTY OF ANY KIND` spans a line break and the gate **failed a
perfectly valid licence**; matching is now whitespace-normalised. (b) All four
original Apache sentinels sat in the **first five lines**, so `head -5
LICENSE-APACHE` passed the gate while granting nothing — the title block of a
licence is not the licence.

⛔ In both cases the **self-test passed while the real run was wrong**, because
its fixtures were tidier than the shipped files. The fixtures are now the real
files, mutated; and the two holes are pinned as named cases that must not be
deleted.

## Consequences

- Readers of this public repository now hold the rights the manifests advertise.
- ⛔ **B4 is unaffected.** A licence grants permissions *in the work*; it says
  nothing about the *name* on it. ADR-001's public-name clearance is a separate
  external gate and `ReasonBraid` remains an uncleared working name.
- Adding a crate with a different licence, or removing a licence file, now fails
  the commit rather than passing unnoticed.
