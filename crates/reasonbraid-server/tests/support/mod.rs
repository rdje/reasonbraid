// Test-only PostgreSQL ownership proof, shared by server, CLI and MCP tests.
// Never export this module from a production library. Destructive fixtures must
// obtain their pool here before migrations, purges, schema drops or role changes.

use std::fs::{self, File, Metadata};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgConnection, PgPool};

const REFUSAL: &str = "disposable PostgreSQL ownership required; run bash scripts/run_pg_tests.sh";

#[derive(Deserialize)]
struct Receipt {
    workspace: String,
    database: String,
    port: u16,
    state: String,
    postgres_pid: u32,
}

#[derive(Clone)]
pub struct Ownership {
    url: String,
    directory: PathBuf,
    database: String,
    token: String,
}

fn same_volume(left: &Metadata, right: &Metadata) -> bool {
    #[cfg(unix)]
    {
        left.dev() == right.dev()
    }
    #[cfg(not(unix))]
    {
        let _ = (left, right);
        false // The supervised runner requires POSIX process groups/signals.
    }
}

fn regular_file(path: &Path, volume: &Metadata) -> Result<String, &'static str> {
    let metadata = fs::symlink_metadata(path).map_err(|_| REFUSAL)?;
    if !metadata.is_file() || !same_volume(&metadata, volume) {
        return Err(REFUSAL);
    }
    let file = File::open(path).map_err(|_| REFUSAL)?;
    if !same_volume(&file.metadata().map_err(|_| REFUSAL)?, volume) {
        return Err(REFUSAL);
    }
    let mut content = String::new();
    file.take(65_537)
        .read_to_string(&mut content)
        .map_err(|_| REFUSAL)?;
    if content.len() > 65_536 {
        return Err(REFUSAL);
    }
    Ok(content)
}

fn hex_token(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

pub fn repository_root() -> Result<PathBuf, &'static str> {
    let cwd = std::env::current_dir().map_err(|_| REFUSAL)?;
    cwd.ancestors()
        .find(|path| {
            path.join("Cargo.toml").is_file()
                && path.join("scripts/project_env.py").is_file()
                && path.join("rust-toolchain.toml").is_file()
        })
        .ok_or(REFUSAL)?
        .canonicalize()
        .map_err(|_| REFUSAL)
}

impl Ownership {
    /// Validate the local receipt and exact URL before attempting any connection.
    /// Explicit inputs let negative controls run without racing process env vars.
    pub fn read(
        root: &Path,
        url: &str,
        relative: &str,
        token: &str,
        database: &str,
    ) -> Result<Self, &'static str> {
        if !hex_token(token, 48)
            || !database
                .strip_prefix("rb_test_")
                .is_some_and(|suffix| hex_token(suffix, 24))
        {
            return Err(REFUSAL);
        }
        let parts: Vec<_> = Path::new(relative).components().collect();
        if parts.len() != 3
            || parts[0] != Component::Normal("target".as_ref())
            || parts[1] != Component::Normal("pg-tests".as_ref())
        {
            return Err(REFUSAL);
        }
        let Component::Normal(name) = parts[2] else {
            return Err(REFUSAL);
        };
        let name = name.to_str().ok_or(REFUSAL)?;
        if !name.starts_with("run-")
            || !(5..=64).contains(&name.len())
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(REFUSAL);
        }
        let root = root.canonicalize().map_err(|_| REFUSAL)?;
        let volume = fs::metadata(&root).map_err(|_| REFUSAL)?;
        let mut workspace = root;
        for part in parts {
            workspace.push(part);
            let metadata = fs::symlink_metadata(&workspace).map_err(|_| REFUSAL)?;
            if !metadata.is_dir() || !same_volume(&metadata, &volume) {
                return Err(REFUSAL);
            }
        }
        let receipt: Receipt =
            serde_json::from_str(&regular_file(&workspace.join("runner.json"), &volume)?)
                .map_err(|_| REFUSAL)?;
        if receipt.workspace != relative
            || receipt.database != database
            || receipt.state != "running"
            || receipt.port == 0
            || url
                != format!(
                    "postgres://postgres@127.0.0.1:{}/{database}?sslmode=disable",
                    receipt.port
                )
        {
            return Err(REFUSAL);
        }
        let directory = workspace.join("data");
        let data_metadata = fs::symlink_metadata(&directory).map_err(|_| REFUSAL)?;
        if !data_metadata.is_dir() || !same_volume(&data_metadata, &volume) {
            return Err(REFUSAL);
        }
        let pid_file = regular_file(&directory.join("postmaster.pid"), &volume)?;
        let pid: u32 = pid_file
            .lines()
            .next()
            .ok_or(REFUSAL)?
            .parse()
            .map_err(|_| REFUSAL)?;
        if pid == 0 || pid != receipt.postgres_pid {
            return Err(REFUSAL);
        }
        Ok(Self {
            url: url.to_owned(),
            directory,
            database: database.to_owned(),
            token: token.to_owned(),
        })
    }

    /// Every new physical connection must prove ownership before entering the pool.
    /// This also covers reconnects after the first successful fixture setup.
    pub async fn connect(&self) -> Result<PgPool, sqlx::Error> {
        let expected = self.clone();
        PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .after_connect(move |connection, _metadata| {
                let expected = expected.clone();
                Box::pin(async move { expected.verify(connection).await })
            })
            .connect(&self.url)
            .await
    }

    /// Role-specific test pools use this callback before publishing each new
    /// connection, after preflighting the runner's canonical administrator URL.
    pub async fn verify(&self, connection: &mut PgConnection) -> Result<(), sqlx::Error> {
        let actual: (String, String, Option<String>) = sqlx::query_as(
            "SELECT current_setting('data_directory'), current_database(), \
             current_setting('reasonbraid.test_owner', true)",
        )
        .fetch_one(connection)
        .await?;
        if Path::new(&actual.0) != self.directory
            || actual.1 != self.database
            || actual.2.as_deref() != Some(self.token.as_str())
        {
            return Err(sqlx::Error::Protocol(REFUSAL.to_owned()));
        }
        Ok(())
    }
}

pub async fn pool() -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => {
            eprintln!(
                "SKIP: DATABASE_URL unset; run bash scripts/run_pg_tests.sh for live verification"
            );
            return None;
        }
        Err(_) => panic!("{REFUSAL}"),
    };
    let required = |name| std::env::var(name).expect(REFUSAL);
    let ownership = Ownership::read(
        &repository_root().expect(REFUSAL),
        &url,
        &required("RB_TEST_CLUSTER"),
        &required("RB_TEST_OWNER"),
        &required("RB_TEST_DATABASE"),
    )
    .expect(REFUSAL);
    Some(ownership.connect().await.expect(REFUSAL))
}
