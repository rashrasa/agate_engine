mod systems;

use std::time::Duration;

pub use systems::{
    AudioSystem, BoundarySystem, CollisionsSystem, DynamicsSystem, EntitySpawnerSystem,
    GravitySystem, MetricsSystem,
};

use crate::core::System;

pub const DEFAULT_SYSTEMS: fn() -> Vec<Box<dyn System>> = || {
    vec![
        Box::new(CollisionsSystem),
        Box::new(MetricsSystem::new(Duration::new(5, 0))),
        Box::new(AudioSystem::new()),
        Box::new(DynamicsSystem),
        Box::new(GravitySystem),
        // Box::new(EntitySpawnerSystem::new(0, 0)),
        Box::new(BoundarySystem::new(
            [-50.0, 50.0],
            [-50.0, 50.0],
            [-50.0, 50.0],
        )),
    ]
};
