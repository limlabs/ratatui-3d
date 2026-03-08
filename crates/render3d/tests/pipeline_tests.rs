use render3d::prelude::*;
use render3d::pipeline::{self, Framebuffer};
use render3d::pipeline::vertex::transform_vertex;
use render3d::pipeline::fragment::shade_fragment;
use render3d::pipeline::rasterize::rasterize_triangle;
use render3d::pipeline::vertex::TransformedVertex;

// ============================================================
// Integration test: full pipeline render
// ============================================================

#[test]
fn integration_render_cube_has_nonblack_pixels() {
    let mut scene = Scene::new();
    scene.add_object(
        SceneObject::new(primitives::cube())
            .with_material(Material::default().with_color(Rgb(100, 150, 255))),
    );
    scene.add_light(Light::ambient(Rgb(255, 255, 255), 0.15));
    scene.add_light(Light::directional(Vec3::new(-1.0, -1.0, -1.0), Rgb(255, 255, 255)));

    let camera = Camera {
        position: Vec3::new(3.0, 2.5, 4.0),
        target: Vec3::new(0.5, 0.0, 0.0),
        ..Camera::default()
    };

    let mut fb = Framebuffer::new(80, 50);
    pipeline::render(&scene, &camera, &mut fb);

    let nonblack = fb.color.iter().filter(|c| **c != Rgb::BLACK).count();
    eprintln!("Integration test: {nonblack} / {} pixels are non-black", fb.color.len());
    assert!(nonblack > 0, "Expected some non-black pixels after rendering a cube");
}

// ============================================================
// Unit test: Fragment shader produces visible output
// ============================================================

#[test]
fn fragment_ambient_only_produces_color() {
    let lights = [Light::ambient(Rgb(255, 255, 255), 0.5)];
    let material = Material::default().with_color(Rgb(200, 100, 50));
    let color = shade_fragment(
        Vec3::ZERO,
        Vec3::Y,      // normal pointing up
        &material,
        &lights,
        Vec3::new(0.0, 0.0, 5.0), // camera
    );
    eprintln!("Ambient-only fragment color: {:?}", color);
    assert!(color.0 > 0 || color.1 > 0 || color.2 > 0,
        "Ambient light should produce non-black output");
}

#[test]
fn fragment_directional_light_produces_color() {
    // Normal faces up, light comes from above (direction = (0, -1, 0) means light travels down)
    let lights = [Light::directional(Vec3::new(0.0, -1.0, 0.0), Rgb(255, 255, 255))];
    let material = Material::default().with_color(Rgb(200, 200, 200));
    let color = shade_fragment(
        Vec3::ZERO,
        Vec3::Y,          // surface normal facing up
        &material,
        &lights,
        Vec3::new(0.0, 1.0, 5.0),
    );
    eprintln!("Directional fragment color: {:?}", color);
    // light_dir = -direction = (0, 1, 0), which is parallel to normal → max diffuse
    assert!(color.0 > 50, "Directional light hitting surface head-on should be bright, got {:?}", color);
}

#[test]
fn fragment_point_light_produces_color() {
    let lights = [Light::point(Vec3::new(0.0, 2.0, 0.0), Rgb(255, 255, 255))];
    let material = Material::default().with_color(Rgb(200, 200, 200));
    let color = shade_fragment(
        Vec3::ZERO,
        Vec3::Y,
        &material,
        &lights,
        Vec3::new(0.0, 0.0, 5.0),
    );
    eprintln!("Point light fragment color: {:?}", color);
    assert!(color.0 > 0, "Point light above surface should illuminate it, got {:?}", color);
}

// ============================================================
// Unit test: Vertex transform produces valid screen coordinates
// ============================================================

#[test]
fn vertex_transform_cube_center_is_on_screen() {
    let camera = Camera {
        position: Vec3::new(0.0, 0.0, 3.0),
        target: Vec3::ZERO,
        ..Camera::default()
    };

    let model = Mat4::IDENTITY;
    let view = camera.view_matrix();
    let proj = camera.projection_matrix(1.6); // 80/50
    let view_proj = proj * view;
    let normal_matrix = model.inverse().transpose();

    let vw = 80.0f32;
    let vh = 50.0f32;

    // Transform the origin (center of cube)
    let result = transform_vertex(
        Vec3::ZERO, Vec3::Z, &model, &view_proj, &normal_matrix, vw, vh,
    );

    eprintln!("Origin screen pos: {:?}", result.map(|v| v.screen_pos));
    assert!(result.is_some(), "Origin should be visible from camera at (0,0,3)");

    let tv = result.unwrap();
    assert!(tv.screen_pos.x > 0.0 && tv.screen_pos.x < vw,
        "X should be on screen: {}", tv.screen_pos.x);
    assert!(tv.screen_pos.y > 0.0 && tv.screen_pos.y < vh,
        "Y should be on screen: {}", tv.screen_pos.y);
    assert!(tv.screen_pos.z > 0.0 && tv.screen_pos.z < 1.0,
        "Depth should be in (0,1): {}", tv.screen_pos.z);
}

