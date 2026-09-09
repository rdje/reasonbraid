//! Bounded local state snapshots. Network request/recovery policy belongs to the
//! caller; this module owns path binding, file lifetime and publication durability.

use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::{CliError, StateFile};

/// Duplicate names or thread keys are ambiguous, even if their values happen to
/// match. The ordinary BTreeMap deserializer would silently keep the last one.
pub(super) fn unique_map<'de, D, T>(deserializer: D) -> Result<BTreeMap<String, T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Unique<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> Visitor<'de> for Unique<T> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("an object with unique keys")
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut entries = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, T>()? {
                if entries.insert(key, value).is_some() {
                    return Err(M::Error::custom("duplicate local state key"));
                }
            }
            Ok(entries)
        }
    }
    deserializer.deserialize_map(Unique(PhantomData))
}

fn invalid(detail: &str) -> CliError {
    CliError::state(detail.to_owned())
}

/// One update lifetime: fresh state is loaded only after acquiring the lock, and
/// the same descriptor remains held through asynchronous work and publication.
/// Dropping an interrupted operation releases exclusion without publishing it.
pub(crate) struct Writer {
    state: StateFile,
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    publication: unix::Publication,
}

impl Writer {
    pub(crate) fn open(path: &std::path::Path) -> Result<Self, CliError> {
        let writer = Self::open_recovery(path)?;
        if writer
            .state
            .bootstrap
            .as_ref()
            .is_some_and(|recovery| recovery.pending.is_some())
        {
            return Err(invalid("bootstrap recovery is pending; use the matching recovery operation before another writer"));
        }
        Ok(writer)
    }

