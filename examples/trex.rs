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
    // Terminal setup
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let result = run(&mut terminal);

    // Cleanup
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    // Load the T-Rex model
    let meshes = ratatui_3d::loader::gltf::load_gltf("assets/trex.glb")
        .expect("Failed to load trex.glb");

    // Build the scene
    let mut scene = Scene::new();

    // The model is very small (~0.31 units tall), scale it up
    let model_scale = 20.0;

    // Track how many objects belong to the T-Rex so we can rotate them all
    let trex_start = scene.objects.len();
    for mesh in meshes {
        scene.add_object(
            SceneObject::new(mesh)
                .with_material(Material::default().with_color(Rgb(100, 160, 80)))
                .with_transform({
                    let mut t = Transform::default();
                    t.scale = Vec3::splat(model_scale);
                    t
                }),
        );
    }
    let trex_end = scene.objects.len();

    // Add a ground plane
    scene.add_object(
        SceneObject::new(primitives::plane())
            .with_material(
                Material::default()
                    .with_color(Rgb(60, 120, 60))
                    .with_specular(0.1),
            )
            .with_transform({
                let mut t = Transform::from_position(Vec3::new(0.0, -3.0, 0.0));
                t.scale = Vec3::splat(20.0);
                t
            }),
    );

    // Lights
    scene.add_light(Light::ambient(Rgb(255, 255, 255), 0.2));
    scene.add_light(Light::directional(
        Vec3::new(-1.0, -1.0, -1.0),
        Rgb(255, 255, 255),
    ));
    scene.add_light(Light::point(Vec3::new(5.0, 8.0, 5.0), Rgb(255, 220, 180)));

    // Viewport state — camera positioned to frame the scaled model
    let mut state = Viewport3DState::default();
    state.camera.position = Vec3::new(5.0, 4.0, 8.0);
    state.camera.target = Vec3::new(0.0, 1.0, 0.0);

    let start = Instant::now();

    loop {
        let elapsed = start.elapsed().as_secs_f32();

        // Rotate all T-Rex meshes together (preserve scale)
        // Base rotation: -90° on X to stand the model upright (it's modeled along Z)
        let base = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        let spin = Quat::from_rotation_y(elapsed * 0.6);
        for obj in &mut scene.objects[trex_start..trex_end] {
            obj.transform.rotation = spin * base;
            obj.transform.scale = Vec3::splat(model_scale);
        }

        // Draw
        terminal.draw(|f| {
            let block = Block::bordered().title(format!(
                " T-Rex | Mode: {:?} | [1/2/3] mode [arrows] orbit [+/-] zoom [q] quit ",
                state.render_mode
            ));
            f.render_stateful_widget(Viewport3D::new(&scene).block(block), f.area(), &mut state);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
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
