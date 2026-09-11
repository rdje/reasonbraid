---
answers:
  - What makes a file an extraction request's own input?
  - When may an extraction input be deleted?
  - Where do the server's private runtime files live, and how is that decided?
---
# A name proposes an input; exclusive creation proves one

- Owner: `SIGNOFF-REPAIR.7.3.3.3.1`; REPAIR-0063.
- Evidence: docs/tasks/artifacts/signoff_review/extraction-owned-input.md.
- Follows: docs/decisions/2026-09-10_extraction-input-boundary.md and
  docs/decisions/2026-09-11_extraction-worker-completion.md.

A path built from values that repeat — a process id, a nanosecond field — is a
guess about uniqueness, and a write that truncates whatever is already there
turns that guess into another caller's loss. Ownership of an extraction input
comes from creating it exclusively, with the operating system refusing the
second creator, and from never adopting a name something else already holds. An
occupied candidate is skipped whole: not opened, not truncated, not inspected.

The input lives on the repository's own volume, under a private store below a
root discovered at runtime from the current directory. Nothing persists an
absolute path, so moving the checkout moves the store. There is no temporary
directory and no home fallback to fall back to — an unusable location is a
refusal, not a quieter place to write. Every parent is proved to be a directory
this process owns, on the expected device, not writable by group or other; the
reference for "this process" is the uid of the file just created, since a new
file carries its creator's effective uid.

Deletion needs two independent facts, and a successful response is neither. No
reader may still hold the input — either no worker was ever started or the
spawner observed its exit — and the file must still be the exact one that was
created, same device, same inode, one link. A file that fails either test is
retained with its repository-relative path named. An input whose fate is unknown
is evidence; deleting it destroys the only record of what a request actually
supplied. The store removes one file it created, or nothing: there is no
recursive deletion anywhere in it.

A runtime root predicate must be specific enough to identify the real root.
`Cargo.toml` plus a `migrations` directory is satisfied by a crate in this
workspace, so a process whose working directory sat there would place private
storage inside that crate. A file that exists only at the root — here
`rust-toolchain.toml` — closes it.

This decision covers the owner. It does not by itself change any production
caller: the R2 API keeps its superseded span until `SIGNOFF-REPAIR.7.3.3.3.2`
wires it, binds the response's parent digest to the owned input's digest before
anything is persisted, and gates cleanup on the reader being finished. Pipe
pressure, total deadlines, descendant containment and aggregate retained storage
remain `SIGNOFF-REPAIR.7.3.4`.
