use tar_ecs::prelude::Component;

use crate::prims::{Quat, Rad, Vec3};

/// This component stored transform attributes
#[derive(Debug, Clone, Component)]
pub struct Transform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}

/// This Component indicates that an entity is rendered
///
/// **Note**: The [`Transform`] component is also required
/// to render
#[derive(Debug, Clone, Component)]
pub struct Rendering {
    pub model_id: uuid::Uuid,
}

/// This Component indicates taht the entity is a camera.
///
/// **Note**: The [`Transform`] component is also required
/// to act like a camera
#[derive(Debug, Clone, Component)]
pub struct Camera {
    pub fovy: Rad,
    pub znear: f32,
    pub zfar: f32,
    pub active: bool,
}
