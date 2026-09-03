//! Compiles the Slint UI into `$OUT_DIR` for `slint::include_modules!()`.

fn main() {
    // "fluent-dark" is the E-OS Crimson baseline: dark widgets on the
    // #0c0202 background the .slint file paints.
    let config = slint_build::CompilerConfiguration::new().with_style("fluent-dark".into());
    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
