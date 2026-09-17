//! The thread command domain for the WP6 control API (`PHASE-0.6.1`).
//!
//! # What this module owns
//!
//! The operation catalogue the CLI drives — `thread.create`, `thread.invite`,
//! `thread.contribute`, `thread.challenge`, `thread.revise`, `thread.close` — with
//! typed, `deny_unknown_fields` bodies, a minimal [`ThreadProjection`] stored in
//! `aggregate_state.state`, and deterministic validation against the core state
//! machines (`reasonbraid_core::state`). The HTTP layer (`crate::api`) resolves the
//! actor, authorizes, and commits; this module decides WHAT a command means and
//! whether the aggregate accepts it.
//!
//! # Dev-profile rules (recorded in `docs/decisions/2026-09-06_control-api-cli.md`)
//!
//! - **Challenge and revise are `thread_contribute` grants** — the WP5 action registry
//!   stays frozen; the three content verbs differ in message kind and event type, not
//!   in authority (`THREAD-004`: typed messages/contributions).
//! - **Auto-accept on first contribution** — `invited → accepted` when the invited
//!   role first contributes; the demo has no explicit accept verb.
//! - **Close folds `open → closing → closed`** into one command: both core-machine
//!   edges are validated, one `thread.closed` event records the terminal state.
//! - **The creator is a participant** from creation, so the organizer can contribute
//!   and challenge.
//! - **Content verbs require the thread to be `open`** and the actor to be a
//!   participant in `invited` or `accepted` state.
//!
//! # The projection
//!
//! The projection keeps only what validation and inspection need (state,
//! participants, counters, close reason, ceiling id, budget); the event log carries
//! the full content. `inspect` reconstructs the timeline from the log — never from
//! SQLite/psql surgery (the `.6.1` acceptance).

use std::collections::BTreeMap;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use reasonbraid_core::{
    AgentRoleId, BudgetDimensions, EventId, ParticipationState, TenantId, ThreadId, ThreadState,
    ThreadTransition, TransitionError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Postgres;

pub const AGGREGATE_TYPE: &str = "thread";

/// Operation names (the `CommandEnvelope.operation` values this module accepts).
/// For each citation, the index of the FIRST citation naming the same §12.1
/// reference — its `(uri, digest)` PAIR (`SIGNOFF-REPAIR.11.14.3.7`).
///
/// A citation whose answer is its own index is registered; every other one
/// reuses the `resource_id` that first registration returned. ⭐ De-duplicating
/// the WORK is derived rather than chosen: a reference's identity is that pair
/// (`.11.14.3.2`), so a repeat can only fetch back the row the first citation
/// just wrote.
///
/// ⛔ The RECORD is not de-duplicated, and this function is why that is
/// structural: it returns one answer per citation, so the caller emits one
/// `RegisteredEvidenceRef` per citation, in order, with its own note.
///
/// ⚠️ Keyed on the PAIR, never on the locator. One locator at two digests is two
/// references (§12.6's changed page), so collapsing them here would erase the
/// distinction `.11.14.3.2` exists to keep.
///
/// It is a free function so it can be falsified directly. The live control can
/// see the O(n) over DISTINCT citations, but nothing the product exposes reveals
/// how many times the store was reached for a repeated one — the row count is 1
/// either way, because the pair replay already returns the existing row. A
/// PostgreSQL scan-counter instrument was tried and discarded: it read the same
/// value with and without the de-duplication, so its assertion could not fail.
fn first_citation_of_each_pair(citations: &[EvidenceRef]) -> Vec<usize> {
    let mut first: std::collections::HashMap<(&str, Option<&str>), usize> =
        std::collections::HashMap::with_capacity(citations.len());
    citations
        .iter()
        .enumerate()
        .map(|(index, citation)| {
            let pair = (citation.uri.as_str(), citation.digest.as_deref());
            *first.entry(pair).or_insert(index)
        })
        .collect()
}

pub const OP_CREATE: &str = "thread.create";
pub const OP_INVITE: &str = "thread.invite";
pub const OP_CONTRIBUTE: &str = "thread.contribute";
pub const OP_CHALLENGE: &str = "thread.challenge";
pub const OP_REVISE: &str = "thread.revise";
pub const OP_CLOSE: &str = "thread.close";
pub const OP_CANCEL: &str = "thread.cancel";
pub const OP_ACCEPT_INVITATION: &str = "thread.accept_invitation";
pub const OP_DECLINE_INVITATION: &str = "thread.decline_invitation";
pub const OP_REMOVE_PARTICIPANT: &str = "thread.remove_participant";
pub const OP_JOIN: &str = "thread.join";
pub const OP_ADVANCE_ROUND: &str = "thread.advance_round";

/// The event types committed for the operations above.
pub const EVENT_CREATED: &str = "thread.created";
pub const EVENT_INVITED: &str = "thread.participant_invited";
pub const EVENT_CONTRIBUTED: &str = "thread.contribution_submitted";
pub const EVENT_CHALLENGED: &str = "thread.challenge_posted";
pub const EVENT_REVISED: &str = "thread.revision_submitted";
pub const EVENT_CLOSED: &str = "thread.closed";
pub const EVENT_CANCELLED: &str = "thread.cancelled";
pub const EVENT_INVITATION_ACCEPTED: &str = "thread.invitation_accepted";
pub const EVENT_INVITATION_DECLINED: &str = "thread.invitation_declined";
pub const EVENT_PARTICIPANT_REMOVED: &str = "thread.participant_removed";
pub const EVENT_JOINED: &str = "thread.participant_joined";
pub const EVENT_ROUND_ADVANCED: &str = "thread.round_advanced";

/// The thread-work kinds an inbox payload carries (`PHASE-0.6.2`): an invitation
/// dispatches a `contribute` work item to the invited role's node; a challenge of a
/// role's contribution dispatches a `revise` work item to that role's node. The
/// node's journal treats exactly these kinds as thread work.
pub const WORK_CONTRIBUTE: &str = "contribute";
pub const WORK_REVISE: &str = "revise";

/// The reservation dimensions one work item requests (a single provider call with
/// a modest token and wall-clock allowance — covered by [`DEFAULT_BUDGET`]).
pub const WORK_RESERVATION: BudgetDimensions = BudgetDimensions {
    calls: Some(1),
    input_tokens: Some(2_000),
    output_tokens: Some(2_000),
    wall_clock_seconds: Some(120),
};

/// The dev-profile default thread budget when a create body names none: every
/// dimension metered (the fail-closed `BudgetDimensions::covers` refuses requests for
/// dimensions a ceiling does not meter).
pub const DEFAULT_BUDGET: BudgetDimensions = BudgetDimensions {
    calls: Some(10),
    input_tokens: Some(10_000),
    output_tokens: Some(10_000),
    wall_clock_seconds: Some(600),
};

// ── Operation bodies (client-supplied intent) ────────────────────────────────────

/// Thread classification (`ROADMAP.md` §3.2's scope fields, the dev registry):
/// `general` is the default; `confidential` marks a thread whose provider use and
/// retention later phases tighten. Recorded and inspectable today — enforcement
/// arrives with the classification-aware policies (`PHASE-1.1.3`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    #[default]
    General,
    Confidential,
}

impl Classification {
    /// The wire name (the snake_case registry form).
    pub fn as_str(self) -> &'static str {
        match self {
            Classification::General => "general",
            Classification::Confidential => "confidential",
        }
    }

    /// The evaluator-access control (`.1.4.3`, the `.1.4.1` contract):
    /// whether the deployment's evaluator registry qualifies a profile for
    /// THIS classification. The dev registry qualifies its built-ins for
    /// `general` only — `confidential` has NO qualified evaluator, so the
    /// dispatch is the typed refusal (ADR-034: a classification without the
    /// controls is never a silent general). A future confidential-qualified
    /// profile flips this arm by joining the registry.
    pub fn has_qualified_evaluator(self) -> bool {
        matches!(self, Classification::General)
    }
}

/// The workflow profile (ADR-016, `PHASE-5.1.2`): the VALIDATED reference
/// to a registered profile — the id string, defaulting to `quick_advice`
/// at the create boundary.
/// Participant rules (`ROADMAP.md` §20.3: explicit participants first): explicit
/// invites on by default, join requests off — the trusted-LAN slice has no
/// request flow yet (`PHASE-1.1.3`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParticipantRules {
    #[serde(default = "default_true")]
    pub allow_explicit_invites: bool,
    #[serde(default)]
    pub allow_join_requests: bool,
}

fn default_true() -> bool {
    true
}

fn default_workflow_steps() -> Vec<String> {
    vec![
        "solicit".to_owned(),
        "synthesize".to_owned(),
        "decide".to_owned(),
    ]
}

impl Default for ParticipantRules {
    fn default() -> Self {
        ParticipantRules {
            allow_explicit_invites: true,
            allow_join_requests: false,
        }
    }
}

/// `thread.create` body: the tenant the thread belongs to, the subject/objective,
/// and the optional budget, classification, workflow profile, and participant
/// rules. The thread id itself is server-assigned.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateBody {
    pub tenant_id: TenantId,
    pub subject: String,
    pub objective: String,
    #[serde(default)]
    pub budget: Option<BudgetSpec>,
    #[serde(default)]
    pub classification: Option<Classification>,
    /// The validated workflow-profile reference (ADR-016: the id of a
    /// registered profile — the unknown id is the typed refusal at the
    /// create boundary).
    #[serde(default)]
    pub workflow_profile: Option<String>,
    /// The routing case class (`.5.2`, ADR-031): a submitted input the
    /// create boundary routes through the rule table ONLY when no explicit
    /// profile is named — the explicit profile always outranks the rule.
    #[serde(default)]
    pub routing_class: Option<String>,
    #[serde(default)]
    pub participant_rules: Option<ParticipantRules>,
}

/// A budget specification for a new thread; unspecified dimensions take the
/// [`DEFAULT_BUDGET`] value (all dimensions metered in the dev profile).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetSpec {
    #[serde(default)]
    pub calls: Option<u64>,
    #[serde(default)]
    pub input_tokens: Option<u64>,
    #[serde(default)]
    pub output_tokens: Option<u64>,
    #[serde(default)]
    pub wall_clock_seconds: Option<u64>,
}

