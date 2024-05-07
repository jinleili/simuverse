RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --no-default-features --release --target wasm32-unknown-unknown \
--bin simuverse

# Generate bindings
for i in target/wasm32-unknown-unknown/release/*.wasm;
do
    wasm-bindgen --no-typescript --out-dir wasm --web "$i";
done

# 优化 wasm 包大小
# 2024/5/5, 4.6MB -> 3.2MB
wasm-opt -Oz --output wasm/optimized.wasm wasm/simuverse_bg.wasm