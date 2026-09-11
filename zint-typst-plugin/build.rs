fn main() {
    // wasm-ld only warns when a call does not match the function's signature,
    // and turns the call into a trap. The stubs in zint-wasm-sys's patch.c
    // stand in for zint functions no compiler checks them against, so this is
    // the one place a mismatch shows. Only wasm-ld takes this flag; host
    // builds of the crate link with something else.
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        println!("cargo:rustc-link-arg-cdylib=--fatal-warnings");
    }
}
