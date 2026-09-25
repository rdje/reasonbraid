//! The R1 evidence snapshot's STORE side (`SIGNOFF-REPAIR.11.24.1.3.2`,
//! ROADMAP §12.6 + §12.9, `migrations/0080`).
//!
//! §12.9 permits a snapshot to *remain addressable for the charter's audit
//! period **or** retain a verifiable external archival reference*. A git
//! acquisition takes the second alternative, and not as a convenience: the
//! object database the acquisition produces is built by the REMOTE's packing
//! configuration, so the same immutable commit yields different bytes across an
//! upstream repack. That measurement lives beside the acquisition, in
//! `git::tests::the_acquired_object_database_is_not_a_stable_identity`, because
//! only there can two REAL acquisitions run — production R1 is https-only by
//! `harden_git_url`, so no integration test can drive one end to end without
//! weakening an SSRF-relevant allowlist.
//!
//! ⭐ **What that leaves for here is everything the store decides**, and the
//! seam between the two halves is `git::external_snapshot_submission`, which
//! both use: the identity it derives is asserted there against real
//! acquisitions, and the behaviour that identity buys is asserted here against
//! real PostgreSQL. Nothing between them is simulated.
//!
//! Measured here:
//!   - the snapshot lands in the `external-reference` class, holds NO bytes,
//!     and carries the resolved commit as §12.6's immutable source version;
//!   - a second acquisition of the same commit REPLAYS — one row, whatever the
//!     remote did to its packing between them;
//!   - a different commit on the same reference is a second snapshot, so the
//!     replay is an identity and not a swallow;
//!   - the row's shape is enforced by the database, not only by the writer:
//!     an inline submission in this class is refused BY NAME, and a hand-built
//!     row that mixes the two shapes is refused by the CHECK;
//!   - a PINNED reference is refused by name — a pin says *these exact bytes*
//!     and this class holds none;
//!   - an excerpt assessment against it is told the snapshot holds no bytes
//!     rather than that its own cited evidence does not exist.
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job. Without
//! `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::snapshots::{
    self, Citer, ExternalSnapshotSubmission, SnapshotError, SnapshotSubmission, EXTERNAL_REFERENCE,
};
use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

const LOCATOR: &str = "https://git.example.invalid/repo.git#main";
const COMMIT: &str = "8be5445a05aeb912c85c1257cdad3282cf2073b3";
const OTHER_COMMIT: &str = "1e35b82e05c6867b3eeef749cf71b50a93d4ead0";

static EVIDENCE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    EVIDENCE_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    pg_cleanup::delete_tables(
        &pool,
        &[
            "site_audit",
            "profile_versions",
            "agent_profiles",
            "outbox_delivery",
            "outbox",
            "node_events",
            "node_inbox_cursors",
            "node_inbox",
            "budget_reservations",
            "budget_ceilings",
            "spend_breakers",
            "administrative_effects",
            "node_enrollment_tokens",
            "authorization_records",
            "authority_grants",
            "enrollments",
            "enrollment_boundaries",
            "node_enroll_audit",
            "node_keys",
            "node_certificates",
            "server_ca",
            "node_leases",
            "runs",
            "incarnations",
            "node_ambiguous_attempts",
            "node_proof_nonces",
            "nodes",
            "hosts",
            "recruitment_panels",
            "recruitment_responses",
            "recruitment_offers",
            "recruitment_calls",
            "card_imports",
            "agent_roles",
            "human_principals",
            "evidence_citations",
            "claim_assessments",
            "derivations",
            "evidence_snapshots",
            "snapshot_objects",
            "reference_registrations",
            "resource_references",
            "quota_events",
            "usage_quotas",
            "federation_agreements",
            "cross_domain_receipts",
            "mcp_listen_state",
            "tenant_bootstrap_requests",
            "routing_recommendations",
            "routing_resolutions",
            "policy_reviews",
            "policy_outcomes",
            "policy_corrections",
            "policy_drift",
            "policy_publications",
            "policy_projections",
            "policy_approvals",
            "policy_decisions",
            "policy_proposals",
            "storm_refusals",
            "resolution_refusals",
            "tenants",
            "idempotency",
            "event_log",
            "aggregate_state",
        ],
    )
    .await
    .expect("purge checked fixture plan");
    Some(pool)
}

