answers: how do I prove a path still names the file or directory I created; why did (dev, ino) identity pass on macOS and fail on Linux; does an open descriptor protect identity; can a link-count check be reused for a directory?

# Proving a path still names what you created

- **Type:** `knowledge`
- **Date:** `2026-09-11`
- **Owner / source:** leaves `SIGNOFF-REPAIR.7.3.3.3.1`, `.7.2.1`, `.11.4.3.1.2.24`

## The question

Before removing a file or directory, how do you prove the path still names the
one you created, and not a successor that took its name?

## The answer

**Hold the descriptor.** Open the entry when you create it and keep it for the
owner's whole life, then compare the path's `symlink_metadata` against that LIVE
descriptor's metadata. An open descriptor keeps the inode allocated, so no
successor can be handed the same `(device, inode)` pair while you hold it.

Comparing against a `Metadata` value captured at creation is NOT the same thing
and is the shape that fails: with the descriptor closed, Linux reuses the inode
number as soon as it is freed, so a replacement presents the identical pair and
gets deleted in your place.

For a FILE, add the link count: the descriptor reports zero links once your file
is unlinked, whatever number the replacement was given, and the path must report
exactly one link so a hard link cannot stand in.

For a DIRECTORY, do NOT reuse that check. A directory's link count grows as
subdirectories appear inside it, and macOS was measured still reporting
`nlink == 2` on a held descriptor AFTER the directory was removed — so it
carries no signal. The pinned inode does the work alone.

## Why the platform hides it

Measured on APFS: a successor at the same path reused the inode in 0 of 300
trials with the descriptor held AND 0 of 300 with it closed. **The development
platform cannot tell the sound shape from the unsound one.** The unsound shape
therefore survives every local run and is exposed only on the runner's ext4.
Treat a passing macOS identity test as no evidence at all about this property.

## Re-verify

Create an entry, record its identity, unlink it, create a successor at the same
path, and compare — once with the original descriptor held, once with it closed.
On ext4 the closed case collides; on APFS neither does, which is the point.

Related: `TOOLBOX.md`, "Remote-only is a hypothesis, not a category".
