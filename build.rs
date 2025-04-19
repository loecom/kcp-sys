use std::env;
use std::path::{Path, PathBuf};
use std::fs;

fn main() {
    println!("cargo:rustc-link-lib=kcp");
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let fulldir = Path::new(&dir).join("kcp");

    let mut config = cc::Build::new();
    config.include(fulldir.clone());
    config.file(fulldir.join("ikcp.c"));
    config.opt_level(3);
    config.warnings(false);
    config.compile("libkcp.a");
    println!("cargo:rustc-link-search=native={}", fulldir.display());

    println!("cargo:rerun-if-changed=kcp/ikcp.h");
    println!("cargo:rerun-if-changed=kcp/ikcp.c");
    println!("cargo:rerun-if-changed=wrapper.h");

    let extra_header_path = std::env::var("KCP_SYS_EXTRA_HEADER_PATH").unwrap_or_default();
    let extra_header_paths = extra_header_path.split(":").filter(|s| !s.is_empty()).collect::<Vec<_>>();

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_args(extra_header_paths.iter().map(|p| format!("-I{}", p)))
        .allowlist_function("ikcp_.*")
        .use_core()
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
        // 修复生成的 bindings.rs
    let bindings_path = out_path.join("bindings.rs");
    let bindings = fs::read_to_string(&bindings_path).unwrap();
    let fixed_bindings = bindings.replace("unsafe extern \"C\" {", "extern \"C\" {");
    fs::write(&bindings_path, fixed_bindings).unwrap();
}
