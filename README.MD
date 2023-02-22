# Simuverse

![Minimum Rust Version](https://img.shields.io/badge/min%20rust-1.64-green.svg)
[![Build Status](https://github.com/gfx-rs/jinleili/simuverse/CI/badge.svg?branch=main)](https://github.com/jinleili/simuverse/actions)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/jinleili/simuverse#license)

Simulate particle fields, fluid dynamics, and natural textures based on [wgpu](https://github.com/gfx-rs/wgpu) + [egui](https://github.com/emilk/egui).

It is not only a complete example but also an extension of the [学习 wgpu](https://github.com/jinleili/learn-wgpu-zh).

## Web Support
Due to the use of [compute pipelines](https://jinleili.github.io/learn-wgpu-zh/intermediate/compute-pipeline/), it's not possible to build a WebAssembly binary using the `--features=webgl` feature. However, once the web platform bug in wgpu 0.15 is fixed, web support will be added. The bug is discussed in this [issue](https://github.com/gfx-rs/wgpu/issues/3430).

## Screenshot
| | |
|---|---|
|![Julia Set](assets/julia_set.png) | ![Spirl](assets/spirl.png)|
|![Wood](assets/wood.png) | ![Marble](assets/marble.png)|

## License
Licensed under the [MIT License](https://github.com/grovesNL/glyphon/blob/main/LICENSE-MIT).