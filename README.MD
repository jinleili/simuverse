# Wgpu Simuverse
基于 wgpu 的交互式模拟程序，提供粒子场、流体场、隐形墨水、自然界纹理的模拟效果。

## Web 端支持
由于使用了[计算管线](https://jinleili.github.io/learn-wgpu-zh/intermediate/compute-pipeline/), 无法通过 `--features=webgl` 来构建 wasm，等 [wgpu 0.15 引入的 web 端 bug](https://github.com/gfx-rs/wgpu/issues/3430) 修复之后将会添加 Web 端支持。

## 运行效果

## 运行
```rust
cargo run
```