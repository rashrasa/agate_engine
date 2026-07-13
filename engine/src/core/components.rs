use crate::Vector3;

#[derive(Default)]
pub struct Position {
    pub p: Vector3,
}

#[derive(Default)]
pub struct Velocity {
    pub v: Vector3,
}

#[derive(Default)]
pub struct Acceleration {
    pub a: Vector3,
}
