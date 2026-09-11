use super::*;

/// Deployment-controlled access; tenant subjects and tenant grants are never
/// consulted. SQL object names are qualified to prevent search-path substitution.
async fn operator_identity(tx: &mut Tx<'_>) -> Result<(String, bool, bool), Error> {
    // Use a freshly parsed simple-protocol statement. PostgreSQL 16 can retain
    // pg_has_role's membership cache across repeated prepared executions inside
    // a transaction, even after a concurrent REVOKE and a guard-lock wait.
    // This static SQL has no caller interpolation. The queued-revocation test
    // must continue to exercise a reused physical connection.
    // The audit table is identified by OID from the catalogue, never by a
    // qualified NAME. Resolving `'public.site_audit'` requires USAGE on the
    // schema, so a caller without it raised SQLSTATE 42501 and the service
    // reported a dependency failure when the truth was simply that the caller
    // is not an operator. The catalogue is readable by every role, so this form
    // keeps the refusal honest. A missing table yields false, not NULL.
    let row = sqlx::Executor::fetch_one(
        &mut **tx,
        "SELECT SESSION_USER::TEXT, r.rolsuper OR EXISTS ( \
             SELECT 1 FROM pg_catalog.pg_roles operator_role \
             WHERE operator_role.rolname = 'reasonbraid_site_operator' \
               AND pg_catalog.pg_has_role(r.oid, operator_role.oid, 'MEMBER')), \
             COALESCE(( \
                 SELECT pg_catalog.has_table_privilege(CURRENT_USER, audit_table.oid, 'INSERT') \
                 FROM pg_catalog.pg_class audit_table \
                 JOIN pg_catalog.pg_namespace audit_schema \
                   ON audit_schema.oid = audit_table.relnamespace \
                 WHERE audit_schema.nspname = 'public' \
                   AND audit_table.relname = 'site_audit'), false) \
         FROM pg_catalog.pg_roles r WHERE r.rolname = SESSION_USER",
    )
    .await?;
    Ok(sqlx::FromRow::from_row(&row)?)
}

async fn begin_operator<'a>(
    pool: &'a PgPool,
    intent: &mut Intent,
) -> Result<(Tx<'a>, DateTime<Utc>), Error> {
    let mut tx = begin(pool).await?;
    let (mut actor, mut allowed, mut can_audit) = operator_identity(&mut tx).await?;
    let at = if allowed {
        let at = lock(&mut tx).await?;
        // Recheck after the wait; preliminary access is not a cached permission.
        (actor, allowed, can_audit) = operator_identity(&mut tx).await?;
        at
    } else {
        sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&mut *tx)
            .await?
    };
    intent.actor = actor;
    if !allowed {
        if can_audit {
            let audit_id = audit(
                &mut tx,
                intent,
                Outcome {
                    grant_id: None,
                    boundary_id: None,
                    outcome: "denied",
                    reason: "operator_required",
                    evaluation: json!({"database_operator": false}),
                    at,
                },
            )
            .await?;
            tx.commit().await?;
            return Err(Error::Refused {
                reason: "operator_required",
                audit_id,
            });
        }
        tx.rollback().await?;
        return Err(Error::OperatorRequired);
    }
    Ok((tx, at))
}

fn intent(action: &'static str, target: Value, reason: &Reason) -> Intent {
    Intent {
        actor_kind: "database",
        actor: String::new(),
        action,
        target,
        requested_reason: reason.as_str().to_owned(),
    }
}

async fn refused(
    mut tx: Tx<'_>,
    intent: &Intent,
    at: DateTime<Utc>,
    reason: &'static str,
) -> Result<Receipt, Error> {
    let audit_id = audit(
        &mut tx,
        intent,
        Outcome {
            grant_id: None,
            boundary_id: None,
            outcome: "denied",
            reason,
            evaluation: json!({"database_operator": true}),
            at,
        },
    )
    .await?;
    tx.commit().await?;
    Err(Error::Refused { reason, audit_id })
}

/// Issue an immutable site boundary through deployment database authority.
pub async fn issue_boundary(
    pool: &PgPool,
    scope: &Scope,
    reason: &Reason,
) -> Result<Receipt, Error> {
    let scope = scope.checked()?;
    let id = identifier("sbd");
    let mut intent = intent(
        "boundary_issue",
        json!({"boundary_id": id, "scope": scope}),
        reason,
    );
    let (mut tx, at) = begin_operator(pool, &mut intent).await?;
    if scope.expires_at <= at {
        return refused(tx, &intent, at, "expired_scope").await;
    }
    sqlx::query(
        "INSERT INTO public.site_boundaries \
         (boundary_id, issued_by, actions, valid_from, expires_at, status, reason) \
         VALUES ($1,$2,$3,$4,$5,'active',$6)",
    )
    .bind(&id)
    .bind(&intent.actor)
    .bind(&scope.actions)
    .bind(scope.valid_from)
    .bind(scope.expires_at)
    .bind(reason.as_str())
    .execute(&mut *tx)
    .await?;
    let audit_id = audit(
        &mut tx,
        &intent,
        Outcome {
            grant_id: None,
            boundary_id: Some(id.clone()),
            outcome: "applied",
            reason: "issued",
            evaluation: json!({"database_operator": true}),
            at,
        },
    )
    .await?;
    tx.commit().await?;
    Ok(Receipt {
        audit_id,
        result: json!({"boundary_id": id, "issued_by": intent.actor, "scope": scope, "status": "active"}),
    })
}

