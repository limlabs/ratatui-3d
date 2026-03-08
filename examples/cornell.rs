// Classic Cornell Box — a well-known benchmark scene for 3D renderers.

use std::io;
use std::time::Duration;

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

fn wall(pos: Vec3, scale: Vec3, color: Rgb) -> SceneObject {
    SceneObject::new(primitives::cube())
        .with_material(
            Material::default()
                .with_color(color)
                .with_diffuse(0.8)
                .with_specular(0.0)
                .with_ambient(0.05),
        )
        .with_transform({
            let mut t = Transform::from_position(pos);
            t.scale = scale;
            t
        })
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut scene = Scene::new();

    // Cornell box dimensions: 5×5×5 box
    let s = 5.0;
    let half = s / 2.0;
    let thick = 0.1;

    // Floor (white)
    scene.add_object(wall(
        Vec3::new(0.0, -half, 0.0),
        Vec3::new(s, thick, s),
        Rgb(200, 200, 200),
    ));

    // Ceiling (white)
    scene.add_object(wall(
        Vec3::new(0.0, half, 0.0),
        Vec3::new(s, thick, s),
        Rgb(200, 200, 200),
    ));

    // Back wall (white)
    scene.add_object(wall(
        Vec3::new(0.0, 0.0, -half),
        Vec3::new(s, s, thick),
        Rgb(200, 200, 200),
    ));

    // Left wall (red)
    scene.add_object(wall(
        Vec3::new(-half, 0.0, 0.0),
        Vec3::new(thick, s, s),
        Rgb(200, 40, 40),
    ));

    // Right wall (green)
    scene.add_object(wall(
        Vec3::new(half, 0.0, 0.0),
        Vec3::new(thick, s, s),
        Rgb(40, 200, 40),
    ));

    // Tall box (white, rotated slightly)
    scene.add_object(
        SceneObject::new(primitives::cube())
            .with_material(
                Material::default()
                    .with_color(Rgb(200, 200, 200))
                    .with_diffuse(0.8)
                    .with_specular(0.05)
                    .with_ambient(0.05),
            )
            .with_transform({
                let mut t = Transform::from_position(Vec3::new(-0.8, -1.0, -0.5));
                t.scale = Vec3::new(1.3, 3.0, 1.3);
                t.rotation = Quat::from_rotation_y(0.3);
                t
            }),
    );

    // Short box (white, rotated slightly)
    scene.add_object(
        SceneObject::new(primitives::cube())
            .with_material(
                Material::default()
                    .with_color(Rgb(200, 200, 200))
                    .with_diffuse(0.8)
                    .with_specular(0.05)
                    .with_ambient(0.05),
            )
            .with_transform({
                let mut t = Transform::from_position(Vec3::new(1.0, -1.75, 0.8));
                t.scale = Vec3::new(1.3, 1.5, 1.3);
                t.rotation = Quat::from_rotation_y(-0.3);
                t
            }),
    );

    // Ceiling light (area light approximated as a bright emissive surface + point light)
    scene.add_object(
        SceneObject::new(primitives::cube())
            .with_material(
                Material::default()
                    .with_color(Rgb(255, 255, 230))
                    .with_ambient(1.0)
                    .with_diffuse(0.0)
                    .with_specular(0.0),
            )
            .with_transform({
                let mut t = Transform::from_position(Vec3::new(0.0, half - thick * 0.5 - 0.01, 0.0));
                t.scale = Vec3::new(1.5, 0.02, 1.5);
                t
            }),
    );

    // Lights
    scene.add_light(Light::ambient(Rgb(255, 255, 255), 0.05));
    scene.add_light(Light::point(
        Vec3::new(0.0, half - 0.3, 0.0),
        Rgb(255, 250, 230),
    ));

    scene = scene.with_background(Rgb(0, 0, 0));

    let mut state = Viewport3DState::default();
    state.camera.position = Vec3::new(0.0, 0.0, 10.0);
    state.camera.target = Vec3::new(0.0, 0.0, 0.0);
    state.pipeline = Pipeline::RaytraceGpu;

    loop {
        terminal.draw(|f| {
            let block = Block::bordered().title(format!(
                " Cornell Box | {:?} {:?} | [1/2/3] mode [r] raytrace [arrows] orbit [+/-] zoom [q] quit ",
                state.pipeline, state.render_mode
            ));
            f.render_stateful_widget(
                Viewport3D::new(&scene).block(block),
                f.area(),
                &mut state,
            );
        })?;

        if event::poll(Duration::from_millis(100))? {
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
