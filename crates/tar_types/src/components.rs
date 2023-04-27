use tar_ecs::prelude::Component;

use crate::prims::{Quat, Vec3};

#[derive(Debug, Clone, Component)]
pub struct Transform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}

#[derive(Debug, Clone, Component)]
pub struct Rendering {
    pub model_id: uuid::Uuid,
}
