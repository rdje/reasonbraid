//! `SIGNOFF-REPAIR.3.4.5` — the comparative measurement ADR-009 never had.
//!
//! ADR-009 chose **chain-in-envelope** over **capability tokens** for the dev
//! profile. Its wire-size evidence was withdrawn by `.3.3.1`, because the test
//! that carried it hand-built some JSON, took its length `N`, and asserted
//! `N < N + 64`. That inequality is true by construction: it encoded no token,
//! and it compared no delegation depth.
//!
//! This instrument replaces it with an actual encoding of both shapes at depths
//! 1, 2 and 3. It is deliberately written so a reader can check that it is not
//! repeating the original mistake:
//!
//! - The depth-1 envelope is the **real shipped** [`CommandEnvelope`] carrying a
//!   real [`AuthorityContext`], serialized by `serde_json`. Nothing is hand-built.
//! - The token shape is encoded end to end — a JWS-style compact serialization
//!   whose header and payload are real JSON, base64url-encoded here, joined by
//!   real separators. The only quantity taken from a specification rather than
//!   produced is the SIGNATURE LENGTH (HMAC-SHA256 is 32 bytes; Ed25519 and
//!   ECDSA P-256 raw signatures are 64). That is a fact about the algorithm, not
//!   an increment invented to make an inequality come out: every other byte of
//!   the token is genuinely encoded, and the signature block is emitted at its
//!   true length rather than added to the other shape's total.
//!
//! ⛔ This instrument asserts NO size advantage in either direction, and the
//! development envelope choice is preserved whatever the numbers say. It records
//! measurements; ADR-009 records what they mean.

use reasonbraid_core::{
    AuthorityContext, ClientContext, CommandEnvelope, TargetSelector, PROTOCOL_VERSION,
};
use serde_json::json;
use sha2::{Digest, Sha256};

/// A P-256 / Ed25519 raw signature. Fixed by the algorithm, not chosen here.
const ASYMMETRIC_SIGNATURE_BYTES: usize = 64;

const SUBJECTS: [&str; 3] = [
    "hpr_00000000-0000-7000-8000-000000000101",
    "rol_00000000-0000-7000-8000-000000000102",
    "rol_00000000-0000-7000-8000-000000000103",
];

fn base64url_len(bytes: &[u8]) -> usize {
    // Unpadded base64url, the JWS compact form: 4 characters per 3 input bytes,
    // with the final partial group emitting ceil(remainder * 4 / 3).
    let (whole, rest) = (bytes.len() / 3, bytes.len() % 3);
    whole * 4 + if rest == 0 { 0 } else { rest + 1 }
}

/// The delegation scope every hop carries — the same value in both shapes, so
/// the comparison is not smuggling a different payload into one of them.
fn scope() -> TargetSelector {
    TargetSelector::Threads {
        threads: vec!["thr_00000000-0000-7000-8000-000000000199".parse().unwrap()],
    }
}

fn authority_context(subject: &str) -> AuthorityContext {
    AuthorityContext {
        on_behalf_of: subject.to_string(),
        purpose: Some("quarterly deliberation".to_string()),
        scope: scope(),
    }
}

/// The request both shapes carry, with no delegation on it. Everything outside
/// the delegation is identical in both designs, so subtracting this baseline
/// isolates the bytes the delegation actually costs.
fn baseline_envelope() -> CommandEnvelope {
    CommandEnvelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        operation: "thread.contribute".to_string(),
        request_id: "req_00000000-0000-7000-8000-000000000001".parse().unwrap(),
        // ⛔ EXACTLY 16 characters, and readable on purpose. The length is
        // load-bearing — `baseline` below asserts 308 bytes — and the previous
        // value was 16 hex digits, which `gitleaks` scored at entropy 3.875 and
        // reported as a `generic-api-key` (`SIGNOFF-REPAIR.11.4.7.2.3`). Every
        // sibling fixture already spells these readably (`k-forged`,
        // `client-key-0001`); this one was the outlier.
        idempotency_key: "key-delegation-1".to_string(),
        expected_aggregate_version: Some(7),
        body: json!({ "content": "the position I am asked to take" }),
        authority_context: None,
        client_context: ClientContext::default(),
    }
}

fn baseline_bytes() -> usize {
    serde_json::to_vec(&baseline_envelope()).unwrap().len()
}

/// Shape A — chain-in-envelope.
///
/// ⚠️ Depth 1 is the SHIPPED type. Depths 2 and 3 are a prototype, because the
/// shipped `AuthorityContext` holds exactly one `on_behalf_of` and cannot
/// express a chain at all — which is itself a finding, recorded in the leaf.
/// The prototype replaces the single context with an array whose members are
/// each a REAL serialized `AuthorityContext`, so only the container is invented.
fn envelope_shape_bytes(depth: usize) -> usize {
    assert!((1..=3).contains(&depth));
    if depth == 1 {
        let mut envelope = baseline_envelope();
        envelope.authority_context = Some(authority_context(SUBJECTS[0]));
        return serde_json::to_vec(&envelope).unwrap().len();
    }
    let mut wire = serde_json::to_value(baseline_envelope()).unwrap();
    let chain: Vec<_> = SUBJECTS[..depth]
        .iter()
        .map(|s| serde_json::to_value(authority_context(s)).unwrap())
        .collect();
    wire.as_object_mut()
        .unwrap()
        .insert("authority_context".into(), json!(chain));
    serde_json::to_vec(&wire).unwrap().len()
}

