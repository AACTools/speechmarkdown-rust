fn main() {
    // Tell cargo to rerun this build script if important files change
    println!("cargo:rerun-if-changed=src/");
}