#[derive(sqlx::FromRow)]
struct Ceiling {
    actions: Vec<String>,
    valid_from: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    status: String,
}

/// Issue a site grant bound to this exact parent. A subject may be independent
/// of tenant enrollment; only explicit operator issuance conveys site authority.
pub async fn issue_grant(
    pool: &PgPool,
    boundary_id: &str,
    subject: &GrantSubject,
    scope: &Scope,
    reason: &Reason,
) -> Result<Receipt, Error> {
    check_id(boundary_id, "sbd")?;
    let scope = scope.checked()?;
    let id = identifier("sgr");
    let (kind, subject_id) = subject_parts(subject);
    let subject_json = json!({"kind": kind, "id": subject_id});
    let mut intent = intent(
        "grant_issue",
        json!({"grant_id": id, "boundary_id": boundary_id, "subject": subject_json, "scope": scope}),
        reason,
    );
    let (mut tx, at) = begin_operator(pool, &mut intent).await?;
    let ceiling: Option<Ceiling> = sqlx::query_as(
        "SELECT actions, valid_from, expires_at, status FROM public.site_boundaries WHERE boundary_id = $1",
    ).bind(boundary_id).fetch_optional(&mut *tx).await?;
    let Some(ceiling) = ceiling else {
        return refused(tx, &intent, at, "boundary_not_found").await;
    };
    if ceiling.status != "active" || ceiling.expires_at <= at {
        return refused(tx, &intent, at, "boundary_unavailable").await;
    }
    if scope.expires_at <= at
        || scope.valid_from < ceiling.valid_from
        || scope.expires_at > ceiling.expires_at
        || !scope
            .actions
            .iter()
            .all(|action| ceiling.actions.contains(action))
    {
        return refused(tx, &intent, at, "grant_exceeds_boundary").await;
    }
    sqlx::query(
        "INSERT INTO public.site_grants \
         (grant_id, boundary_id, issued_by, subject_kind, subject_id, actions, valid_from, expires_at, status, reason) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'active',$9)",
    ).bind(&id).bind(boundary_id).bind(&intent.actor).bind(kind).bind(subject_id)
        .bind(&scope.actions).bind(scope.valid_from).bind(scope.expires_at).bind(reason.as_str())
        .execute(&mut *tx).await?;
    let audit_id = audit(
        &mut tx,
        &intent,
        Outcome {
            grant_id: Some(id.clone()),
            boundary_id: Some(boundary_id.to_owned()),
            outcome: "applied",
            reason: "issued",
            evaluation: json!({"database_operator": true}),
            at,
        },
    )
    .await?;
    tx.commit().await?;
    Ok(Receipt {
        audit_id,
        result: json!({"grant_id": id, "boundary_id": boundary_id, "issued_by": intent.actor, "subject": subject_json, "scope": scope, "status": "active"}),
    })
}

#[derive(Clone, Copy)]
enum Kind {
    Grant,
    Boundary,
}

/// A protected operator inventory. Registry inspection is a separate site grant
/// capability; it does not expose these deployment authority or audit records.
#[derive(Debug, Clone, Copy)]
pub enum Collection {
    Boundaries,
    Grants,
    Audit,
}

impl Collection {
    fn description(self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Boundaries => ("boundaries", "boundary_inspect", "sbd", "boundary_id"),
            Self::Grants => ("grants", "grant_inspect", "sgr", "grant_id"),
            Self::Audit => ("audit", "audit_inspect", "sau", "audit_id"),
        }
    }
}

