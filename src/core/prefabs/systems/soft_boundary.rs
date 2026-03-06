use crate::core;

/// This is a "soft" boundary since it will only flip velocities when out of bounds
/// and no clipping to boundary is done.
pub struct SoftBoundarySystem {
    x: [f32; 2],
    y: [f32; 2],
    z: [f32; 2],
}

impl SoftBoundarySystem {
    pub fn new(mut x: [f32; 2], mut y: [f32; 2], mut z: [f32; 2]) -> Self {
        if x[0] > x[1] {
            x.swap(0, 1);
        }
        if y[0] > y[1] {
            y.swap(0, 1);
        }
        if z[0] > z[1] {
            z.swap(0, 1);
        }
        Self { x, y, z }
    }
}

impl core::System for SoftBoundarySystem {
    fn before_tick(&mut self, args: &mut core::BeforeTickArgs) {
        for entity in args.state.entities_mut() {
            let e_x = entity.translation.x;
            let e_y = entity.translation.y;
            let e_z = entity.translation.z;
            if e_x > self.x[1] && entity.velocity.x > 0.0
                || e_x < self.x[0] && entity.velocity.x < 0.0
            {
                entity.velocity.x = -entity.velocity.x;
            }
            if e_y > self.y[1] && entity.velocity.y > 0.0
                || e_y < self.y[0] && entity.velocity.y < 0.0
            {
                entity.velocity.y = -entity.velocity.y;
            }
            if e_z > self.z[1] && entity.velocity.z > 0.0
                || e_z < self.z[0] && entity.velocity.z < 0.0
            {
                entity.velocity.z = -entity.velocity.z;
            }
        }
    }
}
