cargo build --profile wasm-release --target wasm32-unknown-unknown --bin simuverse

# Generate bindings
for i in target/wasm32-unknown-unknown/wasm-release/*.wasm;
do
    wasm-bindgen --no-typescript --out-dir wasm --web "$i";
    # 优化 wasm 包大小
    # 2024/5/5, 4.6MB -> 3.2MB
    # 2024/6/22, 3.2MB -> 2.2MB
    # 2025/3/1, 2.2MB -> 2.1MB
    filename=$(basename "$i");
    name_no_extension="${filename%.wasm}";
    wasm-opt -Oz --output wasm/"$name_no_extension"_optimized_bg.wasm wasm/"$name_no_extension"_bg.wasm
done