/// `thread.invite` body: the target tenant + thread scope and the invited agent role.
/// `expires_in_seconds` is the typed optional invitation TTL (`.1.3.1`); `None` =
/// the invitation never expires — expiry is DERIVED from the recorded `expires_at`
/// at read/accept time, never swept.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InviteBody {
    pub tenant_id: TenantId,
    pub agent_role: String,
    #[serde(default)]
    pub expires_in_seconds: Option<i64>,
}

/// `thread.accept_invitation` body (`.1.3.1`): the actor IS the invited role —
/// the invitation (a pending record naming this actor) is the acceptance
/// capability; the scope field rides like every other command body.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptInvitationBody {
    pub tenant_id: TenantId,
}

/// `thread.decline_invitation` body (`.1.3.1`): same shape as the accept — the
/// invited role refuses the pending offer.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclineInvitationBody {
    pub tenant_id: TenantId,
}

/// `thread.join` body (`.1.3.2`): the actor is the joining role — a thread whose
/// `allow_join_requests` is on admits it as an ACCEPTED participant through the
/// self-request path (no invitation; the event records `via: "join_request"`).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JoinBody {
    pub tenant_id: TenantId,
}

/// `thread.advance_round` body (`.1.5.2`): the scope only — the new round is
/// SERVER-assigned (current + 1); the client never names one.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdvanceRoundBody {
    pub tenant_id: TenantId,
}

/// `thread.remove_participant` body (`.1.3.1`): a tenant_admin revokes one
/// participant (invited or accepted) — `revoked` in the projection, the event
/// names the removed principal.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoveParticipantBody {
    pub tenant_id: TenantId,
    pub participant: String,
}

/// `thread.contribute` body: the scope and the contribution content.
/// The structured contribution kinds (`ROADMAP.md` §8.5's initial message kinds a
/// CONTRIBUTION can carry, `PHASE-1.5.1`): `position` is the stated default — a
/// contribution without a `kind` IS a position. `evidence_reference` marks a
/// contribution whose whole point is a reference; ANY kind may carry
/// `evidence_refs` alongside it, and every one of them REGISTERS the §12.1
/// resource reference it names (`.11.14.3.2`, ROADMAP §13.2 step 2).
/// Out-of-registry values are typed refusals (deny-unknown at the body
/// boundary), never silently stored.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionKind {
    #[default]
    Position,
    Claim,
    Assumption,
    EvidenceReference,
    /// `.2.4.2` (ADR-029): the evidence request — targets ONE claim digest of
    /// THIS thread (the `target_claim_digest` body field) and is NOT an
    /// acquisition (no budget, no resolver authority).
    EvidenceRequest,
    Question,
    Summary,
    /// `.2.4.2` (ADR-029): the adjudication verdict — the attributable record
    /// (`verdict` body field: the judged digest + the rule + the §13.4
    /// outcome), legal on the `adjudicate` step only.
    Verdict,
    /// `.11.14.3.1` (ROADMAP §13.2 step 6): the evidence assessment — the
    /// §12.7 record linking ONE claim of this thread to ONE snapshot this
    /// tenant cited, legal on the `assess` step only. This is the step's
    /// payload; before it existed, `assess` was a step two shipped profiles
    /// declared and nothing could execute on.
    Assessment,
    /// `.3.2` (ADR-030): the moderation kinds — the CLOSED vocabulary. A
    /// moderation action is a contribution, never a new authority: the
    /// capability-shaped fields (verdict/claims/evidence_refs/target) are
    /// refused on these kinds, so the §13.5 prohibitions hold by
    /// construction.
    Classify,
    RequestClarification,
    ProposeClose,
    DraftSummary,
    IdentifyUnanswered,
}

impl ContributionKind {
    /// Whether this kind is one of the ADR-030 moderation kinds (the closed
    /// set — the capability-shaped fields refuse on them).
    pub fn is_moderation_kind(&self) -> bool {
        matches!(
            self,
            ContributionKind::Classify
                | ContributionKind::RequestClarification
                | ContributionKind::ProposeClose
                | ContributionKind::DraftSummary
                | ContributionKind::IdentifyUnanswered
        )
    }
}

/// One evidence reference attached to a contribution (`PHASE-1.5.1`): a URI, an
/// optional expected digest, and an optional human note. REFERENCES only —
/// accepting a reference is not a promise the core can resolve it, and
/// acquisition stays Phase 4 (`ROADMAP.md` §3.7).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRef {
    pub uri: String,
    /// Absent fields are OMITTED on the wire (never serialized as `null`) — the
    /// event body carries only what the contributor stated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// One evidence reference as the EVENT records it (`.11.14.3.2`, ROADMAP §13.2
/// step 2): what the contributor stated, plus the §12.1 `resource_references`
/// row the citation registered.
///
/// ⛔ Serialize-only, and a separate type rather than an optional field on
/// [`EvidenceRef`]: `resource_id` is the SERVER's, and a `deny_unknown_fields`
/// INPUT type carrying it would have let a caller supply one — the same reason
/// ADR-029 keeps a claim's digest off the wire.
#[derive(Debug, Clone, Serialize)]
pub struct RegisteredEvidenceRef {
    pub uri: String,
    /// Absent fields stay OMITTED, exactly as [`EvidenceRef`] serializes them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// The reference this citation resolves to. §13.2 registers it at step 2;
    /// step 6 acquires against it.
    pub resource_id: String,
}

/// `thread.contribute` body: the scope, the contribution content, the structured
/// kind (default `position`), the evidence references it cites, and (`.2.2`,
/// ADR-029) the structured claims riding the contribution.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContributeBody {
    pub tenant_id: TenantId,
    pub content: String,
    #[serde(default)]
    pub kind: ContributionKind,
    #[serde(default)]
    pub evidence_refs: Vec<EvidenceRef>,
    /// The ADR-029 structured claims: the client submits the CONTENT only —
    /// the digest is computed server-side and never trusted from the wire.
    #[serde(default)]
    pub claims: Vec<ClaimInput>,
    /// `.2.4.2`: the evidence request's target — ONE claim digest of THIS
    /// thread. Legal only on an `evidence_request`-kind contribution.
    #[serde(default)]
    pub target_claim_digest: Option<String>,
    /// `.2.4.2`: the adjudication verdict — legal only on a `verdict`-kind
    /// contribution.
    #[serde(default)]
    pub verdict: Option<VerdictInput>,
    /// `.11.14.3.1`: the assessment. Legal only on an `assessment`-kind
    /// contribution, on the `assess` step.
    #[serde(default)]
    pub assessment: Option<AssessmentInput>,
    /// `.3.2` (ADR-030): the moderated event's id — legal only on a
    /// moderation-kind contribution; the reference must exist in the thread.
    #[serde(default)]
    pub ref_event_id: Option<String>,
    /// `.3.3` (ADR-030): the synthesis record — legal only on a
    /// `summary`-kind contribution during the `synthesize` step.
    #[serde(default)]
    pub synthesis: Option<SynthesisInput>,
}

/// The synthesis record (`.3.3`, ADR-030): derived content, auditable by
/// construction — the synthesizer identity/configuration, the INPUT EVENT
/// RANGE (the event log's version range — the transformation is
/// re-derivable from the named events), the source links, and the coverage
/// report (the `.2.4.1` shapes, generalized). The synthesis never mutates a
/// prior event: it is itself an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SynthesisInput {
    pub synthesizer: String,
    pub input_from: i64,
    pub input_to: i64,
    pub sources: Vec<String>,
    pub coverage: Vec<CoverageItem>,
}

/// The adjudication verdict input (`.2.4.2`, ADR-029): the judged digest, the
/// decision rule applied, and the §13.4 outcome declared. The record is
/// attributable (the contribution's author rides the event) — never a silent
/// rewrite; the canonical outcome persists (the legacy aliases never do).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerdictInput {
    pub target_digest: String,
    pub rule: String,
    pub outcome: CloseOutcome,
}

/// One structured claim riding a contribution (`PHASE-5.2.2`, ADR-029): the
/// content + the SERVER-COMPUTED digest (the ADR-011 `sha256:<hex>` shape).
/// An objection names this digest; since the server derived it, a forged
/// digest simply fails the membership check.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ClaimRecord {
    pub content: String,
    pub digest: String,
}

/// One evidence assessment riding an `assess`-step contribution
/// (`SIGNOFF-REPAIR.11.14.3.1`, ROADMAP §12.7 + §13.2 step 6).
///
/// The claim is named by its SERVER-COMPUTED digest and checked for membership
/// of THIS thread, so the identifier cannot be invented — the defect
/// `SIGNOFF-REPAIR.11.14.2` measured on the standalone route. The snapshot must
/// be one this tenant CITED (`SIGNOFF-REPAIR.11.14.1`), and the excerpt must
/// appear in its acquired bytes, because §12.7 is explicit that citation
/// existence alone never satisfies an evidence gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssessmentInput {
    /// A claim digest of THIS thread.
    pub claim_digest: String,
    /// A snapshot this thread's tenant cited.
    pub snapshot_id: String,
    /// One of the five §12.7 assessments.
    pub assessment: String,
    /// The excerpt, which must appear in the snapshot's raw bytes.
    pub excerpt: String,
    /// The entailment rationale — why the excerpt bears on the claim.
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifier: Option<String>,
    #[serde(default = "unassessed")]
    pub source_authority: String,
    #[serde(default = "unassessed")]
    pub freshness: String,
    #[serde(default = "unassessed")]
    pub independence: String,
    #[serde(default = "unassessed")]
    pub uncertainty: String,
}

fn unassessed() -> String {
    "unassessed".to_owned()
}

/// The client-side claim input: content only (`PHASE-5.2.2`).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimInput {
    pub content: String,
}

/// `thread.challenge` body: the scope, the challenged contribution event, the
/// challenge text, and (`.2.2`, ADR-029) the optional targeted claim digest —
/// the structured objection names ONE claim inside the target contribution.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChallengeBody {
    pub tenant_id: TenantId,
    pub target_event_id: String,
    pub content: String,
    #[serde(default)]
    pub claim_digest: Option<String>,
}

/// `thread.revise` body: the scope, the challenge being answered, and the revision.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviseBody {
    pub tenant_id: TenantId,
    pub target_event_id: String,
    pub content: String,
}

