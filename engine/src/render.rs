pub mod vec_buf;

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::{
    Adapter, BufferUsages, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, Limits, MemoryHints, PowerPreference, Queue, RequestAdapterOptions,
    Surface, Trace, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};
use winit::window::Window;

use crate::{Float, VecBuf, WgpuIndex};

pub struct Renderer {
    instance: Instance,
    device: Arc<Device>,
    queue: Arc<Queue>,
    adapter: Adapter,

    target: Option<RenderTarget>,

    vertex: VecBuf<Vertex>,
    index: VecBuf<WgpuIndex>,
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
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let vertex = VecBuf::with_capacity(&device, &queue, 1024, BufferUsages::VERTEX);
        let index = VecBuf::with_capacity(&device, &queue, 1024, BufferUsages::INDEX);

        let vertex_layout = VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                },
            ],
        };

        Ok(Self {
            instance,
            device,
            queue,
            vertex,
            index,
            adapter,
        })
    }

    pub fn render(&self) {}
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct Vertex {
    pub position: [Float; 3],
    pub color: [Float; 3],
    pub _padding: [Float; 2],
}

struct RenderTarget {
    window: Arc<Window>,
    surface: Surface<'static>,
}
