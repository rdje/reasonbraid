use super::*;

/// Parsed, bounded commands. HTTP extraction/CLI parsing owns malformed wire
/// input; every command reaching this service receives a decision audit unless
/// a database failure prevents the transaction from completing.
#[derive(Debug, Clone)]
pub enum RegistryCommand {
    ListAdapters,
    ListRegions,
    AllowAdapter {
        adapter_id: RegistryName,
        reason: Reason,
    },
    RevokeAdapter {
        adapter_id: RegistryName,
        reason: Reason,
    },
    DeclareRegion {
        region: RegistryName,
        reason: Reason,
    },
    PairRegions {
        from: RegistryName,
        to: RegistryName,
        reason: Reason,
    },
    UnpairRegions {
        from: RegistryName,
        to: RegistryName,
        reason: Reason,
    },
}

impl RegistryCommand {
    fn action(&self) -> Action {
        match self {
            Self::ListAdapters | Self::ListRegions => Action::RegistryInspect,
            Self::AllowAdapter { .. } => Action::AdapterAllow,
            Self::RevokeAdapter { .. } => Action::AdapterRevoke,
            Self::DeclareRegion { .. } => Action::RegionDeclare,
            Self::PairRegions { .. } => Action::RegionPair,
            Self::UnpairRegions { .. } => Action::RegionUnpair,
        }
    }

    fn reason(&self) -> &str {
        match self {
            Self::ListAdapters | Self::ListRegions => "inspect registry",
            Self::AllowAdapter { reason, .. }
            | Self::RevokeAdapter { reason, .. }
            | Self::DeclareRegion { reason, .. }
            | Self::PairRegions { reason, .. }
            | Self::UnpairRegions { reason, .. } => reason.as_str(),
        }
    }

    fn target(&self) -> Value {
        match self {
            Self::ListAdapters => json!({"registry": "adapters"}),
            Self::ListRegions => json!({"registry": "regions"}),
            Self::AllowAdapter { adapter_id, .. } | Self::RevokeAdapter { adapter_id, .. } => {
                json!({"adapter_id": adapter_id})
            }
            Self::DeclareRegion { region, .. } => json!({"region": region}),
            Self::PairRegions { from, to, .. } | Self::UnpairRegions { from, to, .. } => {
                json!({"from": from, "to": to})
            }
        }
    }
}

struct Effect {
    result: Value,
    outcome: &'static str,
}

impl Effect {
    fn write(result: Value, changed: u64) -> Self {
        Self {
            result,
            outcome: if changed == 0 { "noop" } else { "applied" },
        }
    }
}

/// Authorize, mutate/read and audit as one ordered site transaction. A denial
/// commits its own record; an audit failure rolls back an otherwise allowed write.
pub async fn execute(
    pool: &PgPool,
    subject: &GrantSubject,
    command: &RegistryCommand,
) -> Result<Receipt, Error> {
    let mut tx = begin(pool).await?;
    let at = lock(&mut tx).await?;
    let (actor_kind, actor) = subject_parts(subject);
    let intent = Intent {
        actor_kind,
        actor,
        action: command.action().as_str(),
        target: command.target(),
        requested_reason: command.reason().to_owned(),
    };
    let evaluation = evaluate(&mut tx, subject, command.action(), at).await?;
    if evaluation.grant_id.is_none() {
        let audit_id = audit(
            &mut tx,
            &intent,
            Outcome {
                grant_id: None,
                boundary_id: None,
                outcome: "denied",
                reason: "site_authority_required",
                evaluation: evaluation.checks,
                at,
            },
        )
        .await?;
        tx.commit().await?;
        return Err(Error::Refused {
            reason: "site_authority_required",
            audit_id,
        });
    }
    // Domain refusals are separate from SQL errors. In particular, an unavailable
    // database must never masquerade as an undeclared region.
    if let RegistryCommand::PairRegions { from, to, .. } = command {
        let declared: Vec<String> = sqlx::query_scalar(
            "SELECT region_id FROM public.site_regions WHERE region_id = $1 OR region_id = $2",
        )
        .bind(from.as_str())
        .bind(to.as_str())
        .fetch_all(&mut *tx)
        .await?;
        if !declared.iter().any(|value| value == from.as_str())
            || !declared.iter().any(|value| value == to.as_str())
        {
            let audit_id = audit(
                &mut tx,
                &intent,
                Outcome {
                    grant_id: evaluation.grant_id,
                    boundary_id: evaluation.boundary_id,
                    outcome: "denied",
                    reason: "undeclared_region",
                    evaluation: evaluation.checks,
                    at,
                },
            )
            .await?;
            tx.commit().await?;
            return Err(Error::Refused {
                reason: "undeclared_region",
                audit_id,
            });
        }
    }
    let effect = apply(&mut tx, &intent.actor, command).await?;
    let audit_id = audit(
        &mut tx,
        &intent,
        Outcome {
            grant_id: evaluation.grant_id,
            boundary_id: evaluation.boundary_id,
            outcome: effect.outcome,
            reason: effect.outcome,
            evaluation: evaluation.checks,
            at,
        },
    )
    .await?;
    tx.commit().await?;
    Ok(Receipt {
        audit_id,
        result: effect.result,
    })
}