/// The close outcome (`.2.4.1`, §13.4): the TWELVE valid terminals with the two
/// legacy wire words kept as aliases — `decided` (the stated default, a close
/// without an outcome IS a decision) → `accepted_by_rule`, `inconclusive` →
/// `deadlocked`. The canonical names persist (the event + the projection); the
/// legacy words never do. The family rule (the `.1.5.3` refusal, generalized):
/// a DECISION-family terminal carrying a non-empty unresolved register is the
/// typed refusal — listing what prevented a decision while claiming one would
/// be dishonest.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseOutcome {
    /// The legacy wire word — canonicalized to `accepted_by_rule` at persist.
    #[default]
    Decided,
    /// The legacy wire word — canonicalized to `deadlocked` at persist.
    Inconclusive,
    AcceptedUnanimously,
    AcceptedWithRecordedObjections,
    AcceptedByRule,
    AdvisoryAnswerOnly,
    Deadlocked,
    NoQuorum,
    InsufficientEvidence,
    BudgetExhausted,
    Expired,
    Cancelled,
    HumanDecisionRequired,
    UnsafeToContinue,
}

impl CloseOutcome {
    /// Whether this terminal is in the DECISION family (the `decided` side of
    /// the `.1.5.3` rule): a non-empty unresolved register is the typed
    /// refusal. The failure family accepts (and honestly should carry) it.
    pub fn is_decision_family(&self) -> bool {
        matches!(
            self,
            CloseOutcome::Decided
                | CloseOutcome::AcceptedUnanimously
                | CloseOutcome::AcceptedWithRecordedObjections
                | CloseOutcome::AcceptedByRule
                | CloseOutcome::AdvisoryAnswerOnly
        )
    }

    /// The §13.4 canonical terminal name — the legacy words never persist.
    pub fn canonical(&self) -> &'static str {
        match self {
            CloseOutcome::Decided => "accepted_by_rule",
            CloseOutcome::Inconclusive => "deadlocked",
            CloseOutcome::AcceptedUnanimously => "accepted_unanimously",
            CloseOutcome::AcceptedWithRecordedObjections => "accepted_with_recorded_objections",
            CloseOutcome::AcceptedByRule => "accepted_by_rule",
            CloseOutcome::AdvisoryAnswerOnly => "advisory_answer_only",
            CloseOutcome::Deadlocked => "deadlocked",
            CloseOutcome::NoQuorum => "no_quorum",
            CloseOutcome::InsufficientEvidence => "insufficient_evidence",
            CloseOutcome::BudgetExhausted => "budget_exhausted",
            CloseOutcome::Expired => "expired",
            CloseOutcome::Cancelled => "cancelled",
            CloseOutcome::HumanDecisionRequired => "human_decision_required",
            CloseOutcome::UnsafeToContinue => "unsafe_to_continue",
        }
    }
}

/// One coverage item of the minority report (`.2.4.1`, ADR-029 + §13.5): the
/// objection/uncertainty item, whether the synthesis INCLUDED it, and the
/// reason when it was excluded. The coverage report makes the synthesizer's
/// choices checkable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageItem {
    pub item: String,
    pub included: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

/// The minority report riding the close (`.2.4.1`, ADR-029 + §13.5): derived
/// content — the synthesizer identity/configuration, the input event range,
/// the source links, and the coverage report. It rides the close EVENT; the
/// ledger carries it, the inspection reconstructs it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinorityReportInput {
    pub synthesizer: String,
    pub input_event_range: String,
    pub sources: Vec<String>,
    pub coverage: Vec<CoverageItem>,
}

/// `thread.close` body: the scope, the stop reason (preserved for the audit
/// view), the outcome (the §13.4 terminal; `decided`/`inconclusive` stay
/// accepted aliases, default `decided`), the unresolved register (the items
/// that prevented a decision — refused on a decision-family terminal), and
/// the minority report (`.2.4.1`).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloseBody {
    pub tenant_id: TenantId,
    pub reason: String,
    #[serde(default)]
    pub outcome: CloseOutcome,
    #[serde(default)]
    pub unresolved: Vec<String>,
    #[serde(default)]
    pub minority_report: Option<MinorityReportInput>,
}

/// `thread.cancel` body: the scope and the abandonment reason (`PHASE-1.1.3`) —
/// preserved for the audit view, distinct from a decided close.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CancelBody {
    pub tenant_id: TenantId,
    pub reason: String,
}

// ── Projection ────────────────────────────────────────────────────────────────────

/// The recorded offer facts for one invitation (`.1.3.1`): when it was offered and
/// when it expires. The participants map carries the lifecycle STATE (the core
/// machine); this map carries the TIME facts — and expiry is DERIVED from
/// `expires_at` (an `invited` entry whose `expires_at` passed reads as `expired`,
/// exactly like the channel's lease presence: no sweeper, no stored flag).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitationMeta {
    pub invited_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// The minimal thread projection stored in `aggregate_state.state`. It keeps only
/// what validation and inspection need; content lives in the event log. The
/// `PHASE-1.1.3` and `.1.3.1` fields are additive and `#[serde(default)]`-ed, so a
/// projection written before them still parses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreadProjection {
    pub schema: u64,
    pub thread_id: ThreadId,
    pub subject: String,
    pub objective: String,
    pub state: ThreadState,
    /// The creating principal's wire id (`hpr_…`/`rol_…`).
    pub created_by: String,
    /// Principal wire id → participation state. The creator is `accepted` from day one.
    pub participants: BTreeMap<String, ParticipationState>,
    pub contributions: u64,
    pub revisions: u64,
    /// Structured claims riding contributions (`.2.2`; the ADR-029 counter —
    /// the records themselves live in the event log).
    #[serde(default)]
    pub structured_claims: u64,
    /// Challenges that no revision has answered (the unresolved register).
    pub open_challenges: u64,
    pub close_reason: Option<String>,
    /// The §13.4 canonical close terminal (`.2.4.1`; additive — the legacy
    /// projections close without one).
    #[serde(default)]
    pub close_outcome: Option<String>,
    #[serde(default)]
    pub classification: Classification,
    #[serde(default)]
    pub workflow_profile: String,
    /// The resolved profile's step sequence (the ADR-016 composition) —
    /// the projection records it so the inspection shows the plan.
    #[serde(default = "default_workflow_steps")]
    pub workflow_steps: Vec<String>,
    /// The current step index (0 = the first; the close advances to the
    /// terminal step).
    #[serde(default)]
    pub workflow_step: usize,
    #[serde(default)]
    pub participant_rules: ParticipantRules,
    #[serde(default)]
    pub cancel_reason: Option<String>,
    /// Role wire id → the recorded offer facts (`.1.3.1`; additive).
    #[serde(default)]
    pub invitations: BTreeMap<String, InvitationMeta>,
    /// The thread's current round (`.1.5.2`; additive — a projection written
    /// before it defaults to round 1). Server-assigned: contributions land in
    /// the CURRENT round; `thread.advance_round` is the only mover.
    #[serde(default = "default_round")]
    pub current_round: u64,
    pub ceiling_id: String,
    pub budget: BudgetDimensions,
}

/// Round 1 is the stated default (`.1.5.2`): a projection without the field is a
/// thread that never advanced.
fn default_round() -> u64 {
    1
}

// ── Errors ────────────────────────────────────────────────────────────────────────

/// A deterministic domain rejection. The HTTP layer maps each variant to a typed
/// §9.8 reason code and status.
#[derive(Debug)]
pub enum ThreadError {
    /// The body/operation is not a command this module accepts (or is malformed).
    InvalidCommand(String),
    /// The core state machine refused the move (or the thread is not `open` for a
    /// content verb).
    InvalidTransition(TransitionError),
    /// The aggregate has no row for this (tenant, thread) — reported as `scope_hidden`,
    /// never as a confirmed cross-tenant existence fact.
    ThreadNotFound,
    /// The actor has a grant but is not a participant of this thread.
    NotAParticipant { principal: String },
    /// The invite names a role that already has an OPEN membership (invited or
    /// accepted); a terminal record (declined/expired/left/revoked) may be re-invited.
    AlreadyParticipant { principal: String },
    /// The actor is invited but has not accepted yet — explicit participants first
    /// (`.1.3.1`): `thread.accept_invitation` before acting.
    InvitationPending { principal: String },
    /// The actor has no pending invitation to accept or decline.
    NoPendingInvitation { principal: String },
    /// The pending invitation's `expires_at` has passed (derived, `.1.3.1`).
    InvitationExpired { principal: String },
    /// The stored projection does not parse (the database was modified outside the
    /// supported surface — the "no database surgery" acceptance's failure mode).
    CorruptState(String),
    /// The quota check refused the command (`.1.3.2`): the recorded denial
    /// rides the caller's transaction — the refusal is never silent. The
    /// carried error names the exact verdict (exceeded vs unconfigured vs a
    /// storage failure).
    QuotaRefused(crate::quota::QuotaError),
}

impl std::fmt::Display for ThreadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadError::InvalidCommand(msg) => write!(f, "invalid command: {msg}"),
            ThreadError::InvalidTransition(e) => write!(f, "{e}"),
            ThreadError::ThreadNotFound => write!(f, "thread not found"),
            ThreadError::NotAParticipant { principal } => {
                write!(
                    f,
                    "principal `{principal}` is not a participant of this thread"
                )
            }
            ThreadError::AlreadyParticipant { principal } => {
                write!(
                    f,
                    "principal `{principal}` already has an open membership in this thread"
                )
            }
            ThreadError::InvitationPending { principal } => {
                write!(
                    f,
                    "principal `{principal}` is invited but has not accepted yet —                      thread.accept_invitation first"
                )
            }
            ThreadError::NoPendingInvitation { principal } => {
                write!(
                    f,
                    "principal `{principal}` has no pending invitation in this thread"
                )
            }
            ThreadError::InvitationExpired { principal } => {
                write!(f, "principal `{principal}`'s invitation has expired")
            }
            ThreadError::CorruptState(detail) => {
                write!(f, "the stored thread state is corrupt: {detail}")
            }
            ThreadError::QuotaRefused(e) => {
                write!(f, "the quota refused the command: {e}")
            }
        }
    }
}

impl std::error::Error for ThreadError {}

// ── Preparation ───────────────────────────────────────────────────────────────────