#[test]
fn vertex_transform_cube_front_face_all_visible() {
    let camera = Camera {
        position: Vec3::new(0.0, 0.0, 3.0),
        target: Vec3::ZERO,
        ..Camera::default()
    };

    let model = Mat4::IDENTITY;
    let view = camera.view_matrix();
    let proj = camera.projection_matrix(1.6);
    let view_proj = proj * view;
    let normal_matrix = model.inverse().transpose();

    let vw = 80.0f32;
    let vh = 100.0f32;

    // Front face vertices
    let positions = [
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
    ];

    for (i, pos) in positions.iter().enumerate() {
        let result = transform_vertex(
            *pos, Vec3::Z, &model, &view_proj, &normal_matrix, vw, vh,
        );
        eprintln!("Front face vertex {i} ({pos:?}): screen_pos = {:?}", result.map(|v| v.screen_pos));
        assert!(result.is_some(), "Front face vertex {i} should be visible");
    }
}

// ============================================================
// Unit test: Backface culling / winding order
// ============================================================

#[test]
fn rasterize_front_facing_triangle() {
    // After viewport Y-flip, front-facing triangles have NEGATIVE cross_z.
    // p0=(10,40), p1=(50,40), p2=(30,10): edge1=(40,0), edge2=(20,-30)
    // cross_z = 40*(-30) - 0*20 = -1200 < 0 → front-facing → kept

    let v0 = TransformedVertex {
        screen_pos: Vec3::new(10.0, 40.0, 0.5),
        world_pos: Vec3::ZERO,
        world_normal: Vec3::Z,
    };
    let v1 = TransformedVertex {
        screen_pos: Vec3::new(50.0, 40.0, 0.5),
        world_pos: Vec3::X,
        world_normal: Vec3::Z,
    };
    let v2 = TransformedVertex {
        screen_pos: Vec3::new(30.0, 10.0, 0.5),
        world_pos: Vec3::Y,
        world_normal: Vec3::Z,
    };

    let material = Material::default().with_color(Rgb(255, 255, 255));
    let lights = [Light::ambient(Rgb(255, 255, 255), 1.0)];
    let camera_pos = Vec3::new(0.0, 0.0, 5.0);

    let mut fb = Framebuffer::new(80, 50);
    rasterize_triangle(&v0, &v1, &v2, &material, &lights, camera_pos, &mut fb);

    let nonblack = fb.color.iter().filter(|c| **c != Rgb::BLACK).count();
    eprintln!("Front-facing triangle (negative cross): {nonblack} pixels rasterized");
    assert!(nonblack > 0, "Front-facing triangle (negative cross_z) should be rasterized");
}

#[test]
fn rasterize_back_facing_triangle_is_culled() {
    // Positive cross_z = back-facing in screen Y-down → culled
    // p0=(10,10), p1=(50,10), p2=(30,40): cross_z = 40*30 - 0*20 = 1200 > 0
    let v0 = TransformedVertex {
        screen_pos: Vec3::new(10.0, 10.0, 0.5),
        world_pos: Vec3::ZERO,
        world_normal: Vec3::Z,
    };
    let v1 = TransformedVertex {
        screen_pos: Vec3::new(50.0, 10.0, 0.5),
        world_pos: Vec3::X,
        world_normal: Vec3::Z,
    };
    let v2 = TransformedVertex {
        screen_pos: Vec3::new(30.0, 40.0, 0.5),
        world_pos: Vec3::Y,
        world_normal: Vec3::Z,
    };

    let material = Material::default().with_color(Rgb(255, 255, 255));
    let lights = [Light::ambient(Rgb(255, 255, 255), 1.0)];

    let mut fb = Framebuffer::new(80, 50);
    rasterize_triangle(&v0, &v1, &v2, &material, &lights, Vec3::new(0.0, 0.0, 5.0), &mut fb);

    let nonblack = fb.color.iter().filter(|c| **c != Rgb::BLACK).count();
    eprintln!("Back-facing triangle (positive cross): {nonblack} pixels rasterized");
    assert_eq!(nonblack, 0, "Back-facing triangle (positive cross_z) should be culled");
}

