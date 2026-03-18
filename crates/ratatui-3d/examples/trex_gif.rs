use ratatui_3d::color::Rgb;
use ratatui_3d::pipeline::Framebuffer;
#[cfg(feature = "gpu")]
use ratatui_3d::pipeline::raytrace_gpu::GpuRenderer;
use ratatui_3d::prelude::*;

use image::codecs::gif::{GifEncoder, Repeat};
use image::{Frame, RgbaImage};
use std::fs::File;

const FPS: u32 = 20;
const ROTATION_SPEED: f32 = 0.6;

fn main() {
    // One full rotation for a clean loop
    let duration_secs = 2.0 * std::f32::consts::PI / ROTATION_SPEED;
    let frame_count = (duration_secs * FPS as f32).round() as usize;

    // Load the T-Rex model
    let meshes = ratatui_3d::loader::gltf::load_gltf("assets/trex.glb")
        .expect("Failed to load trex.glb");

    // Build scene — no ground plane, no sky (transparent background)
    let mut scene = Scene::new();
    let model_scale = 20.0;

    let trex_start = scene.objects.len();
    for mesh in meshes {
        scene.add_object(
            SceneObject::new(mesh)
                .with_material(Material::default().with_color(Rgb(100, 160, 80)))
                .with_transform({
                    let mut t = Transform::default();
                    t.scale = Vec3::splat(model_scale);
                    t.position = Vec3::new(0.0, -3.0, 0.0);
                    t
                }),
        );
    }
    let trex_end = scene.objects.len();

    scene.add_light(Light::ambient(Rgb(255, 255, 255), 0.25));
    scene.add_light(Light::directional(
        Vec3::new(-1.0, -1.0, -1.0),
        Rgb(255, 255, 255),
    ));

    let camera = {
        let mut c = Camera::default();
        c.position = Vec3::new(5.0, 4.0, 8.0);
        c.target = Vec3::new(0.0, 1.0, 0.0);
        c
    };

    let gpu = GpuRenderer::new();

    // Terminal-style: low-res framebuffer, scaled up to chunky pixels
    render_gif(
        &mut scene,
        trex_start..trex_end,
        &camera,
        &gpu,
        frame_count,
        model_scale,
        80,  // fb width (= terminal cols in HalfBlock)
        80,  // fb height (= terminal rows × 2)
        8,   // pixel scale
        2,   // padding in fb pixels
        "trex_terminal.gif",
    );

    // Smooth: high-res framebuffer, 1:1 pixels
    render_gif(
        &mut scene,
        trex_start..trex_end,
        &camera,
        &gpu,
        frame_count,
        model_scale,
        400,
        300,
        1,  // no scaling
        8,  // padding in fb pixels
        "trex_transparent.gif",
    );
}

fn render_gif(
    scene: &mut Scene,
    trex_range: std::ops::Range<usize>,
    camera: &Camera,
    gpu: &GpuRenderer,
    frame_count: usize,
    model_scale: f32,
    fb_w: u32,
    fb_h: u32,
    pixel_scale: u32,
    padding: u32,
    output_path: &str,
) {
    eprintln!(
        "\n[{}] {} frames, {}x{} framebuffer, {}x scale",
        output_path, frame_count, fb_w, fb_h, pixel_scale
    );

    let mut fb = Framebuffer::new(fb_w, fb_h);

    // --- Render all frames, track union bounding box ---
    let mut saved: Vec<(Vec<Rgb>, Vec<u8>)> = Vec::with_capacity(frame_count);
    let mut min_x = fb_w;
    let mut min_y = fb_h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;

    for i in 0..frame_count {
        let elapsed = i as f32 / FPS as f32;

        let base = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        let spin = Quat::from_rotation_y(elapsed * ROTATION_SPEED);
        for obj in &mut scene.objects[trex_range.clone()] {
            obj.transform.rotation = spin * base;
            obj.transform.scale = Vec3::splat(model_scale);
            obj.transform.position = Vec3::new(0.0, -3.0, 0.0);
        }

        gpu.render(scene, camera, &mut fb);

        for fy in 0..fb_h {
            for fx in 0..fb_w {
                if fb.alpha[fb.index(fx, fy)] > 0 {
                    min_x = min_x.min(fx);
                    min_y = min_y.min(fy);
                    max_x = max_x.max(fx);
                    max_y = max_y.max(fy);
                }
            }
        }

        saved.push((fb.color.clone(), fb.alpha.clone()));
        eprint!("\r  render {}/{}", i + 1, frame_count);
    }
    eprintln!();

    // --- Crop with padding ---
    let crop_x = min_x.saturating_sub(padding);
    let crop_y = min_y.saturating_sub(padding);
    let crop_w = (max_x + 1 + padding).min(fb_w) - crop_x;
    let crop_h = (max_y + 1 + padding).min(fb_h) - crop_y;

    let img_w = crop_w * pixel_scale;
    let img_h = crop_h * pixel_scale;
    eprintln!("  crop {}x{} -> {}x{} (output {}x{})", fb_w, fb_h, crop_w, crop_h, img_w, img_h);

    // --- Crop, scale, encode ---
    let delay = image::Delay::from_numer_denom_ms(1000 / FPS, 1);
    let mut frames: Vec<Frame> = Vec::with_capacity(frame_count);

    for (i, (color, alpha)) in saved.iter().enumerate() {
        let mut rgba = RgbaImage::new(img_w, img_h);

        for cy in 0..crop_h {
            for cx in 0..crop_w {
                let src_idx = ((crop_y + cy) * fb_w + (crop_x + cx)) as usize;
                let c = color[src_idx];
                let a = alpha[src_idx];
                let pixel = image::Rgba([c.0, c.1, c.2, a]);

                let ox = cx * pixel_scale;
                let oy = cy * pixel_scale;
                for dy in 0..pixel_scale {
                    for dx in 0..pixel_scale {
                        rgba.put_pixel(ox + dx, oy + dy, pixel);
                    }
                }
            }
        }

        frames.push(Frame::from_parts(rgba, 0, 0, delay));
        eprint!("\r  encode {}/{}", i + 1, frame_count);
    }
    eprintln!();

    let file = File::create(output_path).expect("Failed to create output file");
    let mut encoder = GifEncoder::new(file);
    encoder.set_repeat(Repeat::Infinite).unwrap();
    encoder
        .encode_frames(frames)
        .expect("Failed to encode GIF");

    eprintln!("  wrote {}", output_path);
}
