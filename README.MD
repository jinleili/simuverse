# Simuverse

![Minimum Rust Version](https://img.shields.io/badge/min%20rust-1.65-green.svg)
[![Build Status](https://github.com/jinleili/simuverse/workflows/CI/badge.svg?branch=main)](https://github.com/jinleili/simuverse/actions)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/jinleili/simuverse#license)

Simulate particle vector fields, fluid dynamics, procedural textures, and position-based dynamics based on [wgpu](https://github.com/gfx-rs/wgpu) + [egui](https://github.com/emilk/egui).

It is not only a complete example but also an extension of the [学习 wgpu](https://github.com/jinleili/learn-wgpu-zh).

## Screenshot
| | |
|---|---|
|![Julia Set](screenshots/julia_set.png) | ![Spirl](screenshots/spirl.png)|
|![Wood](screenshots/wood.png) | ![Marble](screenshots/marble.png)|
|![PBD Cloth](screenshots/cloth.png) | ![LBM Fluid](screenshots/lbm.png)|
|![Load .obj](screenshots/obj.png) | ![B-Spline animation](screenshots/bsp.png)|


## Web Support
Due to the use of [compute pipelines](https://jinleili.github.io/learn-wgpu-zh/intermediate/compute-pipeline/), it's not possible to build a WebAssembly binary using the `--features=webgl` feature. However, once the web platform bug in wgpu 0.15 is fixed, web support will be added. The bug was discussed in this [issue](https://github.com/gfx-rs/wgpu/issues/3430).

## License
Licensed under the [MIT License](https://github.com/jinleili/simuverse/blob/main/LICENSE.MIT).