struct TestServer {
    addr: SocketAddr,
    _handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn start(pool: &PgPool) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().unwrap();
        let router = api_router(pool.clone());
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.expect("serve");
        });
        Self {
            addr,
            _handle: handle,
        }
    }

    fn base(&self) -> String {
        format!("http://{}", self.addr)
    }
}

/// An enrolled human and its tenant — the citer every submission below needs.
async fn citizen(client: &reqwest::Client, base: &str, name: &str) -> Citer {
    let response = client
        .post(format!("{base}/v1/enrollments"))
        .json(&json!({ "kind": "human", "name": name }))
        .send()
        .await
        .expect("enroll request");
    assert_eq!(response.status().as_u16(), 200, "the human enrols");
    let body: Value = response.json().await.expect("enroll json");
    Citer {
        tenant_id: body["tenant_id"].as_str().unwrap().to_owned(),
        principal: body["principal_id"].as_str().unwrap().to_owned(),
    }
}

/// Register a §12.1 reference as that principal, so the registration predicate
/// `submit_external` asks is satisfied the way production satisfies it.
async fn reference(
    client: &reqwest::Client,
    base: &str,
    citer: &Citer,
    locator: &str,
    pin: Option<&str>,
) -> String {
    let mut body = json!({ "original_locator": locator, "scheme": "git" });
    if let Some(pin) = pin {
        body["expected_digest"] = json!(pin);
    }
    let response = client
        .post(format!("{base}/v1/resources"))
        .header(PRINCIPAL_HEADER, &citer.principal)
        .json(&body)
        .send()
        .await
        .expect("resource request");
    let status = response.status().as_u16();
    let submitted: Value = response.json().await.expect("resource json");
    assert_eq!(status, 200, "the reference registers: {submitted}");
    submitted["resource_id"].as_str().unwrap().to_owned()
}

/// The R1 submission, built by the SAME seam production uses, over a receipt
/// whose transport-dependent fields the caller varies.
///
/// `odb_digest` and `odb_bytes` stand in for what an upstream repack changes.
/// They must not touch the identity, and the controls below assert that by
/// varying them between two submissions of one commit.
fn submission(
    reference_id: &str,
    locator: &str,
    commit: &str,
    odb_digest: &str,
    odb_bytes: u64,
) -> ExternalSnapshotSubmission {
    let receipt = reasonbraid_server::git::GitReceipt {
        resolved_commit: commit.to_owned(),
        requested_url: locator.to_owned(),
        requested_ref: "main".to_owned(),
        digest: odb_digest.to_owned(),
        chain: vec![locator.to_owned()],
        object_count: 66,
        file_count: 64,
        max_path_depth: 1,
        odb_bytes,
        manifest: reasonbraid_server::git::GitManifest {
            included: vec!["a.txt".to_owned()],
            excluded: Vec::new(),
        },
        acquired_at: chrono::Utc::now(),
    };
    reasonbraid_server::git::external_snapshot_submission(reference_id, locator, &receipt)
}

