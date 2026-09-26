//! Debug-build kill points for the interruption controls
//! (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.3.3`).
//!
//! Between the CLI's own publications nothing external happens, so a test has no
//! moment to kill the process at. `REASONBRAID_CLI_KILL_AT=<point>[#<n>]` makes
//! the process SIGKILL itself the n-th time (the first by default) it reaches
//! the named point: a real death, running no destructor and no release code,
//! exactly where the test asked. The points are `publish:<checkpoint>` inside
//! every state replacement and `bootstrap:pending-published` /
//! `bootstrap:outcome-published` between the bootstrap's publications.
//!
//! ⛔ Compiled only under `debug_assertions` on Unix. A release build carries no
//! hook and reads no variable.

pub(crate) fn reached(point: &str) {
    #[cfg(all(debug_assertions, unix))]
    {
        use std::sync::atomic::{AtomicUsize, Ordering};

        // Counts arrivals at the named point only; one point is armed per process.
        static SEEN: AtomicUsize = AtomicUsize::new(0);
        let Some(target) = std::env::var_os("REASONBRAID_CLI_KILL_AT") else {
            return;
        };
        let target = target.to_string_lossy();
        let (name, nth) = match target.split_once('#') {
            Some((name, nth)) => (name, nth.parse::<usize>().unwrap_or(0)),
            None => (&*target, 1),
        };
        if name == point && SEEN.fetch_add(1, Ordering::SeqCst) + 1 == nth {
            let _ = rustix::process::kill_process(
                rustix::process::getpid(),
                rustix::process::Signal::KILL,
            );
            // Not reached: SIGKILL cannot be caught or ignored.
            std::process::abort();
        }
    }
    #[cfg(not(all(debug_assertions, unix)))]
    let _ = point;
}
