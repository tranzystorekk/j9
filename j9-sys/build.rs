extern crate autotools;
extern crate bindgen;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR").map(PathBuf::from)?;
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("jq");
    let build_dir = out_dir.join("jq_build");

    // Copy the contents of the src_dir to build_dir within OUT_DIR
    // to avoid modifying the source directory during the build process.
    // This ensures compliance with Cargo's policy that build scripts
    // should not modify anything outside of OUT_DIR.
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir)?;
    }
    fs::create_dir(&build_dir)?;
    for entry in walkdir::WalkDir::new(&src_dir) {
        let entry = entry?;
        let target_path = build_dir.join(entry.path().strip_prefix(&src_dir)?);
        if entry.file_type().is_dir() {
            fs::create_dir_all(target_path)?;
        } else {
            fs::copy(entry.path(), target_path)?;
        }
    }

    // See https://github.com/jqlang/jq/tree/jq-1.7.1?#instructions
    autotools::Config::new(&build_dir)
        .reconf("-i")
        .out_dir(&out_dir)
        .with("oniguruma", Some("builtin"))
        .make_args(vec![
            // See https://github.com/jqlang/jq/issues/1936
            "CPPFLAGS=-D_REENTRANT".into(),
        ])
        .build();

    let lib_dir = out_dir.join("lib");
    let include_dir = out_dir.join("include");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:include={}", include_dir.display());

    for lib in &["onig", "jq"] {
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    let bindings = bindgen::Builder::default()
        .header("jq/src/jq.h")
        .generate()?;

    bindings.write_to_file(out_dir.join("bindings.rs"))?;

    Ok(())
}
