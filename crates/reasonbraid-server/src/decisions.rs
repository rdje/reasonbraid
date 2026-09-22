//! The deterministic decision result (`SIGNOFF-REPAIR.8.1.1`; ROADMAP §13.2
//! step 11, §13.3, §13.4).
//!
//! Before this module a thread's close ASSERTED its ballot result: the closer
//! named `accepted_unanimously`, `accepted_with_recorded_objections`,
//! `accepted_by_rule` or `no_quorum`, and nothing counted anything. This module
//! is the count, and the one function that decides — per declared rule — which
//! close outcomes the server DERIVES, which stay caller-asserted, and which are
//! refused.
//!
//! The contract is
//! `docs/decisions/2026-09-22_a-counted-rule-derives-its-outcome-and-a-rule-it-cannot-count-is-refused.md`;
//! the section numbers below are that record's.
//!
//! ⭐ **Everything here is pure.** The thread aggregate owns the electorate and
//! the ballots; this module only reads a [`Tally`] of them, so every boundary of
//! the count is a unit test rather than a database fixture.

use serde::{Deserialize, Serialize};

use crate::charters::DecisionRule;
use crate::threads::CloseOutcome;

/// One ballot's choice (§4). An abstention is a CAST ballot — never outstanding,
/// never an approval — and what it means depends on the family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BallotChoice {
    Approve,
    Reject,
    Abstain,
}

/// The ballot box at the moment of counting: the electorate's size and the
/// ballots its members cast.
///
/// ⛔ The thread admits a ballot only from an electorate member and only once
/// (§1), so `approve + reject + abstain <= electorate` holds by construction;
/// [`Tally::outstanding`] saturates rather than trusting it blindly.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Tally {
    pub electorate: u64,
    pub approve: u64,
    pub reject: u64,
    pub abstain: u64,
}

impl Tally {
    /// Count the cast ballots against an electorate of `electorate` members.
    pub fn of<'a>(electorate: u64, ballots: impl IntoIterator<Item = &'a BallotChoice>) -> Self {
        let mut tally = Tally {
            electorate,
            ..Tally::default()
        };
        for ballot in ballots {
            match ballot {
                BallotChoice::Approve => tally.approve += 1,
                BallotChoice::Reject => tally.reject += 1,
                BallotChoice::Abstain => tally.abstain += 1,
            }
        }
        tally
    }

    /// Members who have not cast a ballot. There is no timer and no separate
    /// *unreachable* state: at the close, an outstanding ballot is simply not
    /// cast (§7).
    pub fn outstanding(&self) -> u64 {
        self.electorate
            .saturating_sub(self.approve + self.reject + self.abstain)
    }
}

/// How a family reaches its outcome (§2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evaluation {
    /// The outcome is counted from ballots.
    Counted,
    /// `owner_decides`: the outcome is the thread creator's close.
    ByOwner,
    /// `advisory_synthesis`: the only decision terminal is `advisory_answer_only`.
    Advisory,
    /// No bar anything can evaluate yet (`SIGNOFF-REPAIR.8.1.1.4`), so the
    /// family is refused at thread creation rather than accepted and ignored.
    Uncountable,
}

pub fn evaluation(rule: DecisionRule) -> Evaluation {
    match rule {
        DecisionRule::MajorityOfElectorate | DecisionRule::Unanimity | DecisionRule::Consensus => {
            Evaluation::Counted
        }
        DecisionRule::OwnerDecides => Evaluation::ByOwner,
        DecisionRule::AdvisorySynthesis => Evaluation::Advisory,
        DecisionRule::RoleWeighted | DecisionRule::HumanCommittee => Evaluation::Uncountable,
    }
}

/// Why a count could not be taken.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CountError {
    /// The family is not counted from ballots.
    NotCounted(DecisionRule),
    /// `majority_of_electorate` reached the count without its threshold. The
    /// charter refuses to store one without the other, so this is a thread
    /// recorded outside the supported surface.
    ThresholdMissing,
}

impl std::fmt::Display for CountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotCounted(rule) => {
                write!(f, "`{}` is not decided by counting ballots", rule.as_str())
            }
            Self::ThresholdMissing => write!(
                f,
                "`majority_of_electorate` was counted without the approval threshold its \
                 charter must state"
            ),
        }
    }
}

