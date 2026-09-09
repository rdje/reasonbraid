---
answers:
  - Where can I retrieve changelog entries removed from the recent digest?
  - How was the September 2026 changelog rotation verified without losing history?
  - Does changelog rotation imply complete live-document containment adoption?
---
# Changelog rotation through the existing Git history terminal

- Owner: `SIGNOFF-REPAIR.11.4.1`.
- Authority: the existing `README_POLICY.md` local adoption, `.doctrine/readme_routes.txt` and `docs/decisions/2026-09-06_readme-policy-readoption.md` already choose Git history as this ledger's rotation terminal. No cap or storage topology changes.
- Source revision: `25ed7d184203e2d8701800558b785b30c75bb4d0`.
- Source path: `CHANGELOG.md`.
- Source Git blob: `0bc51d581f9158ebafcef94cfb6717464722c6cb`.

The source reached 95,038 bytes under its unchanged 96,000-byte cap. Its first ten
dated records describe the current corrective review. The remaining 120 records
are historical phase chronology; their original success claims stay bound to the
recorded revisions, while current qualification comes from LIVE_STATUS and the
book. The older rotation notice is included in the retired segment and remains
retrievable, preserving the route into earlier Git history.

| Source segment | Byte offsets (half-open) | Bytes | Lines | Dated records | Maximum line bytes |
| --- | --- | --- | --- | --- | --- |
| Complete predecessor | 0–95038 | 95038 | 646 | 130 | 1169 |
| Retained corrective records, including original heading | 0–6968 | 6968 | 110 | 10 | 145 |
| Rotated historical suffix | 6968–95038 | 88070 | 536 | 120 | 1169 |

SHA-256 identities:

- Complete: `f9cd2167b141cc7537b6eef9fffedfb7274b80915fdfa6604fdb4cdf30a9d8b0`.
- Retained: `6a345143d336e2352616469706bbd5d4c1ae40307b2f9470146845b239775a0a`.
- Rotated: `c81596d77187ce3c349e3b68d747eab9200bf668866407852fbbf0e0d4124846`.

The new live digest inserts one rotation record and a bounded retrieval note;
it preserves the ten existing records verbatim. Transition validation recovered
the retained bytes from the new digest and concatenated the exact Git suffix:
all 95,038 original bytes reconstructed, rc=0. Truncating or altering retained
content changed the digest; a missing Git identity refused. The resulting digest
measured 8,462 bytes / 139 lines, maximum content line 145 bytes. These are capture
measurements of this transition, not declarations of a current size.

Retrieve the exact predecessor from the repository root:

```bash
git show 25ed7d184203e2d8701800558b785b30c75bb4d0:CHANGELOG.md
```

`git cat-file blob 0bc51d581f9158ebafcef94cfb6717464722c6cb` is an independent
object-identity retrieval path. Preserve reachable history when cloning and
handing off. If a shallow checkout lacks the named commit, retrieval fails until
that history is available; do not substitute an empty or newer file. Earlier
versions remain discoverable with `git log --follow -- CHANGELOG.md`.

Consumer census found the size/existence guard, commit-workflow appends and
file-level historical references, with no tracked heading-anchor, line-address
or per-record parser consumers. The file already used Git-history rotation.
This applies its existing contract, not a new archive collection.

The director-authorized fsmgen adoption guide was reviewed read-only at SHA-256
`8f77fa39c9bcb9cfc43166259a627a6ced64682030088400b727dccc5d674a53`.
It now describes broader lifecycle, derived-state and verifier contracts. This
repository has README-route containment but no root live-document doctrine;
complete review/adoption and enforcement remain `SIGNOFF-REPAIR.11.4.2`. This
rotation does not claim that broader package is adopted or enforced.
