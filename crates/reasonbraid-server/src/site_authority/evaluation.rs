//! The evaluation harness's writes as site acts (`SIGNOFF-REPAIR.8.2.5`).
//!
//! All seven `evaluation_*` tables carry no tenant column, and every write
//! admitted on bare enrolment — `reader_tenant(...).is_some()` and nothing
//! else, with no principal reaching the service. So any enrolled principal in
//! the deployment set the standard the whole site measured against, and read
//! every other caller's records.
//!
//! ⛔ **The absent tenant column is the DESIGN, not the defect.**
//! `docs/decisions/2026-09-19_the-policy-registry-is-a-shared-control-surface.md`
//! records DOC-0029's verdict for this exact family: site-wide by design, gated
//! by SITE-OPERATOR grants. ROADMAP §19 is release engineering — gate records,
//! corpus versions, CI manifests — not a tenant product surface. What was never
//! applied is the ruled gate.
//!
//! ⭐ **TWO ACTIONS, DERIVED RATHER THAN PREFERRED.** ROADMAP §4.1's charter
//! names separation-of-duties, and §19's release flow has two parties: one
//! maintains the corpus, the runs and the baselines — the STANDARD — while CI
//! evaluates gates against it continuously. A party that measures against a
//! standard must not be able to move the standard, so
//! [`Action::EvaluationRecord`] covers the six standard-setting writes and
//! [`Action::GateEvaluate`] covers running a gate.
//!
//! ⚠️ **VALIDATION RUNS BEFORE THE GATE, IN THE HTTP LAYER**, and the writes
//! re-validate regardless (`SIGNOFF-REPAIR.8.2.5.2`). A refusal here is rendered
//! as an authorization reason, so an input error wrapped in a site act would be
//! a 403 about authority for a 400 about input. This is the contract
//! [`super::workflows`] states in its own words and [`super::policies`] follows.

use super::*;

/// Map a domain failure onto the closure's two-level result.
///
/// ⛔ **THE OUTER `Err` IS NOT A DECISION.** A store fault proves nothing about
/// the caller, so it must roll the act back with NOTHING audited rather than
/// leave a permanent site-audit row asserting a refusal nobody made. That
/// distinction only became expressible when `SIGNOFF-REPAIR.8.2.2` gave
/// [`EvaluationError`] its `Storage` variant — before it, every failure in this
/// module was one undifferentiated `Duplicate`.
///
/// ⚠️ The refusal strings are `&'static str` by `authorized`'s signature, so
/// they name the CLASS. The specific message is not lost: validation runs ahead
/// of the gate and keeps its typed 400, and what remains here is exactly the set
/// of refusals that need the database to decide.
fn refused(
    error: crate::evaluation::EvaluationError,
    taken: &'static str,
    absent: &'static str,
) -> Result<Result<Effect, &'static str>, Error> {
    use crate::evaluation::EvaluationError::*;
    Ok(Err(match error {
        Storage(cause) => return Err(cause.into()),
        // The identity this act asked for is already held.
        Duplicate(_) => taken,
        // Something the act NAMED is not registered, or is not eligible.
        UnknownCorpus(_) | NotRegistered(_) => absent,
        // ⚠️ Reachable only for the refusals that need the stored row — an
        // empty score intersection, or a corrupt stored baseline. Every other
        // malformed input is refused by the pre-gate validator
        // (`SIGNOFF-REPAIR.8.2.5.2`) and never reaches a site act.
        MalformedDigest(_) | Invalid(_) | UndeclaredSeed | InvalidTrialCount(_) => {
            "the request cannot be answered against the stored record"
        }
    }))
}

/// One written row, as the audit's effect.
fn wrote<T: serde::Serialize>(value: &T) -> Effect {
    Effect::write(
        serde_json::to_value(value).expect("the evaluation record serializes"),
        1,
    )
}

