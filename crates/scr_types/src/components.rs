use tar_ecs::prelude::*;
use serde::{Serialize, Deserialize};

use crate::{ecs_serde::SerdeComponent, prims::{Quat, Rad, Vec3}};

/// This component stored transform attributes
#[derive(Debug, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct Transform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl SerdeComponent for Transform {
    const NAME: &'static str = "scr::Transform";
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
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Camera {
    pub fovy: Rad,
    pub znear: f32,
    pub zfar: f32,
    pub active: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fovy: 90.0,
            znear: 0.001,
            zfar: 1000.0,
            active: true,
        }
    }
}

impl SerdeComponent for Camera {
    const NAME: &'static str = "scr::Camera";
}

/// This component stores basic entity info e.g. name
/// it is required for it to be shown in the editor
///
/// **Note**: [`Info`] does not derive Serialize, Deserialize or SerdeComponent,
/// because we use [`Info`] as a top-level entity descriptor and not part of the
/// components section in the serializations of the worlds.
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
