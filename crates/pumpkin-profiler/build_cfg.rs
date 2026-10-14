// Included by the build scripts of crates with profiler hooks. Emits `cfg(profiler)` when the
// `memory-profiler` feature is enabled on a target that pumpkin-profiler supports. Keep the
// platform list in sync with the target dependencies in pumpkin-profiler's Cargo.toml.
fn emit_profiler_cfg() {
    println!("cargo::rustc-check-cfg=cfg(profiler)");
    let var = |name: &str| std::env::var(name).unwrap_or_default();
    let os = var("CARGO_CFG_TARGET_OS");
    let arch = var("CARGO_CFG_TARGET_ARCH");
    let supported_os = os == "windows"
        || os == "macos"
        || (os == "linux" && var("CARGO_CFG_TARGET_ENV") == "gnu");
    let supported_arch = arch == "x86_64" || arch == "aarch64";
    if std::env::var_os("CARGO_FEATURE_MEMORY_PROFILER").is_some()
        && supported_os
        && supported_arch
    {
        println!("cargo::rustc-cfg=profiler");
    }
}