/// Shape B — capability tokens: one signed, expiring credential per hop, all of
/// them presented with the request. Each token is a real JWS compact
/// serialization of real JSON.
fn token_bytes(depth: usize, signature_bytes: usize) -> usize {
    assert!((1..=3).contains(&depth));
    let mut total = baseline_bytes();
    for (hop, subject) in SUBJECTS[..depth].iter().enumerate() {
        // The header a verifier needs to pick the key and the algorithm.
        let header = json!({
            "alg": if signature_bytes == 32 { "HS256" } else { "ES256" },
            "typ": "JWT",
            "kid": "key_00000000-0000-7000-8000-00000000000f",
        });
        // The claims. The first five carry what the envelope's AuthorityContext
        // carries; the rest are what a CREDENTIAL must add and an in-request
        // context does not need — an issuer, an audience, a lifetime and a
        // replay identifier. That asymmetry is the real subject of ADR-009's
        // question, so it is encoded rather than argued about.
        let payload = json!({
            "sub": subject,
            "purpose": "quarterly deliberation",
            "scope": scope(),
            "act": SUBJECTS
                .get(hop + 1)
                .copied()
                .unwrap_or("agt_00000000-0000-7000-8000-000000000104"),
            "dep": hop + 1,
            "iss": "https://authority.example.invalid/tenants/ten_0000000000000001",
            "aud": "https://api.example.invalid/v1/commands",
            "iat": 1_789_000_000u64,
            "exp": 1_789_003_600u64,
            "jti": "tok_00000000-0000-7000-8000-00000000010f",
        });
        let header = serde_json::to_vec(&header).unwrap();
        let payload = serde_json::to_vec(&payload).unwrap();
        // The signature bytes themselves are not a valid signature — only their
        // LENGTH is load-bearing for a size measurement, and the length is the
        // algorithm's. The signing input is still really derived from the real
        // header and payload, so the token is a complete structure rather than
        // a number added to the other shape's total.
        let signing_input = [header.as_slice(), b".", payload.as_slice()].concat();
        let digest = Sha256::digest(&signing_input);
        let signature: Vec<u8> = digest
            .iter()
            .copied()
            .cycle()
            .take(signature_bytes)
            .collect();
        // `header.payload.signature`, plus the two dots.
        total +=
            base64url_len(&header) + 1 + base64url_len(&payload) + 1 + base64url_len(&signature);
    }
    total
}

/// The measurement. It asserts the recorded numbers so ADR-009 and this
/// instrument cannot drift: if an envelope field changes, this fails and the
/// ADR's table must be updated with it.
#[test]
fn the_two_delegation_representations_are_measured_at_depths_one_to_three() {
    let baseline = baseline_bytes();
    assert_eq!(baseline, 308, "the undelegated baseline request");

    let mut table = Vec::new();
    for depth in 1..=3 {
        let envelope = envelope_shape_bytes(depth);
        let token_ec = token_bytes(depth, ASYMMETRIC_SIGNATURE_BYTES);
        let token_hs = token_bytes(depth, 32);
        table.push((depth, envelope, token_ec, token_hs));
        println!(
            "depth {depth}: envelope {envelope} B (+{} delegation) | \
             token ES256 {token_ec} B (+{}) | token HS256 {token_hs} B (+{})",
            envelope - baseline,
            token_ec - baseline,
            token_hs - baseline,
        );
    }

    // The recorded measurement. These are the numbers ADR-009 now carries.
    // The recorded measurement. These are the numbers ADR-009 now carries, and
    // asserting them keeps the ADR and this instrument from drifting: change an
    // envelope field and this fails, in the commit that changed it.
    assert_eq!(
        table,
        vec![
            (1, 505, 1_066, 1_023),
            (2, 684, 1_824, 1_738),
            (3, 861, 2_582, 2_453),
        ],
        "the measured sizes changed — re-measure and update ADR-009's table \
         in the same commit (SIGNOFF-REPAIR.3.4.5)"
    );

    // The per-hop cost, which is what actually answers "does this scale with
    // depth" — and which the withdrawn fixed-increment claim could not have
    // produced. A token is a constant cost per hop because each hop needs its
    // own complete credential; the envelope's chain rows share one container.
    let token_ec_step = table[1].2 - table[0].2;
    let token_hs_step = table[1].3 - table[0].3;
    assert_eq!((token_ec_step, table[2].2 - table[1].2), (758, 758));
    assert_eq!((token_hs_step, table[2].3 - table[1].3), (715, 715));
    // 2 -> 3 is one more `AuthorityContext` inside an array that already exists.
    // 1 -> 2 costs 2 B more because the container itself changes from an object
    // to an array, which is an artifact of the prototype, not of the design.
    assert_eq!(table[2].1 - table[1].1, 177);
    assert_eq!(table[1].1 - table[0].1, 179);
}

/// The structural finding the size table cannot show, asserted so it cannot be
/// quietly forgotten: the SHIPPED envelope cannot carry a chain at all. ADR-009
/// is titled "chain-in-envelope", and what exists is one delegated subject.
#[test]
fn the_shipped_envelope_carries_one_hop_not_a_chain() {
    let mut envelope = baseline_envelope();
    envelope.authority_context = Some(authority_context(SUBJECTS[0]));
    let wire = serde_json::to_value(&envelope).unwrap();
    let context = &wire["authority_context"];
    assert!(
        context.is_object() && !context.is_array(),
        "authority_context is a single object; a depth-2 chain has nowhere to go"
    );
    assert!(
        context["on_behalf_of"].is_string(),
        "one subject, as a string — not a list of hops"
    );
    // And the depth-2/3 rows above are therefore prototype-vs-prototype, not
    // shipped-vs-prototype. The leaf says so; this pins it.
    let round_tripped: CommandEnvelope = serde_json::from_value(wire).unwrap();
    assert!(round_tripped.authority_context.is_some());
}
