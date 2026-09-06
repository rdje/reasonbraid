//! Strong, non-interchangeable identifier newtypes.
//!
//! `ROADMAP.md` §8.3: "use newtypes for every ID; never interchange plain UUID strings
//! inside domain code." §8.1 names the identity hierarchy. This module turns that rule
//! into a type: [`Id<K>`] is a branded newtype over [`uuid::Uuid`] (v7, sortable per
//! §17.2), and each identifier family is a distinct marker `K`.

use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

/// A type-level marker naming one identifier family.
///
/// Each family carries a distinct wire prefix and human label, so two different
/// identifier types can never serialize to the same form, and deserializing a value
/// into the wrong family is rejected.
pub trait IdKind {
    /// Wire prefix (for example `"ten"`, `"thr"`). It appears in the serialized form
    /// `"{PREFIX}_{uuid}"` and is validated on the way back in.
    const PREFIX: &'static str;

    /// Human-readable label, used by [`fmt::Debug`] and diagnostics.
    const LABEL: &'static str;
}

/// A strongly typed, globally unique, non-semantic identifier (`ROADMAP.md` §8.3).
///
/// `Id<K>` is a newtype over [`uuid::Uuid`] branded by the marker `K`. Identifiers of
/// different kinds are different Rust types: they cannot be assigned, compared, or
/// passed for one another without an explicit, prefix-checked conversion.
///
/// On the wire each `Id<K>` serializes to `"{PREFIX}_{uuid}"` (for example
/// `"thr_018f…"` for a [`ThreadId`]); deserialization re-checks the prefix, so a
/// `ThreadId` can never be read back as a [`TenantId`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id<K: IdKind> {
    uuid: Uuid,
    _kind: PhantomData<K>,
}

impl<K: IdKind> Id<K> {
    /// Generate a new identifier of kind `K` backed by a random, sortable UUIDv7.
    pub fn new() -> Self {
        Self::from_uuid(Uuid::now_v7())
    }

    /// Wrap an existing UUID as an identifier of kind `K`.
    ///
    /// This is deliberate and explicit — there is deliberately no blanket `From<Uuid>`
    /// for every `Id<K>`, because that would let a caller silently re-brand any UUID as
    /// any kind. Naming `K` here is the visible act of choosing the kind.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self {
            uuid,
            _kind: PhantomData,
        }
    }

    /// The underlying UUID (by reference).
    pub fn as_uuid(&self) -> &Uuid {
        &self.uuid
    }

    /// Consume the identifier, returning its underlying UUID.
    pub fn into_uuid(self) -> Uuid {
        self.uuid
    }
}

impl<K: IdKind> Default for Id<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: IdKind> fmt::Display for Id<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", K::PREFIX, self.uuid)
    }
}

impl<K: IdKind> fmt::Debug for Id<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}Id({})", K::LABEL, self)
    }
}

impl<K: IdKind> Serialize for Id<K> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, K: IdKind> Deserialize<'de> for Id<K> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

impl<K: IdKind> FromStr for Id<K> {
    type Err = IdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s
            .strip_prefix(K::PREFIX)
            .ok_or_else(|| IdParseError::WrongPrefix {
                expected: K::PREFIX,
                got: s.to_owned(),
            })?;
        let rest = rest
            .strip_prefix('_')
            .ok_or_else(|| IdParseError::MissingSeparator {
                value: s.to_owned(),
            })?;
        let uuid = Uuid::parse_str(rest).map_err(|e| IdParseError::InvalidUuid {
            value: rest.to_owned(),
            reason: e.to_string(),
        })?;
        Ok(Self::from_uuid(uuid))
    }
}

/// Error parsing an [`Id`] from its wire form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdParseError {
    /// The value does not begin with this identifier family's prefix.
    WrongPrefix { expected: &'static str, got: String },
    /// The prefix is present but not followed by the `_` separator.
    MissingSeparator { value: String },
    /// The suffix after the prefix is not a valid UUID.
    InvalidUuid { value: String, reason: String },
}

impl fmt::Display for IdParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdParseError::WrongPrefix { expected, got } => {
                write!(f, "expected id prefix `{expected}_` but got `{got}`")
            }
            IdParseError::MissingSeparator { value } => {
                write!(
                    f,
                    "id `{value}` is missing the `_` separator after its prefix"
                )
            }
            IdParseError::InvalidUuid { value, reason } => {
                write!(f, "id suffix `{value}` is not a valid UUID: {reason}")
            }
        }
    }
}

