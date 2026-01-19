# WGSL template engine

A template engine for wgsl files using [ppx](https://github.com/Jomy10/ppx).

## Usage

To compile a wgsl template at compile time, this crate provides a macro
similar to [`include_wgsl!` in wgpu](https://docs.rs/wgpu/latest/wgpu/macro.include_wgsl.html).

```rust
let shader = device.create_shader_module(include_wgsl_template!("wgsl/file.wgsl", "wgsl/", ["1", "35"]));
```

To compile a wgsl template at runtime, use the [ppx library](https://crates.io/crate/ppx) directly.