/// The approvals a threshold `t` requires of an electorate of `n`: the smallest
/// whole number `need` with `need >= t·n`.
///
/// ⚠️ The epsilon is load-bearing (§3). Thresholds are human-entered decimals,
/// and `0.56 * 25.0` is `14.000000000000002` in `f64` — a bare `ceil` would
/// demand 15 approvals of 25 where 14 are exactly 56%. `1e-9` is far below
/// `1/n` for any electorate a thread can hold.
fn approvals_needed(threshold: f64, electorate: u64) -> u64 {
    (threshold * electorate as f64 - 1e-9).ceil().max(0.0) as u64
}

/// The counted result (§3) — the §13.2 step 11 *deterministic decision result*.
pub fn count(
    rule: DecisionRule,
    threshold: Option<f64>,
    tally: &Tally,
) -> Result<CloseOutcome, CountError> {
    if evaluation(rule) != Evaluation::Counted {
        return Err(CountError::NotCounted(rule));
    }
    // An empty electorate — or a vote that never opened — decided nothing.
    if tally.electorate == 0 {
        return Ok(CloseOutcome::NoQuorum);
    }
    let Tally {
        electorate: n,
        approve: a,
        reject: r,
        abstain: s,
    } = *tally;
    let o = tally.outstanding();
    Ok(match rule {
        DecisionRule::MajorityOfElectorate => {
            let need = approvals_needed(threshold.ok_or(CountError::ThresholdMissing)?, n);
            if a >= need {
                if a == n {
                    CloseOutcome::AcceptedUnanimously
                } else if r > 0 {
                    CloseOutcome::AcceptedWithRecordedObjections
                } else {
                    CloseOutcome::AcceptedByRule
                }
            } else if a + o >= need {
                // The ballots not yet cast could still have carried it.
                CloseOutcome::NoQuorum
            } else {
                CloseOutcome::Deadlocked
            }
        }
        // §13.4: *An initiator who requests unanimity receives failure/deadlock
        // if any applicable member withholds it* — and an abstainer has not
        // recused, so they are still applicable (§4).
        DecisionRule::Unanimity => {
            if r > 0 || s > 0 {
                CloseOutcome::Deadlocked
            } else if o > 0 {
                CloseOutcome::NoQuorum
            } else {
                CloseOutcome::AcceptedUnanimously
            }
        }
        // A `reject` is the blocking objection; an abstention stands aside.
        // ⛔ Silence is never consent: a member who has not voted may still
        // block, so outstanding ballots are `no_quorum`, not agreement.
        DecisionRule::Consensus => {
            if r > 0 {
                CloseOutcome::Deadlocked
            } else if o > 0 {
                CloseOutcome::NoQuorum
            } else if a == 0 {
                CloseOutcome::Deadlocked
            } else if s == 0 {
                CloseOutcome::AcceptedUnanimously
            } else {
                CloseOutcome::AcceptedByRule
            }
        }
        DecisionRule::OwnerDecides
        | DecisionRule::RoleWeighted
        | DecisionRule::HumanCommittee
        | DecisionRule::AdvisorySynthesis => unreachable!("only counted families reach here"),
    })
}

/// Where a close outcome came from — recorded on the close event, because an
/// implicit provenance is the defect this module repairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    /// The server computed it from the declared rule.
    Derived,
    /// The closer stated it and nothing checked it.
    CallerAsserted,
}

/// A close the classification accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosedAs {
    pub outcome: CloseOutcome,
    pub provenance: Provenance,
}

/// Every way the classification refuses a close. Each names the ASSERTED word,
/// so the refusal says what was wrong with the request rather than only what
/// the server would have preferred.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CloseRefusal {
    /// A counted family's result is not the one the closer asserted.
    Disagrees {
        asserted: &'static str,
        counted: &'static str,
    },
    /// A ballot word under a family that counts no ballots.
    NothingCounted {
        asserted: &'static str,
        rule: DecisionRule,
    },
    /// `advisory_answer_only` under a family that binds.
    RuleBinds {
        rule: DecisionRule,
    },
    /// A binding or ballot word under `advisory_synthesis`.
    NoBindingDecision {
        asserted: &'static str,
    },
    /// `owner_decides`, closed with a decision by someone other than the owner.
    NotTheOwner,
    /// A family nothing can evaluate. Creation refuses it, so reaching this
    /// means a thread was recorded outside the supported surface.
    Uncountable(DecisionRule),
    Count(CountError),
}

