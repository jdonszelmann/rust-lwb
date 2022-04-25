use rustc::{nightly, not};

#[nightly]
fn on_nightly() {
    println!(r#"cargo:rustc-cfg=feature="nightly""#);
}

#[not(nightly)]
fn on_nightly() {}


fn main() {
    println!("cargo:rerun-if-changed=src/build.rs");

    on_nightly();
}