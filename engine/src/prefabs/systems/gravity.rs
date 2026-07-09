use nalgebra::Vector3;

use crate::core::{G, System, Unique, entity::Entity};

pub struct GravitySystem;

impl System for GravitySystem {
    fn before_tick(&mut self, args: &mut crate::core::BeforeTickArgs) {
        // SAFETY: As long as only 1 thread has access to entities, and this function does not
        // insert, remove, or re-allocate the backing entities Vec this operation is safe.
        // This is to avoid performing an initial iteration to calculate new accelerations,
        // Then doing another iteration with iter_mut to update them.
        // If multithreading is introduced, this may have to be updated.
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
                if accel.x.is_nan() {
                    accel.x = 0.0;
                }
                if accel.y.is_nan() {
                    accel.y = 0.0;
                }
                if accel.z.is_nan() {
                    accel.z = 0.0;
                }
                (*a.cast_mut()).acceleration = accel;
            }
        }
    }
}
