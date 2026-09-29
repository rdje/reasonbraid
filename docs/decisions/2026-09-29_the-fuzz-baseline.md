---
answers:
  - Does ReasonBraid fuzz its untrusted-input parsers, and which ones?
  - What happened to the fuzz baseline Phase 1 deferred to "the first untrusted parser"?
  - How is the untrusted-parser population derived?
  - What does the fuzz baseline still not catch?
---
# The fuzz baseline: every in-repo parser of untrusted bytes answers every mutation

- **Type:** decision
- **Status:** active. Discharges Phase 1's deferral #5 (`docs/decisions/2026-09-07_phase1-gate-record.md`), whose trigger, *"the first untrusted parser (Phase 4's resource packs)"*, fired at Phase 4 and was never acted on.
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.1`
- **Date:** 2026-09-29
- **Work unit:** `REASONBRAID-REPAIR-0575`

## The population, derived rather than listed

`python3 -B scripts/census_untrusted_parsers.py` derives it from where untrusted bytes enter the product. It covers HTTP response bodies, a worker's stdin, a child process's piped output, a peer certificate, a request body and a Git transfer. The population is every product function that takes raw bytes (or, when named as a parser, text) and is defined beside a producer, or called from one by path or by a method name the product defines once. Its self-test holds each rule, including the five mistakes its first versions made:

- `str::bytes()` counted as an HTTP body;
- an I/O `read` counted as a parser;
- a certificate reached by a method call missed;
- a parser of hex dropped as a "hex helper";
- the extraction worker's five format parsers missed, because their signatures span several lines.

On 2026-09-29 it lists **25** functions. This record classifies each one.

| Function | In the baseline? | Why |
| --- | --- | --- |
| `reasonbraid-extract`: `extract_pdf`, `extract_zip`, `extract_tar`, `extract_feed`, `extract_rss` | ✅ fuzzed | `the_format_parsers_answer_every_mutation_without_panicking`, seeded with a valid input of each format |
| `reasonbraid-extract`: `root_element`, `is_archive_bytes`, `text_eligible` | ✅ fuzzed, inside the above | each is reached on every mutated input of its format |
| `fetcher.rs`: `decode_body`, `inflate` | ✅ fuzzed | `the_body_decoder_and_the_sniffer_answer_every_mutation`: gzip, deflate, brotli and identity seeds, under the size and ratio ceilings |
| `fetcher.rs`: `sniff_kind`, `skip_whitespace_and_bom`, `prefix_ignoring_case` | ✅ fuzzed | the same test: HTML, PDF, ZIP and BOM-prefixed RSS heads under three declared types |
| `git.rs`: `is_lfs_pointer` | ✅ fuzzed | `the_lfs_pointer_test_answers_every_mutation` |
| `node_channel.rs`: `decode_hex` | ✅ fuzzed | `the_hex_decoder_answers_every_mutation`: a node's proof arrives as hex |
| `api.rs`: `parse_principal` | ✅ fuzzed | `the_principal_parser_answers_every_mutation`: every request names its principal in a header |
| `ca.rs`: `extract_point`, `verify_leaf` (with `leaf_not_after`) | ✅ fuzzed | `the_certificate_parsers_answer_every_mutation`, seeded with a real leaf from the module's own CA |
| `ca.rs`: `verify_signature` | not fuzzed | `ring`'s signature verification over a point, a message and a signature it treats as opaque; the in-repo code only passes them through |
| `subprocess.rs`: `stderr_tail` | not fuzzed | `String::from_utf8_lossy` over a byte tail: total on every input by construction |
| `lifetime.rs` (browse): `parse_endpoint` | not fuzzed | reads a line from the browser the worker itself launched, inside the worker's own process |
| `extraction_input.rs`: `extract_acquired_bytes`; `snapshots.rs`: `submit` | not a parser | they store bytes and hash them; nothing interprets them |
| `mtls.rs`: `build_server_config`, `build_client_config` | not untrusted | they take the server's own certificate and key; the peer's certificate is verified by `rustls` and WebPKI |

Request and channel bodies are also producers, decoded by `serde` with `deny_unknown_fields` and no in-repo parser behind them. They are outside this baseline, which fuzzes code this repository wrote.

## What the baseline is

- **A seeded mutation harness on the stable toolchain.** It is `reasonbraid-extract`'s `Mutator` and `reasonbraid-server`'s `fuzz_support`, the same xorshift in both, with no crate added. Each round applies one to four structural edits (bit flip, boundary byte, truncation, insertion, duplication, zeroed run) to a VALID seed, and the parser must answer a result or a typed refusal without panicking. A failure names the round and prints the input, so it reproduces.
- **It runs in every test run.** CI's `cargo test --all` runs it at 300 rounds per seed, well under a second in all. `RB_FUZZ_ROUNDS` raises it for a deeper run.
- **Measured on 2026-09-29.** 300 rounds per seed pass everywhere, and so do 20,000 rounds per seed: 100,000 mutated inputs through the extraction worker's parsers in 2.3 s, and the server's five tests in 1.7 s. No panic was found.
- **Seen failing, as `docs/CLAIM_VERIFICATION.md` leg 2 requires.** An `.expect()` was planted where the feed parser maps a malformed event to a refusal, and the harness failed on round 3, printing the input. The same kind of plant in `decode_hex` failed on round 2. Both files were restored byte for byte.

## What it still permits

- **It is unguided.** There is no coverage feedback: this environment has no nightly toolchain and no `cargo-fuzz`, so it cannot run libFuzzer. The mutations explore around valid seeds and cannot learn which ones reach new code. A guided fuzzer remains worth adding when a nightly toolchain is available.
- **It finds panics, not hangs.** A mutation that makes a parser loop forever would hang the test. In production the extraction worker runs under a killing time budget, and the fetcher's decoder under its size and ratio ceilings.
- **Third-party parsers are exercised only through these wrappers.** That covers `lopdf`, `zip`, `tar`, `quick-xml`, `flate2`, `brotli` and `x509-parser`. `gix`'s pack parsing is not reached by any seed here, and no claim is made about upstream fuzzing.
- **No corpus is kept between runs.** Each run starts from the same valid seeds.
