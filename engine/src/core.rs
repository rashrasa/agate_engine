/*
   Main set of modules containing all data types, logic, systems, constants, and traits, which are concerned with
   representing and modelling app/engine behaviour, all of which is windowing- and rendering-agnostic.
*/

use std::hash::Hash;

pub mod components;
pub mod entity;
mod lifecycle;
pub mod world;

// Exports
pub use lifecycle::{
    AfterRenderArgs, AfterTickArgs, BeforeInputArgs, BeforeRenderArgs, BeforeStartArgs,
    BeforeTickArgs, DisposeArgs, HandleInputArgs, HandleTickArgs, System,
};

use crate::{Integrator, render::storage::textures::MipLevel};

pub const G: f64 = 6.6743e-11;

pub const GLOBAL_INTEGRATOR: Integrator = Integrator::RK4;

/// Number of vertices per chunk per side (regardless of chunk size). Higher numbers increase performance demands.
pub const CHUNK_RESOLUTION: usize = 4;

/// Units of distance covered by a chunk. Lower numbers increase performance demands.
pub const CHUNK_SIZE: f32 = 16.0;

pub const CAMERA_SPEED: f32 = 20.0;
pub const CAMERA_USES_PITCH: bool = true;
pub const RENDER_DISTANCE: f32 = 16.0;

pub const MUTE: bool = false;

// must be in decreasing quality
pub const MIPMAP_LEVELS: [MipLevel; 1] = [MipLevel::Square(2048)];

pub trait Instanced<I> {
    fn instance(&self) -> I;
}

pub trait Unique<U: Hash + Eq> {
    fn id(&self) -> &U;
}

pub trait Meshed<U: Hash + Eq> {
    fn mesh_id(&self) -> &U;
}

pub trait Textured<U: Hash + Eq> {
    fn texture_id(&self) -> &U;
}