/// The class, the absent bytes, the immutable source version — and a
/// re-acquisition across an upstream repack that does NOT duplicate the row.
#[tokio::test]
async fn the_git_snapshot_is_an_external_reference_keyed_on_its_commit() {
    let _guard = guard().await;
    let Some(pool) = pool().await else {
        return;
    };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let citer = citizen(&client, &base, "git-evidence-owner").await;
    let reference_id = reference(&client, &base, &citer, LOCATOR, None).await;

    let first = snapshots::submit_external(
        &pool,
        &submission(&reference_id, LOCATOR, COMMIT, "sha256:aa", 6857),
        chrono::Utc::now(),
        &citer,
    )
    .await
    .expect("the first acquisition files a snapshot");
    assert!(!first.replay, "the first submission is a fresh row");

    let stored = snapshots::get_for_tenant(&pool, &first.snapshot_id, &citer.tenant_id)
        .await
        .expect("the read succeeds")
        .expect("the citing tenant reads its own snapshot");
    assert_eq!(stored.storage_class, EXTERNAL_REFERENCE);
    assert_eq!(
        stored.raw_digest, None,
        "the store holds no bytes for this class"
    );
    assert_eq!(
        stored.byte_length, 0,
        "and says so in the length the CHECK pins to it"
    );
    assert_eq!(
        stored.immutable_source_version.as_deref(),
        Some(COMMIT),
        "§12.6's immutable source version is the resolved commit"
    );
    assert_eq!(stored.resolver_id, "r1-git-fetcher");
    assert_eq!(stored.media_type, "application/x-git-repository");
    let reference_json = stored
        .external_reference
        .as_ref()
        .expect("the external archival reference is recorded");
    assert_eq!(reference_json["kind"], json!("git-commit"));
    assert_eq!(reference_json["remote"], json!(LOCATOR));
    assert_eq!(reference_json["resolved_commit"], json!(COMMIT));
    assert_eq!(
        reference_json.get("digest"),
        None,
        "the odb digest is NOT in the reference — it is not stable across acquisitions"
    );
    assert_eq!(
        stored.provider_receipt["git"]["digest"],
        json!("sha256:aa"),
        "it is in the provider receipt, as the record of ONE acquisition"
    );

    // ⛔ THE RE-ACQUISITION, with the two fields an upstream repack changes
    // changed. Same commit, so it must be the SAME snapshot.
    let again = snapshots::submit_external(
        &pool,
        &submission(&reference_id, LOCATOR, COMMIT, "sha256:bb", 10071),
        chrono::Utc::now(),
        &citer,
    )
    .await
    .expect("the re-acquisition succeeds");
    assert!(again.replay, "a re-acquisition of the same commit replays");
    assert_eq!(
        again.snapshot_id, first.snapshot_id,
        "and replays onto the same row, though the object database differed"
    );

    // ⭐ THE POSITIVE CONTROL for that replay: a DIFFERENT commit on the same
    // reference is a second snapshot. Without it, a writer that silently
    // swallowed every submission would pass the assertion above.
    let moved_on = snapshots::submit_external(
        &pool,
        &submission(&reference_id, LOCATOR, OTHER_COMMIT, "sha256:aa", 6857),
        chrono::Utc::now(),
        &citer,
    )
    .await
    .expect("a new commit files a new snapshot");
    assert!(!moved_on.replay, "a different commit is not a replay");
    assert_ne!(moved_on.snapshot_id, first.snapshot_id);

    let rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM evidence_snapshots WHERE reference_id = $1 AND storage_class = $2",
    )
    .bind(&reference_id)
    .bind(EXTERNAL_REFERENCE)
    .fetch_one(&pool)
    .await
    .expect("count the reference's snapshots");
    assert_eq!(rows, 2, "three acquisitions, two commits, two rows");
}

/// `SIGNOFF-REPAIR.7.4.6`: a commit whose acquisition was TOMBSTONED, acquired
/// again, is a new row — the replay used to hand back the dead one, and the
/// unique identity index (`0080`) would have forced it to. A third acquisition
/// then replays onto the NEW row, so identity still holds among live rows.
#[tokio::test]
async fn a_tombstoned_commit_acquired_again_is_a_new_row() {
    let _guard = guard().await;
    let Some(pool) = pool().await else {
        return;
    };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let citer = citizen(&client, &base, "git-evidence-retired").await;
    let reference_id = reference(&client, &base, &citer, LOCATOR, None).await;

    let first = snapshots::submit_external(
        &pool,
        &submission(&reference_id, LOCATOR, COMMIT, "sha256:aa", 6857),
        chrono::Utc::now(),
        &citer,
    )
    .await
    .expect("the first acquisition files a snapshot");
    const REASON: &str = "the remote was force-pushed";
    assert!(
        snapshots::tombstone(&pool, &first.snapshot_id, REASON)
            .await
            .expect("tombstone the acquisition"),
        "the first acquisition is tombstoned"
    );

    let again = snapshots::submit_external(
        &pool,
        &submission(&reference_id, LOCATOR, COMMIT, "sha256:bb", 10071),
        chrono::Utc::now(),
        &citer,
    )
    .await
    .expect("the commit acquired again files a snapshot");
    assert!(!again.replay, "not a replay of the tombstoned row");
    assert_ne!(
        again.snapshot_id, first.snapshot_id,
        "a new acquisition never re-cites the tombstoned row"
    );
    let retired = snapshots::get_for_tenant(&pool, &first.snapshot_id, &citer.tenant_id)
        .await
        .expect("the read succeeds")
        .expect("the citer still reads the tombstoned row");
    assert!(retired.deleted_at.is_some(), "the tombstone stands");

    let third = snapshots::submit_external(
        &pool,
        &submission(&reference_id, LOCATOR, COMMIT, "sha256:cc", 7000),
        chrono::Utc::now(),
        &citer,
    )
    .await
    .expect("a third acquisition succeeds");
    assert!(third.replay, "identity holds among live rows");
    assert_eq!(third.snapshot_id, again.snapshot_id);

    let rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM evidence_snapshots WHERE reference_id = $1 AND storage_class = $2",
    )
    .bind(&reference_id)
    .bind(EXTERNAL_REFERENCE)
    .fetch_one(&pool)
    .await
    .expect("count the reference's snapshots");
    assert_eq!(rows, 2, "one retired row and one live row for the commit");
}

