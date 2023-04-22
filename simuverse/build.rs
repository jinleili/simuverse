use std::error::Error;

mod wgsl_preprocess;
use wgsl_preprocess::preprocess_wgsl;

// build.rs 配置：https://blog.csdn.net/weixin_33910434/article/details/87943334
fn main() -> Result<(), Box<dyn Error>> {
    // 这一行告诉 cargo 如果 /wgsl/ 目录中的内容发生了变化，就重新运行脚本
    println!("cargo:rerun-if-changed=../assets/wgsl/");
    preprocess_wgsl()
}
