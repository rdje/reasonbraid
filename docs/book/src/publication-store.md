# The publication store

A policy publication ends up in a **bare Git repository** inside the directory
the deployment declares with `rb-server --publication-repo-root` (see
[Where publications are written](cli.md#where-publications-are-written)).
`POST /v1/policy-publications/{id}/publish` writes it there, and ROADMAP §15.7
describes the steps. This chapter covers what is written, which refs move and
what refuses them. It also covers how an interrupted publish is recovered.

The examples come from a real run of the publisher against a scratch
repository, not from hand-written output.

## What a publish writes

Each publication is **one root commit** holding two files:

| File | Content |
| --- | --- |
| `manifest.json` | the publication's identity: `publication_id`, `proposal_id`, `decision_id`, `approval_id`, `projection_id` and `projection_digest` |
| `bundle.txt` | the compiled policy bytes, which hash to `projection_digest` |

The commit's author and committer are `reasonbraid-publisher`. Its timestamp
is the publication's **staging time**, not the moment of writing, so the same
publication always produces the same commit id. The message is
`publication <id>`.

## The three refs

| Ref | Holds | What moves it | What refuses |
| --- | --- | --- | --- |
| `refs/rb/staging/<id>` | the commit being published | every publish of this publication, to the same commit | nothing directly; a publish refused by the write-once rule never reaches it |
| `refs/rb/publications/<id>` | the publication, **for ever** | the first successful write only | **write-once**: a different commit under the same id is refused (`ImmutableExists`); rewriting the identical commit is a no-op |
| `refs/rb/effective` | the currently effective publication | a **compare-and-swap** from the id the publish expected | any other current value (`CasMismatch`); it is never force-pushed |

A real repository after three publishes: `pub-1`, then `pub-2` (expecting
`pub-1` as effective), then `pub-3` (wrongly expecting `pub-1` again):

```text
$ git --git-dir=<repository> for-each-ref refs/rb
7d4bb4e29b6f56286ed5eeef1c8154ddadf6b9eb commit	refs/rb/effective
324c107e0604c8d87dab3141577f0817297b2a1a commit	refs/rb/publications/pub-1
7d4bb4e29b6f56286ed5eeef1c8154ddadf6b9eb commit	refs/rb/publications/pub-2
e0a32dc5a4b105152787a9b48d85eb253768668b commit	refs/rb/publications/pub-3
324c107e0604c8d87dab3141577f0817297b2a1a commit	refs/rb/staging/pub-1
7d4bb4e29b6f56286ed5eeef1c8154ddadf6b9eb commit	refs/rb/staging/pub-2
e0a32dc5a4b105152787a9b48d85eb253768668b commit	refs/rb/staging/pub-3
```

`pub-3` shows the one state a single publish cannot finish. Its immutable ref
was written, and then its compare-and-swap was refused, so `refs/rb/effective`
still names `pub-2`. That publication stays `staged`.
[`rb-reconciler`](#recovering-an-interrupted-publish-rb-reconciler) reports it
rather than forcing the channel.

## The order of a publish

1. **Validate before anything is written.** The repository must be inside the
   root and must open, and the manifest must still digest to what was staged.
2. **Record the Git operation** in the database (below).
3. **Write the objects**: the two blobs, the tree and the commit. Git objects
   are content-addressed, so rewriting them is harmless.
4. **Fetch back** the manifest blob and check that it hashes to what was
   written.
5. **The write-once check.** If the immutable ref already holds a *different*
   commit, the publish is refused **before any ref moves**.
6. Move `refs/rb/staging/<id>`, then `refs/rb/publications/<id>`, then
   `refs/rb/effective` by compare-and-swap.
7. Mark the publication `effective` and record the object ids.

## What refuses a publish

Every refusal from the Git half reaches the caller as `400 invalid_command`
with one of these messages. The first three below are copied from the real run
above; paths are shortened.

| Refusal | Produced by | Message |
| --- | --- | --- |
| `ImmutableExists` | publishing **different** content under an id already published | *the immutable publication ref already exists — written once, never moved* |
| `CasMismatch` | an `expected_effective` that is not the channel's current value, including expecting one when there is none | *the effective channel is not what the compare-and-swap expected — expected \`324c107…\`, found \`7d4bb4e…\` — never a force-push* |
| `Open` | a `repo_path` inside the root that is not a Git repository | *the repository does not open: "…/nowhere" does not appear to be a git repository* |
| `FetchBack` | the object store returning bytes that do not hash to what was just written | *the fetch-back verification failed: …* |
| `Write` | the object store or a ref edit failing for any other reason | *the write failed: …* |

`FetchBack` and `Write` come from a failing store, not from a request shape, so
the example run could not produce them. Their text is the server's own.

⚠️ Publishing the **same** publication again is not a refusal. It rewrites the
identical commit and every ref edit is a no-op, which is what makes recovery
safe to retry.

## The operation is recorded before the write

A publish records **what it is about to do before it touches Git**. ROADMAP
§15.7 step 4 calls this the *desired Git operation*. The publication row gets:

- `repository`: the repository, **relative to** the publication root (for
  example `"live"`), so the record stays valid if the root is moved;
- `expected_effective`: the effective id the compare-and-swap expects. When
  `repository` is recorded and this is absent, the publish expected **no**
  effective channel yet.

Both appear on the publication when recorded:

```json
{ "publication_id": "pub_…", "state": "staged", "repository": "live",
  "expected_effective": "1111…" }
```

This is what lets an interrupted publish be found. If the server dies between
writing to Git and marking the row `effective`, the row still names the
repository it was writing to.

- A publish refused **before** anything is written records nothing. That covers
  a path outside the root, a location that is not a repository, and a manifest
  that no longer matches the staged digest.
- Once a publication has recorded its operation, a later publish must ask for
  the **same** repository and `expected_effective`. A different request is
  refused (*already recorded its Git operation*) rather than re-pointed,
  because re-pointing it would strand whatever the first attempt wrote.

**The publication commit is reproducible.** Its timestamp is the publication's
**staging time**, not the moment of writing. Publishing the same publication's
content twice therefore produces the **same commit id**, and the id can be
recomputed from the record.

⚠️ Publications staged before this rule carry no `repository`. Nothing recorded
where they were written, and they are not guessed at.

## Recovering an interrupted publish: `rb-reconciler`

`rb-reconciler` is the §15.8 reconciler. It makes one pass over every
publication that has recorded a Git operation. For each one it reads the
publication's refs from the recorded repository, compares them with what the
row says, and then either recovers the publication or reports it. Point it at
the same database and publication root as `rb-server`:

```text
$ rb-reconciler --database-url postgres://… --publication-repo-root /srv/rb/publications
pub_a: applied RetryStagedWrite
pub_b: applied VerifyAndAdvance
pub_c: requires a human: StopSecurityAlert — the immutable ref holds a commit this publication's content does not commit to — never pick a side
pub_f: cannot be reconciled: no recorded Git operation — …
```

`--publication <id>` reconciles one publication. The reconciler never changes
the database schema; migrating is `rb-server`'s job.

| What it finds | What it does |
| --- | --- |
| staged, nothing written to Git | **recovers**: writes the recorded operation and marks the row `effective` |
| staged, the Git write happened but the row never heard | **recovers**: verifies the commit and marks the row `effective` |
| staged, but the immutable ref holds a different commit | **reports** — never picks a side |
| `failed`, but its write appeared later | **reports** — quarantine and adjudicate; the row stays `failed` |
| `effective`, but its ref is missing or moved | **reports** — freeze and repair through the authorized path |
| the recorded compare-and-swap can no longer hold | **reports** — advancing would overwrite a publication the record does not know about |
| database and Git agree | nothing |
| no recorded Git operation | nothing to observe, so nothing is guessed |

Recovery is **idempotent**. The commit is reproducible and rewriting an
identical ref is a no-op, so a second pass over a recovered publication
reports `consistent` and writes nothing.

Exit status: `0` when every publication is consistent or recovered, `3` when at
least one needs a human, and `1` when the pass itself failed.


## Reading what was published

`GET /v1/policy-bundles/{manifest_digest}` serves an effective publication's
manifest and bundle, checking both against the record first. See
[Reading a published bundle](policy-lifecycle.md#reading-a-published-bundle).

## Beside it: the evidence store

Evidence snapshots use a **different**, content-addressed store: identical bytes
are one object, and an object is never rewritten. It is described in
[Evidence snapshots](evidence.md). The publication store is not
content-addressed by bytes. Its identity is the publication id, and its guarantee
is the write-once ref.