/// The shape is the DATABASE's rule, not the writer's good manners — and the
/// inline surface refuses the class by name rather than by constraint fault.
#[tokio::test]
async fn the_storage_class_shape_is_enforced_by_the_row() {
    let _guard = guard().await;
    let Some(pool) = pool().await else {
        return;
    };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let citer = citizen(&client, &base, "git-evidence-shape").await;
    let reference_id = reference(&client, &base, &citer, LOCATOR, None).await;

    // The inline surface, handed the external class.
    let bytes = b"not the repository";
    let inline = SnapshotSubmission {
        reference_id: reference_id.clone(),
        original_locator: LOCATOR.to_owned(),
        final_locator: LOCATOR.to_owned(),
        resolver_id: "r1-git-fetcher".to_owned(),
        resolver_version: "0.1.0".to_owned(),
        network_class: "public".to_owned(),
        auth_class: "none".to_owned(),
        provider_receipt: json!({}),
        immutable_source_version: Some(COMMIT.to_owned()),
        raw_digest: reasonbraid_server::fetcher::digest_sha256_hex(bytes),
        byte_length: bytes.len() as i64,
        media_type: "application/x-git-repository".to_owned(),
        storage_class: EXTERNAL_REFERENCE.to_owned(),
        retention_class: "standard".to_owned(),
        extraction_version: None,
        quarantine_status: "none".to_owned(),
        redactions: json!([]),
        disclosure_policy: json!({}),
        license: None,
        fresh_until: None,
    };
    let refused = snapshots::submit(&pool, &inline, bytes, chrono::Utc::now(), &citer).await;
    assert!(
        matches!(refused, Err(SnapshotError::StorageClassNotInline)),
        "the inline surface refuses the external class by name, not as a store fault: {refused:?}"
    );

    // ⭐ THE POSITIVE CONTROL: the SAME submission in the default class lands,
    // so the refusal above is about the class and not about the fixture.
    let allowed = SnapshotSubmission {
        storage_class: "standard".to_owned(),
        ..inline
    };
    snapshots::submit(&pool, &allowed, bytes, chrono::Utc::now(), &citer)
        .await
        .expect("the same submission in the default class lands");

    // And the row's own CHECK, reached past the writer entirely: a hand-built
    // external-reference row that also claims inline bytes is refused.
    let mixed = sqlx::query(
        "INSERT INTO evidence_snapshots \
         (snapshot_id, reference_id, original_locator, final_locator, retrieved_at, \
          resolver_id, resolver_version, immutable_source_version, raw_digest, \
          external_reference, byte_length, media_type, storage_class) \
         VALUES ('snp_mixed', $1, $2, $2, now(), 'r1-git-fetcher', '0.1.0', $3, $4, \
                 '{}'::jsonb, 0, 'application/x-git-repository', $5)",
    )
    .bind(&reference_id)
    .bind(LOCATOR)
    .bind(COMMIT)
    .bind(reasonbraid_server::fetcher::digest_sha256_hex(bytes))
    .bind(EXTERNAL_REFERENCE)
    .execute(&pool)
    .await;
    assert!(
        mixed.is_err(),
        "the CHECK refuses a row that is both classes at once"
    );

    // ⛔ And an external-reference row with no immutable source version has no
    // identity, so the CHECK refuses that too.
    let identityless = sqlx::query(
        "INSERT INTO evidence_snapshots \
         (snapshot_id, reference_id, original_locator, final_locator, retrieved_at, \
          resolver_id, resolver_version, external_reference, byte_length, media_type, \
          storage_class) \
         VALUES ('snp_noident', $1, $2, $2, now(), 'r1-git-fetcher', '0.1.0', \
                 '{}'::jsonb, 0, 'application/x-git-repository', $3)",
    )
    .bind(&reference_id)
    .bind(LOCATOR)
    .bind(EXTERNAL_REFERENCE)
    .execute(&pool)
    .await;
    assert!(
        identityless.is_err(),
        "the CHECK refuses an external-reference row with no immutable source version"
    );
}

