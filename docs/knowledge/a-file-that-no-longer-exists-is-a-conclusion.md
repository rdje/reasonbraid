answers: my docs cite a file that is not in the repo — what happened to it; how do I find out whether a file was renamed or deleted; a reference does not resolve, what are the possible causes; how should I cite a dependency's source code; my citation into a library broke after an upgrade

# "A file that no longer exists" is a CONCLUSION, not an observation

- **Type:** `knowledge`
- **Date:** `2026-09-18`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.17.1`

## The question

Prose cites a source file by name and line number. No file of that name is in
the tree. The obvious reading is *"a file that was renamed or deleted"*. Is it?

## The answer

> Ask git before you conclude. The file's absence **today** cannot tell you
> whether it was ever here — and that difference changes the repair.

```bash
git log --all --diff-filter=A --name-only -- '*something.rs'
```

An empty result means the file was **never tracked, under any name, ever**.

⭐ "Unresolvable" has at least four causes, and they take different repairs:

| cause | how to tell | repair |
| --- | --- | --- |
| renamed | `git log --follow` / the add-commit shows the old path | update the path |
| deleted | an add commit exists, a delete commit exists | cite the commit it was exact at, or drop the line |
| a typo | no add commit, and no plausible neighbour | correct the name |
| **never ours** — a dependency's source | no add commit, but the crate is in the lock file | qualify with crate and version |

## The measured instance

A leaf recorded eight unresolvable citations as naming *"a file that no longer
exists"*, and opened a task to decide whether to correct them or annotate them
with the commit they were exact at.

🔴 **Nobody had asked git.** The file had never been tracked. It was
`gix-0.87.1/src/config/cache/init.rs` — a dependency's source, cited to explain
how that library loads configuration — and every one of the seven cited lines
resolved exactly, to the code the prose quoted, at the version the lock file pins.

⛔ The wrong premise was expensive in the way wrong premises are: it made the
obvious repair *"annotate each citation with the commit it was exact at"*, which
is the **wrong axis entirely**. The drift in a dependency citation comes from the
DEPENDENCY's version, not from this repository's history. The right repair is
invisible while you believe the file was deleted.

## Citing a dependency's source

> Write it **crate-and-version qualified**: `gix-0.87.1/src/config/cache/init.rs:229`.

⭐ That resolves for a reader *and* dates itself, which a bare basename never
could. A line number into a dependency moves on every upgrade, and the version is
the only thing in the citation that says which source it was exact against.

⚠️ Rejected alternatives, because each looks reasonable: annotating with this
repo's commit (wrong axis, above); accepting it as a property of dated records (a
path added to a record does not change what the record *says*, only how precisely
it says it); and exempting third-party references from the resolvability gate (an
exemption list to maintain and argue about, which also hides the genuinely broken
references among the legitimate ones).

## And check your key twice

🔎 The same leaf's census was too narrow a second time. It counted **8** bare
`init.rs` citations; there were **17**, because nine more were written
*partially* pathed — `src/init.rs`, `src/lib.rs`, `src/open/permissions.rs` — in
the same tables, and the key only matched the bare form. See
[[a-census-is-as-wide-as-its-key]].

## Related

- [[a-census-is-as-wide-as-its-key]] — the same leaf, the same day, twice.
- [[a-key-too-loose-returns-the-wrong-instance]] — the opposite failure of the
  same probe-design question.
- [[a-schema-object-is-its-latest-migration]] — the sibling idea: an artifact's
  identity is a function of history, not of the current tree.
