fn main() {
    // The node embeds its own journal schema, independently of server migrations.
    let manifest = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies the crate directory"),
    );
    let root = manifest.ancestors().nth(2).expect("workspace crate layout");
    println!(
        "cargo:rerun-if-changed={}",
        root.join("crates/reasonbraid-node/migrations").display()
    );
}
