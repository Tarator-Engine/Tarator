use cgmath::Zero;
use tar_ecs::prelude::*;

use crate::prims::{Euler, Rad, Vec3};

/// This component stored transform attributes
#[derive(Debug, Clone, Component)]
pub struct Transform {
    pub pos: Vec3,
    pub rot: Euler,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vec3::zero(),
            rot: Euler::new(Rad::zero(), Rad::zero(), Rad::zero()),
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
    pub amount_left: f32,
    pub amount_right: f32,
    pub amount_forward: f32,
    pub amount_backward: f32,
    pub amount_up: f32,
    pub amount_down: f32,
    pub rotate_horizontal: f32,
    pub rotate_vertical: f32,
    pub speed: f32,
    pub sensitivity: f32,
    pub mouse_pressed: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fovy: cgmath::Rad(90.0),
            znear: 0.001,
            zfar: 1000.0,
            active: true,
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            speed: 1.0,
            sensitivity: 0.4,
            mouse_pressed: false,
        }
    }
}

/// This component stores basic entity info e.g. name
/// it is required for it to be shown in the editor
#[derive(Debug, Clone, Component)]
pub struct Info {
    pub name: String,
    pub id: uuid::Uuid,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            name: "test".into(),
            id: uuid::Uuid::new_v4(),
        }
    }
}
