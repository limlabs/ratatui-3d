use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
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
    // Build the scene
    let mut scene = Scene::new();

    // Add a cube
    scene.add_object(
        SceneObject::new(primitives::cube())
            .with_material(Material::default().with_color(Rgb(100, 150, 255))),
    );

    // Add a sphere
    scene.add_object(
        SceneObject::new(primitives::sphere(16, 24))
            .with_material(Material::default().with_color(Rgb(255, 100, 100)))
            .with_transform(Transform::from_position(Vec3::new(2.0, 0.0, 0.0))),
    );

    // Add a ground plane
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

    // Lights
    scene.add_light(Light::ambient(Rgb(255, 255, 255), 0.15));
    scene.add_light(Light::directional(
        Vec3::new(-1.0, -1.0, -1.0),
        Rgb(255, 255, 255),
    ));
    scene.add_light(Light::point(Vec3::new(2.0, 3.0, 2.0), Rgb(255, 220, 180)));

    // Viewport state
    let mut state = Viewport3DState::default();
    state.camera.position = Vec3::new(3.0, 2.5, 4.0);
    state.camera.target = Vec3::new(0.5, 0.0, 0.0);

    let start = Instant::now();

    loop {
        let elapsed = start.elapsed().as_secs_f32();

        // Animate cube rotation
        scene.objects[0].transform.rotation = Quat::from_rotation_y(elapsed * 0.8)
            * Quat::from_rotation_x(elapsed * 0.3);

        // Draw
        terminal.draw(|f| {
            let block = Block::bordered().title(format!(
                " 3D Renderer | Mode: {:?} | [1/2/3] mode [arrows] orbit [q] quit ",
                state.render_mode
            ));
            f.render_stateful_widget(Viewport3D::new(&scene).block(block), f.area(), &mut state);
        })?;

        // Handle input (poll with short timeout for animation)
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
