#!/bin/bash

set -e 

export RUSTFLAGS='--cfg getrandom_backend="wasm_js"'

cargo build --profile wasm-release --target wasm32-unknown-unknown --bin simuverse

# Generate bindings
for i in target/wasm32-unknown-unknown/wasm-release/*.wasm;
do
    wasm-bindgen --no-typescript --out-dir wasm --web "$i";
    # 优化 wasm 包大小 (未压缩)
    # 2024/5/5, 4.6MB -> 3.2MB
    # 2024/6/22, 3.2MB -> 2.2MB
    # 2025/3/1, 2.2MB -> 2.1MB
    # 2025/6/22, 2.1MB -> 1.5MB
    filename=$(basename "$i");
    name_no_extension="${filename%.wasm}";
    wasm-opt -Oz --output wasm/"$name_no_extension"_optimized_bg.wasm wasm/"$name_no_extension"_bg.wasm
    cd wasm
    # 删除旧的压缩文件
    rm -f "$name_no_extension"_bg.wasm.br
    # 压缩并保留原文件
    brotli -Zk "$name_no_extension"_optimized_bg.wasm -o "$name_no_extension"_bg.wasm.br
    cd ../
done