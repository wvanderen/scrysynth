fn main() {
    println!("cargo:rerun-if-changed=src/domain/session.rs");
    tauri_build::build()
}
