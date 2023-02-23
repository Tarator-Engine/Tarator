use cgmath::Zero;
use tar_ecs::prelude::Component;

use crate::prims::{Quat, Rad, Vec3};

/// This component stored transform attributes
#[derive(Debug, Clone, Component)]
pub struct Transform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vec3::zero(),
            rot: Quat::zero(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
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

impl Default for Camera {
    fn default() -> Self {
        Self {
            fovy: cgmath::Rad(90.0),
            znear: 0.001,
            zfar: 1000.0,
            active: true,
        }
    }
}
