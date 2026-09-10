use super::{pg_cleanup, pg_test_support};
use pg_cleanup::{delete_tables, CleanupError};
use sqlx::PgPool;

struct PlanFixture {
    pool: PgPool,
    parent: String,
    child: String,
    witness: String,
    constraint: String,
}

impl PlanFixture {
    async fn new(prefix: &str) -> Option<Self> {
        assert!(prefix.len() <= 25);
        assert!(prefix.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'));
        let pool = pg_test_support::pool().await?;
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("migrate owned fixture");
        let fixture = Self {
            pool,
            parent: format!("{prefix}_parent"),
            child: format!("{prefix}_child"),
            witness: format!("{prefix}_witness"),
            constraint: format!("{prefix}_child_fk"),
        };
        fixture
            .sql(&format!(
                "CREATE TABLE public.\"{}\" (id INTEGER PRIMARY KEY);
                 CREATE TABLE public.\"{}\" (parent_id INTEGER,
                     CONSTRAINT \"{}\" FOREIGN KEY (parent_id) REFERENCES public.\"{}\" (id));
                 CREATE TABLE public.\"{}\" (id INTEGER PRIMARY KEY);
                 INSERT INTO public.\"{}\" VALUES (1);
                 INSERT INTO public.\"{}\" VALUES (1);",
                fixture.parent,
                fixture.child,
                fixture.constraint,
                fixture.parent,
                fixture.witness,
                fixture.parent,
                fixture.witness,
            ))
            .await;
        Some(fixture)
    }

    async fn sql(&self, statement: &str) {
        sqlx::raw_sql(statement)
            .execute(&self.pool)
            .await
            .expect("owned fixture SQL");
    }

    async fn counts(&self) -> (i64, i64, i64) {
        sqlx::query_as(&format!(
            "SELECT (SELECT count(*) FROM public.\"{}\"),
                    (SELECT count(*) FROM public.\"{}\"),
                    (SELECT count(*) FROM public.\"{}\")",
            self.parent, self.child, self.witness,
        ))
        .fetch_one(&self.pool)
        .await
        .expect("read parent, child and unrelated witness")
    }

    async fn seed_child(&self) {
        self.sql(&format!("INSERT INTO public.\"{}\" VALUES (1)", self.child))
            .await;
    }

    async fn finish(self) {
        self.sql(&format!(
            "DROP TABLE public.\"{}\"; DROP TABLE public.\"{}\"; DROP TABLE public.\"{}\";",
            self.child, self.parent, self.witness,
        ))
        .await;
        self.pool.close().await;
    }
}

#[tokio::test]
async fn invalid_or_incomplete_plans_refuse_before_any_delete() {
    let Some(f) = PlanFixture::new("fixture_plan_input").await else {
        return;
    };
    let before = f.counts().await;
    assert_eq!(before, (1, 0, 1));
    for invalid in [
        String::new(),
        "Uppercase".to_owned(),
        "0leading_digit".to_owned(),
        "public.qualified".to_owned(),
        "x\"; DELETE FROM tenants; --".to_owned(),
        "é".to_owned(),
        "nul\0name".to_owned(),
        "a".repeat(64),
    ] {
        let error = delete_tables(&f.pool, &[&f.witness, &invalid])
            .await
            .expect_err("invalid plan must refuse");
        assert!(matches!(&error, CleanupError::InvalidName(name) if name == &invalid));
        assert_eq!(f.counts().await, before, "invalid plan: {invalid:?}");
    }
    let duplicate = delete_tables(&f.pool, &[&f.witness, &f.witness])
        .await
        .unwrap_err();
    assert!(matches!(duplicate, CleanupError::Duplicate(ref name) if name == &f.witness));
    assert_eq!(f.counts().await, before);
    let unknown = delete_tables(&f.pool, &[&f.witness, "fixture_plan_missing"])
        .await
        .unwrap_err();
    assert!(matches!(unknown, CleanupError::UnsupportedTable(_)));
    assert_eq!(f.counts().await, before);

    // Even an empty child is a required dependency. The witness deliberately
    // precedes the bad entry, exposing validation interleaved with deletion.
    let missing = delete_tables(&f.pool, &[&f.witness, &f.parent])
        .await
        .unwrap_err();
    assert!(
        matches!(&missing, CleanupError::MissingDependency { parent, child, constraint }
        if parent == &f.parent && child == &format!("public.{}", f.child)
            && constraint == &f.constraint)
    );
    assert!(missing.to_string().contains(&f.constraint));
    assert_eq!(f.counts().await, before);

    f.seed_child().await;
    let reversed = delete_tables(&f.pool, &[&f.witness, &f.parent, &f.child])
        .await
        .unwrap_err();
    assert!(matches!(reversed, CleanupError::DependencyOrder { .. }));
    assert_eq!(f.counts().await, (1, 1, 1));
    delete_tables(&f.pool, &[]).await.unwrap();
    assert_eq!(f.counts().await, (1, 1, 1));
    delete_tables(&f.pool, &[&f.child, &f.parent])
        .await
        .unwrap();
    assert_eq!(f.counts().await, (0, 0, 1));
    f.finish().await;
}

#[tokio::test]
async fn views_inheritance_cross_schema_and_cycles_refuse_without_deletion() {
    let Some(f) = PlanFixture::new("fixture_plan_scope").await else {
        return;
    };
    f.seed_child().await;
    f.sql(&format!(
        "CREATE VIEW fixture_plan_view AS SELECT * FROM public.\"{}\";
         CREATE TABLE fixture_plan_inherited_parent (id INTEGER);
         CREATE TABLE fixture_plan_inherited_child () INHERITS (fixture_plan_inherited_parent);",
        f.parent,
    ))
    .await;
    for unsupported in [
        "fixture_plan_view",
        "fixture_plan_inherited_parent",
        "fixture_plan_inherited_child",
    ] {
        let error = delete_tables(&f.pool, &[&f.witness, unsupported])
            .await
            .unwrap_err();
        assert!(matches!(error, CleanupError::UnsupportedTable(ref name) if name == unsupported));
        assert_eq!(f.counts().await, (1, 1, 1));
    }
    f.sql(&format!(
        "CREATE SCHEMA fixture_plan_external;
         CREATE TABLE fixture_plan_external.\"{}\" (parent_id INTEGER,
             CONSTRAINT fixture_plan_external_fk FOREIGN KEY (parent_id) REFERENCES public.\"{}\" (id));
         INSERT INTO fixture_plan_external.\"{}\" VALUES (1);",
        f.child, f.parent, f.child,
    ))
    .await;
    let cross_schema = delete_tables(&f.pool, &[&f.witness, &f.child, &f.parent])
        .await
        .unwrap_err();
    assert!(
        matches!(&cross_schema, CleanupError::MissingDependency { child, constraint, .. }
        if child == &format!("fixture_plan_external.{}", f.child)
            && constraint == "fixture_plan_external_fk")
    );
    let external_count: i64 = sqlx::query_scalar(&format!(
        "SELECT count(*) FROM fixture_plan_external.\"{}\"",
        f.child,
    ))
    .fetch_one(&f.pool)
    .await
    .unwrap();
    assert_eq!(external_count, 1);
    assert_eq!(f.counts().await, (1, 1, 1));
    f.sql(&format!(
        "DROP TABLE fixture_plan_external.\"{}\"; DROP SCHEMA fixture_plan_external;
         CREATE TABLE fixture_plan_cycle_a (id INTEGER PRIMARY KEY, peer INTEGER);
         CREATE TABLE fixture_plan_cycle_b (id INTEGER PRIMARY KEY, peer INTEGER);
         ALTER TABLE fixture_plan_cycle_a ADD CONSTRAINT fixture_plan_a_fk
             FOREIGN KEY (peer) REFERENCES fixture_plan_cycle_b (id);
         ALTER TABLE fixture_plan_cycle_b ADD CONSTRAINT fixture_plan_b_fk
             FOREIGN KEY (peer) REFERENCES fixture_plan_cycle_a (id);",
        f.child,
    ))
    .await;
    for (first, second) in [
        ("fixture_plan_cycle_a", "fixture_plan_cycle_b"),
        ("fixture_plan_cycle_b", "fixture_plan_cycle_a"),
    ] {
        let error = delete_tables(&f.pool, &[&f.witness, first, second])
            .await
            .unwrap_err();
        assert!(matches!(error, CleanupError::DependencyOrder { .. }));
        assert_eq!(f.counts().await, (1, 1, 1));
    }
    f.sql(
        "ALTER TABLE fixture_plan_cycle_a DROP CONSTRAINT fixture_plan_a_fk;
         ALTER TABLE fixture_plan_cycle_b DROP CONSTRAINT fixture_plan_b_fk;
         DROP TABLE fixture_plan_cycle_a; DROP TABLE fixture_plan_cycle_b;
         DROP VIEW fixture_plan_view; DROP TABLE fixture_plan_inherited_child;
         DROP TABLE fixture_plan_inherited_parent;",
    )
    .await;
    f.finish().await;
}

#[tokio::test]
async fn cascading_and_set_actions_require_explicit_children() {
    let Some(f) = PlanFixture::new("fixture_plan_actions").await else {
        return;
    };
    f.seed_child().await;
    for (suffix, action) in [
        ("cascade", "CASCADE"),
        ("null", "SET NULL"),
        ("default", "SET DEFAULT"),
    ] {
        let table = format!("fixture_plan_{suffix}");
        let constraint = format!("{table}_fk");
        f.sql(&format!(
            "CREATE TABLE public.\"{table}\" (parent_id INTEGER DEFAULT NULL,
                CONSTRAINT \"{constraint}\" FOREIGN KEY (parent_id)
                    REFERENCES public.\"{}\" (id) ON DELETE {action});
             INSERT INTO public.\"{table}\" VALUES (1);",
            f.parent,
        ))
        .await;
        let error = delete_tables(&f.pool, &[&f.witness, &f.child, &f.parent])
            .await
            .unwrap_err();
        assert!(
            matches!(&error, CleanupError::MissingDependency { child, constraint: actual, .. }
            if child == &format!("public.{table}") && actual == &constraint)
        );
        assert_eq!(f.counts().await, (1, 1, 1));
        let referenced: i64 = sqlx::query_scalar(&format!(
            "SELECT count(*) FROM public.\"{table}\" WHERE parent_id = 1",
        ))
        .fetch_one(&f.pool)
        .await
        .unwrap();
        assert_eq!(referenced, 1);
        delete_tables(&f.pool, &[&f.child, &table, &f.parent])
            .await
            .unwrap();
        assert_eq!(f.counts().await, (0, 0, 1));
        let remaining: i64 =
            sqlx::query_scalar(&format!("SELECT count(*) FROM public.\"{table}\""))
                .fetch_one(&f.pool)
                .await
                .unwrap();
        assert_eq!(remaining, 0);
        f.sql(&format!(
            "DROP TABLE public.\"{table}\"; INSERT INTO public.\"{}\" VALUES (1);",
            f.parent
        ))
        .await;
        f.seed_child().await;
    }
    f.finish().await;
}

#[tokio::test]
async fn database_errors_keep_their_cause_and_late_partial_effects_visible() {
    let Some(f) = PlanFixture::new("fixture_plan_error").await else {
        return;
    };
    f.seed_child().await;
    let refused = sqlx::query(&format!("DELETE FROM public.\"{}\"", f.parent))
        .execute(&f.pool)
        .await
        .unwrap_err();
    assert_eq!(
        refused.as_database_error().unwrap().code().as_deref(),
        Some("23503")
    );
    f.sql(&format!(
        "CREATE FUNCTION fixture_plan_refuse_delete() RETURNS trigger LANGUAGE plpgsql AS $$
             BEGIN RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'owned cleanup failure'; END $$;
         CREATE TRIGGER fixture_plan_refuse_delete BEFORE DELETE ON public.\"{}\"
             FOR EACH ROW EXECUTE FUNCTION fixture_plan_refuse_delete();",
        f.parent,
    ))
    .await;
    let error = delete_tables(&f.pool, &[&f.child, &f.parent])
        .await
        .unwrap_err();
    let CleanupError::Database(cause) = &error else {
        panic!("expected database cause: {error}")
    };
    assert_eq!(
        cause.as_database_error().unwrap().code().as_deref(),
        Some("P0001")
    );
    assert!(std::error::Error::source(&error).is_some());
    // Preflight errors are mutation-free; later SQL errors retain the existing
    // statement-by-statement semantics and must not imply an atomic rollback.
    assert_eq!(f.counts().await, (1, 0, 1));
    f.sql(&format!(
        "DROP TRIGGER fixture_plan_refuse_delete ON public.\"{}\";
         DROP FUNCTION fixture_plan_refuse_delete();",
        f.parent,
    ))
    .await;
    f.seed_child().await;
    delete_tables(&f.pool, &[&f.child, &f.parent])
        .await
        .unwrap();
    assert_eq!(f.counts().await, (0, 0, 1));
    let orphan = sqlx::query(&format!("INSERT INTO public.\"{}\" VALUES (999)", f.child))
        .execute(&f.pool)
        .await
        .unwrap_err();
    assert_eq!(
        orphan.as_database_error().unwrap().constraint(),
        Some(f.constraint.as_str())
    );
    f.finish().await;
}

#[tokio::test]
async fn self_referencing_table_is_deleted_as_one_statement() {
    let Some(f) = PlanFixture::new("fixture_plan_self").await else {
        return;
    };
    f.sql(
        "CREATE TABLE fixture_plan_self_tree (id INTEGER PRIMARY KEY, parent_id INTEGER,
             CONSTRAINT fixture_plan_self_fk FOREIGN KEY (parent_id) REFERENCES fixture_plan_self_tree (id));
         INSERT INTO fixture_plan_self_tree VALUES (1, NULL), (2, 1);",
    )
    .await;
    delete_tables(&f.pool, &["fixture_plan_self_tree"])
        .await
        .unwrap();
    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM fixture_plan_self_tree")
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert_eq!(remaining, 0);
    assert_eq!(f.counts().await, (1, 0, 1));
    f.sql("DROP TABLE fixture_plan_self_tree").await;
    f.finish().await;
}
