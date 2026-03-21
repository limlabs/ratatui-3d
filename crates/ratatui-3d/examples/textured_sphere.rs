use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::Block;

use ratatui_3d::prelude::*;

fn load_texture(path: &str) -> Arc<Texture> {
    let img = image::open(path)
        .unwrap_or_else(|e| panic!("Failed to load texture {path}: {e}"))
        .into_rgba8();
    let (w, h) = img.dimensions();
    Arc::new(Texture::from_rgba(w, h, img.into_raw()))
}

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
    let texture = load_texture("assets/vaporwave.png");

    let mut scene = Scene::new();

    // Textured sphere
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

    // Lights
    scene.add_light(Light::ambient(Rgb(255, 255, 255), 0.5));
    scene.add_light(Light::directional(
        Vec3::new(-0.5, -1.0, -0.5),
        Rgb(255, 255, 255),
    ));
    scene.add_light(Light::point(
        Vec3::new(3.0, 2.0, 3.0),
        Rgb(255, 240, 220),
    ));
    scene.add_light(Light::point(
        Vec3::new(-3.0, 1.0, 2.0),
        Rgb(220, 220, 255),
    ));

    let mut state = Viewport3DState::default();
    state.camera.position = Vec3::new(0.0, 0.0, 3.5);
    state.camera.target = Vec3::ZERO;

    let start = Instant::now();

    loop {
        let elapsed = start.elapsed().as_secs_f32();

        // Slow rotation
        scene.objects[0].transform.rotation = Quat::from_rotation_y(elapsed * 0.5);

        terminal.draw(|f| {
            let block = Block::bordered().title(
                " Textured Sphere | [1/2/3] mode [arrows] orbit [+/-] zoom [q] quit ",
            );
            f.render_stateful_widget(Viewport3D::new(&scene).block(block), f.area(), &mut state);
        })?;

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
                    KeyCode::Char('+') | KeyCode::Char('=') => state.camera.zoom(-0.3),
                    KeyCode::Char('-') => state.camera.zoom(0.3),
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
