use napi::bindgen_prelude::*;
use napi_derive::napi;
use render3d::pipeline::Framebuffer;
use render3d::prelude::*;

/// A 3D scene that can be rendered to a pixel buffer.
#[napi]
pub struct Renderer {
    scene: Scene,
    camera: Camera,
}

#[napi]
impl Renderer {
    /// Create a new renderer with an empty scene.
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            scene: Scene::new(),
            camera: Camera::default(),
        }
    }

    /// Set the camera position.
    #[napi]
    pub fn set_camera(&mut self, px: f64, py: f64, pz: f64, tx: f64, ty: f64, tz: f64) {
        self.camera.position = Vec3::new(px as f32, py as f32, pz as f32);
        self.camera.target = Vec3::new(tx as f32, ty as f32, tz as f32);
    }

    /// Set the background color.
    #[napi]
    pub fn set_background(&mut self, r: u32, g: u32, b: u32) {
        self.scene.background = Rgb(r as u8, g as u8, b as u8);
    }

    /// Add a cube to the scene. Returns the object index.
    #[napi]
    pub fn add_cube(&mut self, x: f64, y: f64, z: f64, r: u32, g: u32, b: u32) -> u32 {
        let idx = self.scene.objects.len() as u32;
        self.scene.add_object(
            SceneObject::new(primitives::cube())
                .with_material(Material::default().with_color(Rgb(r as u8, g as u8, b as u8)))
                .with_transform(Transform::from_position(Vec3::new(
                    x as f32, y as f32, z as f32,
                ))),
        );
        idx
    }

    /// Add a sphere to the scene. Returns the object index.
    #[napi]
    pub fn add_sphere(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        r: u32,
        g: u32,
        b: u32,
        stacks: Option<u32>,
        slices: Option<u32>,
    ) -> u32 {
        let idx = self.scene.objects.len() as u32;
        self.scene.add_object(
            SceneObject::new(primitives::sphere(
                stacks.unwrap_or(16),
                slices.unwrap_or(24),
            ))
            .with_material(Material::default().with_color(Rgb(r as u8, g as u8, b as u8)))
            .with_transform(Transform::from_position(Vec3::new(
                x as f32, y as f32, z as f32,
            ))),
        );
        idx
    }

    /// Add an ambient light.
    #[napi]
    pub fn add_ambient_light(&mut self, r: u32, g: u32, b: u32, intensity: f64) {
        self.scene
            .add_light(Light::ambient(Rgb(r as u8, g as u8, b as u8), intensity as f32));
    }

    /// Add a directional light.
    #[napi]
    pub fn add_directional_light(&mut self, dx: f64, dy: f64, dz: f64, r: u32, g: u32, b: u32) {
        self.scene.add_light(Light::directional(
            Vec3::new(dx as f32, dy as f32, dz as f32),
            Rgb(r as u8, g as u8, b as u8),
        ));
    }

    /// Add a point light.
    #[napi]
    pub fn add_point_light(&mut self, x: f64, y: f64, z: f64, r: u32, g: u32, b: u32) {
        self.scene.add_light(Light::point(
            Vec3::new(x as f32, y as f32, z as f32),
            Rgb(r as u8, g as u8, b as u8),
        ));
    }

    /// Render the scene to an RGB pixel buffer (3 bytes per pixel).
    #[napi]
    pub fn render(&self, width: u32, height: u32) -> Buffer {
        let mut fb = Framebuffer::new(width, height);
        render3d::pipeline::render(&self.scene, &self.camera, &mut fb);
        let bytes: Vec<u8> = fb.color.iter().flat_map(|c| [c.0, c.1, c.2]).collect();
        bytes.into()
    }

    /// Ray-trace the scene to an RGB pixel buffer (3 bytes per pixel).
    #[napi]
    pub fn render_raytrace(&self, width: u32, height: u32) -> Buffer {
        let mut fb = Framebuffer::new(width, height);
        render3d::pipeline::raytrace::render(&self.scene, &self.camera, &mut fb);
        let bytes: Vec<u8> = fb.color.iter().flat_map(|c| [c.0, c.1, c.2]).collect();
        bytes.into()
    }
}
