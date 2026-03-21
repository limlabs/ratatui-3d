# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

3D software renderer for terminal UIs built on ratatui. Cargo workspace with three crates:

- **render3d** (`crates/render3d/`) — Core engine: rasterizer, ray tracer (CPU + GPU via wgpu), Phong shading, mesh primitives, OBJ/glTF loaders
- **ratatui-3d** (`crates/ratatui-3d/`) — Terminal widget integration: `Viewport3D` (stateful) and `Viewport3DStatic` (one-shot), three render modes (HalfBlock, Braille, ASCII)
- **render3d-node** (`crates/render3d-node/`) — Node.js bindings via NAPI-RS

## Build & Test Commands

```bash
# Build entire workspace
cargo build

# Run tests (all in render3d crate)
cargo test -p render3d

# Run a single test
cargo test -p render3d -- test_name

# Run examples (from workspace root)
cargo run -p ratatui-3d --example cube
cargo run -p ratatui-3d --example trex --features gltf,gpu
cargo run -p ratatui-3d --example cornell --features gpu
cargo run -p ratatui-3d --example dna --features gltf,gpu

# GIF export example (requires gltf,gpu features + assets/trex.glb)
cargo run -p ratatui-3d --example trex_gif --features gltf,gpu

# Node.js bindings (from crates/render3d-node/)
npm run build          # napi build --release
npm run build:debug    # napi build
```

## Architecture

### Rendering Pipeline (`render3d/src/pipeline/`)

Three pipeline modes selected via `Pipeline` enum:
1. **Rasterize** — Scanline rasterization with barycentric interpolation, depth buffering, backface culling
2. **Raytrace** — CPU recursive ray tracing with reflection/refraction
3. **RaytraceGpu** — WGPU compute shader (`raytrace.wgsl`), feature-gated behind `gpu`

Pipeline stages: vertex transformation (`vertex.rs`) → rasterization/ray casting → fragment shading (`fragment.rs`) with Phong lighting → framebuffer (`framebuffer.rs`) with depth test.

### Widget Layer (`ratatui-3d/src/`)

`Viewport3D` implements ratatui's `StatefulWidget`. Render modes map framebuffer pixels to terminal cells:
- **HalfBlock**: `▀` char with fg/bg colors (1×2 pixel resolution per cell)
- **Braille**: 2×4 supersampled, luminance-averaged (high-res monochrome)
- **ASCII**: 2×2 blocks mapped to luminance ramp ` .:-=+*#%@`

### Feature Flags

`render3d` default features: `obj`, `gltf`, `gpu`. Each can be toggled independently. `ratatui-3d` mirrors these flags. `render3d-node` excludes `gpu` by default.

### Key Types

- `Scene` — Container for objects, lights, background color, sky gradient
- `SceneObject` — Mesh + Material + Transform + visibility
- `Material` — Phong parameters: color, ambient, diffuse, specular, shininess
- `Light` — Ambient, Directional (direction), Point (position)
- `Camera` — Position/target/up with `orbit()` and `zoom()` methods
- All math via `glam` crate (Vec3, Mat4, Quat)

### Node.js Bindings

`render3d-node` exposes a `Renderer` class via NAPI. Methods for scene construction, camera setup, and `render()`/`render_raytrace()` returning raw RGB pixel buffers.

### GIF Export

The `Framebuffer` carries a per-pixel `alpha` channel (0 = background/transparent, 255 = geometry hit). All three pipeline backends (rasterize, raytrace CPU, raytrace GPU) write alpha on fragment output. This enables rendering to image files with transparent backgrounds.

The `trex_gif` example demonstrates the workflow: render frames to a `Framebuffer`, use the `alpha` buffer to produce RGBA pixels, auto-crop to the bounding box of opaque pixels, and encode with the `image` crate's GIF encoder. It outputs two variants — a chunky pixel-scaled "terminal style" GIF and a smooth high-res GIF.

The `image` dependency (GIF feature only) lives in `ratatui-3d` for the example; the core `render3d` crate has no image I/O dependency.

## Tests

All tests live in `crates/render3d/tests/pipeline_tests.rs`. They cover the full pipeline (vertex transforms, fragment shading, rasterization, backface culling, depth testing, integration renders).
