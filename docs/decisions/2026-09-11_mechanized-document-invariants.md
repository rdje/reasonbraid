---
answers:
  - Why is the public-repository policy enforced by a check rather than a census?
  - May a book page state the current corrective frontier?
  - What makes a documentation rule enforceable rather than advisory?
---
# A document invariant that nothing checks is a suggestion

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.13`; REPAIR-0061.
- Evidence: the leaf's verification bullet in docs/tasks/SIGNOFF-REPAIR.md.
- Follows: docs/decisions/2026-09-09_public-repository-policy.md.

The director's public-repository correction was applied twice by hand and leaked
twice anyway — into the CI guide, then into the book's introduction and the
governance charter. The corrections were not careless; the instrument was. A
census is a one-shot measurement of a moving corpus, and a policy that depends
on someone remembering to re-run it is advisory whatever the prose says.

Every tracked-Markdown sentence that states or instructs a private visibility
for this repository is therefore reviewed mechanically. Legitimate ones exist —
a prohibition, a negation, a historical line quoted inside preserved evidence —
so the rule is not that the words may never appear. It is that each such
sentence is listed verbatim with its reason, and that a new or reworded one
fails a commit. A listed exception that no longer matches also fails, so the
allowlist describes what is actually there instead of decaying into a blanket
exclusion. Broad matching was rejected deliberately: an allowlist thirty entries
long teaches bypass, while a precise one stays reviewable.

The book may not hold a second copy of a fact the task tree owns. Its roadmap
page named a corrective frontier seven committed leaves after that leaf closed,
because nothing compared the copy with the original. The repair is not a
re-synchronization — it is the removal of the duplicate, with the reader routed
to the page that is already updated per leaf. The check enforces the weaker,
sufficient rule: a page may name the frontier only if it names the same leaf as
the tree, and naming none always passes.

This generalizes. Where a document restates a fact some other tracked artifact
owns, the drift is not a question of diligence; it is a question of whether
anything compares them. Prefer deleting the copy. Where the copy must exist,
make a check own the comparison — and prove the check by reintroducing the exact
defect it was written for and watching it fail.

These are documentation and enforcement changes. No repository visibility
setting, remote, production source, qualification category, name-clearance gate
or release claim changes with them.