/// The fully validated result of preparing one command: everything the transaction
/// needs to write, and the semantic result stored for idempotent replay.
pub struct PreparedCommand {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub event_body: Value,
    pub next_state: Value,
    /// The success result (`{"ok": true, …}`); failures are stored by the caller.
    pub result: Value,
    /// `thread.create` only: the budget ceiling to write in the SAME transaction.
    pub ceiling: Option<(String, BudgetDimensions)>,
}

/// The semantic result every accepted command stores (and a replay returns verbatim).
fn success_result(
    thread_id: &ThreadId,
    event_id: &EventId,
    event_type: &str,
    state: ThreadState,
) -> Value {
    json!({
        "ok": true,
        "thread_id": thread_id.to_string(),
        "event_id": event_id.to_string(),
        "event_type": event_type,
        "thread_state": state.as_str(),
    })
}

fn ceiling_for(spec: Option<&BudgetSpec>) -> BudgetDimensions {
    match spec {
        None => DEFAULT_BUDGET,
        Some(spec) => BudgetDimensions {
            calls: spec.calls.or(DEFAULT_BUDGET.calls),
            input_tokens: spec.input_tokens.or(DEFAULT_BUDGET.input_tokens),
            output_tokens: spec.output_tokens.or(DEFAULT_BUDGET.output_tokens),
            wall_clock_seconds: spec
                .wall_clock_seconds
                .or(DEFAULT_BUDGET.wall_clock_seconds),
        },
    }
}

/// Prepare a `thread.create` — pure: a new thread has no prior state. The thread id
/// is server-assigned (the client proposes intent, never aggregate identity).
pub fn prepare_create(
    tenant_id: &TenantId,
    thread_id: &ThreadId,
    principal: &str,
    body: &CreateBody,
    workflow_steps: Vec<String>,
) -> PreparedCommand {
    let budget = ceiling_for(body.budget.as_ref());
    let ceiling_id = format!("ceil_{thread_id}");
    let projection = ThreadProjection {
        invitations: BTreeMap::new(),
        current_round: default_round(),
        schema: 1,
        thread_id: *thread_id,
        subject: body.subject.clone(),
        objective: body.objective.clone(),
        state: ThreadState::Open,
        created_by: principal.to_string(),
        participants: BTreeMap::from([(principal.to_string(), ParticipationState::Accepted)]),
        contributions: 0,
        revisions: 0,
        structured_claims: 0,
        open_challenges: 0,
        close_reason: None,
        close_outcome: None,
        classification: body.classification.unwrap_or_default(),
        workflow_profile: body
            .workflow_profile
            .clone()
            .unwrap_or_else(|| crate::workflows::DEFAULT_PROFILE_ID.to_owned()),
        workflow_steps,
        workflow_step: 0,
        participant_rules: body.participant_rules.clone().unwrap_or_default(),
        cancel_reason: None,
        ceiling_id: ceiling_id.clone(),
        budget,
    };
    let event_id = EventId::new();
    let event_body = json!({
        "operation": OP_CREATE,
        "thread_id": thread_id.to_string(),
        "tenant_id": tenant_id.to_string(),
        "actor_principal_id": principal,
        "subject": body.subject,
        "objective": body.objective,
        "budget": budget,
        "classification": projection.classification,
        "workflow_profile": projection.workflow_profile,
        "workflow_steps": projection.workflow_steps,
        "participant_rules": projection.participant_rules,
    });
    PreparedCommand {
        event_id,
        event_type: EVENT_CREATED,
        event_body,
        next_state: serde_json::to_value(&projection).expect("projection serializes"),
        result: success_result(thread_id, &event_id, EVENT_CREATED, ThreadState::Open),
        ceiling: Some((ceiling_id, budget)),
    }
}

/// Require the thread to be `open` for a content/lifecycle verb; the rejection is a
/// core-machine-shaped [`TransitionError`] (the §9.8 `invalid_transition` path).
fn require_open(projection: &ThreadProjection, verb: &'static str) -> Result<(), ThreadError> {
    if projection.state != ThreadState::Open {
        return Err(ThreadError::InvalidTransition(TransitionError {
            aggregate: "Thread",
            from: projection.state.as_str(),
            event: verb,
        }));
    }
    Ok(())
}

/// The actor may act only as an ACCEPTED participant (`.1.3.1`: explicit
/// participants first — the auto-accept on first contribution is gone). An
/// invited-but-unaccepted role gets the typed `invitation_pending` refusal that
/// names the accept verb.
fn ensure_participant(projection: &ThreadProjection, principal: &str) -> Result<(), ThreadError> {
    match projection.participants.get(principal) {
        None => Err(ThreadError::NotAParticipant {
            principal: principal.to_string(),
        }),
        Some(ParticipationState::Accepted) => Ok(()),
        Some(ParticipationState::Invited) => Err(ThreadError::InvitationPending {
            principal: principal.to_string(),
        }),
        Some(_) => Err(ThreadError::NotAParticipant {
            principal: principal.to_string(),
        }),
    }
}

/// The derived expiry predicate (`.1.3.1`): a pending invitation whose recorded
/// `expires_at` has passed IS expired — a time fact, like the channel's lease
/// presence. No sweeper, no stored `expired` flag.
fn invitation_is_expired(meta: &InvitationMeta, now: DateTime<Utc>) -> bool {
    meta.expires_at.is_some_and(|at| at <= now)
}

/// The inspection view of a projection: `invited` entries whose invitation has
/// expired read as `expired` (derived at read time; the stored projection is
/// untouched — the next accept/decline enforces the same predicate).
pub fn derived_view(mut projection: ThreadProjection) -> ThreadProjection {
    let now = Utc::now();
    for (role, state) in projection.participants.iter_mut() {
        if *state == ParticipationState::Invited
            && projection
                .invitations
                .get(role)
                .is_some_and(|m| invitation_is_expired(m, now))
        {
            *state = ParticipationState::Expired;
        }
    }
    projection
}

/// The event type of one event in this thread, if it exists (the challenge/revise
/// target check).
async fn event_type_in_thread<'e, E>(
    mut tx: E,
    tenant_id: &TenantId,
    thread_id: &ThreadId,
    event_id: &str,
) -> Result<Option<String>, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    sqlx::query_scalar(
        "SELECT event_type FROM event_log \
         WHERE event_id = $1 AND tenant_id = $2 AND aggregate_id = $3",
    )
    .bind(event_id)
    .bind(tenant_id.to_string())
    .bind(thread_id.to_string())
    .fetch_optional(&mut *tx)
    .await
}

/// The event BODY + version of one event in this thread (`.2.2`: the structured
/// challenge validates its claim digest against the target contribution's
/// server-computed claim records — the body, not the projection, carries them;
/// `.2.3`: the blind-target guard needs the version for the commitment scan).
async fn event_body_and_version_in_thread<'e, E>(
    mut tx: E,
    tenant_id: &TenantId,
    thread_id: &ThreadId,
    event_id: &str,
) -> Result<Option<(serde_json::Value, i64)>, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        "SELECT body, aggregate_version FROM event_log \
         WHERE event_id = $1 AND tenant_id = $2 AND aggregate_id = $3",
    )
    .bind(event_id)
    .bind(tenant_id.to_string())
    .bind(thread_id.to_string())
    .fetch_optional(&mut *tx)
    .await
}

/// `.3.3` (ADR-030): the thread's highest event-log version — the synthesis's
/// input range must name events that exist.
async fn max_event_version_in_thread<'e, E>(
    mut tx: E,
    tenant_id: &TenantId,
    thread_id: &ThreadId,
) -> Result<Option<i64>, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    sqlx::query_scalar(
        "SELECT MAX(aggregate_version) FROM event_log WHERE tenant_id = $1 AND aggregate_id = $2",
    )
    .bind(tenant_id.to_string())
    .bind(thread_id.to_string())
    .fetch_one(&mut *tx)
    .await
}

/// `.2.4.2` (ADR-029): whether ONE claim digest is a claim of ANY contribution
/// of this thread — the evidence request targets a claim that exists HERE (the
/// request is not an acquisition, and it cannot demand evidence for a claim
/// that was never made). The JSONB containment matches the server-computed
/// `claims[].digest` records.
async fn claim_exists_in_thread<'e, E>(
    mut tx: E,
    tenant_id: &TenantId,
    thread_id: &ThreadId,
    claim_digest: &str,
) -> Result<bool, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM event_log          WHERE tenant_id = $1 AND aggregate_id = $2          AND body -> 'claims' @> jsonb_build_array(jsonb_build_object('digest', $3::text)))",
    )
    .bind(tenant_id.to_string())
    .bind(thread_id.to_string())
    .bind(claim_digest)
    .fetch_one(&mut *tx)
    .await
}

/// `.2.3` (ADR-029): whether a blind contribution's phase has committed — a
/// LATER event that is either a round advance carrying `blind_committed: true`
/// (the commitment point) or the close/cancel (which ends the phase with the
/// thread). The version bound makes the scan replay-precise.
async fn blind_phase_committed<'e, E>(
    mut tx: E,
    tenant_id: &TenantId,
    thread_id: &ThreadId,
    after_version: i64,
) -> Result<bool, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM event_log \
         WHERE tenant_id = $1 AND aggregate_id = $2 AND aggregate_version > $3 \
         AND (event_type IN ('thread.closed', 'thread.cancelled') \
         OR (event_type = 'thread.round_advanced' AND body ->> 'blind_committed' = 'true')))",
    )
    .bind(tenant_id.to_string())
    .bind(thread_id.to_string())
    .bind(after_version)
    .fetch_one(&mut *tx)
    .await
}

/// The `author` of one contribution/challenge event in this thread, if the event
/// exists and carries one (`PHASE-0.6.2`: the challenge dispatch sends its revise
/// work to the challenged contribution's author — a role's node — while a human
/// author revises through the CLI, so no dispatch happens).
pub(crate) async fn event_author_in_thread<'e, E>(
    mut tx: E,
    tenant_id: &TenantId,
    thread_id: &ThreadId,
    event_id: &str,
) -> Result<Option<String>, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    let body: Option<Value> = sqlx::query_scalar(
        "SELECT body FROM event_log \
         WHERE event_id = $1 AND tenant_id = $2 AND aggregate_id = $3",
    )
    .bind(event_id)
    .bind(tenant_id.to_string())
    .bind(thread_id.to_string())
    .fetch_optional(&mut *tx)
    .await?;
    Ok(body.and_then(|b| b.get("author").and_then(|a| a.as_str()).map(str::to_string)))
}