// ============================================================
// Unit test: Actual cube vertex winding after transform
// ============================================================

#[test]
fn cube_front_face_winding_after_transform() {
    // Camera looking straight at the front face (+Z)
    let camera = Camera {
        position: Vec3::new(0.0, 0.0, 3.0),
        target: Vec3::ZERO,
        ..Camera::default()
    };

    let model = Mat4::IDENTITY;
    let view = camera.view_matrix();
    let proj = camera.projection_matrix(1.6);
    let view_proj = proj * view;
    let normal_matrix = model.inverse().transpose();

    let vw = 80.0f32;
    let vh = 100.0f32;

    // First triangle of front face: v0=(-0.5,-0.5,0.5), v1=(0.5,-0.5,0.5), v2=(0.5,0.5,0.5)
    let tv0 = transform_vertex(
        Vec3::new(-0.5, -0.5, 0.5), Vec3::Z, &model, &view_proj, &normal_matrix, vw, vh,
    ).expect("v0 should be visible");

    let tv1 = transform_vertex(
        Vec3::new(0.5, -0.5, 0.5), Vec3::Z, &model, &view_proj, &normal_matrix, vw, vh,
    ).expect("v1 should be visible");

    let tv2 = transform_vertex(
        Vec3::new(0.5, 0.5, 0.5), Vec3::Z, &model, &view_proj, &normal_matrix, vw, vh,
    ).expect("v2 should be visible");

    eprintln!("v0 screen: {:?}", tv0.screen_pos);
    eprintln!("v1 screen: {:?}", tv1.screen_pos);
    eprintln!("v2 screen: {:?}", tv2.screen_pos);

    let p0 = tv0.screen_pos;
    let p1 = tv1.screen_pos;
    let p2 = tv2.screen_pos;

    let edge1 = p1 - p0;
    let edge2 = p2 - p0;
    let cross_z = edge1.x * edge2.y - edge1.y * edge2.x;

    eprintln!("Front face cross_z = {cross_z} (negative=front-facing=kept, positive=back-facing=culled)");
    assert!(cross_z < 0.0,
        "Front face of cube should have negative cross_z after Y-flip, got {cross_z}");
}

// ============================================================
// Unit test: Full pipeline with simple camera
// ============================================================

#[test]
fn render_cube_simple_camera_produces_pixels() {
    let mut scene = Scene::new();
    scene.add_object(
        SceneObject::new(primitives::cube())
            .with_material(Material::default().with_color(Rgb(255, 255, 255))),
    );
    scene.add_light(Light::ambient(Rgb(255, 255, 255), 1.0));

    // Simple camera straight on
    let camera = Camera {
        position: Vec3::new(0.0, 0.0, 3.0),
        target: Vec3::ZERO,
        ..Camera::default()
    };

    let mut fb = Framebuffer::new(80, 100);
    pipeline::render(&scene, &camera, &mut fb);

    let nonblack = fb.color.iter().filter(|c| **c != Rgb::BLACK).count();
    eprintln!("Simple camera cube render: {nonblack} / {} pixels are non-black", fb.color.len());

    // Print a few pixels around center
    for y in 45..55 {
        let row: String = (35..45).map(|x| {
            let c = fb.get_pixel(x, y);
            if c == Rgb::BLACK { '.' } else { '#' }
        }).collect();
        eprintln!("  row {y}: {row}");
    }

    assert!(nonblack > 0, "Should have rendered some pixels for a cube with ambient light");
}

// ============================================================
// Unit test: Framebuffer set_pixel / depth test
// ============================================================

#[test]
fn framebuffer_depth_test() {
    let mut fb = Framebuffer::new(10, 10);

    // Write a pixel at depth 0.5
    fb.set_pixel(5, 5, 0.5, Rgb(255, 0, 0));
    assert_eq!(fb.get_pixel(5, 5), Rgb(255, 0, 0));

    // Closer pixel (depth 0.3) should overwrite
    fb.set_pixel(5, 5, 0.3, Rgb(0, 255, 0));
    assert_eq!(fb.get_pixel(5, 5), Rgb(0, 255, 0));

    // Farther pixel (depth 0.8) should NOT overwrite
    fb.set_pixel(5, 5, 0.8, Rgb(0, 0, 255));
    assert_eq!(fb.get_pixel(5, 5), Rgb(0, 255, 0));
}
