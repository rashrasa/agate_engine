use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use log::info;
use nalgebra::{ArrayStorage, Const, Dim, ToConst, Vector3};
use serde_json::{Number, Value};

use crate::core;

pub struct MetricsSystem {
    window_start: Instant,
    window: Duration,

    // cpu
    start_tick: Instant,
    window_ticking: Duration,
    n_ticks: u64,

    // gpu
    start_render: Instant,
    window_rendering: Duration,
    n_renders: u64,

    gui_data: Option<Arc<RwLock<HashMap<String, Value>>>>,
}

impl MetricsSystem {
    pub fn new(window: Duration) -> Self {
        Self {
            window_start: Instant::now(),
            window,

            start_tick: Instant::now(),
            window_ticking: Duration::ZERO,
            n_ticks: 0,

            start_render: Instant::now(),
            window_rendering: Duration::ZERO,
            n_renders: 0,

            gui_data: None,
        }
    }
}

impl core::System for MetricsSystem {
    fn before_start(&mut self, args: &mut core::BeforeStartArgs) {
        self.window_start = Instant::now();

        self.start_tick = Instant::now();
        self.window_ticking = Duration::ZERO;
        self.n_ticks = 0;

        self.start_render = Instant::now();
        self.window_rendering = Duration::ZERO;
        self.n_renders = 0;

        self.gui_data = Some(args.renderer.gui_data());
    }

    fn before_input(&mut self, _args: &mut core::BeforeInputArgs) {
        self.start_tick = Instant::now();
    }

    fn after_tick(&mut self, _args: &mut core::AfterTickArgs) {
        self.window_ticking += self.start_tick.elapsed();
        self.n_ticks += 1;
    }

    fn before_render(&mut self, _args: &mut core::BeforeRenderArgs) {
        self.start_render = Instant::now();
    }
    fn after_render(&mut self, args: &mut core::AfterRenderArgs) {
        self.window_rendering += self.start_render.elapsed();
        self.n_renders += 1;

        // evaluate
        let window_time = self.window_start.elapsed();
        if window_time > self.window {
            let window_time = window_time.as_secs_f64();

            let cpu_time = (self.window_ticking.as_secs_f64() / self.n_ticks as f64) * 1000.0;
            let gpu_time = (self.window_rendering.as_secs_f64() / self.n_renders as f64) * 1000.0;
            let fps = self.n_renders as f64 / window_time;

            let anomalies = args
                .state
                .entities()
                .iter()
                .filter(|e| has_nan(&e.acceleration))
                .count();
            info!(
                "\nCPU/IO: {:.2}ms\nRender: {:.2}ms\nFPS: {:.2}\nEntities with NaN accelerations: {}",
                cpu_time, gpu_time, fps, anomalies
            );

            if let Some(gui_data) = &self.gui_data {
                if let Ok(mut gui_data) = gui_data.write() {
                    gui_data.insert(
                        "cpu".into(),
                        Value::Number(Number::from_f64(cpu_time).unwrap_or(Number::from(0))),
                    );

                    gui_data.insert(
                        "gpu".into(),
                        Value::Number(Number::from_f64(gpu_time).unwrap_or(Number::from(0))),
                    );

                    gui_data.insert(
                        "fps".into(),
                        Value::Number(Number::from_f64(fps).unwrap_or(Number::from(0))),
                    );

                    gui_data.insert("anomalies".into(), Value::Number(Number::from(anomalies)));
                }
            }

            self.window_rendering = Duration::ZERO;
            self.window_ticking = Duration::ZERO;
            self.n_renders = 0;
            self.n_ticks = 0;
            self.window_start = Instant::now();
        }
    }
}

fn has_nan<const R: usize, const C: usize>(
    data: &nalgebra::Matrix<f32, Const<R>, Const<C>, ArrayStorage<f32, R, C>>,
) -> bool {
    for a in data.iter() {
        if a.is_nan() {
            return true;
        }
    }

    return false;
}
