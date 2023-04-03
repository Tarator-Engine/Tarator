use cgmath::InnerSpace;
use winit::dpi::PhysicalSize;

use crate::{
    components::{Camera, Transform},
    prims::{Mat4, Point3, Vec3},
};

#[derive(Debug)]
pub struct CameraParams {
    pub position: Vec3,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
}

/// takes in the output from a call to [`tar_ecs::world::World::component_collect`]
/// and transforms it into the parameters required for rendering
#[must_use] pub fn get_cam_params(cam: (Transform, Camera), size: PhysicalSize<u32>) -> CameraParams {
    let cam_comp = cam.1;
    let transform = cam.0;

    let a = size.width as f32 / size.height as f32;
    let y = cam_comp.fovy.0;
    let n = cam_comp.znear;
    let pos = transform.pos;

    let euler = transform.rot;

    let (sin_pitch, cos_pitch) = euler.x.0.sin_cos();
    let (sin_yaw, cos_yaw) = euler.y.0.sin_cos();

    let view_matrix = Mat4::look_to_rh(
        Point3::new(transform.pos.x, transform.pos.y, transform.pos.z),
        Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
        Vec3::unit_y(),
    );

    CameraParams {
        position: pos,
        view_matrix,
        projection_matrix: Mat4::new(
            1.0 / (a * (0.5 * y).tan()),
            0.0,
            0.0,
            0.0, // NOTE: first column!
            0.0,
            1.0 / (0.5 * y).tan(),
            0.0,
            0.0,
            0.0,
            0.0,
            -1.0,
            -1.0,
            0.0,
            0.0,
            -2.0 * n,
            0.0,
        ),
    }
}