/// Read one bounded newest-ID-first page and commit its attributable inspection.
/// Pages are current views; concurrent changes do not form a repeatable snapshot.
/// New inspection audit records do not extend a descending history walk.
pub async fn inspect(
    pool: &PgPool,
    collection: Collection,
    limit: u16,
    before: Option<&str>,
    reason: &Reason,
) -> Result<Receipt, Error> {
    if !(1..=100).contains(&limit) {
        return Err(Error::InvalidInput("page limit must be between 1 and 100"));
    }
    let (name, action, prefix, id_field) = collection.description();
    if let Some(id) = before {
        check_id(id, prefix)?;
    }
    let mut intent = intent(
        action,
        json!({"collection": name, "limit": limit, "before": before}),
        reason,
    );
    let (mut tx, at) = begin_operator(pool, &mut intent).await?;
    let query = match collection {
        Collection::Boundaries => "SELECT to_jsonb(b) FROM public.site_boundaries b WHERE $1::TEXT IS NULL OR boundary_id < $1 ORDER BY boundary_id DESC LIMIT $2",
        Collection::Grants => "SELECT to_jsonb(g) FROM public.site_grants g WHERE $1::TEXT IS NULL OR grant_id < $1 ORDER BY grant_id DESC LIMIT $2",
        Collection::Audit => "SELECT to_jsonb(a) FROM public.site_audit a WHERE $1::TEXT IS NULL OR audit_id < $1 ORDER BY audit_id DESC LIMIT $2",
    };
    let mut items: Vec<Value> = sqlx::query_scalar(query)
        .bind(before)
        .bind(i64::from(limit) + 1)
        .fetch_all(&mut *tx)
        .await?;
    let next_before = if items.len() > usize::from(limit) {
        items.truncate(usize::from(limit));
        items.last().map(|item| item[id_field].clone())
    } else {
        None
    };
    let audit_id = audit(
        &mut tx,
        &intent,
        Outcome {
            grant_id: None,
            boundary_id: None,
            outcome: "inspected",
            reason: "inspected",
            evaluation: json!({"database_operator": true}),
            at,
        },
    )
    .await?;
    tx.commit().await?;
    Ok(Receipt {
        audit_id,
        result: json!({"collection": name, "items": items, "next_before": next_before}),
    })
}

/// Suspension/revocation takes the same serialization guard as registry access.
pub async fn disable_grant(
    pool: &PgPool,
    id: &str,
    operation: Disable,
    reason: &Reason,
) -> Result<Receipt, Error> {
    disable(pool, Kind::Grant, id, operation, reason).await
}

pub async fn disable_boundary(
    pool: &PgPool,
    id: &str,
    operation: Disable,
    reason: &Reason,
) -> Result<Receipt, Error> {
    disable(pool, Kind::Boundary, id, operation, reason).await
}

async fn disable(
    pool: &PgPool,
    kind: Kind,
    id: &str,
    operation: Disable,
    reason: &Reason,
) -> Result<Receipt, Error> {
    check_id(
        id,
        match kind {
            Kind::Grant => "sgr",
            Kind::Boundary => "sbd",
        },
    )?;
    let action = match (kind, operation) {
        (Kind::Grant, Disable::Suspend) => "grant_suspend",
        (Kind::Grant, Disable::Revoke) => "grant_revoke",
        (Kind::Boundary, Disable::Suspend) => "boundary_suspend",
        (Kind::Boundary, Disable::Revoke) => "boundary_revoke",
    };
    let target = match kind {
        Kind::Grant => json!({"grant_id": id}),
        Kind::Boundary => json!({"boundary_id": id}),
    };
    let mut intent = intent(action, target, reason);
    let (mut tx, at) = begin_operator(pool, &mut intent).await?;
    let query = match kind {
        Kind::Grant => "SELECT status, boundary_id FROM public.site_grants WHERE grant_id = $1",
        Kind::Boundary => {
            "SELECT status, boundary_id FROM public.site_boundaries WHERE boundary_id = $1"
        }
    };
    let state: Option<(String, String)> = sqlx::query_as(query)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    let Some((previous, boundary_id)) = state else {
        return refused(tx, &intent, at, "authority_not_found").await;
    };
    let changed = previous != "revoked" && previous != operation.status();
    let status = if changed {
        operation.status()
    } else {
        &previous
    };
    if changed {
        let query = match kind {
            Kind::Grant => "UPDATE public.site_grants SET status = $2 WHERE grant_id = $1",
            Kind::Boundary => {
                "UPDATE public.site_boundaries SET status = $2 WHERE boundary_id = $1"
            }
        };
        sqlx::query(query)
            .bind(id)
            .bind(status)
            .execute(&mut *tx)
            .await?;
    }
    let audit_id = audit(&mut tx, &intent, Outcome {
        grant_id: matches!(kind, Kind::Grant).then(|| id.to_owned()), boundary_id: Some(boundary_id),
        outcome: if changed { "applied" } else { "noop" }, reason: if changed { "disabled" } else { "already_disabled" },
        evaluation: json!({"database_operator": true, "previous_status": previous, "status": status}), at,
    }).await?;
    tx.commit().await?;
    Ok(Receipt {
        audit_id,
        result: json!({"authority_id": id, "status": status, "changed": changed}),
    })
}
