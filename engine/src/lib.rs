pub mod core;
pub mod render;

pub use render::vec_buf::VecBuf;

use nalgebra::{ArrayStorage, Const};

pub const GLOBAL_INTEGRATOR: Integrator = Integrator::RK4;

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
pub type Id = u64;
pub type Index = u32;

pub type Matrix<const N: usize, const M: usize> =
    nalgebra::Matrix<Float, Const<N>, Const<M>, ArrayStorage<Float, N, M>>;
pub type Vector<const N: usize> = Matrix<N, 1>;
pub type Vector3 = Vector<3>;
