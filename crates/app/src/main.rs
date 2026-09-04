//! Starter binary for a project scaffolded from the `bedrock` discipline-spine template.
//!
//! Replace this crate with your project. The first real work should be owned by a
//! task-tree leaf under `docs/tasks/` (see `CLAUDE.md` and `docs/TASK_TREE.md`).

fn main() {
    println!("bedrock: replace this crate with your project — start from ROADMAP.md.");
}

#[cfg(test)]
mod tests {
    #[test]
    fn scaffold_builds_and_tests_run() {
        // A trivial passing test so `make check` / CI is green from commit one.
        assert_eq!(2 + 2, 4);
    }
}