    /// Only the bootstrap coordinator may select and reconcile pending intent.
    /// The same path, snapshot and lock checks apply to both entrypoints.
    pub(crate) fn open_recovery(path: &std::path::Path) -> Result<Self, CliError> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let publication = unix::Publication::open(path)?;
            let state = publication.load()?;
            Ok(Self { state, publication })
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            let _ = path;
            Err(invalid(
                "verified local state storage currently requires Linux or macOS",
            ))
        }
    }

    pub(crate) fn state(&self) -> &StateFile {
        &self.state
    }

    pub(crate) fn state_mut(&mut self) -> &mut StateFile {
        &mut self.state
    }

    /// Publish an intermediate snapshot while retaining this operation's lock.
    pub(crate) fn persist(&self) -> Result<(), CliError> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            self.publication.publish(&codec::encode(&self.state)?)
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err(invalid(
                "verified local state storage currently requires Linux or macOS",
            ))
        }
    }
    pub(crate) fn publish(self) -> Result<(), CliError> {
        self.persist()
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod codec {
    use super::{invalid, CliError, StateFile};
    use reasonbraid_core::{AgentRoleId, HumanPrincipalId, TenantId, ThreadId};
    use std::fmt;
    use std::io::{self, Write};
    use std::str::FromStr;

    pub(super) const MAX_STATE_BYTES: usize = 8 * 1024 * 1024;
    fn canonical<T: FromStr + fmt::Display>(raw: &str) -> bool {
        raw.parse::<T>().is_ok_and(|id| id.to_string() == raw)
    }

    fn validate(state: &StateFile) -> Result<(), CliError> {
        if state.version > 2
            || (state.version == 0 && (!state.principals.is_empty() || !state.threads.is_empty()))
            || (state.version == 2) != state.bootstrap.is_some()
        {
            return Err(invalid("unsupported or inconsistent local state version"));
        }
        if let Some(recovery) = &state.bootstrap {
            crate::bootstrap_state::validate(recovery)?;
        }
        for principal in state.principals.values() {
            let id_valid = match principal.kind.as_str() {
                "human" => canonical::<HumanPrincipalId>(&principal.id),
                "role" => canonical::<AgentRoleId>(&principal.id),
                _ => false,
            };
            if !id_valid || !canonical::<TenantId>(&principal.tenant) {
                return Err(invalid(
                    "local principal kind, identity or tenant is malformed",
                ));
            }
        }
        for (id, thread) in &state.threads {
            if !canonical::<ThreadId>(id) || !canonical::<TenantId>(&thread.tenant_id) {
                return Err(invalid("local thread identity or tenant is malformed"));
            }
        }
        Ok(())
    }

    pub(super) fn decode(raw: &[u8]) -> Result<StateFile, CliError> {
        // Deserialize the original bytes so duplicate struct fields and map keys
        // cannot be normalized away by a serde_json::Value intermediary.
        let state: StateFile = serde_json::from_slice(raw).map_err(|_| {
            invalid("local state JSON is malformed or has unknown/duplicate fields")
        })?;
        let shape: serde_json::Value =
            serde_json::from_slice(raw).map_err(|_| invalid("local state JSON is malformed"))?;
        if !shape.is_object()
            || ["principals", "threads"].iter().any(|field| {
                shape[*field]
                    .as_object()
                    .is_none_or(|entries| entries.values().any(|value| !value.is_object()))
            })
        {
            return Err(invalid("local state and its records must be JSON objects"));
        }
        crate::bootstrap_state::validate_shape(&shape, state.version)?;
        validate(&state)?;
        Ok(state)
    }

    pub(super) fn encode(state: &StateFile) -> Result<Vec<u8>, CliError> {
        validate(state)?;
        struct Bounded(Vec<u8>);
        impl Write for Bounded {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                if bytes.len() > MAX_STATE_BYTES.saturating_sub(self.0.len()) {
                    return Err(io::Error::other("local state exceeds the 8 MiB limit"));
                }
                self.0.extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut bytes = Bounded(Vec::new());
        serde_json::to_writer_pretty(&mut bytes, state)
            .map_err(|_| invalid("cannot encode local state within the 8 MiB limit"))?;
        Ok(bytes.0)
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(super) use unix::{load, save};

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub(super) fn load(_: &std::path::Path) -> Result<StateFile, CliError> {
    Err(invalid(
        "verified local state storage currently requires Linux or macOS",
    ))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub(super) fn save(_: &std::path::Path, _: &StateFile) -> Result<(), CliError> {
    Err(invalid(
        "verified local state storage currently requires Linux or macOS",
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix {
    use std::ffi::OsString;
    use std::fs::{File, Metadata};
    use std::io::{Read, Write};
    use std::os::unix::fs::MetadataExt;
    use std::path::{Component, Path, PathBuf};

    use rustix::fs::{self, AtFlags, FlockOperation, Mode, OFlags};
    use rustix::io::Errno;

    use super::codec::{decode, encode, MAX_STATE_BYTES};
    use super::{invalid, CliError, StateFile};

    const STATE: &str = "state.json";
    const LOCK: &str = "state.lock";
    const NEXT: &str = "state.json.next";
    const DIRECTORY: OFlags = OFlags::RDONLY
        .union(OFlags::DIRECTORY)
        .union(OFlags::CLOEXEC)
        .union(OFlags::NOFOLLOW);
    const READ_FILE: OFlags = OFlags::RDONLY
        .union(OFlags::CLOEXEC)
        .union(OFlags::NOFOLLOW)
        .union(OFlags::NONBLOCK);

    fn failure(operation: &str, error: impl std::fmt::Display) -> CliError {
        invalid(&format!("local state {operation}: {error}"))
    }

    fn synchronize(file: &File) -> Result<(), CliError> {
        fs::fsync(file).map_err(|error| failure("synchronization failed", error))?;
        #[cfg(target_os = "macos")]
        fs::fcntl_fullfsync(file).map_err(|error| failure("full synchronization failed", error))?;
        Ok(())
    }

    fn metadata(file: &File, device: u64, directory: bool) -> Result<Metadata, CliError> {
        let meta = file
            .metadata()
            .map_err(|error| failure("metadata failed", error))?;
        let correct_type = if directory {
            meta.is_dir()
        } else {
            meta.is_file() && meta.nlink() == 1
        };
        if !correct_type
            || meta.dev() != device
            || meta.uid() != rustix::process::geteuid().as_raw()
            || meta.mode() & 0o022 != 0
        {
            return Err(invalid("local state requires owned, non-writable-by-others directories and single-link regular files on the repository volume"));
        }
        Ok(meta)
    }

    fn repository(cwd: &Path) -> Result<PathBuf, CliError> {
        cwd.ancestors()
            .find(|path| {
                path.join("Cargo.toml").is_file()
                    && path.join("crates/reasonbraid-cli/Cargo.toml").is_file()
                    && path.join("migrations").is_dir()
            })
            .map(Path::to_path_buf)
            .ok_or_else(|| invalid("run rb from within the ReasonBraid repository"))
    }

    fn components(path: &Path) -> Result<Vec<OsString>, CliError> {
        let mut parts = Vec::new();
        for part in path.components() {
            match part {
                Component::Normal(name) => {
                    if parts.len() == 64 {
                        return Err(invalid("local state path exceeds 64 components"));
                    }
                    parts.push(name.to_owned());
                }
                Component::CurDir => (),
                _ => {
                    return Err(invalid(
                        "local state path must stay inside the repository without parent traversal",
                    ))
                }
            }
        }
        if parts.is_empty() {
            return Err(invalid(
                "local state needs a directory below the repository root",
            ));
        }
        Ok(parts)
    }

    fn descend(
        root: &File,
        parts: &[OsString],
        device: u64,
        create: bool,
    ) -> Result<Option<File>, CliError> {
        let mut parent = root
            .try_clone()
            .map_err(|error| failure("directory handle failed", error))?;
        for part in parts {
            let next = match fs::openat(&parent, part, DIRECTORY, Mode::empty()) {
                Ok(fd) => File::from(fd),
                Err(Errno::NOENT) if !create => return Ok(None),
                Err(Errno::NOENT) => {
                    match fs::mkdirat(&parent, part, Mode::from_raw_mode(0o700)) {
                        Ok(()) | Err(Errno::EXIST) => (),
                        Err(error) => return Err(failure("directory creation failed", error)),
                    }
                    File::from(
                        fs::openat(&parent, part, DIRECTORY, Mode::empty())
                            .map_err(|error| failure("directory open failed", error))?,
                    )
                }
                Err(error) => return Err(failure("directory open failed", error)),
            };
            metadata(&next, device, true)?;
            if create {
                synchronize(&parent)?;
            }
            parent = next;
        }
        if create {
            synchronize(&parent)?;
        }
        Ok(Some(parent))
    }

    struct Directory {
        file: File,
        device: u64,
        #[cfg(test)]
        fault: Option<Checkpoint>,
    }

    impl Directory {
        fn open(path: &Path, create: bool) -> Result<Option<Self>, CliError> {
            let cwd = std::env::current_dir()
                .map_err(|error| failure("current directory unavailable", error))?;
            Self::from_cwd(path, create, &cwd)
        }

        fn from_cwd(path: &Path, create: bool, cwd: &Path) -> Result<Option<Self>, CliError> {
            let root_path = repository(cwd)?;
            let relative = if path.is_absolute() {
                path.strip_prefix(&root_path).map_err(|_| {
                    invalid("absolute local state path is outside the current repository")
                })?
            } else {
                path
            };
            let parts = components(relative)?;
            let root = File::from(
                fs::open(&root_path, DIRECTORY, Mode::empty())
                    .map_err(|error| failure("repository open failed", error))?,
            );
            let device = root
                .metadata()
                .map_err(|error| failure("repository metadata failed", error))?
                .dev();
            metadata(&root, device, true)?;
            // Old releases interpreted relative input from CWD. Do not silently
            // abandon an existing different store when adopting root-relative input.
            if !path.is_absolute() && cwd != root_path {
                let legacy = cwd
                    .strip_prefix(&root_path)
                    .map_err(|_| invalid("current directory left the repository"))?
                    .join(relative);
                let legacy_parts = components(&legacy)?;
                if descend(&root, &legacy_parts, device, false)?.is_some() {
                    return Err(invalid("ambiguous legacy CWD-relative state directory; select its explicit repository-relative path"));
                }
            }
            Ok(descend(&root, &parts, device, create)?.map(|file| Self {
                file,
                device,
                #[cfg(test)]
                fault: None,
            }))
        }

        fn open_file(&self, name: &str) -> Result<Option<File>, CliError> {
            match fs::openat(&self.file, name, READ_FILE, Mode::empty()) {
                Ok(fd) => {
                    let file = File::from(fd);
                    metadata(&file, self.device, false)?;
                    Ok(Some(file))
                }
                Err(Errno::NOENT) => Ok(None),
                Err(error) => Err(failure("file open failed", error)),
            }
        }

        fn load(&self) -> Result<StateFile, CliError> {
            let Some(file) = self.open_file(STATE)? else {
                return Ok(StateFile::default());
            };
            if file
                .metadata()
                .map_err(|error| failure("file size failed", error))?
                .len()
                > MAX_STATE_BYTES as u64
            {
                return Err(invalid("local state exceeds the 8 MiB limit"));
            }
            let mut bytes = Vec::new();
            file.take(MAX_STATE_BYTES as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(|error| failure("read failed", error))?;
            if bytes.len() > MAX_STATE_BYTES {
                return Err(invalid("local state exceeds the 8 MiB limit"));
            }
            decode(&bytes)
        }

        fn lock(&self) -> Result<File, CliError> {
            let file = File::from(
                fs::openat(
                    &self.file,
                    LOCK,
                    OFlags::RDWR
                        | OFlags::CREATE
                        | OFlags::CLOEXEC
                        | OFlags::NOFOLLOW
                        | OFlags::NONBLOCK,
                    Mode::from_raw_mode(0o600),
                )
                .map_err(|error| failure("lock open failed", error))?,
            );
            metadata(&file, self.device, false)?;
            fs::flock(&file, FlockOperation::NonBlockingLockExclusive).map_err(|error| {
                if error == Errno::WOULDBLOCK {
                    invalid(
                        "another local state writer holds this directory; retry after it finishes",
                    )
                } else {
                    failure("lock failed", error)
                }
            })?;
            self.check_lock(&file)?;
            synchronize(&self.file)?;
            Ok(file)
        }

        fn check_lock(&self, held: &File) -> Result<(), CliError> {
            let current = self
                .open_file(LOCK)?
                .ok_or_else(|| invalid("local state lock was removed while held"))?;
            let a = held
                .metadata()
                .map_err(|error| failure("lock metadata failed", error))?;
            let b = metadata(&current, self.device, false)?;
            if (a.dev(), a.ino()) != (b.dev(), b.ino()) {
                return Err(invalid("local state lock identity changed while held"));
            }
            metadata(held, self.device, false)?;
            Ok(())
        }

        fn checkpoint(&self, point: Checkpoint) -> Result<(), CliError> {
            #[cfg(test)]
            if self.fault == Some(point) {
                return Err(invalid("owned publication fault"));
            }
            let _ = point;
            Ok(())
        }

        fn publish(&self, held: &File, bytes: &[u8]) -> Result<(), CliError> {
            self.check_lock(held)?;
            // Refuse corrupt/linked current state instead of silently erasing it.
            let current = self.load()?;
            // Validate continuity against the exact encoded replacement, before
            // any working-file cleanup or write can change the old snapshot.
            let replacement = decode(bytes)?;
            crate::bootstrap_state::validate_transition(&current, &replacement)?;
            // NEXT is a reserved private working file, never a source of truth.
            // Only remove an owned, private, bounded single-link regular residue.
            if let Some(stale) = self.open_file(NEXT)? {
                let meta = metadata(&stale, self.device, false)?;
                if meta.mode() & 0o7777 != 0o600 || meta.len() > MAX_STATE_BYTES as u64 {
                    return Err(invalid(
                        "ambiguous local state working file; preserve it for inspection",
                    ));
                }
                fs::unlinkat(&self.file, NEXT, AtFlags::empty())
                    .map_err(|error| failure("working-file cleanup failed", error))?;
                synchronize(&self.file)?;
            }
            let mut next = File::from(
                fs::openat(
                    &self.file,
                    NEXT,
                    OFlags::WRONLY
                        | OFlags::CREATE
                        | OFlags::EXCL
                        | OFlags::CLOEXEC
                        | OFlags::NOFOLLOW,
                    Mode::from_raw_mode(0o600),
                )
                .map_err(|error| failure("working-file creation failed", error))?,
            );
            metadata(&next, self.device, false)?;
            let mut replacing = false;
            let result = (|| {
                self.checkpoint(Checkpoint::Created)?;
                next.write_all(bytes)
                    .map_err(|error| failure("write failed", error))?;
                self.checkpoint(Checkpoint::Written)?;
                synchronize(&next)?;
                self.checkpoint(Checkpoint::FileSynced)?;
                self.check_lock(held)?;
                replacing = true;
                fs::renameat(&self.file, NEXT, &self.file, STATE)
                    .map_err(|error| failure("replacement failed", error))?;
                self.checkpoint(Checkpoint::Renamed)?;
                synchronize(&self.file)?;
                self.checkpoint(Checkpoint::DirectorySynced)?;
                Ok(())
            })();
            if let Err(error) = result {
                // Preserve the exact reserved residue on error. The next holder
                // validates it before cleanup; never unlink a possibly swapped
                // pathname during failure unwinding or delete the published file.
                return Err(if replacing {
                    invalid(&format!(
                        "local state replacement/durability is unconfirmed: {error}"
                    ))
                } else {
                    error
                });
            }
            Ok(())
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Checkpoint {
        Created,
        Written,
        FileSynced,
        Renamed,
        DirectorySynced,
    }

    pub(crate) fn load(path: &Path) -> Result<StateFile, CliError> {
        Directory::open(path, false)?.map_or_else(|| Ok(StateFile::default()), |dir| dir.load())
    }

    pub(super) struct Publication {
        directory: Directory,
        lock: File,
    }

    impl Publication {
        pub(super) fn open(path: &Path) -> Result<Self, CliError> {
            let directory = Directory::open(path, true)?
                .ok_or_else(|| invalid("local state directory is missing"))?;
            let lock = directory.lock()?;
            Ok(Self { directory, lock })
        }

        pub(super) fn load(&self) -> Result<StateFile, CliError> {
            self.directory.load()
        }

        pub(super) fn publish(&self, bytes: &[u8]) -> Result<(), CliError> {
            self.directory.publish(&self.lock, bytes)
        }
    }

    pub(crate) fn save(path: &Path, state: &StateFile) -> Result<(), CliError> {
        // Preserve the convenience API's validation-before-filesystem-effects
        // boundary, even though complete CLI updates open their guard earlier.
        let bytes = encode(state)?;
        Publication::open(path)?.publish(&bytes)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::state_store::Writer;
        use crate::StoredPrincipal;
        use crate::{BootstrapOutcome, BootstrapRecovery, BootstrapRequest, CompletedBootstrap};
        use std::os::unix::fs::{symlink, PermissionsExt};

        struct Fixture(PathBuf);
        impl Fixture {
            fn new() -> Self {
                let root = repository(&std::env::current_dir().unwrap()).unwrap();
                let base = root.join("target/cli-state-controls");
                std::fs::create_dir_all(&base).unwrap();
                let path = base.join(format!("unit-{}", uuid::Uuid::now_v7()));
                std::fs::create_dir(&path).unwrap();
                assert_eq!(
                    std::fs::metadata(&path).unwrap().dev(),
                    std::fs::metadata(root).unwrap().dev()
                );
                Self(path)
            }
            fn dir(&self) -> PathBuf {
                self.0.join("store")
            }
        }
        impl Drop for Fixture {
            fn drop(&mut self) {
                std::fs::remove_dir_all(&self.0).unwrap();
            }
        }
        fn state(name: &str) -> StateFile {
            let mut state = StateFile {
                version: 1,
                ..StateFile::default()
            };
            state.principals.insert(
                name.into(),
                StoredPrincipal {
                    kind: "human".into(),
                    id: "hpr_00000000-0000-7000-8000-000000000001".into(),
                    tenant: "ten_00000000-0000-7000-8000-000000000001".into(),
                },
            );
            state
        }

        fn pending_state() -> StateFile {
            StateFile {
                version: 2,
                bootstrap: Some(BootstrapRecovery {
                    pending: Some(BootstrapRequest {
                        request_id: "req_00000000-0000-7000-8000-000000000001".into(),
                        server: "http://127.0.0.1:4310".into(),
                        name: "alice".into(),
                        actions: None,
                    }),
                    completed: None,
                }),
                ..StateFile::default()
            }
        }

        fn add_completed_outcome(value: &mut StateFile) {
            let request = value.bootstrap.as_ref().unwrap().pending.clone().unwrap();
            let principal = state("alice").principals.remove("alice").unwrap();
            let outcome = BootstrapOutcome {
                bootstrap_request_id: request.request_id.clone(),
                kind: "human".into(),
                name: request.name.clone(),
                principal_id: principal.id.clone(),
                tenant_id: principal.tenant.clone(),
                boundary_id: format!("bnd_{}", principal.tenant),
                grant_id: format!("grt_{}", principal.id),
                replayed: false,
            };
            value.principals.insert(request.name.clone(), principal);
            value.bootstrap.as_mut().unwrap().completed =
                Some(CompletedBootstrap { request, outcome });
        }

        fn assert_exclusion(path: &Path) {
            let probe = File::open(path.join(LOCK)).unwrap();
            assert_eq!(
                fs::flock(&probe, FlockOperation::NonBlockingLockExclusive),
                Err(Errno::WOULDBLOCK)
            );
        }

        #[test]
        fn borrowed_publications_retain_one_guard_through_pending_completion_and_cleanup() {
            let fixture = Fixture::new();
            let path = fixture.dir();
            let mut writer = Writer::open(&path).unwrap();
            *writer.state_mut() = pending_state();
            writer.persist().unwrap();
            assert_exclusion(&path);
            let before = std::fs::read(path.join(STATE)).unwrap();
            writer
                .state_mut()
                .bootstrap
                .as_mut()
                .unwrap()
                .pending
                .as_mut()
                .unwrap()
                .name = "different".into();
            assert!(writer.persist().is_err());
            assert_eq!(std::fs::read(path.join(STATE)).unwrap(), before);
            assert_exclusion(&path);
            *writer.state_mut() = pending_state();
            add_completed_outcome(writer.state_mut());
            writer.persist().unwrap();
            assert_exclusion(&path);
            let published = StateFile::load(&path).unwrap();
            assert_eq!(published.principals["alice"].kind, "human");
            assert!(published.bootstrap.unwrap().pending.is_some());
            writer.state_mut().bootstrap.as_mut().unwrap().pending = None;
            writer.persist().unwrap();
            assert_exclusion(&path);
            drop(writer);
            let reopened = Writer::open(&path).unwrap();
            assert!(reopened
                .state()
                .bootstrap
                .as_ref()
                .unwrap()
                .pending
                .is_none());
            assert!(reopened
                .state()
                .bootstrap
                .as_ref()
                .unwrap()
                .completed
                .is_some());
        }

        #[test]
        fn intermediate_failure_preserves_the_recoverable_key_and_does_not_release_the_guard() {
            for point in [
                Checkpoint::Created,
                Checkpoint::Written,
                Checkpoint::FileSynced,
                Checkpoint::Renamed,
                Checkpoint::DirectorySynced,
            ] {
                let fixture = Fixture::new();
                let path = fixture.dir();
                state("original").save(&path).unwrap();
                let mut writer = Writer::open(&path).unwrap();
                let original_principals = writer.state().principals.clone();
                *writer.state_mut() = pending_state();
                writer.state_mut().principals = original_principals;
                writer.publication.directory.fault = Some(point);
                let error = writer.persist().unwrap_err();
                assert_exclusion(&path);
                let replacement =
                    matches!(point, Checkpoint::Renamed | Checkpoint::DirectorySynced);
                let observed = StateFile::load(&path).unwrap();
                assert!(observed.principals.contains_key("original"));
                assert_eq!(observed.bootstrap.is_some(), replacement);
                assert_eq!(error.to_string().contains("unconfirmed"), replacement);
                drop(writer);
                if replacement {
                    assert_eq!(
                        observed.bootstrap.unwrap().pending.unwrap().request_id,
                        "req_00000000-0000-7000-8000-000000000001"
                    );
                    let error = match Writer::open(&path) {
                        Ok(_) => panic!("ordinary writer must refuse unresolved recovery"),
                        Err(error) => error,
                    };
                    assert!(error.to_string().contains("pending"));
                } else {
                    assert!(Writer::open(&path).is_ok());
                }
            }
        }

        #[test]
        fn publication_failures_keep_complete_snapshots_and_recover_reserved_work() {
            for point in [
                Checkpoint::Created,
                Checkpoint::Written,
                Checkpoint::FileSynced,
                Checkpoint::Renamed,
                Checkpoint::DirectorySynced,
            ] {
                let f = Fixture::new();
                let path = f.dir();
                state("original").save(&path).unwrap();
                let mut directory = Directory::open(&path, true).unwrap().unwrap();
                directory.fault = Some(point);
                let held = directory.lock().unwrap();
                let error = directory
                    .publish(&held, &encode(&state("replacement")).unwrap())
                    .unwrap_err();
                let published = matches!(point, Checkpoint::Renamed | Checkpoint::DirectorySynced);
                let observed = StateFile::load(&path).unwrap();
                assert!(observed.principals.contains_key(if published {
                    "replacement"
                } else {
                    "original"
                }));
                assert_eq!(
                    error.to_string().contains("unconfirmed"),
                    published,
                    "{point:?}: {error}"
                );
                assert_eq!(path.join(NEXT).exists(), !published);
                drop(held);
                drop(directory);
                state("recovered").save(&path).unwrap();
                let loaded = StateFile::load(&path).unwrap();
                assert_eq!(loaded.principals.len(), 1);
                assert!(loaded.principals.contains_key("recovered"));
                assert!(!path.join(NEXT).exists());
                assert_eq!(
                    std::fs::metadata(path.join(STATE)).unwrap().mode() & 0o777,
                    0o600
                );
            }
        }

        #[test]
        fn invalid_state_is_refused_and_preserved_instead_of_overwritten() {
            let principal = r#"{"kind":"human","id":"hpr_00000000-0000-7000-8000-000000000001","tenant":"ten_00000000-0000-7000-8000-000000000001"}"#;
            let cases = [
                "not-json".into(),
                r#"[1,{},{}]"#.into(),
                r#"{"version":2,"principals":{},"threads":{}}"#.into(),
                r#"{"version":1,"principals":{},"threads":{},"extra":true}"#.into(),
                r#"{"version":1,"version":1,"principals":{},"threads":{}}"#.into(),
                format!(r#"{{"version":0,"principals":{{"a":{principal}}},"threads":{{}}}}"#),
                format!(r#"{{"version":1,"principals":{{"a":{principal},"a":{principal}}},"threads":{{}}}}"#),
                r#"{"version":1,"principals":{"a":["human","hpr_00000000-0000-7000-8000-000000000001","ten_00000000-0000-7000-8000-000000000001"]},"threads":{}}"#.into(),
                format!(r#"{{"version":1,"principals":{{"a":{}}},"threads":{{}}}}"#, principal.replace("human", "unknown")),
                format!(r#"{{"version":1,"principals":{{"a":{}}},"threads":{{}}}}"#, principal.replace("hpr_", "rol_")),
                format!(r#"{{"version":1,"principals":{{"a":{}}},"threads":{{}}}}"#, principal.replace("ten_", "bad_")),
                r#"{"version":1,"principals":{},"threads":{"bad":{"tenant_id":"ten_00000000-0000-7000-8000-000000000001","subject":"test"}}}"#.into(),
            ];
            for (index, raw) in cases.into_iter().enumerate() {
                let f = Fixture::new();
                let path = f.dir();
                std::fs::create_dir(&path).unwrap();
                std::fs::write(path.join(STATE), &raw).unwrap();
                assert!(StateFile::load(&path).is_err(), "case {index}");
                assert!(state("replacement").save(&path).is_err(), "case {index}");
                assert_eq!(std::fs::read_to_string(path.join(STATE)).unwrap(), raw);
            }
            // Empty legacy version zero remains valid. Canonical older UUID
            // versions remain valid identities; UUIDv7 is specific to new keys.
            let f = Fixture::new();
            StateFile::default().save(&f.dir()).unwrap();
            assert_eq!(StateFile::load(&f.dir()).unwrap().version, 0);
            let mut legacy = state("legacy");
            legacy.principals.get_mut("legacy").unwrap().tenant =
                "ten_00000000-0000-4000-8000-000000000001".into();
            legacy.save(&f.dir()).unwrap();
            assert_eq!(
                StateFile::load(&f.dir()).unwrap().principals["legacy"].tenant,
                "ten_00000000-0000-4000-8000-000000000001"
            );
        }

        #[test]
        fn oversized_reads_and_writes_are_bounded_without_erasing_state() {
            let f = Fixture::new();
            let path = f.dir();
            std::fs::create_dir(&path).unwrap();
            let file = File::create(path.join(STATE)).unwrap();
            file.set_len(MAX_STATE_BYTES as u64 + 1).unwrap();
            let error = StateFile::load(&path).unwrap_err();
            assert!(error.to_string().contains("8 MiB"));
            assert!(state("replacement").save(&path).is_err());
            assert_eq!(file.metadata().unwrap().len(), MAX_STATE_BYTES as u64 + 1);
            let absent = f.0.join("oversized-output");
            let too_large = state(&"x".repeat(MAX_STATE_BYTES + 1));
            assert!(too_large
                .save(&absent)
                .unwrap_err()
                .to_string()
                .contains("8 MiB"));
            assert!(!absent.exists());
        }

        #[test]
        fn unsafe_working_files_and_lock_replacement_are_preserved_and_refused() {
            for kind in ["symlink", "hardlink", "unreserved-mode", "directory"] {
                let f = Fixture::new();
                let path = f.dir();
                state("original").save(&path).unwrap();
                let original = std::fs::read(path.join(STATE)).unwrap();
                let target = f.0.join("owned-target");
                std::fs::write(&target, b"preserve owned target").unwrap();
                match kind {
                    "symlink" => symlink(&target, path.join(NEXT)).unwrap(),
                    "hardlink" => std::fs::hard_link(&target, path.join(NEXT)).unwrap(),
                    "unreserved-mode" => {
                        std::fs::write(path.join(NEXT), b"ambiguous").unwrap();
                        std::fs::set_permissions(
                            path.join(NEXT),
                            std::fs::Permissions::from_mode(0o644),
                        )
                        .unwrap();
                    }
                    "directory" => std::fs::create_dir(path.join(NEXT)).unwrap(),
                    _ => unreachable!(),
                }
                assert!(state("replacement").save(&path).is_err(), "{kind}");
                assert_eq!(std::fs::read(path.join(STATE)).unwrap(), original);
                assert_eq!(std::fs::read(target).unwrap(), b"preserve owned target");
                assert!(std::fs::symlink_metadata(path.join(NEXT)).is_ok());
            }
            let f = Fixture::new();
            let path = f.dir();
            state("original").save(&path).unwrap();
            let directory = Directory::open(&path, true).unwrap().unwrap();
            let held = directory.lock().unwrap();
            std::fs::remove_file(path.join(LOCK)).unwrap();
            let replacement = File::create(path.join(LOCK)).unwrap();
            let error = directory
                .publish(&held, &encode(&state("replacement")).unwrap())
                .unwrap_err();
            assert!(error.to_string().contains("lock identity changed"));
            assert!(StateFile::load(&path)
                .unwrap()
                .principals
                .contains_key("original"));
            drop(replacement);
        }

        #[test]
        fn root_relative_paths_refuse_traversal_links_and_ambiguous_legacy_locations() {
            let f = Fixture::new();
            let root = repository(&f.0).unwrap();
            let path = f.dir();
            state("original").save(&path).unwrap();
            let relative = path.strip_prefix(&root).unwrap();
            let cwd = f.0.join("cwd");
            std::fs::create_dir(&cwd).unwrap();
            let selected = Directory::from_cwd(relative, false, &cwd).unwrap().unwrap();
            assert!(selected.load().unwrap().principals.contains_key("original"));
            std::fs::create_dir_all(cwd.join(relative)).unwrap();
            assert!(Directory::from_cwd(relative, false, &cwd).is_err());
            let alias = f.0.join("alias");
            symlink(&path, &alias).unwrap();
            assert!(StateFile::load(&alias).is_err());
            assert!(state("replacement").save(&alias).is_err());
            assert!(StateFile::load(&path.join("..").join("store")).is_err());
            assert!(StateFile::load(Path::new("/outside-reasonbraid-owned-state")).is_err());
            let excessive: PathBuf = f.0.join((0..65).map(|_| "component").collect::<PathBuf>());
            assert!(Directory::from_cwd(&excessive, true, &cwd).is_err());
            assert!(!f.0.join("component").exists());
        }
    }
}
