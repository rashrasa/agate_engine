use nalgebra::Vector3;

use crate::core::{G, System, Unique, entity::Entity};

pub struct GravitySystem;

impl System for GravitySystem {
    fn before_tick(&mut self, args: &mut crate::core::BeforeTickArgs) {
        unsafe {
            for a in args.state.entities() {
                let a = a as *const Entity;
                let mut accel = Vector3::zeros();
                for b in args.state.entities() {
                    let a = a.as_ref().unwrap();
                    if a.id() != b.id() {
                        let vec = b.translation - a.translation;
                        accel += (G as f32 * b.mass / vec.magnitude().powi(2)) * vec.normalize();
                    }
                }
                (*a.cast_mut()).acceleration = accel;
            }
        }
    }
}
