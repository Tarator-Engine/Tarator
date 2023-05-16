pub mod components;
pub mod prims;
pub mod ecs_serde;

use std::fmt::Debug;

pub use macros::{InitSystems, System};

pub type System = fn(&mut tar_ecs::prelude::World);

pub mod prelude {
    pub use super::components::*;
    pub use super::ecs_serde::SerdeComponent;
    pub use super::InitSystems;
    pub use super::System;
    pub use super::Systems;
}

#[repr(C)]
pub struct Systems {
    /// list of systems with a function pointer and a bool indicating wether it depends on the
    /// previous system
    ///
    /// ## Note:
    /// this is for internal use only!
    pub systems: Vec<(System, bool)>,
}

impl Debug for Systems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Systems")
            .field("number_of_systems", &self.systems.len())
            .finish()
    }
}

impl Systems {
    /// Create a new Systems struct it will be used to define which systems you want to use during
    /// execution of the game
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// add a system you want to use
    ///
    /// ## Note:
    /// the systems added using this function may be executed in **any order** for systems that
    /// have to be executed in series use [`add_serial`]
    pub fn add(mut self, func: System) -> Self {
        self.addr(func);
        self
    }

    /// add a system you want to use
    ///
    /// this function is intended for internal use but you can use it instead of the builder pattern
    /// ## Note:
    /// the systems added using this function may be executed in **any order** for systems that
    /// have to be executed in series use [`addr_serial`]
    pub fn addr(&mut self, func: System) {
        self.systems.push((func, false));
    }

    /// add a system you want to use which will be executed after the one you specified before
    pub fn add_serial(mut self, func: System) -> Self {
        self.addr_serial(func);
        self
    }

    /// add a system you want to use which will be executed after the one you specified before
    ///
    /// this function is intended for internal use but you can use it instead of the builder pattern
    pub fn addr_serial(&mut self, func: System) {
        self.systems.push((func, true));
    }
}
