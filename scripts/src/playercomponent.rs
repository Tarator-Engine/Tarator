use scr_types::Component;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Component, Serialize, Deserialize)]
pub struct PlayerComponent {
    name: String,
}
