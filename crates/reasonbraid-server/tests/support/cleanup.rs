//! Checked DELETE plans for exclusive, ownership-verified PostgreSQL fixtures.
//!
//! Callers obtain the pool through `pg_test_support` and finish migrations first.
//! No concurrent fixture mutation or DDL is supported. Validation precedes every
//! DELETE, but a later database error can leave earlier DELETE statements committed.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use sqlx::PgPool;

#[derive(Debug)]
pub enum CleanupError {
    InvalidName(String),
    Duplicate(String),
    UnsupportedTable(String),
    MissingDependency {
        parent: String,
        child: String,
        constraint: String,
    },
    DependencyOrder {
        parent: String,
        child: String,
        constraint: String,
    },
    Database(sqlx::Error),
}

impl fmt::Display for CleanupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName(name) => write!(f, "invalid fixture table name: {name:?}"),
            Self::Duplicate(name) => write!(f, "duplicate fixture table: {name}"),
            Self::UnsupportedTable(name) => {
                write!(f, "missing or unsupported fixture table: public.{name}")
            }
            Self::MissingDependency {
                parent,
                child,
                constraint,
            } => write!(
                f,
                "fixture cleanup of public.{parent} omits {child} (constraint {constraint})"
            ),
            Self::DependencyOrder {
                parent,
                child,
                constraint,
            } => write!(
                f,
                "fixture cleanup must delete {child} before public.{parent} (constraint {constraint})"
            ),
            Self::Database(error) => write!(f, "fixture cleanup database error: {error}"),
        }
    }
}

impl std::error::Error for CleanupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for CleanupError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

fn canonical_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 63
        && name.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase() || byte == b'_' || (index > 0 && byte.is_ascii_digit())
        })
}

/// Delete an explicit child-before-parent plan after validating its whole scope.
///
/// Only ordinary public tables without inheritance or partitioning are supported.
/// All incoming FK actions require a declared earlier child, including CASCADE
/// and SET NULL/DEFAULT. Cross-schema dependencies and cycles refuse. A self-FK
/// is left to PostgreSQL's checks during the complete single-table DELETE.
pub async fn delete_tables(pool: &PgPool, tables: &[&str]) -> Result<(), CleanupError> {
    let mut positions = BTreeMap::new();
    for (index, table) in tables.iter().copied().enumerate() {
        if !canonical_name(table) {
            return Err(CleanupError::InvalidName(table.to_owned()));
        }
        if positions.insert(table, index).is_some() {
            return Err(CleanupError::Duplicate(table.to_owned()));
        }
    }
    if tables.is_empty() {
        return Ok(());
    }

    let mut connection = pool.acquire().await?;
    let supported: BTreeSet<String> = sqlx::query_scalar::<_, String>(
        "SELECT c.relname::text FROM pg_catalog.pg_class c
         JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = 'public' AND c.relkind = 'r'
           AND NOT EXISTS (SELECT 1 FROM pg_catalog.pg_inherits i
                           WHERE i.inhparent = c.oid OR i.inhrelid = c.oid)",
    )
    .fetch_all(&mut *connection)
    .await?
    .into_iter()
    .collect();
    for table in tables {
        if !supported.contains(*table) {
            return Err(CleanupError::UnsupportedTable((*table).to_owned()));
        }
    }

    let dependencies: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT fk.conname::text, child_ns.nspname::text, child.relname::text,
                parent.relname::text
         FROM pg_catalog.pg_constraint fk
         JOIN pg_catalog.pg_class child ON child.oid = fk.conrelid
         JOIN pg_catalog.pg_namespace child_ns ON child_ns.oid = child.relnamespace
         JOIN pg_catalog.pg_class parent ON parent.oid = fk.confrelid
         JOIN pg_catalog.pg_namespace parent_ns ON parent_ns.oid = parent.relnamespace
         WHERE fk.contype = 'f' AND parent_ns.nspname = 'public'
           AND fk.conrelid <> fk.confrelid
         ORDER BY parent.relname, child_ns.nspname, child.relname, fk.conname",
    )
    .fetch_all(&mut *connection)
    .await?;
    for (constraint, child_schema, child, parent) in dependencies {
        let Some(&parent_position) = positions.get(parent.as_str()) else {
            continue;
        };
        let child_position = (child_schema == "public")
            .then(|| positions.get(child.as_str()).copied())
            .flatten();
        let child = format!("{child_schema}.{child}");
        let Some(child_position) = child_position else {
            return Err(CleanupError::MissingDependency {
                parent,
                child,
                constraint,
            });
        };
        if child_position >= parent_position {
            return Err(CleanupError::DependencyOrder {
                parent,
                child,
                constraint,
            });
        }
    }

    for table in tables {
        sqlx::query(&format!("DELETE FROM public.\"{table}\""))
            .execute(&mut *connection)
            .await?;
    }
    Ok(())
}
