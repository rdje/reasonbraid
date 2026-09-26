---
answers:
  - What does the server answer when a request body does not deserialize?
  - Why is a malformed body 422 on some routes and 400 on others?
  - Why did the repair for missing reason codes keep the 422?
---
# A body refusal keeps its status and gains a code

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.11.36`
- **Date:** 2026-09-26
- **Work unit:** `REASONBRAID-REPAIR-0539`
- **Cites:** `docs/book/src/errors.md` (*every refusal carries a stable reason
  code*); `SIGNOFF-REPAIR.9.2.1.2.3` and
  `docs/knowledge/a-convention-is-what-the-corpus-asserts-not-what-one-surface-does.md`
  (the two-level contract); `SIGNOFF-REPAIR.7.4.7` (`api::json_body`)

## The fact / decision

1. **A body that does not deserialize keeps axum's status**: `422` when it parses
   but is not the handler's declared shape (an unknown, missing or mistyped
   field), `400` when it does not parse at all, `413` when it is too large, `415`
   when it is not declared JSON.
2. **It carries a code**: `{"code": "invalid_command", "message": <the parser's
   own sentence>}`, on the control API and the node channel alike. The sentence
   names the unknown or missing field.
3. **Every handler reads its body through one extractor per surface** —
   `api::ApiJson` and `node_channel::NodeJson`, both through
   `api::json_rejection` — so a new route cannot bring axum's plain-text default
   back; the unit test `api::json_extraction::no_handler_takes_a_bare_json` walks
   every source file of the server.
4. **The exception stays an exception**: the site acts' `api::site_request`
   answers `400` with a FIXED message, because an operator surface does not echo
   malformed input (its own comment says so).

## Why

- **The defect was the missing code, not the status.** 52 handlers answered axum's
  default, a plain-text sentence with no `code`, against the errors chapter's
  promise. That is what `SIGNOFF-REPAIR.11.36` owns.
- **The `422` is a deliberate level.** `.9.2.1.2.3` measured the corpus: `422` =
  the body is not the declared shape and the handler never ran; `400
  invalid_command` = the handler ran and the request is wrong. Four suites assert
  it and a repair relies on it. The first cut of this leaf flattened it to `400`
  and the corpus refused that at once — the mistake the knowledge note records.
- **`json_body` was the deviation.** It had mapped every shape error to `400`
  (`.7.4.7`), so `POST /v1/assessments` answered unlike every other typed route.
  It now answers `422` too.
- **Nodes are unaffected.** The node treats `400`, `413` and `422` alike as a
  permanent refusal of an event's bytes (`SIGNOFF-REPAIR.4.4.10.2`); it gains the
  code in place of "unknown".

## How to apply

- Take a body as `ApiJson<T>` (control API) or `NodeJson<T>` (node channel), never
  `Json<T>`.
- Branch on `code` and on the status's level: `422` means fix the body's shape,
  `400 invalid_command` from the handler means the request itself is wrong.