/// A PINNED reference cannot be satisfied by evidence this store holds no bytes
/// of, and the refusal says so instead of filing a snapshot that escapes it.
#[tokio::test]
async fn a_pinned_reference_refuses_an_external_reference_snapshot() {
    let _guard = guard().await;
    let Some(pool) = pool().await else {
        return;
    };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let citer = citizen(&client, &base, "git-evidence-pin").await;
    let pin = format!("sha256:{}", "a".repeat(64));
    let pinned = reference(&client, &base, &citer, LOCATOR, Some(&pin)).await;

    let refused = snapshots::submit_external(
        &pool,
        &submission(&pinned, LOCATOR, COMMIT, "sha256:aa", 6857),
        chrono::Utc::now(),
        &citer,
    )
    .await;
    assert!(
        matches!(refused, Err(SnapshotError::PinnedReferenceHoldsNoBytes)),
        "a pin over bytes this class does not hold is refused by name: {refused:?}"
    );

    // ⭐ THE POSITIVE CONTROL: the same submission against an UNPINNED
    // reference to the same repository lands, so the refusal is the pin's and
    // not the submission's.
    let unpinned = reference(
        &client,
        &base,
        &citer,
        "https://git.example.invalid/repo.git#release",
        None,
    )
    .await;
    snapshots::submit_external(
        &pool,
        &submission(
            &unpinned,
            "https://git.example.invalid/repo.git#release",
            COMMIT,
            "sha256:aa",
            6857,
        ),
        chrono::Utc::now(),
        &citer,
    )
    .await
    .expect("an unpinned reference accepts it");
}

/// An excerpt assessment over a repository snapshot is told WHY it cannot be
/// checked. Before `migrations/0080` the join could only say "does not exist",
/// which for a tenant reading its own cited evidence is false.
#[tokio::test]
async fn an_excerpt_against_a_byteless_snapshot_is_told_what_it_hit() {
    let _guard = guard().await;
    let Some(pool) = pool().await else {
        return;
    };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let citer = citizen(&client, &base, "git-evidence-excerpt").await;
    let reference_id = reference(&client, &base, &citer, LOCATOR, None).await;
    let filed = snapshots::submit_external(
        &pool,
        &submission(&reference_id, LOCATOR, COMMIT, "sha256:aa", 6857),
        chrono::Utc::now(),
        &citer,
    )
    .await
    .expect("the snapshot files");

    let response = client
        .post(format!("{base}/v1/assessments"))
        .header(PRINCIPAL_HEADER, &citer.principal)
        .json(&json!({
            "claim_id": "clm_git_evidence",
            "snapshot_id": filed.snapshot_id,
            "assessment": "supports",
            "excerpt": "anything at all",
            "rationale": "the excerpt cannot be checked against a repository this store does not hold",
        }))
        .send()
        .await
        .expect("assessment request");
    let status = response.status().as_u16();
    let body: Value = response.json().await.expect("assessment json");
    let message = body.to_string();
    assert_eq!(status, 400, "the assessment is refused: {message}");
    assert!(
        message.contains(EXTERNAL_REFERENCE) && message.contains("holds none"),
        "and the refusal names the class rather than claiming the snapshot is absent: {message}"
    );
    assert!(
        !message.contains("does not exist"),
        "the tenant is never told its own cited evidence is absent: {message}"
    );
}
