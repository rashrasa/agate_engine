use std::{f32::consts::PI, time::Instant};

use nalgebra::{UnitQuaternion, Vector3};

use crate::core::{System, components::EntityDescription, entity::BoundingBox};

pub struct EntitySpawnerSystem {
    mesh_id: u64,
    texture_id: u64,
    last: Instant,
}

impl EntitySpawnerSystem {
    pub const DESCRIPTION: EntityDescription = EntityDescription {
        mesh_id: self.mesh_id,
        texture_id: self.texture_id,
        velocity: Vector3::new(
            rand::random::<f32>() / f32::MAX * 15.0,
            rand::random::<f32>() / f32::MAX * 15.0,
            rand::random::<f32>() / f32::MAX * 15.0,
        ),
        acceleration: Vector3::zeros(),
        bounding_box: BoundingBox::ZERO,
        scale: Vector3::new(1.5, 1.5, 1.5),
        rotation: UnitQuaternion::from_euler_angles(
            rand::random::<f32>().abs() / f32::MAX * PI * 2.0,
            rand::random::<f32>().abs() / f32::MAX * PI * 2.0,
            rand::random::<f32>().abs() / f32::MAX * PI * 2.0,
        ),
        translation: Vector3::new(
            rand::random::<f32>().abs() / f32::MAX * 5.0,
            rand::random::<f32>().abs() / f32::MAX * 5.0,
            rand::random::<f32>().abs() / f32::MAX * 5.0,
        ),
        response: crate::core::entity::CollisionResponse::Inelastic(1.0),
        mass: 5.0e8,
    };
    pub fn new(mesh_id: u64, texture_id: u64) -> Self {
        Self {
            mesh_id,
            texture_id,
            last: Instant::now(),
        }
    }
}

impl System for EntitySpawnerSystem {
    fn before_tick(&mut self, args: &mut crate::core::BeforeTickArgs) {
        if self.last.elapsed().as_secs_f32() > 0.01 {
            args.state.add_entity(&Self::DESCRIPTION);
            self.last = Instant::now()
        }
    }
}
