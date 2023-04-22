use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
use std::path::PathBuf;
fn main() {
    // run-wasm 的执行目录：target/release/run-wasm

    // Prepare what to copy and how
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets/preprocessed-wgsl");
    let out_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/wasm-examples/simuverse/");
    let _ = copy_items(&[&path], out_dir, &copy_options);

    cargo_run_wasm::run_wasm_with_css("body { margin: 0px; }");
}
