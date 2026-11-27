//! Sets the `lumis` and `syntect` cfgs the code gates each highlighter on.
//!
//! They follow the Cargo features of the same names, except on
//! `target_os = "wasi"`, where both are always set and Cargo.toml always pulls
//! in their dependencies. A WASI build comes from tools like ampoule, which
//! pass no feature flags, and the default features cannot turn the
//! highlighters on because the plain precompiled NIF is built with them.

use std::env;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-check-cfg=cfg(lumis, syntect)");

    let wasi = env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "wasi");

    for (cfg, feature) in [
        ("lumis", "CARGO_FEATURE_LUMIS"),
        ("syntect", "CARGO_FEATURE_SYNTECT"),
    ] {
        if wasi || env::var_os(feature).is_some() {
            println!("cargo::rustc-cfg={cfg}");
        }
    }
}