impl std::fmt::Display for CloseRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disagrees { asserted, counted } => write!(
                f,
                "the close asserts `{asserted}` and the counted ballot is `{counted}` — \
                 under this thread's decision rule the outcome is derived; omit it to \
                 close with the count"
            ),
            Self::NothingCounted { asserted, rule } => write!(
                f,
                "`{asserted}` names a ballot result, and `{}` counts no ballots",
                rule.as_str()
            ),
            Self::RuleBinds { rule } => write!(
                f,
                "`{}` is a binding rule, so the close cannot be `advisory_answer_only`",
                rule.as_str()
            ),
            Self::NoBindingDecision { asserted } => write!(
                f,
                "`advisory_synthesis` has no binding decision, so the close cannot be \
                 `{asserted}`"
            ),
            Self::NotTheOwner => write!(
                f,
                "under `owner_decides` only the thread's creator closes with a decision"
            ),
            Self::Uncountable(rule) => write!(
                f,
                "`{}` has no bar the server can evaluate (SIGNOFF-REPAIR.8.1.1.4)",
                rule.as_str()
            ),
            Self::Count(e) => write!(f, "{e}"),
        }
    }
}

/// The six terminals that state a fact about the PROCESS rather than the
/// ballot, so they stay caller-asserted under every rule (§5).
pub fn is_process_terminal(outcome: CloseOutcome) -> bool {
    match outcome {
        CloseOutcome::InsufficientEvidence
        | CloseOutcome::BudgetExhausted
        | CloseOutcome::Expired
        | CloseOutcome::Cancelled
        | CloseOutcome::HumanDecisionRequired
        | CloseOutcome::UnsafeToContinue => true,
        CloseOutcome::Decided
        | CloseOutcome::Inconclusive
        | CloseOutcome::AcceptedUnanimously
        | CloseOutcome::AcceptedWithRecordedObjections
        | CloseOutcome::AcceptedByRule
        | CloseOutcome::AdvisoryAnswerOnly
        | CloseOutcome::Deadlocked
        | CloseOutcome::NoQuorum => false,
    }
}

fn derived(outcome: CloseOutcome) -> ClosedAs {
    ClosedAs {
        outcome,
        provenance: Provenance::Derived,
    }
}

fn caller_asserted(outcome: CloseOutcome) -> ClosedAs {
    ClosedAs {
        outcome,
        provenance: Provenance::CallerAsserted,
    }
}

