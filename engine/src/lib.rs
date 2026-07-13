/*  agate_engine

   Notes:
       - Tick and Render tied to WindowEvent::RedrawRequested event from the main window
       - Entities and World live in app::ActiveState
       - lifecycle::System's act on state, input, camera through lifecycle hooks
    Issues:
       - Inter-System communication is not currently possible (merging systems is necessary)
       - Textures do not work, the first texture added is the one applied to all objects currently
*/

use nalgebra::{ArrayStorage, Const};

pub mod app;
pub mod core;
pub mod input;
pub mod prefabs;
pub mod render;

pub fn init_logging(level: log::LevelFilter) {
    env_logger::builder()
        .filter_level(level)
        .target(env_logger::Target::Stdout)
        .init();
}

pub type Vector<const N: usize> =
    nalgebra::Matrix<Float, Const<3>, Const<1>, ArrayStorage<Float, 3, 1>>;

pub type Vector3 = Vector<3>;

const GLOBAL_INTEGRATOR: Integrator = Integrator::RK4;

#[derive(Clone, Debug)]
pub enum Integrator {
    Euler,
    RK4,
}

impl Integrator {
    pub fn integrate<const N: usize>(&self, dt: Float, y: &mut Vector<N>, dy: &Vector<N>) {
        match self {
            Integrator::Euler => {
                *y += dy * dt;
            }
            Integrator::RK4 => {
                let k1 = dy;
                let k2 = dy + dt / 2.0 * k1;
                let k3 = dy + dt / 2.0 * k2;
                let k4 = dy + dt * k3;
                *y += (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0 * dt
            }
        }
    }
}

pub type Float = f32;
