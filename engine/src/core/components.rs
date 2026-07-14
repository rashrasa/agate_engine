use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::{Float, GLOBAL_INTEGRATOR, Vector3};

pub struct Entity {
    id: u64,
}

#[derive(Default, Debug)]
pub struct Position {
    pub p: Vector3,
}

#[derive(Default, Debug)]
pub struct Velocity {
    pub v: Vector3,
}

#[derive(Default, Debug)]
pub struct Acceleration {
    pub a: Vector3,
}

#[derive(Default, Debug)]
pub struct Runtime {
    id: IdStorage,
    components: ComponentStorage,
}

#[derive(Debug)]
pub struct Component<T> {
    entity: u64,
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

#[derive(Default, Debug)]
pub struct ComponentStorage {
    p: Vec<Component<Position>>,
    v: Vec<Component<Velocity>>,
    a: Vec<Component<Acceleration>>,
}

impl Runtime {
    pub fn add_entity(
        &mut self,
        position: Option<Position>,
        velocity: Option<Velocity>,
        acceleration: Option<Acceleration>,
    ) -> Entity {
        let id = self.id.id();
        if let Some(inner) = position {
            self.components.p.push(Component { entity: id, inner });
        }
        if let Some(inner) = velocity {
            self.components.v.push(Component { entity: id, inner });
        }
        if let Some(inner) = acceleration {
            self.components.a.push(Component { entity: id, inner });
        }
        Entity { id }
    }

    // Manually implemented systems
    pub fn tick(&mut self, dt: Float) {
        apply2(
            self.components.a.iter(),
            self.components.v.iter_mut(),
            |a, v| {
                GLOBAL_INTEGRATOR.integrate::<2>(dt, &mut v.v, &a.a);
            },
        );

        apply2(
            self.components.v.iter(),
            self.components.p.iter_mut(),
            |v, p| {
                GLOBAL_INTEGRATOR.integrate::<2>(dt, &mut p.p, &v.v);
            },
        );
    }
}

impl Display for Runtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();

        let mut i = self.components.p.iter().peekable();
        let mut j = self.components.v.iter().peekable();
        let mut k = self.components.a.iter().peekable();
        for id in 0..self.id.next {
            string += &format!("({id}");

            if let Some(p) = i.peek().cloned() {
                if p.entity == id {
                    i.next();
                    string += &format!("({:.2}, {:.2}, {:.2}), ", p.p.x, p.p.y, p.p.z);
                } else {
                    string += "None, ";
                };
            }

            if let Some(v) = j.peek().cloned() {
                if v.entity == id {
                    j.next();
                    string += &format!("({:.2}, {:.2}, {:.2}), ", v.v.x, v.v.y, v.v.z);
                } else {
                    string += "None, ";
                };
            }

            if let Some(a) = k.peek().cloned() {
                if a.entity == id {
                    k.next();
                    string += &format!("({:.2}, {:.2}, {:.2}), ", a.a.x, a.a.y, a.a.z);
                } else {
                    string += "None";
                };
            }

            string += ")\n";
        }
        write!(f, "{}", string)
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
    next: u64,
}

impl IdStorage {
    pub fn id(&mut self) -> u64 {
        let v = self.next;
        self.next += 1;
        v
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Write, time::Duration};

    use super::*;

    #[test]
    fn it_works() {
        let mut rt = Runtime::default();

        rt.add_entity(None, None, None);
        rt.add_entity(
            Some(Default::default()),
            Some(Default::default()),
            Some(Default::default()),
        );

        rt.add_entity(
            Some(Default::default()),
            Some(Velocity {
                v: Vector3::new(0.0, 1.0, 0.0),
            }),
            None,
        );

        rt.add_entity(
            Some(Default::default()),
            Some(Default::default()),
            Some(Acceleration {
                a: Vector3::new(0.0, 1.0, 0.0),
            }),
        );

        for _ in 0..100 {
            rt.tick(0.0167);
            println!("{}[2J{}", 27 as char, rt);
            std::io::stdout().flush().unwrap();
            std::thread::sleep(Duration::from_millis(16));
        }
    }
}