/// The close classification (§5): given the thread's declared rule, its count,
/// who is closing and what they asserted, the outcome the close records and
/// where it came from — or the refusal.
///
/// `asserted` is `None` when the close names no outcome, which under a declared
/// rule is the normal path: the server supplies the derived result.
pub fn classify_close(
    rule: Option<DecisionRule>,
    threshold: Option<f64>,
    tally: &Tally,
    closer_is_owner: bool,
    asserted: Option<CloseOutcome>,
) -> Result<ClosedAs, CloseRefusal> {
    // §6: a thread that declares no rule closes exactly as before — and the
    // caller-asserted provenance is now recorded rather than implicit.
    let Some(rule) = rule else {
        return Ok(caller_asserted(asserted.unwrap_or_default()));
    };
    if let Some(process) = asserted.filter(|a| is_process_terminal(*a)) {
        return Ok(caller_asserted(process));
    }
    match evaluation(rule) {
        Evaluation::Counted => {
            let counted = count(rule, threshold, tally).map_err(CloseRefusal::Count)?;
            match asserted {
                None => Ok(derived(counted)),
                Some(CloseOutcome::AdvisoryAnswerOnly) => Err(CloseRefusal::RuleBinds { rule }),
                // The legacy aliases compare by their canonical word.
                Some(a) if a.canonical() == counted.canonical() => Ok(derived(counted)),
                Some(a) => Err(CloseRefusal::Disagrees {
                    asserted: a.canonical(),
                    counted: counted.canonical(),
                }),
            }
        }
        Evaluation::ByOwner => match asserted {
            None | Some(CloseOutcome::Decided | CloseOutcome::AcceptedByRule) => {
                if closer_is_owner {
                    Ok(derived(CloseOutcome::AcceptedByRule))
                } else {
                    Err(CloseRefusal::NotTheOwner)
                }
            }
            Some(CloseOutcome::AdvisoryAnswerOnly) => Err(CloseRefusal::RuleBinds { rule }),
            Some(a @ (CloseOutcome::Deadlocked | CloseOutcome::Inconclusive)) => {
                Ok(caller_asserted(a))
            }
            Some(
                a @ (CloseOutcome::AcceptedUnanimously
                | CloseOutcome::AcceptedWithRecordedObjections
                | CloseOutcome::NoQuorum),
            ) => Err(CloseRefusal::NothingCounted {
                asserted: a.canonical(),
                rule,
            }),
            Some(
                CloseOutcome::InsufficientEvidence
                | CloseOutcome::BudgetExhausted
                | CloseOutcome::Expired
                | CloseOutcome::Cancelled
                | CloseOutcome::HumanDecisionRequired
                | CloseOutcome::UnsafeToContinue,
            ) => unreachable!("the process terminals returned above"),
        },
        Evaluation::Advisory => match asserted {
            None | Some(CloseOutcome::AdvisoryAnswerOnly) => {
                Ok(derived(CloseOutcome::AdvisoryAnswerOnly))
            }
            Some(a @ (CloseOutcome::Deadlocked | CloseOutcome::Inconclusive)) => {
                Ok(caller_asserted(a))
            }
            Some(
                a @ (CloseOutcome::Decided
                | CloseOutcome::AcceptedUnanimously
                | CloseOutcome::AcceptedWithRecordedObjections
                | CloseOutcome::AcceptedByRule
                | CloseOutcome::NoQuorum),
            ) => Err(CloseRefusal::NoBindingDecision {
                asserted: a.canonical(),
            }),
            Some(
                CloseOutcome::InsufficientEvidence
                | CloseOutcome::BudgetExhausted
                | CloseOutcome::Expired
                | CloseOutcome::Cancelled
                | CloseOutcome::HumanDecisionRequired
                | CloseOutcome::UnsafeToContinue,
            ) => unreachable!("the process terminals returned above"),
        },
        Evaluation::Uncountable => Err(CloseRefusal::Uncountable(rule)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_OUTCOMES: [CloseOutcome; 14] = [
        CloseOutcome::Decided,
        CloseOutcome::Inconclusive,
        CloseOutcome::AcceptedUnanimously,
        CloseOutcome::AcceptedWithRecordedObjections,
        CloseOutcome::AcceptedByRule,
        CloseOutcome::AdvisoryAnswerOnly,
        CloseOutcome::Deadlocked,
        CloseOutcome::NoQuorum,
        CloseOutcome::InsufficientEvidence,
        CloseOutcome::BudgetExhausted,
        CloseOutcome::Expired,
        CloseOutcome::Cancelled,
        CloseOutcome::HumanDecisionRequired,
        CloseOutcome::UnsafeToContinue,
    ];

    fn tally(n: u64, a: u64, r: u64, s: u64) -> Tally {
        Tally {
            electorate: n,
            approve: a,
            reject: r,
            abstain: s,
        }
    }

    fn majority(t: f64, x: Tally) -> &'static str {
        count(DecisionRule::MajorityOfElectorate, Some(t), &x)
            .unwrap()
            .canonical()
    }

    fn counted(rule: DecisionRule, x: Tally) -> &'static str {
        count(rule, None, &x).unwrap().canonical()
    }

    #[test]
    fn the_tally_counts_each_choice_and_derives_the_outstanding() {
        use BallotChoice::*;
        let t = Tally::of(5, &[Approve, Reject, Abstain, Approve]);
        assert_eq!(t, tally(5, 2, 1, 1));
        assert_eq!(t.outstanding(), 1);
    }

    #[test]
    fn the_families_split_three_counted_two_derived_two_refused() {
        let by = |e: Evaluation| -> Vec<&str> {
            DecisionRule::ALL
                .into_iter()
                .filter(|r| evaluation(*r) == e)
                .map(|r| r.as_str())
                .collect()
        };
        assert_eq!(
            by(Evaluation::Counted),
            ["majority_of_electorate", "unanimity", "consensus"]
        );
        assert_eq!(by(Evaluation::ByOwner), ["owner_decides"]);
        assert_eq!(by(Evaluation::Advisory), ["advisory_synthesis"]);
        assert_eq!(
            by(Evaluation::Uncountable),
            ["role_weighted", "human_committee"]
        );
    }

    #[test]
    fn the_approvals_needed_sit_exactly_at_the_threshold() {
        // Measured witnesses: both products land one ulp ABOVE a whole number.
        assert_eq!(
            approvals_needed(0.56, 25),
            14,
            "0.56 * 25.0 is 14.000000000000002"
        );
        assert_eq!(
            approvals_needed(0.55, 100),
            55,
            "0.55 * 100.0 is 55.00000000000001"
        );
        assert_eq!(approvals_needed(0.51, 100), 51);
        assert_eq!(approvals_needed(2.0 / 3.0, 3), 2);
        assert_eq!(approvals_needed(0.667, 3), 3, "0.667 is above two thirds");
        assert_eq!(approvals_needed(0.6, 5), 3);
        assert_eq!(approvals_needed(1.0, 7), 7);
        assert_eq!(approvals_needed(0.51, 2), 2, "a tie never carries");
    }

    #[test]
    fn a_majority_carries_at_its_bar_and_not_one_short() {
        // 0.6 of 5 needs 3.
        assert_eq!(majority(0.6, tally(5, 3, 0, 0)), "accepted_by_rule");
        assert_eq!(
            majority(0.6, tally(5, 2, 0, 0)),
            "no_quorum",
            "three outstanding could carry it"
        );
        assert_eq!(
            majority(0.6, tally(5, 2, 3, 0)),
            "deadlocked",
            "no outstanding ballot remains"
        );
        assert_eq!(
            majority(0.6, tally(5, 2, 2, 0)),
            "no_quorum",
            "one outstanding could make three"
        );
        assert_eq!(
            majority(0.6, tally(5, 2, 1, 2)),
            "deadlocked",
            "abstentions are cast"
        );
    }

    #[test]
    fn a_majority_names_how_it_carried() {
        assert_eq!(majority(0.6, tally(5, 5, 0, 0)), "accepted_unanimously");
        assert_eq!(
            majority(0.6, tally(5, 4, 1, 0)),
            "accepted_with_recorded_objections"
        );
        assert_eq!(
            majority(0.6, tally(5, 4, 0, 1)),
            "accepted_by_rule",
            "an abstention is no objection"
        );
        assert_eq!(
            majority(0.6, tally(5, 4, 0, 0)),
            "accepted_by_rule",
            "one never voted"
        );
    }

    #[test]
    fn a_majority_tie_never_carries() {
        assert_eq!(majority(0.51, tally(4, 2, 2, 0)), "deadlocked");
        assert_eq!(majority(0.51, tally(4, 2, 0, 0)), "no_quorum");
    }

    #[test]
    fn a_majority_without_its_threshold_is_refused_rather_than_guessed() {
        assert_eq!(
            count(DecisionRule::MajorityOfElectorate, None, &tally(3, 3, 0, 0)),
            Err(CountError::ThresholdMissing)
        );
    }

    #[test]
    fn unanimity_is_withheld_by_a_reject_or_an_abstention() {
        use DecisionRule::Unanimity as U;
        assert_eq!(counted(U, tally(3, 3, 0, 0)), "accepted_unanimously");
        assert_eq!(counted(U, tally(3, 2, 1, 0)), "deadlocked");
        assert_eq!(
            counted(U, tally(3, 2, 0, 1)),
            "deadlocked",
            "§13.4: an abstainer is still applicable and withheld"
        );
        assert_eq!(counted(U, tally(3, 2, 0, 0)), "no_quorum");
        assert_eq!(
            counted(U, tally(3, 1, 1, 0)),
            "deadlocked",
            "a reject settles it even with ballots outstanding"
        );
    }

    #[test]
    fn consensus_is_blocked_by_a_reject_and_not_by_a_stand_aside() {
        use DecisionRule::Consensus as C;
        assert_eq!(counted(C, tally(3, 3, 0, 0)), "accepted_unanimously");
        assert_eq!(
            counted(C, tally(3, 2, 0, 1)),
            "accepted_by_rule",
            "a stand-aside does not block"
        );
        assert_eq!(
            counted(C, tally(3, 2, 1, 0)),
            "deadlocked",
            "a reject is the block"
        );
        assert_eq!(
            counted(C, tally(3, 2, 0, 0)),
            "no_quorum",
            "silence is never consent"
        );
        assert_eq!(
            counted(C, tally(3, 0, 0, 3)),
            "deadlocked",
            "nobody supports it"
        );
    }

    #[test]
    fn consensus_never_records_objections_because_an_objection_blocks() {
        use DecisionRule::Consensus as C;
        for a in 0..=3 {
            for r in 0..=(3 - a) {
                for s in 0..=(3 - a - r) {
                    assert_ne!(
                        counted(C, tally(3, a, r, s)),
                        "accepted_with_recorded_objections"
                    );
                }
            }
        }
    }

    #[test]
    fn an_empty_electorate_decides_nothing_under_every_counted_family() {
        for rule in DecisionRule::ALL
            .into_iter()
            .filter(|r| evaluation(*r) == Evaluation::Counted)
        {
            assert_eq!(
                count(rule, Some(0.6), &Tally::default()).unwrap(),
                CloseOutcome::NoQuorum,
                "{}",
                rule.as_str()
            );
        }
    }

    #[test]
    fn a_family_that_counts_no_ballots_refuses_to_be_counted() {
        for rule in [
            DecisionRule::OwnerDecides,
            DecisionRule::AdvisorySynthesis,
            DecisionRule::RoleWeighted,
            DecisionRule::HumanCommittee,
        ] {
            assert_eq!(
                count(rule, None, &tally(1, 1, 0, 0)),
                Err(CountError::NotCounted(rule))
            );
        }
    }

    /// What the §5 table says for one cell.
    #[derive(Debug, PartialEq)]
    enum Cell {
        Derived(&'static str),
        Caller,
        Refused,
    }

    fn cell(r: Result<ClosedAs, CloseRefusal>) -> Cell {
        match r {
            Ok(ClosedAs {
                outcome,
                provenance: Provenance::Derived,
            }) => Cell::Derived(outcome.canonical()),
            Ok(ClosedAs {
                provenance: Provenance::CallerAsserted,
                ..
            }) => Cell::Caller,
            Err(_) => Cell::Refused,
        }
    }

    /// §5, every terminal under every rule, as a table rather than examples.
    #[test]
    fn every_terminal_is_classified_under_every_rule() {
        // A unanimous ballot, so a counted family's result is known per family.
        let box3 = tally(3, 3, 0, 0);
        for asserted in ALL_OUTCOMES {
            let word = asserted.canonical();
            let process = is_process_terminal(asserted);

            // No rule: everything caller-asserted, exactly as before.
            assert_eq!(
                cell(classify_close(None, None, &box3, false, Some(asserted))),
                Cell::Caller,
                "no rule / {word}"
            );

            // The counted families: the counted word is derived, any other
            // ballot word disagrees, `advisory_answer_only` is refused.
            for rule in [
                DecisionRule::MajorityOfElectorate,
                DecisionRule::Unanimity,
                DecisionRule::Consensus,
            ] {
                let got = cell(classify_close(
                    Some(rule),
                    Some(0.6),
                    &box3,
                    false,
                    Some(asserted),
                ));
                let expected = if process {
                    Cell::Caller
                } else if word == "accepted_unanimously" {
                    Cell::Derived("accepted_unanimously")
                } else {
                    Cell::Refused
                };
                assert_eq!(got, expected, "{} / {word}", rule.as_str());
            }

            // owner_decides, closed by the owner.
            let owner = cell(classify_close(
                Some(DecisionRule::OwnerDecides),
                None,
                &box3,
                true,
                Some(asserted),
            ));
            let expected = match word {
                _ if process => Cell::Caller,
                "accepted_by_rule" => Cell::Derived("accepted_by_rule"),
                "deadlocked" => Cell::Caller,
                _ => Cell::Refused,
            };
            assert_eq!(owner, expected, "owner_decides / {word}");

            // advisory_synthesis.
            let advisory = cell(classify_close(
                Some(DecisionRule::AdvisorySynthesis),
                None,
                &box3,
                true,
                Some(asserted),
            ));
            let expected = match word {
                _ if process => Cell::Caller,
                "advisory_answer_only" => Cell::Derived("advisory_answer_only"),
                "deadlocked" => Cell::Caller,
                _ => Cell::Refused,
            };
            assert_eq!(advisory, expected, "advisory_synthesis / {word}");
        }
    }

    #[test]
    fn the_four_ballot_words_are_derived_under_a_counted_rule() {
        // One tally per word, under majority 0.6 of 5.
        for (x, word) in [
            (tally(5, 5, 0, 0), CloseOutcome::AcceptedUnanimously),
            (
                tally(5, 4, 1, 0),
                CloseOutcome::AcceptedWithRecordedObjections,
            ),
            (tally(5, 3, 0, 0), CloseOutcome::AcceptedByRule),
            (tally(5, 1, 0, 0), CloseOutcome::NoQuorum),
        ] {
            let rule = Some(DecisionRule::MajorityOfElectorate);
            assert_eq!(
                classify_close(rule, Some(0.6), &x, false, Some(word)),
                Ok(derived(word)),
                "{} agrees with its count",
                word.canonical()
            );
            assert_eq!(
                classify_close(rule, Some(0.6), &x, false, None),
                Ok(derived(word)),
                "an omitted outcome closes with the count"
            );
        }
    }

    #[test]
    fn a_close_that_disagrees_with_the_count_names_both_words() {
        let refused = classify_close(
            Some(DecisionRule::Unanimity),
            None,
            &tally(3, 2, 1, 0),
            true,
            Some(CloseOutcome::AcceptedUnanimously),
        )
        .unwrap_err();
        assert_eq!(
            refused,
            CloseRefusal::Disagrees {
                asserted: "accepted_unanimously",
                counted: "deadlocked"
            }
        );
        let rendered = refused.to_string();
        assert!(rendered.contains("accepted_unanimously") && rendered.contains("deadlocked"));
    }

    #[test]
    fn a_legacy_alias_is_compared_by_its_canonical_word() {
        let rule = Some(DecisionRule::MajorityOfElectorate);
        assert_eq!(
            classify_close(
                rule,
                Some(0.6),
                &tally(5, 3, 0, 0),
                false,
                Some(CloseOutcome::Decided)
            ),
            Ok(derived(CloseOutcome::AcceptedByRule))
        );
        assert_eq!(
            classify_close(
                rule,
                Some(0.6),
                &tally(5, 1, 4, 0),
                false,
                Some(CloseOutcome::Inconclusive)
            ),
            Ok(derived(CloseOutcome::Deadlocked))
        );
    }

    #[test]
    fn owner_decides_is_the_owners_close_and_nobody_elses() {
        let rule = Some(DecisionRule::OwnerDecides);
        let empty = Tally::default();
        assert_eq!(
            classify_close(rule, None, &empty, true, None),
            Ok(derived(CloseOutcome::AcceptedByRule))
        );
        assert_eq!(
            classify_close(rule, None, &empty, false, None),
            Err(CloseRefusal::NotTheOwner)
        );
        assert_eq!(
            classify_close(
                rule,
                None,
                &empty,
                false,
                Some(CloseOutcome::AcceptedByRule)
            ),
            Err(CloseRefusal::NotTheOwner)
        );
    }

    #[test]
    fn a_process_terminal_is_caller_asserted_even_over_a_carried_count() {
        assert_eq!(
            classify_close(
                Some(DecisionRule::Unanimity),
                None,
                &tally(3, 3, 0, 0),
                false,
                Some(CloseOutcome::UnsafeToContinue)
            ),
            Ok(caller_asserted(CloseOutcome::UnsafeToContinue)),
            "overriding agreement with a safety stop is the direction §13.4 permits"
        );
    }

    #[test]
    fn an_uncountable_family_is_refused_at_the_close_too() {
        for rule in [DecisionRule::RoleWeighted, DecisionRule::HumanCommittee] {
            assert_eq!(
                classify_close(Some(rule), None, &tally(1, 1, 0, 0), true, None),
                Err(CloseRefusal::Uncountable(rule))
            );
        }
    }

    #[test]
    fn a_rule_less_close_keeps_the_legacy_default() {
        assert_eq!(
            classify_close(None, None, &Tally::default(), false, None),
            Ok(caller_asserted(CloseOutcome::Decided))
        );
    }
}
