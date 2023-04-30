use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
use std::path::PathBuf;
fn main() {
    // run-wasm 的执行目录：target/release/run-wasm

    // Prepare what to copy and how
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;

    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wgsl_path = base_path.join("../assets/preprocessed-wgsl");
    let img_path = base_path.join("../assets/cloth_500x500.png");
    let out_dir = base_path.join("../target/wasm-examples/simuverse/assets");
    // 创建目录
    let _ = std::fs::create_dir_all(&out_dir);
    let _ = copy_items(&[&wgsl_path, &img_path], out_dir, &copy_options);

    cargo_run_wasm::run_wasm_with_css("body { margin: 0px; }");
}
