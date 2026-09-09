//! Deterministic candidate selection on the command's transaction executor.
use super::*;

const CANDIDATE_PAGE_SIZE: usize = 32;

pub(super) struct AuthoritySelection {
    pub(super) boundary: Option<EnrollmentAuthorityBoundary>,
    pub(super) grant: Option<AuthorityGrant>,
    pub(super) decision: Decision,
}

impl AuthoritySelection {
    fn absent() -> Self {
        Self {
            boundary: None,
            grant: None,
            decision: Decision::Denied {
                reason: "no applicable grant: tenant membership alone grants no authority".into(),
            },
        }
    }
}

/// Select a usable grant, including requested delegation attenuation. A refusal
/// retains the first candidate's actual parent and decision; absence has neither.
/// Pages bound the number of buffered rows without a total cutoff that could hide
/// valid authority. This loader does not itself serialize concurrent revocation.
pub(super) async fn select_authority_in_tx<E>(
    mut conn: E,
    authz: &CommandAuthz,
    at: DateTime<Utc>,
) -> Result<AuthoritySelection, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let subject = authz.delegate_subject.as_ref().unwrap_or(&authz.principal);
    let (kind, id) = subject_parts(subject);
    let tenant = authz.target.tenant_id().to_string();
    let mut cursor: Option<(DateTime<Utc>, String)> = None;
    let mut first_refusal = None;
    loop {
        let rows: Vec<GrantRow> = sqlx::query_as(
            "SELECT grant_id, boundary_id, tenant_id, issuer, subject_kind, subject_id, actions, \
                    selector, risk_ceiling, spend_limits, delegable, valid_from, expires_at, status \
             FROM authority_grants \
             WHERE tenant_id = $1 AND subject_kind = $2 AND subject_id = $3 AND status = 'active' \
               AND ($4::timestamptz IS NULL OR valid_from < $4 \
                    OR (valid_from = $4 AND grant_id COLLATE \"C\" > $5::text)) \
             ORDER BY valid_from DESC, grant_id COLLATE \"C\" ASC LIMIT $6",
        )
        .bind(&tenant)
        .bind(kind)
        .bind(&id)
        .bind(cursor.as_ref().map(|(time, _)| *time))
        .bind(cursor.as_ref().map(|(_, id)| id.as_str()))
        .bind(CANDIDATE_PAGE_SIZE as i64)
        .fetch_all(&mut *conn)
        .await?;
        let last_page = rows.len() < CANDIDATE_PAGE_SIZE;
        for row in rows {
            let grant = grant_from_row(row).ok_or_else(|| {
                sqlx::Error::Protocol("stored authority grant is malformed".into())
            })?;
            cursor = Some((grant.valid_from, grant.grant_id.clone()));
            let boundary = load_boundary_by_id_in_tx(&mut *conn, &grant.boundary_id).await?;
            let mut decision = evaluate(Some(&boundary), Some(&grant), authz, at);
            if decision == Decision::Allowed && authz.delegate_subject.is_some() {
                if let Some(scope) = &authz.delegation_scope {
                    if !delegation_scope_is_subset(&target_to_selector(&authz.target), scope)
                        || !delegation_scope_is_subset(scope, &grant.selector)
                    {
                        decision = Decision::Denied {
                            reason: "the delegation scope is wider than the subject's grant or \
                                     does not cover the request's target (the §16.3 widening invariant)"
                                .into(),
                        };
                    }
                }
            }
            let selected = AuthoritySelection {
                boundary: Some(boundary),
                grant: Some(grant),
                decision,
            };
            if selected.decision == Decision::Allowed {
                return Ok(selected);
            }
            if first_refusal.is_none() {
                first_refusal = Some(selected);
            }
        }
        if last_page {
            return Ok(first_refusal.unwrap_or_else(AuthoritySelection::absent));
        }
    }
}