macro_rules! site_write {
    (
        $(#[$meta:meta])*
        $name:ident,
        $action:expr,
        $registry:literal,
        $taken:literal,
        $absent:literal,
        |$conn:ident| $call:expr,
        $($arg:ident : $ty:ty),* $(,)?
    ) => {
        $(#[$meta])*
        pub async fn $name(
            pool: &PgPool,
            subject: &GrantSubject,
            $($arg: $ty,)*
            reason: &Reason,
        ) -> Result<Receipt, Error> {
            $(let $arg = $arg.to_owned();)*
            authorized(
                pool,
                subject,
                $action,
                json!({ "registry": $registry }),
                reason.as_str(),
                move |$conn, _at| {
                    $(let $arg = $arg.clone();)*
                    Box::pin(async move {
                        match $call {
                            Ok(row) => Ok(Ok(wrote(&row))),
                            Err(error) => refused(error, $taken, $absent),
                        }
                    })
                },
            )
            .await
        }
    };
}

site_write!(
    /// Register a corpus version as a site act.
    register_corpus,
    Action::EvaluationRecord,
    "evaluation_corpora",
    "that corpus version is already registered",
    "the corpus registry refused this registration",
    |conn| crate::evaluation::register_corpus(conn, &registration).await,
    registration: &crate::evaluation::CorpusRegistration,
);

site_write!(
    /// Record a run as a site act.
    record_run,
    Action::EvaluationRecord,
    "evaluation_runs",
    "that run id is already recorded",
    "the named corpus version is not registered",
    |conn| crate::evaluation::record_run(conn, &run).await,
    run: &crate::evaluation::RunRecord,
);

site_write!(
    /// Create a shadow trial as a site act.
    create_trial,
    Action::EvaluationRecord,
    "evaluation_trials",
    "that trial id already exists",
    "the named corpus version is not registered",
    |conn| crate::evaluation::create_trial(conn, &submission).await,
    submission: &crate::evaluation::TrialSubmission,
);

site_write!(
    /// Record a calibration as a site act.
    record_calibration,
    Action::EvaluationRecord,
    "evaluation_calibrations",
    "that calibration id already exists",
    "a named run is not registered, or was taken against another corpus version or workflow",
    |conn| crate::evaluation::record_calibration(conn, &submission).await,
    submission: &crate::evaluation::CalibrationSubmission,
);

site_write!(
    /// Record a gate as a site act.
    record_gate,
    Action::EvaluationRecord,
    "evaluation_gates",
    "that gate id is already registered",
    "the named corpus version is not registered",
    |conn| crate::evaluation::record_gate(conn, &submission).await,
    submission: &crate::evaluation::GateSubmission,
);

/// Append a trial's results as a site act.
///
/// ⚠️ Written out rather than through the macro because its inner call returns
/// `()` — there is no row to serialize — so its effect is the trial it appended
/// to rather than the value it wrote.
pub async fn record_trial_results(
    pool: &PgPool,
    subject: &GrantSubject,
    trial_id: &str,
    results: &Value,
    reason: &Reason,
) -> Result<Receipt, Error> {
    let trial_id = trial_id.to_owned();
    let results = results.clone();
    authorized(
        pool,
        subject,
        Action::EvaluationRecord,
        json!({ "registry": "evaluation_trial_results", "trial_id": trial_id }),
        reason.as_str(),
        move |conn, _at| {
            let trial_id = trial_id.clone();
            let results = results.clone();
            Box::pin(async move {
                match crate::evaluation::record_trial_results(conn, &trial_id, &results).await {
                    Ok(()) => Ok(Ok(Effect::write(json!({ "trial_id": trial_id }), 1))),
                    Err(error) => refused(
                        error,
                        "that trial's results are already recorded",
                        "the results append to a REGISTERED trial",
                    ),
                }
            })
        },
    )
    .await
}

/// Evaluate a gate as a site act — the MEASURING capability.
///
/// ⭐ [`Action::GateEvaluate`], not [`Action::EvaluationRecord`]: a party that
/// measures against a standard must not be able to move the standard.
pub async fn evaluate_gate(
    pool: &PgPool,
    subject: &GrantSubject,
    gate_id: &str,
    scores: &Value,
    reason: &Reason,
) -> Result<Receipt, Error> {
    let gate_id = gate_id.to_owned();
    let scores = scores.clone();
    authorized(
        pool,
        subject,
        Action::GateEvaluate,
        json!({ "registry": "evaluation_gate_results", "gate_id": gate_id }),
        reason.as_str(),
        move |conn, _at| {
            let gate_id = gate_id.clone();
            let scores = scores.clone();
            Box::pin(async move {
                match crate::evaluation::evaluate_gate(conn, &gate_id, &scores).await {
                    Ok(row) => Ok(Ok(wrote(&row))),
                    Err(error) => refused(
                        error,
                        "that gate result is already recorded",
                        "that gate is not registered",
                    ),
                }
            })
        },
    )
    .await
}
