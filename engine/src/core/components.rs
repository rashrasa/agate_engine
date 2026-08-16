use std::ops::{Deref, DerefMut};

use crate::{Id, Vector3};

pub struct Position {
    pub p: Vector3,
}

pub struct Render {
    pub mesh: Id,
    pub texture: Id,
}

pub struct Components {}

pub struct Component<T> {
    pub(crate) entity: Id,
    pub(crate) inner: T,
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
