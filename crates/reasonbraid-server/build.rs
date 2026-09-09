fn main() {
    // SQLx can track existing SQL files, but stable proc macros cannot discover
    // added migrations without Cargo watching the containing directory.
    let manifest = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies the crate directory"),
    );
    let root = manifest.ancestors().nth(2).expect("workspace crate layout");
    println!(
        "cargo:rerun-if-changed={}",
        root.join("migrations").display()
    );
}
