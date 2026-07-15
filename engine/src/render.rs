pub mod vec_buf;

use wgpu::{
    Adapter, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, Limits, MemoryHints, PowerPreference, Queue, RequestAdapterOptions, Trace,
};

use crate::Float;

pub struct Renderer {
    instance: Instance,
    device: Device,
    queue: Queue,
    adapter: Adapter,
}

impl Renderer {
    pub async fn new() -> anyhow::Result<Self> {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Device"),
                required_features: !Features::all_webgpu_mask(),
                required_limits: Limits::defaults(),
                experimental_features: ExperimentalFeatures::disabled(),
                memory_hints: MemoryHints::Performance,
                trace: Trace::Off,
            })
            .await?;

        Ok(Self {
            instance,
            device,
            queue,
            adapter,
        })
    }
}

#[repr(C)]
pub struct Vertex {
    pub position: [Float; 3],
    pub uv: [Float; 2],
    pub normal: [Float; 3],
}
