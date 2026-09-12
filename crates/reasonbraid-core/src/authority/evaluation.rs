//! Explicit audit provenance with object-only, duplicate-preserving decoding.
//!
//! [`object_only`] is shared with `authority/effect.rs`, which needs the same
//! refusal of the sequence and unit-marker alternatives for its own tagged types.
use super::{AuthorizationRecordId, BoundaryStatus, GrantSubject, TargetSelector};
use crate::ThreadId;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::{fmt, marker::PhantomData};

/// The exact administrative inspection admitted by the frozen-read exception.
/// Declaring a variant does not imply that its HTTP route is implemented.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TenantAdminInspection {
    NodesPresence {},
    Grants {},
    Boundaries {},
    Incarnations {},
    Runs {},
    Breakers {},
    Usage {},
    AuthorizationRecord { record_id: AuthorizationRecordId },
}

/// Which evaluator produced an admission, not whether a domain effect or response
/// delivery succeeded. Historical records retain unknown provenance explicitly.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthorizationEvaluation {
    LegacyUnspecified {},
    /// Ordinary evaluation requiring an active boundary, including ordinary reads.
    BoundaryChecked {},
    /// Named direct own-tenant read exception. A frozen parent keeps its actual
    /// status; evidence is absent only when no authority source was selected.
    TenantAdminInspection {
        principal: GrantSubject,
        inspection: TenantAdminInspection,
        boundary_status: Option<BoundaryStatus>,
        grant_selector: Option<TargetSelector>,
    },
}

impl Default for AuthorizationEvaluation {
    fn default() -> Self {
        Self::LegacyUnspecified {}
    }
}

// The pinned Serde internal-tag decoder accepts sequence alternatives and unit
// variants discard extra fields. First require a map, then use empty struct
// markers for strict field decoding. Passing MapAccess directly preserves
// duplicate keys; normalizing through serde_json::Value would lose that evidence.
pub(super) fn object_only<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct ObjectVisitor<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> de::Visitor<'de> for ObjectVisitor<T> {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a tagged authority metadata object")
        }

        fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<T, A::Error> {
            T::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }
    deserializer.deserialize_map(ObjectVisitor(PhantomData))
}

// Scope is shared with grants and command envelopes. Preserve its public variants
// and schema while refusing a tenant-wide marker carrying discarded thread fields.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum SelectorWire {
    TenantWide {},
    Threads { threads: Vec<ThreadId> },
}

impl<'de> Deserialize<'de> for TargetSelector {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match object_only::<D, SelectorWire>(deserializer)? {
            SelectorWire::TenantWide {} => Self::TenantWide,
            SelectorWire::Threads { threads } => Self::Threads { threads },
        })
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum InspectionWire {
    NodesPresence {},
    Grants {},
    Boundaries {},
    Incarnations {},
    Runs {},
    Breakers {},
    Usage {},
    AuthorizationRecord { record_id: AuthorizationRecordId },
}

impl<'de> Deserialize<'de> for TenantAdminInspection {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match object_only::<D, InspectionWire>(deserializer)? {
            InspectionWire::NodesPresence {} => Self::NodesPresence {},
            InspectionWire::Grants {} => Self::Grants {},
            InspectionWire::Boundaries {} => Self::Boundaries {},
            InspectionWire::Incarnations {} => Self::Incarnations {},
            InspectionWire::Runs {} => Self::Runs {},
            InspectionWire::Breakers {} => Self::Breakers {},
            InspectionWire::Usage {} => Self::Usage {},
            InspectionWire::AuthorizationRecord { record_id } => {
                Self::AuthorizationRecord { record_id }
            }
        })
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum EvaluationWire {
    LegacyUnspecified {},
    BoundaryChecked {},
    TenantAdminInspection {
        principal: GrantSubject,
        inspection: TenantAdminInspection,
        boundary_status: Option<BoundaryStatus>,
        grant_selector: Option<TargetSelector>,
    },
}

impl<'de> Deserialize<'de> for AuthorizationEvaluation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match object_only::<D, EvaluationWire>(deserializer)? {
            EvaluationWire::LegacyUnspecified {} => Self::LegacyUnspecified {},
            EvaluationWire::BoundaryChecked {} => Self::BoundaryChecked {},
            EvaluationWire::TenantAdminInspection {
                principal,
                inspection,
                boundary_status,
                grant_selector,
            } => Self::TenantAdminInspection {
                principal,
                inspection,
                boundary_status,
                grant_selector,
            },
        })
    }
}