impl std::error::Error for IdParseError {}

impl<K: IdKind> JsonSchema for Id<K> {
    fn schema_name() -> Cow<'static, str> {
        format!("{}Id", K::LABEL).into()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "description": format!(
                "{} identifier; wire form `{}_{{uuid}}` (UUIDv7).",
                K::LABEL, K::PREFIX
            ),
            "pattern": format!("^{}_", K::PREFIX)
        })
    }
}

// ── The eight identifier families (ROADMAP.md §8.1) ─────────────────────────

/// Declares one identifier family: its zero-sized marker, its [`IdKind`] impl, and its
/// `*Id` type alias. Prefixes are distinct so serialized forms can never collide and
/// deserialization is prefix-checked.
macro_rules! id_family {
    ($(#[$doc:meta])* marker $marker:ident, alias $alias:ident, prefix $prefix:literal, label $label:literal $(,)?) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $marker;

        impl IdKind for $marker {
            const PREFIX: &'static str = $prefix;
            const LABEL: &'static str = $label;
        }

        #[doc = concat!("A strongly typed identifier for the `", stringify!($marker), "` family.")]
        pub type $alias = Id<$marker>;
    };
}

id_family! {
    /// A tenant — the organization/scope boundary (`ROADMAP.md` §8.1, §16.8).
    marker Tenant, alias TenantId, prefix "ten", label "Tenant"
}
id_family! {
    /// A durable human identity that can hold authority and act (`ROADMAP.md` §8.1).
    marker HumanPrincipal, alias HumanPrincipalId, prefix "hpr", label "HumanPrincipal"
}
id_family! {
    /// A machine (physical or virtual) that runs one or more nodes (`ROADMAP.md` §8.1).
    marker Host, alias HostId, prefix "hst", label "Host"
}
id_family! {
    /// A running `reasonbraid-node` daemon on a host (`ROADMAP.md` §8.1, §11.1).
    marker NodeInstance, alias NodeId, prefix "nod", label "NodeInstance"
}
id_family! {
    /// A durable agent role — purpose, subscriptions, authority, history (`ROADMAP.md` §8.1).
    marker AgentRole, alias AgentRoleId, prefix "rol", label "AgentRole"
}
id_family! {
    /// One provider/model/harness/configuration of a role (`ROADMAP.md` §8.1).
    marker AgentIncarnation, alias AgentIncarnationId, prefix "inc", label "AgentIncarnation"
}
id_family! {
    /// One supervised execution with a budget and provider receipts (`ROADMAP.md` §8.1).
    marker Run, alias RunId, prefix "run", label "Run"
}
id_family! {
    /// A conversation/deliberation aggregate — scope, question, workflow, visibility (`ROADMAP.md` §8.2).
    marker Thread, alias ThreadId, prefix "thr", label "Thread"
}

// ── Envelope-scoped identifiers (ROADMAP.md §9.1) — added in PHASE-0.1.2 ────

id_family! {
    /// A committed event's identity (`ROADMAP.md` §9.1, §8.3).
    marker Event, alias EventId, prefix "evt", label "Event"
}
id_family! {
    /// A client request identity; also the `causation_id` target of an event (`ROADMAP.md` §9.1).
    marker Request, alias RequestId, prefix "req", label "Request"
}
id_family! {
    /// A correlation scope spanning several commands/events (`ROADMAP.md` §9.1).
    marker Correlation, alias CorrelationId, prefix "corr", label "Correlation"
}
id_family! {
    /// The authenticated actor principal the server resolved for a command (`ROADMAP.md` §9.1).
    /// Opaque here: which principal kind (`hpr`/`rol`) an `agt` names is a WP5 identity concern.
    marker ActorPrincipal, alias ActorPrincipalId, prefix "agt", label "ActorPrincipal"
}
id_family! {
    /// A server-assigned authorization audit record reference (`ROADMAP.md` §9.1).
    marker AuthorizationRecord, alias AuthorizationRecordId, prefix "authz", label "AuthorizationRecord"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The core guarantee: distinct things stay distinct in types. If any two
    /// identifiers collapsed into one type (e.g. all plain `String` aliases), the whole
    /// point of the newtype rule would be lost — so assert all identifier kinds are
    /// distinct concrete types, not aliases of one another.
    #[test]
    fn id_kinds_are_distinct_types() {
        let kinds = [
            std::any::TypeId::of::<TenantId>(),
            std::any::TypeId::of::<HumanPrincipalId>(),
            std::any::TypeId::of::<HostId>(),
            std::any::TypeId::of::<NodeId>(),
            std::any::TypeId::of::<AgentRoleId>(),
            std::any::TypeId::of::<AgentIncarnationId>(),
            std::any::TypeId::of::<RunId>(),
            std::any::TypeId::of::<ThreadId>(),
            std::any::TypeId::of::<EventId>(),
            std::any::TypeId::of::<RequestId>(),
            std::any::TypeId::of::<CorrelationId>(),
            std::any::TypeId::of::<ActorPrincipalId>(),
            std::any::TypeId::of::<AuthorizationRecordId>(),
        ];
        for i in 0..kinds.len() {
            for j in (i + 1)..kinds.len() {
                assert_ne!(
                    kinds[i], kinds[j],
                    "identifier kinds at indices {i} and {j} collapsed into one type"
                );
            }
        }
    }

    /// New identifiers are unique and carry the sortable v7 version (ROADMAP.md §17.2).
    #[test]
    fn new_ids_are_unique_v7() {
        let a = ThreadId::new();
        let b = ThreadId::new();
        assert_ne!(a, b);
        assert_eq!(a.as_uuid().get_version_num(), 7);
        assert_eq!(b.as_uuid().get_version_num(), 7);
    }

    /// Wire forms are prefix-distinct: each family serializes under its own three-letter
    /// prefix and the suffix is always a valid UUID.
    #[test]
    fn wire_forms_carry_distinct_prefixes() {
        let cases = [
            (TenantId::new().to_string(), "ten"),
            (HumanPrincipalId::new().to_string(), "hpr"),
            (HostId::new().to_string(), "hst"),
            (NodeId::new().to_string(), "nod"),
            (AgentRoleId::new().to_string(), "rol"),
            (AgentIncarnationId::new().to_string(), "inc"),
            (RunId::new().to_string(), "run"),
            (ThreadId::new().to_string(), "thr"),
            (EventId::new().to_string(), "evt"),
            (RequestId::new().to_string(), "req"),
            (CorrelationId::new().to_string(), "corr"),
            (ActorPrincipalId::new().to_string(), "agt"),
            (AuthorizationRecordId::new().to_string(), "authz"),
        ];
        let mut seen = std::collections::HashSet::new();
        for (value, prefix) in cases {
            let rest = value
                .strip_prefix(prefix)
                .unwrap_or_else(|| panic!("{value} lacks prefix {prefix}"));
            let uuid_str = rest
                .strip_prefix('_')
                .unwrap_or_else(|| panic!("{value} lacks `_` separator after {prefix}"));
            assert!(Uuid::parse_str(uuid_str).is_ok());
            assert!(
                seen.insert(prefix),
                "prefix {prefix} used by more than one family"
            );
        }
    }

    /// The wire non-confusability guarantee: round-trip preserves the kind, and a value
    /// serialized for one family is rejected when read back as another.
    #[test]
    fn serde_round_trip_preserves_kind_and_rejects_wrong_prefix() {
        let tenant = TenantId::new();
        let json = serde_json::to_string(&tenant).unwrap();
        assert_eq!(json, format!("\"ten_{}\"", tenant.as_uuid()));

        let back: TenantId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, tenant);

        // A ThreadId string must NOT deserialize as a TenantId: the prefix is checked.
        let thread_json = serde_json::to_string(&ThreadId::new()).unwrap();
        assert!(serde_json::from_str::<TenantId>(&thread_json).is_err());
    }

    #[test]
    fn from_str_accepts_correct_prefix_and_rejects_others() {
        let id = RunId::new();
        let s = id.to_string();
        assert_eq!(s.parse::<RunId>().unwrap(), id);

        assert!(matches!(
            s.parse::<ThreadId>(),
            Err(IdParseError::WrongPrefix {
                expected: "thr",
                ..
            })
        ));
    }

    #[test]
    fn display_and_debug_are_self_describing() {
        let id = AgentRoleId::new();
        let display = id.to_string();
        assert!(display.starts_with("rol_"));
        let debug = format!("{id:?}");
        assert!(debug.starts_with("AgentRoleId(") && debug.ends_with(')'));
    }
}
