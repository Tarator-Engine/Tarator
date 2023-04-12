pub mod components;
pub mod prims;
pub use macros::{InitSystems, System};

pub type System = fn(&mut tar_ecs::prelude::World);

#[repr(C)]
pub struct Systems {
    /// list of systems with a function pointer and a bool indicating wether it depends on the
    /// previous system
    ///
    /// ## Note:
    /// this is for internal use only!
    pub systems: Vec<(System, bool)>,
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
    /// have to be executed in serial use [`add_serial`]
    pub fn add(mut self, func: System) -> Self {
        self.addr(func);
        self
    }

    /// add a system you want to use
    ///
    /// this is intended for internal use but you can use it instead of the builder pattern
    /// ## Note:
    /// the systems added using this function may be executed in **any order** for systems that
    /// have to be executed in serial use [`addr_serial`]
    pub fn addr(&mut self, func: System) {
        self.systems.push((func, false));
    }

    /// add a system you want to use but it will be executed after the one you specified before
    pub fn add_serial(mut self, func: System) -> Self {
        self.addr_serial(func);
        self
    }

    /// add a system you want to use but it will be executed after the one you specified before
    ///
    /// this is intended for internal use but you can use it instead of the builder pattern
    pub fn addr_serial(&mut self, func: System) {
        self.systems.push((func, true));
    }
}
