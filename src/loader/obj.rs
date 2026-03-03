use crate::math::Vec3;
use crate::mesh::{Mesh, Vertex};
use std::path::Path;

/// Load meshes from an OBJ file. Returns one `Mesh` per model in the file.
pub fn load_obj(path: impl AsRef<Path>) -> Result<Vec<Mesh>, String> {
    let (models, _materials) =
        tobj::load_obj(path.as_ref(), &tobj::GPU_LOAD_OPTIONS).map_err(|e| e.to_string())?;

    let mut meshes = Vec::with_capacity(models.len());

    for model in &models {
        let m = &model.mesh;
        let mut vertices = Vec::with_capacity(m.positions.len() / 3);
        let has_normals = !m.normals.is_empty();

        for i in 0..(m.positions.len() / 3) {
            let pos = Vec3::new(m.positions[i * 3], m.positions[i * 3 + 1], m.positions[i * 3 + 2]);
            let normal = if has_normals {
                Vec3::new(m.normals[i * 3], m.normals[i * 3 + 1], m.normals[i * 3 + 2])
            } else {
                Vec3::ZERO
            };
            let mut v = Vertex::new(pos, normal);
            if !m.texcoords.is_empty() && i * 2 + 1 < m.texcoords.len() {
                v = v.with_uv(m.texcoords[i * 2], m.texcoords[i * 2 + 1]);
            }
            vertices.push(v);
        }

        // Compute face normals if normals weren't provided
        if !has_normals {
            compute_normals(&mut vertices, &m.indices);
        }

        meshes.push(Mesh::new(vertices, m.indices.clone()));
    }

    Ok(meshes)
}

/// Compute smooth vertex normals by averaging face normals.
fn compute_normals(vertices: &mut [Vertex], indices: &[u32]) {
    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let v0 = vertices[i0].position;
        let v1 = vertices[i1].position;
        let v2 = vertices[i2].position;
        let face_normal = (v1 - v0).cross(v2 - v0);
        vertices[i0].normal += face_normal;
        vertices[i1].normal += face_normal;
        vertices[i2].normal += face_normal;
    }
    for v in vertices.iter_mut() {
        v.normal = v.normal.normalize_or_zero();
    }
}
