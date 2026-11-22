fn main() {
    // Boa parses the popup page prelude on the event-loop/CLI main thread.
    // Windows reserves only 1 MiB there by default, which overflows during
    // that parse; reserve 16 MiB for this executable. Other platforms already
    // give the main thread a larger stack.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-arg=/STACK:16777216");
    }
}
