use std::ops::{Deref, DerefMut};

use crate::{Float, GLOBAL_INTEGRATOR, Ident, Vector3, render::camera::NoClipCamera};

pub struct Entity {
    id: Ident,
}

#[derive(Default, Debug, Clone)]
pub struct Position {
    pub p: Vector3,
}

#[derive(Default, Debug, Clone)]
pub struct Dynamic {
    pub vel: Vector3,
    pub accel: Vector3,
}

impl Dynamic {
    pub fn tick(&mut self, dt: Float) {
        crate::GLOBAL_INTEGRATOR.integrate(dt, &mut self.vel, &self.accel);
    }
}

#[derive(Default, Debug, Clone)]
pub struct CollisionBox {
    pub start: Vector3,
    pub size: Vector3,
}
impl CollisionBox {
    pub const ZERO: Self = CollisionBox {
        start: Vector3::new(0.0, 0.0, 0.0),
        size: Vector3::new(0.0, 0.0, 0.0),
    };
    pub const fn new(start: Vector3, size: Vector3) -> Self {
        Self { start, size }
    }
    pub fn intersects(&self, other: &Self) -> Option<Vector3> {
        let f = (other.start + other.size) / 2.0 - (self.start + self.size) / 2.0;

        let min_size = self.size.inf(&other.size);

        if f.x.abs() > min_size.x || f.y.abs() > min_size.y || f.z.abs() > min_size.z {
            return None;
        }

        Some(f)
    }
}

pub struct Camera {
    pub camera: NoClipCamera,
}

impl Camera {
    fn update_position(&mut self, position: &Vector3) {
        self.camera.set_position(position);
    }
}

pub struct InputController {}

pub struct Render {
    mesh_id: Ident,
    texture_id: Ident,
}

/// Elastic collisions have CollisionResponse::Inelastic(1.0).
/// Inelastic takes any value. Values exceeding 1.0 will result in
/// energy magically being added to the system. Values below 0.0 will
/// be clamped to 0.0.
#[derive(Debug)]
pub enum CollisionResponse {
    Immovable,
    Inelastic(f32),
}

#[derive(Default, Debug)]
pub struct Runtime {
    id: IdStorage,
    components: ComponentStorage,
}

#[derive(Default, Debug)]
pub struct ComponentStorage {
    positions: Vec<Component<Position>>,
    dynamics: Vec<Component<Dynamic>>,
    collisions: Vec<Component<CollisionBox>>,
}

pub struct EntityDescription {
    pub position: Option<Position>,
    pub dynamic: Option<Dynamic>,
    pub collision: Option<CollisionBox>,
    pub camera: Option<Camera>,
    pub render: Option<Render>,
}

impl Runtime {
    pub fn add_entity(&mut self, desc: &EntityDescription) -> Entity {
        let id = self.id.id();
        if let Some(inner) = &desc.position {
            self.components.positions.push(Component {
                entity: id,
                inner: inner.clone(),
            });
        }
        if let Some(inner) = &desc.dynamic {
            self.components.dynamics.push(Component {
                entity: id,
                inner: inner.clone(),
            });
        }
        if let Some(inner) = &desc.collision {
            self.components.collisions.push(Component {
                entity: id,
                inner: inner.clone(),
            });
        }
        Entity { id }
    }

    // Manually implemented systems
    pub fn tick(&mut self, dt: Float) {
        for d in self.components.dynamics.iter_mut() {
            d.tick(dt);
        }

        apply2(
            self.components.dynamics.iter(),
            self.components.positions.iter_mut(),
            |v, p| {
                GLOBAL_INTEGRATOR.integrate(dt, &mut p.p, &v.vel);
            },
        );
    }
}

fn apply2<'a, T, S, Src, Dst, F>(src: Src, dst: Dst, mut f: F)
where
    Src: Iterator<Item = &'a Component<T>>,
    Dst: Iterator<Item = &'a mut Component<S>>,
    F: FnMut(&T, &mut S),
    T: 'static,
    S: 'static,
{
    let mut src = src.peekable();
    let mut dst = dst.peekable();

    while let Some(d) = dst.peek() {
        if let Some(s) = src.peek() {
            if s.entity == d.entity {
                // advance both and use values
                f(src.next().unwrap(), dst.next().unwrap());
            } else if s.entity > d.entity {
                // src exceeded dst without finding match. discard dst
                dst.next();
            } else if s.entity < d.entity {
                // dst exceeded src without finding match. discard src
                src.next();
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct IdStorage {
    next: Ident,
}

impl IdStorage {
    pub fn id(&mut self) -> Ident {
        let v = self.next;
        self.next += 1;
        v
    }
}

#[derive(Debug)]
pub struct Component<T> {
    entity: Ident,
    inner: T,
}

impl<T> Deref for Component<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Component<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
