use std::sync::Arc;

use cgmath::{Quaternion, Vector3, Matrix4};
use gltf::Mesh;

pub struct Node {
    pub index: usize,
    pub children: Vec<usize>,
    pub mesh: Option<Arc<Mesh>>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    // TODO: weights
    // weights_id: usize,
    pub translation: Vector3<f32>,
    // TODO: camera importing
    // pub camera: Option<Camera>,

    pub name: Option<String>,
    pub final_transform: Matrix4<f32>, // includes parent transforms
}