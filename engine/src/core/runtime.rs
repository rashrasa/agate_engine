use crate::{
    Float, GLOBAL_INTEGRATOR,
    core::{
        components::{Component, Components, Entity, EntityDescription, Ids, apply2},
        resources::Resources,
    },
};

#[derive(Default, Debug)]
pub struct Runtime {
    id: Ids,
    components: Components,
    resources: Resources,
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
