#!/usr/bin/env python3
"""The untrusted-parser population, derived from where untrusted bytes enter
(`SIGNOFF-REPAIR.11.4.7.2.1.1`).

Phase 1 deferred a fuzz baseline until *"the first untrusted parser"*, and Phase
4's resource packs fired that trigger without anyone acting on it. Deciding
what a baseline covers needs the population, and a hand-written list is the
thing this project keeps finding stale. So the population is DERIVED:

1. A PRODUCER is a site where bytes this product does not control enter it: an
   HTTP response body, a worker's stdin, a child process's piped output, a peer
   certificate, a request body, a Git transfer.
2. A PARSER is a product function that takes raw bytes (`&[u8]`, `Vec<u8>`,
   `Bytes`, a reader) and is not a pure digest or encoding helper, which accepts
   every input by construction.
3. The population is every parser defined in a file that holds a producer, or
   called by path (`module::function(`) from one.

    python3 -B scripts/census_untrusted_parsers.py            # the population
    python3 -B scripts/census_untrusted_parsers.py --json
    python3 -B scripts/census_untrusted_parsers.py --self-test

⚠️ WHAT THIS DOES NOT DO. It reads source text, not a call graph: a parser
reached through a method call on a value, or through a third crate, is found
only if its file also holds a producer. JSON handled by `serde` is a producer
here (a request body) with no in-repo parser behind it, and is reported as
such. Third-party parsers (`gix`, `quick-xml`, the archive and PDF crates) are
outside the population; the decision record says what covers them.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PRODUCERS: list[tuple[str, re.Pattern[str]]] = [
    # `.bytes()` alone is also `str::bytes()`, which is no producer: a body read
    # is awaited, or streamed.
    ("http_body", re.compile(r"\.(bytes|chunk|text)\(\)\s*\.await|bytes_stream\(\)")),
    ("process_input", re.compile(r"\bstdin\(\)")),
    ("child_output", re.compile(r"Stdio::piped\(\)")),
    ("peer_certificate", re.compile(r"peer_certificates|client_cert")),
    # A handler parameter that destructures an extractor (`ApiJson(x): ApiJson<…>`,
    # the node channel's `NodeJson`, or axum's `Json`); a response built with
    # `Json(…)` is not one.
    ("request_body", re.compile(r"\b(?:Api|Node)?Json\([a-z_]+\)\s*:\s*(?:Api|Node)?Json<")),
    ("git_transfer", re.compile(r"prepare_fetch|fetch_only|\breceive\(")),
]
FUNCTION = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*(?:<[^>]*>)?\s*\(([^)]*)\)")
FUNCTION_START = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+[a-z_][a-z0-9_]*")
BYTES_PARAM = re.compile(r":\s*(?:&\s*(?:mut\s+)?\[u8\]|Vec<u8>|Bytes\b|impl\s+(?:Buf)?Read\b|&mut\s+impl\s+(?:Buf)?Read\b)")
# A parser of untrusted TEXT (hex, a header, a line) takes `&str`, like thousands
# of functions that parse nothing, so text is counted only for functions named as
# parsers.
TEXT_PARAM = re.compile(r":\s*&\s*str\b")
PARSER_NAME = re.compile(r"^(parse|decode|extract|sniff)_")
# Method names third-party types define too (`base64::Engine::decode`,
# `io::Read::read`): a text census cannot tell whose method a `.decode(` is, so
# these are never resolved by method, only by path.
FOREIGN_METHOD_NAMES = {"decode", "encode", "read", "write", "parse", "new", "from"}
# Excluded by what the function DOES, never by where it sits: hashing and encoding
# accept every input by construction, `read`/`write` are I/O trait methods that
# move bytes without interpreting them, and the storage writers persist bytes.
# `decode_hex` PARSES hex and stays in; `to_hex` only encodes and goes.
PURE_HELPER = re.compile(r"(^|_)(digest|fingerprint|splitmix64)($|_)|^(to_hex|hex|hex_lower)$|^(read|write|write_new|write_input|publish|backup|create|create_named)$")
# A call by path (`ca::parse(`) or by method (`.verify_leaf(`): the mTLS layer
# hands a peer certificate to the CA through a method, and a path-only rule
# missed it.
PATH_CALL = re.compile(r"\b[a-z_][a-z0-9_]*::([a-z_][a-z0-9_]*)\(")
METHOD_CALL = re.compile(r"\.([a-z_][a-z0-9_]*)\(")


def product_files() -> list[Path]:
    listed = subprocess.run(
        ["git", "ls-files", "crates/*/src/*.rs", "crates/*/src/**/*.rs"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout.split()
    return [ROOT / f for f in listed if "/tests/" not in f]


def product_lines(text: str) -> list[str]:
    """The lines before the first `#[cfg(test)]`: test modules are not product."""
    lines = text.split("\n")
    for index, line in enumerate(lines):
        if line.strip() == "#[cfg(test)]":
            return lines[:index]
    return lines


def signatures(lines: list[str]) -> list[str]:
    """Each function signature on one line: rustfmt breaks a long parameter list
    across lines, and the extraction worker's five format parsers are written
    that way (a one-line rule missed all five)."""
    out = []
    for index, line in enumerate(lines):
        if FUNCTION_START.match(line):
            joined = line
            for more in lines[index + 1 : index + 20]:
                if ")" in joined:
                    break
                joined += " " + more.strip()
            out.append(joined)
    return out


def parsers_in(lines: list[str]) -> list[str]:
    found = []
    for line in signatures(lines):
        match = FUNCTION.match(line)
        takes_input = BYTES_PARAM.search(match.group(2)) if match else None
        if match and not takes_input and PARSER_NAME.search(match.group(1)):
            takes_input = TEXT_PARAM.search(match.group(2))
        if match and takes_input and not PURE_HELPER.search(match.group(1)):
            found.append(match.group(1))
    return found


def census(files: dict[str, str]) -> dict:
    """`files` maps a repository-relative path to its text (so the self-test can
    hand in fixtures)."""
    producers: dict[str, list[str]] = {}
    parsers: dict[str, list[str]] = {}
    calls: dict[str, set[str]] = {}
    for rel, text in files.items():
        lines = product_lines(text)
        kinds = sorted({kind for kind, pattern in PRODUCERS for line in lines if pattern.search(line)})
        if kinds:
            producers[rel] = kinds
            by_path = {m.group(1) for line in lines for m in PATH_CALL.finditer(line)}
            by_method = {m.group(1) for line in lines for m in METHOD_CALL.finditer(line)}
            calls[rel] = (by_path, by_method)
        found = parsers_in(lines)
        if found:
            parsers[rel] = found
    defined: dict[str, int] = {}
    for rel, text in files.items():
        for line in signatures(product_lines(text)):
            match = FUNCTION.match(line)
            if match:
                defined[match.group(1)] = defined.get(match.group(1), 0) + 1

    def reaches(rel: str, name: str) -> bool:
        by_path, by_method = calls[rel]
        if name in by_path:
            return True
        return name in by_method and defined.get(name) == 1 and name not in FOREIGN_METHOD_NAMES

    population: list[dict] = []
    for rel, names in sorted(parsers.items()):
        for name in names:
            via = []
            if rel in producers:
                via.append(f"defined beside {', '.join(producers[rel])}")
            callers = sorted(p for p in calls if p != rel and reaches(p, name))
            via += [f"called from {c} ({', '.join(producers[c])})" for c in callers]
            if via:
                population.append({"file": rel, "function": name, "reached": via})
    serde_only = sorted(
        rel for rel, kinds in producers.items() if kinds == ["request_body"] and rel not in parsers
    )
    return {
        "producers": {rel: kinds for rel, kinds in sorted(producers.items())},
        "population": population,
        "serde_only_producers": serde_only,
    }


def self_test() -> int:
    fixture = {
        "crates/a/src/fetch.rs": "\n".join([
            "async fn get() { let s = r.bytes_stream(); sniff(&head); ca::parse_cert(&der); v.check_leaf(&der); b.decode(x); }",
            "fn count(t: &str) -> usize { t.bytes().count() }",
            "fn sniff(head: &[u8]) -> bool { true }",
            "fn extract_pdf(",
            "    bytes: &[u8],",
            "    request: &Request,",
            ") -> Result<(), ()> { Ok(()) }",
            "pub fn digest_sha256_hex(bytes: &[u8]) -> String { String::new() }",
            "#[cfg(test)]",
            "fn only_in_tests(b: &[u8]) {}",
        ]),
        "crates/a/src/ca.rs": "pub fn parse_cert(der: &[u8]) -> Result<(), String> { Ok(()) }\npub fn check_leaf(der: &[u8]) -> bool { true }\nfn unreached(b: &[u8]) {}\nfn read(&mut self, buf: &mut [u8]) -> usize { 0 }",
        "crates/a/src/api.rs": "async fn h(ApiJson(body): ApiJson<Body>) { Ok(Json(row)) }\nfn decode_hex(s: &str) -> Option<Vec<u8>> { None }",
        "crates/a/src/state.rs": "pub fn decode(raw: &[u8]) -> Result<(), ()> { Ok(()) }",
        "crates/a/src/resp.rs": "async fn r() { Ok(Json(row)) }",
        "crates/a/src/quiet.rs": "fn lonely(b: &[u8]) {}\nfn words(t: &str) { t.bytes(); }",
    }
    result = census(fixture)
    names = sorted((p["file"], p["function"]) for p in result["population"])
    assert names == [
        ("crates/a/src/api.rs", "decode_hex"),
        ("crates/a/src/ca.rs", "check_leaf"),
        ("crates/a/src/ca.rs", "parse_cert"),
        ("crates/a/src/fetch.rs", "extract_pdf"),
        ("crates/a/src/fetch.rs", "sniff"),
    ], names
    assert "crates/a/src/quiet.rs" not in result["producers"], "`str::bytes()` is no producer"
    assert result["producers"]["crates/a/src/fetch.rs"] == ["http_body"], result
    assert ("crates/a/src/api.rs", "decode_hex") in names, "a parser of untrusted text beside a producer"
    assert ("crates/a/src/state.rs", "decode") not in names, "`.decode(` is a foreign method name"
    assert "crates/a/src/resp.rs" not in result["producers"], "a response built with Json is no producer"
    assert result["serde_only_producers"] == [], result
    print("census_untrusted_parsers: self-test OK (a parser beside a producer, one called by path, "
          "one called by method, a digest helper, an I/O `read` and a test-only function excluded, "
          "an unreached parser excluded, `str::bytes()` and a `Json` response not producers, "
          "a text parser named as one included, a foreign method name never resolved by method, "
          "a signature split across lines read whole)")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    files = {str(p.relative_to(ROOT)): p.read_text(encoding="utf-8") for p in product_files()}
    result = census(files)
    if "--json" in argv:
        print(json.dumps(result, indent=2))
        return 0
    print(f"producers: {len(result['producers'])} files")
    for rel, kinds in result["producers"].items():
        print(f"  {rel}: {', '.join(kinds)}")
    print(f"untrusted parsers: {len(result['population'])}")
    for entry in result["population"]:
        print(f"  {entry['file']}::{entry['function']} — {'; '.join(entry['reached'])}")
    print(f"serde-only producers (no in-repo parser): {', '.join(result['serde_only_producers']) or 'none'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
