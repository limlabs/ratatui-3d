// "DNA" (https://poly.pizza/m/eqeVBmTlgiN) by nathan stevens is licensed under CC-BY 3.0

use std::f32::consts::FRAC_PI_2;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::Block;

use ratatui_3d::prelude::*;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let meshes = ratatui_3d::loader::gltf::load_gltf("assets/dna.glb")
        .expect("Failed to load dna.glb");

    let mut scene = Scene::new();

    // Model is ~580 units along Z, ~124 wide. Scale to ~8 units tall and stand upright.
    let model_scale = 0.014;

    let dna_start = scene.objects.len();
    for mesh in meshes {
        scene.add_object(
            SceneObject::new(mesh)
                .with_material(
                    Material::default()
                        .with_color(Rgb(100, 180, 255))
                        .with_specular(0.6)
                        .with_shininess(32.0),
                )
                .with_transform({
                    let mut t = Transform::default();
                    t.scale = Vec3::splat(model_scale);
                    // Rotate to stand upright (model extends along Z, we want Y)
                    t.rotation = Quat::from_rotation_x(-FRAC_PI_2);
                    // Center vertically: model center is at Z≈290, after rotation that's Y
                    t.position = Vec3::new(0.0, -290.0 * model_scale, 0.0);
                    t
                }),
        );
    }
    let dna_end = scene.objects.len();

    // Lighting
    scene.add_light(Light::ambient(Rgb(255, 255, 255), 0.2));
    scene.add_light(Light::directional(
        Vec3::new(-1.0, -1.0, -1.0),
        Rgb(255, 255, 255),
    ));
    scene.add_light(Light::point(
        Vec3::new(3.0, 4.0, 4.0),
        Rgb(255, 200, 150),
    ));

    scene = scene.with_background(Rgb(10, 10, 20));

    let mut state = Viewport3DState::default();
    state.camera.position = Vec3::new(0.0, 2.0, 8.0);
    state.camera.target = Vec3::ZERO;
    state.pipeline = Pipeline::RaytraceGpu;

    let start = Instant::now();

    loop {
        let elapsed = start.elapsed().as_secs_f32();

        // Spin the DNA around Y, keeping it upright
        let base = Quat::from_rotation_x(-FRAC_PI_2);
        let spin = Quat::from_rotation_y(elapsed * 0.5);
        for obj in &mut scene.objects[dna_start..dna_end] {
            obj.transform.rotation = spin * base;
            obj.transform.scale = Vec3::splat(model_scale);
        }

        // Draw
        terminal.draw(|f| {
            let block = Block::bordered().title(format!(
                " DNA Helix | {:?} {:?} | [1/2/3] mode [r] raytrace [arrows] orbit [+/-] zoom [q] quit ",
                state.pipeline, state.render_mode
            ));
            f.render_stateful_widget(
                Viewport3D::new(&scene).block(block),
                f.area(),
                &mut state,
            );
        })?;

        // Handle input
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => {
                        state.pipeline = match state.pipeline {
                            Pipeline::Rasterize => Pipeline::Raytrace,
                            Pipeline::Raytrace => Pipeline::RaytraceGpu,
                            Pipeline::RaytraceGpu => Pipeline::Rasterize,
                        };
                    }
                    KeyCode::Char('1') => state.render_mode = RenderMode::HalfBlock,
                    KeyCode::Char('2') => state.render_mode = RenderMode::Braille,
                    KeyCode::Char('3') => state.render_mode = RenderMode::Ascii,
                    KeyCode::Left => state.camera.orbit(-0.1, 0.0),
                    KeyCode::Right => state.camera.orbit(0.1, 0.0),
                    KeyCode::Up => state.camera.orbit(0.0, -0.1),
                    KeyCode::Down => state.camera.orbit(0.0, 0.1),
                    KeyCode::Char('+') | KeyCode::Char('=') => state.camera.zoom(-0.5),
                    KeyCode::Char('-') => state.camera.zoom(0.5),
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
