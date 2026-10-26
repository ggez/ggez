//! Points `src/lib.rs` at the example named by `GGEZ_EXAMPLE` (default `05_astroblasto`).

use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=GGEZ_EXAMPLE");
    let example = env::var("GGEZ_EXAMPLE").unwrap_or_else(|_| String::from("05_astroblasto"));
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source = manifest_dir.join("..").join(format!("{example}.rs"));
    let source = source
        .canonicalize()
        .unwrap_or_else(|_| panic!("no example named {example:?} at {source:?}"));
    println!("cargo:rerun-if-changed={}", source.display());

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("example.rs");
    fs::write(&out, format!("#[path = {:?}]\nmod example;\n", source)).unwrap();
}
