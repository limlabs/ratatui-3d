use crate::color::Rgb;
use crate::light::Light;
use crate::object::SceneObject;

/// The 3D scene containing objects and lights.
#[derive(Debug, Clone)]
pub struct Scene {
    pub objects: Vec<SceneObject>,
    pub lights: Vec<Light>,
    pub background: Rgb,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            lights: Vec::new(),
            background: Rgb::BLACK,
        }
    }

    pub fn add_object(&mut self, object: SceneObject) -> &mut Self {
        self.objects.push(object);
        self
    }

    pub fn add_light(&mut self, light: Light) -> &mut Self {
        self.lights.push(light);
        self
    }

    pub fn with_background(mut self, color: Rgb) -> Self {
        self.background = color;
        self
    }
}
