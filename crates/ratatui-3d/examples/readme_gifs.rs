//! Renders the README demo GIFs for the `cube` and `textured_sphere` examples.
//!
//! Each scene is drawn headlessly through the real `Viewport3D` widget into a
//! ratatui `Buffer`. Every terminal cell is then painted as an 8×16 pixel block
//! (`▀` → fg on the top half, bg on the bottom half), so the output looks like
//! a capture of the interactive example running in an 80×23 terminal.
//!
//! Run from the workspace root:
//!
//! ```sh
//! cargo run --release -p ratatui-3d --example readme_gifs
//! ```
//!
//! Writes `docs/cube.gif` and `docs/textured_sphere.gif`.

use std::f32::consts::PI;
use std::fs::File;
use std::sync::Arc;

use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, Rgba, RgbaImage};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::StatefulWidget;

use ratatui_3d::prelude::*;

const FPS: u32 = 20;
const COLS: u16 = 80;
const ROWS: u16 = 23;
const CELL_W: u32 = 8;
const CELL_H: u32 = 16;

fn main() {
    render_cube();
    render_textured_sphere();
}

/// Same scene and animation as `examples/cube.rs`.
fn render_cube() {
    let mut scene = Scene::new();
    scene.add_object(
        SceneObject::new(primitives::cube())
            .with_material(Material::default().with_color(Rgb(100, 150, 255))),
    );
    scene.add_object(
        SceneObject::new(primitives::sphere(16, 24))
            .with_material(Material::default().with_color(Rgb(255, 100, 100)))
            .with_transform(Transform::from_position(Vec3::new(2.0, 0.0, 0.0))),
    );
    scene.add_object(
        SceneObject::new(primitives::plane())
            .with_material(
                Material::default()
                    .with_color(Rgb(80, 180, 80))
                    .with_specular(0.1),
            )
            .with_transform({
                let mut t = Transform::from_position(Vec3::new(0.0, -0.6, 0.0));
                t.scale = Vec3::splat(6.0);
                t
            }),
    );
    scene.add_light(Light::ambient(Rgb(255, 255, 255), 0.15));
    scene.add_light(Light::directional(
        Vec3::new(-1.0, -1.0, -1.0),
        Rgb(255, 255, 255),
    ));
    scene.add_light(Light::point(Vec3::new(2.0, 3.0, 2.0), Rgb(255, 220, 180)));

    let mut state = Viewport3DState::default();
    state.camera.position = Vec3::new(3.0, 2.5, 4.0);
    state.camera.target = Vec3::new(0.5, 0.0, 0.0);

    // The cube spins at 0.8 rad/s around Y and 0.3 rad/s around X. After 5π
    // seconds those are 4π and 1.5π — the latter is a quarter-turn multiple,
    // which maps a cube onto itself, so the GIF loops seamlessly.
    let duration = 5.0 * PI;

    render_gif(
        &mut scene,
        &mut state,
        duration,
        "docs/cube.gif",
        |scene, t| {
            scene.objects[0].transform.rotation =
                Quat::from_rotation_y(t * 0.8) * Quat::from_rotation_x(t * 0.3);
        },
    );
}

/// Same scene and animation as `examples/textured_sphere.rs`.
fn render_textured_sphere() {
    let img = image::open("assets/vaporwave.png")
        .expect("Failed to load assets/vaporwave.png (run from the workspace root)")
        .into_rgba8();
    let (w, h) = img.dimensions();
    let texture = Arc::new(Texture::from_rgba(w, h, img.into_raw()));

    let mut scene = Scene::new();
    scene.add_object(
        SceneObject::new(primitives::sphere(32, 48))
            .with_material(
                Material::default()
                    .with_color(Rgb(255, 255, 255))
                    .with_texture(texture)
                    .with_ambient(0.4)
                    .with_diffuse(0.8)
                    .with_specular(0.2)
                    .with_shininess(16.0),
            )
            .with_transform({
                let mut t = Transform::from_position(Vec3::ZERO);
                t.scale = Vec3::splat(2.0);
                t
            }),
    );
    scene.add_light(Light::ambient(Rgb(255, 255, 255), 0.5));
    scene.add_light(Light::directional(
        Vec3::new(-0.5, -1.0, -0.5),
        Rgb(255, 255, 255),
    ));
    scene.add_light(Light::point(Vec3::new(3.0, 2.0, 3.0), Rgb(255, 240, 220)));
    scene.add_light(Light::point(Vec3::new(-3.0, 1.0, 2.0), Rgb(220, 220, 255)));

    let mut state = Viewport3DState::default();
    state.camera.position = Vec3::new(0.0, 0.0, 3.5);
    state.camera.target = Vec3::ZERO;

    // One full revolution at 0.5 rad/s.
    let duration = 2.0 * PI / 0.5;

    render_gif(
        &mut scene,
        &mut state,
        duration,
        "docs/textured_sphere.gif",
        |scene, t| {
            scene.objects[0].transform.rotation = Quat::from_rotation_y(t * 0.5);
        },
    );
}

fn render_gif(
    scene: &mut Scene,
    state: &mut Viewport3DState,
    duration_secs: f32,
    output_path: &str,
    animate: impl Fn(&mut Scene, f32),
) {
    let frame_count = (duration_secs * FPS as f32).round() as usize;
    let area = Rect::new(0, 0, COLS, ROWS);
    let delay = Delay::from_numer_denom_ms(1000 / FPS, 1);

    let file =
        File::create(output_path).unwrap_or_else(|e| panic!("Failed to create {output_path}: {e}"));
    let mut encoder = GifEncoder::new_with_speed(file, 10);
    encoder.set_repeat(Repeat::Infinite).unwrap();

    for i in 0..frame_count {
        let t = i as f32 / FPS as f32;
        animate(scene, t);

        let mut buf = Buffer::empty(area);
        StatefulWidget::render(Viewport3D::new(scene), area, &mut buf, state);

        let frame = Frame::from_parts(buffer_to_image(&buf), 0, 0, delay);
        encoder
            .encode_frame(frame)
            .expect("Failed to encode GIF frame");
        eprint!("\r[{output_path}] frame {}/{frame_count}", i + 1);
    }
    eprintln!();
}

/// Paint each `▀` cell as a block: fg fills the top half, bg the bottom half.
fn buffer_to_image(buf: &Buffer) -> RgbaImage {
    let area = buf.area;
    let mut img = RgbaImage::new(area.width as u32 * CELL_W, area.height as u32 * CELL_H);

    for row in 0..area.height {
        for col in 0..area.width {
            let cell = &buf[(col, row)];
            let (top, bottom) = match cell.symbol() {
                "▀" => (cell.fg, cell.bg),
                _ => (cell.bg, cell.bg),
            };
            let top = to_rgba(top);
            let bottom = to_rgba(bottom);

            let x0 = col as u32 * CELL_W;
            let y0 = row as u32 * CELL_H;
            for dy in 0..CELL_H {
                let px = if dy < CELL_H / 2 { top } else { bottom };
                for dx in 0..CELL_W {
                    img.put_pixel(x0 + dx, y0 + dy, px);
                }
            }
        }
    }
    img
}

fn to_rgba(color: Color) -> Rgba<u8> {
    match color {
        Color::Rgb(r, g, b) => Rgba([r, g, b, 255]),
        _ => Rgba([0, 0, 0, 255]),
    }
}