async fn apply(tx: &mut Tx<'_>, actor: &str, command: &RegistryCommand) -> Result<Effect, Error> {
    Ok(match command {
        RegistryCommand::ListAdapters => {
            let rows: Vec<(String, String, String, DateTime<Utc>)> = sqlx::query_as(
                "SELECT adapter_id, added_by, reason, added_at FROM public.adapter_allowlist ORDER BY adapter_id",
            ).fetch_all(&mut **tx).await?;
            Effect {
                result: json!({"adapters": rows.into_iter().map(|(id, by, reason, at)| json!({"adapter_id": id, "added_by": by, "reason": reason, "added_at": at})).collect::<Vec<_>>()}),
                outcome: "inspected",
            }
        }
        RegistryCommand::ListRegions => {
            let regions: Vec<String> =
                sqlx::query_scalar("SELECT region_id FROM public.site_regions ORDER BY region_id")
                    .fetch_all(&mut **tx)
                    .await?;
            let pairs: Vec<(String, String)> = sqlx::query_as("SELECT from_region, to_region FROM public.region_pairs ORDER BY from_region, to_region")
                .fetch_all(&mut **tx).await?;
            Effect {
                result: json!({"regions": regions, "pairs": pairs.into_iter().map(|(from, to)| json!({"from": from, "to": to})).collect::<Vec<_>>()}),
                outcome: "inspected",
            }
        }
        RegistryCommand::AllowAdapter { adapter_id, reason } => {
            let changed = sqlx::query(
                "INSERT INTO public.adapter_allowlist (adapter_id, added_by, reason) VALUES ($1,$2,$3) \
                 ON CONFLICT (adapter_id) DO NOTHING",
            ).bind(adapter_id.as_str()).bind(actor).bind(reason.as_str()).execute(&mut **tx).await?.rows_affected();
            Effect::write(json!({"adapter_id": adapter_id, "allowed": true}), changed)
        }
        RegistryCommand::RevokeAdapter { adapter_id, .. } => {
            let changed = sqlx::query("DELETE FROM public.adapter_allowlist WHERE adapter_id = $1")
                .bind(adapter_id.as_str())
                .execute(&mut **tx)
                .await?
                .rows_affected();
            Effect::write(json!({"adapter_id": adapter_id, "revoked": true}), changed)
        }
        RegistryCommand::DeclareRegion { region, .. } => {
            let changed = sqlx::query("INSERT INTO public.site_regions (region_id) VALUES ($1) ON CONFLICT (region_id) DO NOTHING")
                .bind(region.as_str()).execute(&mut **tx).await?.rows_affected();
            Effect::write(json!({"region": region, "declared": true}), changed)
        }
        RegistryCommand::PairRegions { from, to, .. } => {
            let changed = sqlx::query("INSERT INTO public.region_pairs (from_region, to_region) VALUES ($1,$2) ON CONFLICT (from_region, to_region) DO NOTHING")
                .bind(from.as_str()).bind(to.as_str()).execute(&mut **tx).await?.rows_affected();
            Effect::write(json!({"from": from, "to": to, "paired": true}), changed)
        }
        RegistryCommand::UnpairRegions { from, to, .. } => {
            let changed = sqlx::query(
                "DELETE FROM public.region_pairs WHERE from_region = $1 AND to_region = $2",
            )
            .bind(from.as_str())
            .bind(to.as_str())
            .execute(&mut **tx)
            .await?
            .rows_affected();
            Effect::write(json!({"from": from, "to": to, "unpaired": true}), changed)
        }
    })
}
