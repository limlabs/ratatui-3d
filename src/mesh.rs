use crate::math::Vec3;

/// A single vertex with position, normal, and texture coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3) -> Self {
        Self {
            position,
            normal,
            uv: [0.0, 0.0],
        }
    }

    pub fn with_uv(mut self, u: f32, v: f32) -> Self {
        self.uv = [u, v];
        self
    }
}

/// An indexed triangle mesh.
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    /// Triangle indices (length must be a multiple of 3).
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        debug_assert!(indices.len().is_multiple_of(3), "indices length must be a multiple of 3");
        Self { vertices, indices }
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}