/// Build the inbox payload for one thread work item (`PHASE-0.6.2`): the work
/// kind, the role it is for, the thread context the node's request carries, the
/// revise target when present, and the reservation (or the denial reason when the
/// ceiling refused to reserve).
pub fn work_payload(
    kind: &str,
    agent_role: &str,
    subject: &str,
    objective: &str,
    target_event_id: Option<&str>,
    reservation: Option<&reasonbraid_core::ReservationReference>,
    reservation_reason: Option<&str>,
) -> Value {
    let mut payload = json!({
        "kind": kind,
        "agent_role": agent_role,
        "subject": subject,
        "objective": objective,
        "reservation": reservation,
        "reservation_reason": reservation_reason,
        // The §14.6 retry authorization (`.2.3`): the dev profile dispatches
        // nothing with a possible-duplicate risk — the flag rides the wire so
        // the node's retry gate is typed, and it stays false until a surface
        // (Phase 5+ correction machinery) authorizes a risky re-run.
        "allow_possible_duplicate": false,
    });
    if let Some(target) = target_event_id {
        payload["target_event_id"] = json!(target);
    }
    payload
}

/// Prepare a command against an EXISTING thread: read the locked projection, validate
/// the operation against the core state machines and the dev rules, and build the
/// next projection + event. The caller holds the transaction; the `FOR UPDATE` read
/// here is the same row the aggregate library's fresh apply re-reads
/// (`agg::apply_fresh_in_tx`, `PHASE-1.1.1`), so the version derived there matches
/// the state validated here.
pub(crate) async fn prepare_thread_command<'e, E>(
    mut tx: E,
    tenant_id: &TenantId,
    thread_id: &ThreadId,
    operation: &str,
    principal: &str,
    body: &Value,
) -> Result<PreparedCommand, ThreadError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    let row: Option<(i64, Value)> = sqlx::query_as(
        "SELECT aggregate_version, state FROM aggregate_state \
         WHERE tenant_id = $1 AND aggregate_id = $2 FOR UPDATE",
    )
    .bind(tenant_id.to_string())
    .bind(thread_id.to_string())
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ThreadError::CorruptState(e.to_string()))?;
    let Some((_, state_json)) = row else {
        return Err(ThreadError::ThreadNotFound);
    };
    let mut projection: ThreadProjection =
        serde_json::from_value(state_json).map_err(|e| ThreadError::CorruptState(e.to_string()))?;

    let event_id = EventId::new();
    let (event_type, event_body, next_state) = match operation {
        OP_INVITE => {
            let body: InviteBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            require_open(&projection, "invite")?;
            if !projection.participant_rules.allow_explicit_invites {
                return Err(ThreadError::InvalidCommand(
                    "the thread does not allow explicit invitations (allow_explicit_invites is off)"
                        .to_string(),
                ));
            }
            let role: AgentRoleId = body.agent_role.parse().map_err(|_| {
                ThreadError::InvalidCommand(format!(
                    "agent_role `{}` is not a valid role identifier",
                    body.agent_role
                ))
            })?;
            // An OPEN membership (invited or accepted) refuses a second offer; a
            // terminal record (declined/expired/left/revoked) may be re-invited —
            // the new offer overwrites the meta (`.1.3.1`).
            match projection.participants.get(body.agent_role.as_str()) {
                Some(ParticipationState::Invited) | Some(ParticipationState::Accepted) => {
                    return Err(ThreadError::AlreadyParticipant {
                        principal: body.agent_role.clone(),
                    });
                }
                _ => {}
            }
            let now = Utc::now();
            // The per-tenant INVITE quota (`.1.3.2`, ADR-034 §16.11): the
            // invitation-storm bound — the check rides THIS transaction (a
            // use/denial event commits with the invitation; a refusal is a
            // recorded event, never silent; the unconfigured scope refuses
            // fail-closed).
            crate::quota::check_in_tx(
                &mut *tx,
                &tenant_id.to_string(),
                crate::quota::SCOPE_TENANT,
                &tenant_id.to_string(),
                now,
            )
            .await
            .map_err(ThreadError::QuotaRefused)?;
            let expires_at = body
                .expires_in_seconds
                .map(|secs| now + ChronoDuration::seconds(secs));
            projection
                .participants
                .insert(body.agent_role.clone(), ParticipationState::Invited);
            projection.invitations.insert(
                body.agent_role.clone(),
                InvitationMeta {
                    invited_at: now,
                    expires_at,
                },
            );
            (
                EVENT_INVITED,
                json!({
                    "operation": OP_INVITE,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "agent_role": role.to_string(),
                    "invited_at": now.to_rfc3339(),
                    "expires_at": expires_at.map(|at| at.to_rfc3339()),
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_ACCEPT_INVITATION => {
            let body: AcceptInvitationBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            let _ = body;
            require_open(&projection, "accept_invitation")?;
            if !projection.participants.contains_key(principal) {
                return Err(ThreadError::NoPendingInvitation {
                    principal: principal.to_string(),
                });
            }
            if projection.participants[principal] != ParticipationState::Invited {
                return Err(ThreadError::NoPendingInvitation {
                    principal: principal.to_string(),
                });
            }
            // Expiry is DERIVED: a pending offer past its `expires_at` refuses
            // here (and reads as `expired` in the inspection view).
            if let Some(meta) = projection.invitations.get(principal) {
                if invitation_is_expired(meta, Utc::now()) {
                    return Err(ThreadError::InvitationExpired {
                        principal: principal.to_string(),
                    });
                }
            }
            projection
                .participants
                .insert(principal.to_string(), ParticipationState::Accepted);
            (
                EVENT_INVITATION_ACCEPTED,
                json!({
                    "operation": OP_ACCEPT_INVITATION,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "agent_role": principal,
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_DECLINE_INVITATION => {
            let body: DeclineInvitationBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            let _ = body;
            require_open(&projection, "decline_invitation")?;
            if !projection.participants.contains_key(principal) {
                return Err(ThreadError::NoPendingInvitation {
                    principal: principal.to_string(),
                });
            }
            if projection.participants[principal] != ParticipationState::Invited {
                return Err(ThreadError::NoPendingInvitation {
                    principal: principal.to_string(),
                });
            }
            if let Some(meta) = projection.invitations.get(principal) {
                if invitation_is_expired(meta, Utc::now()) {
                    return Err(ThreadError::InvitationExpired {
                        principal: principal.to_string(),
                    });
                }
            }
            projection
                .participants
                .insert(principal.to_string(), ParticipationState::Declined);
            (
                EVENT_INVITATION_DECLINED,
                json!({
                    "operation": OP_DECLINE_INVITATION,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "agent_role": principal,
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_REMOVE_PARTICIPANT => {
            let body: RemoveParticipantBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            require_open(&projection, "remove_participant")?;
            match projection.participants.get(body.participant.as_str()) {
                Some(ParticipationState::Invited) | Some(ParticipationState::Accepted) => {
                    projection
                        .participants
                        .insert(body.participant.clone(), ParticipationState::Revoked);
                    (
                        EVENT_PARTICIPANT_REMOVED,
                        json!({
                            "operation": OP_REMOVE_PARTICIPANT,
                            "thread_id": thread_id.to_string(),
                            "tenant_id": tenant_id.to_string(),
                            "actor_principal_id": principal,
                            "removed_principal": body.participant,
                        }),
                        serde_json::to_value(&projection).expect("projection serializes"),
                    )
                }
                _ => {
                    return Err(ThreadError::InvalidCommand(format!(
                        "principal `{}` is not an invited or accepted participant",
                        body.participant
                    )));
                }
            }
        }
        OP_JOIN => {
            let body: JoinBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            let _ = body;
            require_open(&projection, "join")?;
            // `.1.3.2`: the self-request path requires the thread's join door open.
            if !projection.participant_rules.allow_join_requests {
                return Err(ThreadError::InvalidCommand(
                    "the thread does not allow join requests (allow_join_requests is off)"
                        .to_string(),
                ));
            }
            // An OPEN membership refuses (already joined/invited); a terminal
            // record (declined/expired/revoked) may join — the fresh membership
            // overwrites it.
            match projection.participants.get(principal) {
                Some(ParticipationState::Invited) | Some(ParticipationState::Accepted) => {
                    return Err(ThreadError::AlreadyParticipant {
                        principal: principal.to_string(),
                    });
                }
                _ => {}
            }
            projection
                .participants
                .insert(principal.to_string(), ParticipationState::Accepted);
            (
                EVENT_JOINED,
                json!({
                    "operation": OP_JOIN,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "agent_role": principal,
                    "via": "join_request",
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_ADVANCE_ROUND => {
            let body: AdvanceRoundBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            let _ = body;
            require_open(&projection, "advance_round")?;
            ensure_participant(&projection, principal)?;
            // `.2.3`/`.2.4.2` (ADR-016/029): the round advance IS the step
            // advance — the composition executes one step per round (clamped
            // at the terminal step; the close still seats it). The
            // `blind_committed` flag rides the event when the advance moved
            // PAST `blind_solicit` (the blind phase's commitment point — the
            // read surface's redaction scan keys on it). No new verb: the
            // existing transition.
            let commits_blind = projection
                .workflow_steps
                .get(projection.workflow_step)
                .map(|s| s.as_str())
                == Some("blind_solicit");
            projection.current_round += 1;
            projection.workflow_step = (projection.workflow_step + 1)
                .min(projection.workflow_steps.len().saturating_sub(1));
            (
                EVENT_ROUND_ADVANCED,
                json!({
                    "operation": OP_ADVANCE_ROUND,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "round": projection.current_round,
                    "blind_committed": commits_blind,
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_CONTRIBUTE => {
            let body: ContributeBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            require_open(&projection, "contribute")?;
            ensure_participant(&projection, principal)?;
            // `.2.2` (ADR-029): structured claims ride a `claim`-kind
            // contribution only — the kind is part of the claim's identity
            // (a position carrying claims would blur what is being claimed).
            if !body.claims.is_empty() && body.kind != ContributionKind::Claim {
                return Err(ThreadError::InvalidCommand(
                    "structured claims ride a `claim`-kind contribution only".to_string(),
                ));
            }
            // The current step name (the `.2.4.2`/`.3.2` gates).
            let step = projection
                .workflow_steps
                .get(projection.workflow_step)
                .map(|s| s.as_str());
            // `.3.2` (ADR-030): the moderation kinds FIRST — the CLOSED
            // vocabulary IS the structural prohibition. The capability-shaped
            // fields refuse on them (no vote/verdict, no evidence fabrication,
            // no claim targeting); the `ref_event_id` rides only moderation
            // kinds and must exist in the thread (the action can never erase —
            // it references, never rewrites).
            if body.kind.is_moderation_kind() {
                if body.verdict.is_some()
                    || body.assessment.is_some()
                    || !body.claims.is_empty()
                    || !body.evidence_refs.is_empty()
                    || body.target_claim_digest.is_some()
                {
                    return Err(ThreadError::InvalidCommand(
                        "a moderation action carries no verdict, assessment, claims, evidence \
                         references, or claim target — the prohibition is the vocabulary's \
                         negative space"
                            .to_string(),
                    ));
                }
                if step != Some("moderate") {
                    return Err(ThreadError::InvalidCommand(format!(
                        "the moderation action requires the current step to be `moderate` \
                         (it is `{}`)",
                        step.unwrap_or("none")
                    )));
                }
                if let Some(ref_event_id) = body.ref_event_id.clone() {
                    match event_type_in_thread(&mut *tx, tenant_id, thread_id, &ref_event_id)
                        .await
                        .map_err(|e| ThreadError::CorruptState(e.to_string()))?
                    {
                        Some(_) => {}
                        None => {
                            return Err(ThreadError::InvalidCommand(format!(
                                "the moderated event `{ref_event_id}` does not exist in this thread"
                            )))
                        }
                    }
                }
            } else if body.ref_event_id.is_some() {
                return Err(ThreadError::InvalidCommand(
                    "the moderated-event reference rides a moderation kind only".to_string(),
                ));
            }
            // `.2.4.2` (ADR-029): the kind-specific fields ride their kind —
            // the target names a claim only for an evidence request, the
            // verdict only for a verdict.
            if body.target_claim_digest.is_some() && body.kind != ContributionKind::EvidenceRequest
            {
                return Err(ThreadError::InvalidCommand(
                    "the claim target rides an `evidence_request`-kind contribution only"
                        .to_string(),
                ));
            }
            if body.verdict.is_some() && body.kind != ContributionKind::Verdict {
                return Err(ThreadError::InvalidCommand(
                    "the verdict rides a `verdict`-kind contribution only".to_string(),
                ));
            }
            if body.assessment.is_some() && body.kind != ContributionKind::Assessment {
                return Err(ThreadError::InvalidCommand(
                    "the assessment rides an `assessment`-kind contribution only".to_string(),
                ));
            }
            // The evidence_reference kind CARRIES evidence — an empty refs
            // list would claim support it does not show.
            if body.kind == ContributionKind::EvidenceReference && body.evidence_refs.is_empty() {
                return Err(ThreadError::InvalidCommand(
                    "an `evidence_reference` contribution requires at least one evidence reference"
                        .to_string(),
                ));
            }
            // The step gates (the ADR-016 composition executing): the request
            // belongs to the `evidence_request` step, the verdict to the
            // `adjudicate` step.
            if body.kind == ContributionKind::EvidenceRequest {
                let Some(target) = body.target_claim_digest.clone() else {
                    return Err(ThreadError::InvalidCommand(
                        "an `evidence_request` contribution requires `target_claim_digest`"
                            .to_string(),
                    ));
                };
                if step != Some("evidence_request") {
                    return Err(ThreadError::InvalidCommand(format!(
                        "the `evidence_request` contribution requires the current step to be                          `evidence_request` (it is `{}`)",
                        step.unwrap_or("none")
                    )));
                }
                if !claim_exists_in_thread(&mut *tx, tenant_id, thread_id, &target)
                    .await
                    .map_err(|e| ThreadError::CorruptState(e.to_string()))?
                {
                    return Err(ThreadError::InvalidCommand(format!(
                        "claim digest `{target}` is not a claim of this thread"
                    )));
                }
            }
            if body.kind == ContributionKind::Verdict && step != Some("adjudicate") {
                return Err(ThreadError::InvalidCommand(format!(
                    "the `verdict` contribution requires the current step to be `adjudicate` \
                     (it is `{}`)",
                    step.unwrap_or("none")
                )));
            }
            // `.3.3` (ADR-030): the synthesis rides a `summary`-kind
            // contribution on the `synthesize` step — derived content whose
            // input range must name events that EXIST (the transformation is
            // re-derivable, never a claim over nothing).
            if body.synthesis.is_some() {
                if body.kind != ContributionKind::Summary {
                    return Err(ThreadError::InvalidCommand(
                        "the synthesis rides a `summary`-kind contribution only".to_string(),
                    ));
                }
                if step != Some("synthesize") {
                    return Err(ThreadError::InvalidCommand(format!(
                        "the synthesis requires the current step to be `synthesize` \
                         (it is `{}`)",
                        step.unwrap_or("none")
                    )));
                }
                let Some(record) = body.synthesis.as_ref() else {
                    unreachable!("the is_some guard holds above");
                };
                if record.input_from < 1 || record.input_from > record.input_to {
                    return Err(ThreadError::InvalidCommand(format!(
                        "the synthesis input range {}..{} is invalid (1 <= from <= to)",
                        record.input_from, record.input_to
                    )));
                }
                let max_version = max_event_version_in_thread(&mut *tx, tenant_id, thread_id)
                    .await
                    .map_err(|e| ThreadError::CorruptState(e.to_string()))?
                    .unwrap_or(0);
                if record.input_to > max_version {
                    return Err(ThreadError::InvalidCommand(format!(
                        "the synthesis input range {}..{} exceeds the thread's event log \
                         (max version {max_version})",
                        record.input_from, record.input_to
                    )));
                }
            }
            // `.11.14.3.2` (ROADMAP §13.2 step 2, "register context and
            // resource references"): a citation registers the §12.1 reference
            // it names, on THIS transaction — so the contribution event and the
            // row its citation resolves to commit together or not at all.
            // ⛔ Registering, not refusing: step 2 registers and step 6
            // acquires, so citing something the network has not yet fetched is
            // the flow working
            // (`docs/decisions/2026-09-16_a-citation-registers-the-reference-it-names.md`).
            let submitted_by = crate::resources::actor_handle(principal);
            let first_of_pair = first_citation_of_each_pair(&body.evidence_refs);
            let mut evidence_refs: Vec<RegisteredEvidenceRef> =
                Vec::with_capacity(body.evidence_refs.len());
            // ⭐ The registration work is DE-DUPLICATED within one contribution
            // (`SIGNOFF-REPAIR.11.14.3.7`), and this is derived rather than
            // chosen: a reference's identity is the `(original_locator,
            // expected_digest)` PAIR (`.11.14.3.2`), so two citations naming the
            // same pair name ONE row — the second registration is a lookup that
            // can only return what the first just wrote.
            //
            // ⛔ It de-duplicates the WORK, never the record. Every citation the
            // contributor wrote still rides the event below, in its own order,
            // with its own note; only the trip to the store is shared. A
            // contribution that cites one pair three times with three different
            // notes still shows three citations.
            //
            // ⚠️ This is not a bound. It removes the trivially amplifying case
            // without inventing a number — the cost of N DISTINCT citations is
            // unchanged, and what limits that today is the request body, not a
            // quota. The leaf records the measurement.
            let mut registered: Vec<Option<String>> = vec![None; body.evidence_refs.len()];
            for (index, citation) in body.evidence_refs.iter().enumerate() {
                if first_of_pair[index] != index {
                    let resource_id = registered[first_of_pair[index]]
                        .clone()
                        .expect("the first citation of a pair is registered before its repeats");
                    evidence_refs.push(RegisteredEvidenceRef {
                        uri: citation.uri.clone(),
                        digest: citation.digest.clone(),
                        note: citation.note.clone(),
                        resource_id,
                    });
                    continue;
                }
                let Some(scheme) = crate::resources::scheme_of(&citation.uri) else {
                    return Err(ThreadError::InvalidCommand(format!(
                        "the evidence reference `{}` carries no URI scheme — a citation \
                         registers a §12.1 resource reference, and that contract names one",
                        citation.uri
                    )));
                };
                let reference = crate::resources::ResourceReference {
                    original_locator: citation.uri.clone(),
                    scheme: scheme.to_owned(),
                    media_type_hint: None,
                    expected_digest: citation.digest.clone(),
                    fragment_or_selector: None,
                    credential_binding_ref: None,
                    owning_node_or_capability: None,
                    // The contributor states neither, so the row takes the
                    // values `migrations/0023` declares as its column defaults
                    // — which an explicit INSERT would otherwise bypass.
                    visibility_scope: "network".to_owned(),
                    purpose: None,
                    retention_class: None,
                    risk_class: "low".to_owned(),
                };
                // The thread's tenant registers the citation's reference, so a
                // contributor's own tenant can read the §12.1 row it just
                // created (`SIGNOFF-REPAIR.11.14.3.4`).
                let registrant = crate::resources::Registrant {
                    tenant_id: tenant_id.to_string(),
                    principal: submitted_by.clone(),
                    // A contributor cites a URI; it never names a credential,
                    // which is why the reference above carries `None` too.
                    credential_binding_ref: None,
                };
                let outcome = match crate::resources::submit(
                    &mut *tx,
                    &reference,
                    &submitted_by,
                    &registrant,
                )
                .await
                {
                    Ok(outcome) => outcome,
                    // A store fault is the server's problem and must not be
                    // reported as though the caller's input were wrong
                    // (`.7.4.2`); every other variant IS about the input.
                    Err(crate::resources::ReferenceError::Storage(cause)) => {
                        return Err(ThreadError::CorruptState(cause.to_string()))
                    }
                    Err(error) => return Err(ThreadError::InvalidCommand(error.to_string())),
                };
                registered[index] = Some(outcome.resource_id.clone());
                evidence_refs.push(RegisteredEvidenceRef {
                    uri: citation.uri.clone(),
                    digest: citation.digest.clone(),
                    note: citation.note.clone(),
                    resource_id: outcome.resource_id,
                });
            }
            // `.11.14.3.1` (ROADMAP §13.2 step 6): the `assess` step's payload.
            // Every gate below already existed somewhere; what was missing was
            // the step that composes them, which is why two shipped profiles
            // declared `assess` and nothing could execute on it.
            let recorded_assessment = if body.kind == ContributionKind::Assessment {
                let Some(input) = body.assessment.as_ref() else {
                    return Err(ThreadError::InvalidCommand(
                        "an `assessment` contribution requires an `assessment` payload".to_string(),
                    ));
                };
                if step != Some("assess") {
                    return Err(ThreadError::InvalidCommand(format!(
                        "the `assessment` contribution requires the current step to be `assess` \
                         (it is `{}`)",
                        step.unwrap_or("none")
                    )));
                }
                // The claim is named by a digest the SERVER computed, and it
                // must be a claim of THIS thread — the same check an evidence
                // request already gets, and the reason the identifier cannot be
                // invented the way the standalone route's `claim_id` can.
                if !claim_exists_in_thread(&mut *tx, tenant_id, thread_id, &input.claim_digest)
                    .await
                    .map_err(|e| ThreadError::CorruptState(e.to_string()))?
                {
                    return Err(ThreadError::InvalidCommand(format!(
                        "claim digest `{}` is not a claim of this thread",
                        input.claim_digest
                    )));
                }
                // The evidence must be evidence this tenant ACQUIRED. Without
                // this, an assessment would be a way to learn that a snapshot
                // exists — the enumeration `.11.14.1` closed on the read side.
                if !crate::snapshots::is_cited_by(
                    &mut *tx,
                    &input.snapshot_id,
                    &tenant_id.to_string(),
                )
                .await
                .map_err(|e| ThreadError::CorruptState(e.to_string()))?
                {
                    return Err(ThreadError::InvalidCommand(format!(
                        "snapshot `{}` is not cited by this tenant — assess evidence this \
                         deliberation acquired",
                        input.snapshot_id
                    )));
                }
                // The store owns the §12.7 vocabulary, the excerpt validation
                // against the acquired bytes, and the replay. It runs on THIS
                // transaction, so the contribution event and the assessment row
                // commit together or not at all.
                let submission = crate::claims::AssessmentSubmission {
                    claim_id: input.claim_digest.clone(),
                    snapshot_id: input.snapshot_id.clone(),
                    assessment: input.assessment.clone(),
                    author: principal.to_owned(),
                    verifier: input.verifier.clone(),
                    excerpt: input.excerpt.clone(),
                    selector: input.selector.clone(),
                    rationale: input.rationale.clone(),
                    source_authority: input.source_authority.clone(),
                    freshness: input.freshness.clone(),
                    independence: input.independence.clone(),
                    uncertainty: input.uncertainty.clone(),
                };
                // The `thread` namespace: this identifier is a digest the server
                // minted and membership-checked above, which is what makes it
                // a different kind of identifier from the standalone route's
                // caller label (`.11.14.3.3`).
                match crate::claims::submit(
                    &mut *tx,
                    &submission,
                    &tenant_id.to_string(),
                    crate::claims::ClaimNamespace::Thread,
                )
                .await
                {
                    Ok(assessment_id) => Some(assessment_id),
                    // A store fault is the server's problem and must not be
                    // reported as though the caller's input were wrong
                    // (`.7.4.2`); every other variant IS about the input.
                    Err(crate::claims::AssessmentError::Storage(cause)) => {
                        return Err(ThreadError::CorruptState(cause.to_string()))
                    }
                    Err(error) => return Err(ThreadError::InvalidCommand(error.to_string())),
                }
            } else {
                None
            };
            // `.2.2` (ADR-029): the digest is SERVER-computed over the claim
            // content — the client never supplies it, so an objection can
            // only name a digest the server derived.
            let claims: Vec<ClaimRecord> = body
                .claims
                .iter()
                .map(|c| ClaimRecord {
                    content: c.content.clone(),
                    digest: crate::fetcher::digest_sha256_hex(c.content.as_bytes()),
                })
                .collect();
            projection.structured_claims += claims.len() as u64;
            projection.contributions += 1;
            // `.2.3` (ADR-029): a contribution posted while the CURRENT step is
            // `blind_solicit` is blind — the marker rides the event; the read
            // surface defers its content until the commitment point.
            let blind = projection
                .workflow_steps
                .get(projection.workflow_step)
                .map(|s| s.as_str())
                == Some("blind_solicit");
            (
                EVENT_CONTRIBUTED,
                json!({
                    "operation": OP_CONTRIBUTE,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "author": principal,
                    "content": body.content,
                    "kind": body.kind,
                    "evidence_refs": evidence_refs,
                    "claims": claims,
                    "target_claim_digest": body.target_claim_digest,
                    "ref_event_id": body.ref_event_id,
                    "synthesis": body.synthesis,
                    "verdict": body.verdict.as_ref().map(|v| json!({
                        "target_digest": v.target_digest,
                        "rule": v.rule,
                        "outcome": v.outcome.canonical(),
                    })),
                    "assessment": body.assessment.as_ref().map(|a| json!({
                        "claim_digest": a.claim_digest,
                        "snapshot_id": a.snapshot_id,
                        "assessment": a.assessment,
                        "excerpt": a.excerpt,
                        "rationale": a.rationale,
                        "selector": a.selector,
                        "assessment_id": recorded_assessment,
                    })),
                    "round": projection.current_round,
                    "blind": blind,
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_CHALLENGE => {
            let body: ChallengeBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            require_open(&projection, "challenge")?;
            ensure_participant(&projection, principal)?;
            match event_type_in_thread(&mut *tx, tenant_id, thread_id, &body.target_event_id)
                .await
                .map_err(|e| ThreadError::CorruptState(e.to_string()))?
            {
                Some(t) if t == EVENT_CONTRIBUTED => {}
                Some(t) => {
                    return Err(ThreadError::InvalidCommand(format!(
                        "challenge target `{}` is a `{t}` event, not a contribution",
                        body.target_event_id
                    )))
                }
                None => {
                    return Err(ThreadError::InvalidCommand(format!(
                        "challenge target `{}` does not exist in this thread",
                        body.target_event_id
                    )))
                }
            }
            // `.2.3` (ADR-029): a NON-AUTHOR cannot target a still-blind
            // contribution — the objection would name content the challenger
            // cannot read. The author (and the post-commitment readers) may.
            // The commitment point: a later round-advance that committed the
            // blind phase, or the close/cancel.
            let (target_body, target_version) = event_body_and_version_in_thread(
                &mut *tx,
                tenant_id,
                thread_id,
                &body.target_event_id,
            )
            .await
            .map_err(|e| ThreadError::CorruptState(e.to_string()))?
            .ok_or_else(|| {
                ThreadError::InvalidCommand(format!(
                    "challenge target `{}` does not exist in this thread",
                    body.target_event_id
                ))
            })?;
            if target_body.get("blind").and_then(|b| b.as_bool()) == Some(true)
                && target_body.get("author").and_then(|a| a.as_str()) != Some(principal)
                && !blind_phase_committed(&mut *tx, tenant_id, thread_id, target_version)
                    .await
                    .map_err(|e| ThreadError::CorruptState(e.to_string()))?
            {
                return Err(ThreadError::InvalidCommand(format!(
                    "challenge target `{}` is blind until the round advance",
                    body.target_event_id
                )));
            }
            // `.2.2` (ADR-029): the structured objection names ONE claim inside
            // the target contribution — a digest that is not among the target's
            // server-computed claim digests is the typed refusal (the digest is
            // never matched against client-supplied text).
            if let Some(claim_digest) = body.claim_digest.clone() {
                let claims: Vec<ClaimRecord> = target_body
                    .get("claims")
                    .and_then(|c| serde_json::from_value(c.clone()).ok())
                    .unwrap_or_default();
                if !claims.iter().any(|c| c.digest == claim_digest) {
                    return Err(ThreadError::InvalidCommand(format!(
                        "claim digest `{claim_digest}` is not a claim of contribution `{}`",
                        body.target_event_id
                    )));
                }
            }
            projection.open_challenges += 1;
            (
                EVENT_CHALLENGED,
                json!({
                    "operation": OP_CHALLENGE,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "author": principal,
                    "target_event_id": body.target_event_id,
                    "claim_digest": body.claim_digest,
                    "content": body.content,
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_REVISE => {
            let body: ReviseBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            require_open(&projection, "revise")?;
            ensure_participant(&projection, principal)?;
            match event_type_in_thread(&mut *tx, tenant_id, thread_id, &body.target_event_id)
                .await
                .map_err(|e| ThreadError::CorruptState(e.to_string()))?
            {
                Some(t) if t == EVENT_CHALLENGED => {}
                Some(t) => {
                    return Err(ThreadError::InvalidCommand(format!(
                        "revision target `{}` is a `{t}` event, not a challenge",
                        body.target_event_id
                    )))
                }
                None => {
                    return Err(ThreadError::InvalidCommand(format!(
                        "revision target `{}` does not exist in this thread",
                        body.target_event_id
                    )))
                }
            }
            projection.open_challenges = projection.open_challenges.saturating_sub(1);
            projection.revisions += 1;
            (
                EVENT_REVISED,
                json!({
                    "operation": OP_REVISE,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "author": principal,
                    "target_event_id": body.target_event_id,
                    "content": body.content,
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_CLOSE => {
            let body: CloseBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            // `.2.4.1` (the `.1.5.3` refusal, generalized to the family): a
            // DECISION-family terminal must not carry an unresolved register —
            // listing the items that prevented a decision while claiming one
            // would be dishonest. The failure family accepts it.
            if body.outcome.is_decision_family() && !body.unresolved.is_empty() {
                return Err(ThreadError::InvalidCommand(
                    "a decision terminal cannot carry unresolved items — name a failure terminal                      (deadlocked, no_quorum, insufficient_evidence, budget_exhausted, expired,                      cancelled, human_decision_required, unsafe_to_continue)"
                        .to_string(),
                ));
            }
            // Fold open → closing → terminal (both core edges validated; one
            // terminal event records the result). A thread already `closing`
            // only finalizes. The terminal state is the OUTCOME family: the
            // decision family → closed, the failure family → the honest
            // `Inconclusive` state (the core machine's edge, `.1.5.3`).
            let terminal = match projection.state {
                ThreadState::Open => {
                    ThreadState::Open
                        .apply(ThreadTransition::BeginClose)
                        .map_err(ThreadError::InvalidTransition)?;
                    if body.outcome.is_decision_family() {
                        ThreadState::Closing
                            .apply(ThreadTransition::FinalizeClose)
                            .expect("finalize_close from closing is deterministic")
                    } else {
                        ThreadState::Closing
                            .apply(ThreadTransition::FinalizeInconclusive)
                            .expect("finalize_inconclusive from closing is deterministic")
                    }
                }
                ThreadState::Closing => {
                    if body.outcome.is_decision_family() {
                        ThreadState::Closing
                            .apply(ThreadTransition::FinalizeClose)
                            .expect("finalize_close from closing is deterministic")
                    } else {
                        ThreadState::Closing
                            .apply(ThreadTransition::FinalizeInconclusive)
                            .expect("finalize_inconclusive from closing is deterministic")
                    }
                }
                other => {
                    return Err(ThreadError::InvalidTransition(TransitionError {
                        aggregate: "Thread",
                        from: other.as_str(),
                        event: "close",
                    }))
                }
            };
            projection.state = terminal;
            projection.close_reason = Some(body.reason.clone());
            // The canonical terminal persists; the legacy words never do.
            projection.close_outcome = Some(body.outcome.canonical().to_string());
            // The profile's terminal step (the ADR-016 sequence's last).
            projection.workflow_step = projection.workflow_steps.len().saturating_sub(1);
            (
                EVENT_CLOSED,
                json!({
                    "operation": OP_CLOSE,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "reason": body.reason,
                    "outcome": body.outcome.canonical(),
                    "unresolved": body.unresolved,
                    "minority_report": body.minority_report,
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        OP_CANCEL => {
            let body: CancelBody = serde_json::from_value(body.clone())
                .map_err(|e| ThreadError::InvalidCommand(e.to_string()))?;
            // The abandonment terminal (`PHASE-1.1.3`): `open` or `closing` →
            // `cancelled`, one core-machine edge, one event, reason preserved.
            // Distinct from a decided close; the two terminals never mix.
            let cancelled = match projection.state {
                ThreadState::Open => ThreadState::Open
                    .apply(ThreadTransition::Cancel)
                    .map_err(ThreadError::InvalidTransition)?,
                ThreadState::Closing => ThreadState::Closing
                    .apply(ThreadTransition::Cancel)
                    .map_err(ThreadError::InvalidTransition)?,
                other => {
                    return Err(ThreadError::InvalidTransition(TransitionError {
                        aggregate: "Thread",
                        from: other.as_str(),
                        event: "cancel",
                    }))
                }
            };
            projection.state = cancelled;
            projection.cancel_reason = Some(body.reason.clone());
            (
                EVENT_CANCELLED,
                json!({
                    "operation": OP_CANCEL,
                    "thread_id": thread_id.to_string(),
                    "tenant_id": tenant_id.to_string(),
                    "actor_principal_id": principal,
                    "reason": body.reason,
                }),
                serde_json::to_value(&projection).expect("projection serializes"),
            )
        }
        other => {
            return Err(ThreadError::InvalidCommand(format!(
                "unknown thread operation `{other}`"
            )))
        }
    };

    Ok(PreparedCommand {
        event_id,
        event_type,
        event_body,
        next_state,
        result: success_result(thread_id, &event_id, event_type, projection.state),
        ceiling: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The evaluator-access registry (`.1.4.3`): the dev registry qualifies
    /// `general` only — `confidential` has NO qualified evaluator, so the
    /// dispatch refuses (never a silent general).
    #[test]
    fn the_evaluator_registry_qualifies_general_only() {
        assert!(Classification::General.has_qualified_evaluator());
        assert!(
            !Classification::Confidential.has_qualified_evaluator(),
            "the dev registry registers no confidential-qualified evaluator"
        );
        assert_eq!(Classification::Confidential.as_str(), "confidential");
        assert_eq!(Classification::General.as_str(), "general");
    }

    /// The close fold validates BOTH core edges: a closed thread rejects a second
    /// close, and the projection lands on `closed` with the reason preserved.
    #[test]
    fn close_fold_is_deterministic_and_terminal() {
        // A pure check that the fold is the documented one (the DB-backed path is
        // covered by the live-PG `command_api` suite).
        let from_open = ThreadState::Open
            .apply(ThreadTransition::BeginClose)
            .unwrap()
            .apply(ThreadTransition::FinalizeClose)
            .unwrap();
        assert_eq!(from_open, ThreadState::Closed);
        assert!(ThreadState::Closed
            .apply(ThreadTransition::BeginClose)
            .is_err());
    }

    /// Budget specs fill unspecified dimensions from the defaults — every dimension
    /// of a dev thread's ceiling is metered (fail-closed reservation coverage).
    #[test]
    fn budget_spec_defaults_every_dimension() {
        let dims = ceiling_for(Some(&BudgetSpec {
            calls: Some(1),
            input_tokens: None,
            output_tokens: None,
            wall_clock_seconds: None,
        }));
        assert_eq!(dims.calls, Some(1));
        assert_eq!(dims.input_tokens, DEFAULT_BUDGET.input_tokens);
        assert_eq!(dims.output_tokens, DEFAULT_BUDGET.output_tokens);
        assert_eq!(dims.wall_clock_seconds, DEFAULT_BUDGET.wall_clock_seconds);

        let all_default = ceiling_for(None);
        assert_eq!(all_default, DEFAULT_BUDGET);
    }

    /// The create projection seats the creator as an accepted participant — the
    /// organizer can contribute and challenge without a separate accept verb.
    #[test]
    fn create_projection_seats_the_creator() {
        let tenant: TenantId = "ten_00000000-0000-7000-8000-000000000001".parse().unwrap();
        let thread: ThreadId = "thr_00000000-0000-7000-8000-000000000001".parse().unwrap();
        let prepared = prepare_create(
            &tenant,
            &thread,
            "hpr_00000000-0000-7000-8000-000000000001",
            &CreateBody {
                tenant_id: tenant,
                subject: "subject".to_string(),
                objective: "objective".to_string(),
                budget: None,
                classification: None,
                workflow_profile: None,
                routing_class: None,
                participant_rules: None,
            },
            default_workflow_steps(),
        );
        let projection: ThreadProjection =
            serde_json::from_value(prepared.next_state).expect("projection parses");
        assert_eq!(projection.state, ThreadState::Open);
        assert_eq!(
            projection.participants["hpr_00000000-0000-7000-8000-000000000001"],
            ParticipationState::Accepted
        );
        // The `.1.1.3` defaults are stated, not empty: general / single-agent /
        // explicit-invites-only.
        assert_eq!(projection.classification, Classification::General);
        assert_eq!(
            projection.workflow_profile,
            crate::workflows::DEFAULT_PROFILE_ID
        );
        assert_eq!(projection.cancel_reason, None);
        assert!(
            projection.participant_rules.allow_explicit_invites,
            "explicit invites on by default"
        );
        assert!(
            !projection.participant_rules.allow_join_requests,
            "join requests off by default"
        );
        assert_eq!(projection.ceiling_id, format!("ceil_{thread}"));
        assert_eq!(
            prepared.ceiling.as_ref().map(|(id, _)| id.as_str()),
            Some(format!("ceil_{thread}").as_str())
        );
        assert_eq!(prepared.event_type, EVENT_CREATED);
    }

    /// `SIGNOFF-REPAIR.11.14.3.7`: the citation de-duplication, falsified
    /// directly — because no product surface can see it.
    ///
    /// ⚠️ This test exists in this form for a measured reason. The live control
    /// can observe that N DISTINCT citations register N references, but a
    /// REPEATED pair produces one row either way: `resources::submit`'s replay
    /// returns the existing row, so the count is 1 with or without the
    /// de-duplication. A `pg_stat_user_tables` scan-counter instrument was
    /// written and discarded — it reported the same value against the repaired
    /// and the unrepaired handler, so its assertion could not fail, which is a
    /// control that measures nothing.
    #[test]
    fn the_citation_pairs_are_de_duplicated_by_pair_not_by_locator() {
        let cite = |uri: &str, digest: Option<&str>| EvidenceRef {
            uri: uri.to_owned(),
            digest: digest.map(str::to_owned),
            note: None,
        };

        // Distinct locators: every citation registers.
        let distinct = [
            cite("https://a.example/1", None),
            cite("https://a.example/2", None),
            cite("https://a.example/3", None),
        ];
        assert_eq!(
            super::first_citation_of_each_pair(&distinct),
            vec![0, 1, 2],
            "distinct pairs each register"
        );

        // The same pair repeated: one registration, and the repeats point AT it.
        let repeated = [
            cite("https://a.example/report", None),
            cite("https://a.example/report", None),
            cite("https://a.example/report", None),
        ];
        assert_eq!(
            super::first_citation_of_each_pair(&repeated),
            vec![0, 0, 0],
            "a repeated pair registers once"
        );

        // ⛔ Keyed on the PAIR. One locator at two digests is TWO references
        // (§12.6's changed page), and an implementation keyed on the locator
        // alone would answer `[0, 0, 0, 0]` here.
        let a = "sha256:aaaa";
        let b = "sha256:bbbb";
        let pairs = [
            cite("https://a.example/page", Some(a)),
            cite("https://a.example/page", Some(b)),
            cite("https://a.example/page", Some(a)),
            cite("https://a.example/page", None),
        ];
        assert_eq!(
            super::first_citation_of_each_pair(&pairs),
            vec![0, 1, 0, 3],
            "the pair is the key: two digests and the unpinned form are three \
             references, and the repeat of the first points at it"
        );

        // The order is preserved and the length matches, which is what keeps the
        // RECORD undeduplicated: one answer per citation, in order.
        assert_eq!(super::first_citation_of_each_pair(&[]).len(), 0);
        assert_eq!(
            super::first_citation_of_each_pair(&repeated).len(),
            repeated.len()
        );
    }
}
