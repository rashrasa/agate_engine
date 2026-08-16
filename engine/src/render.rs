pub mod vec_buf;

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::{
    Adapter, BufferUsages, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, Limits, MemoryHints, PowerPreference, PresentMode, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages, Trace, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexStepMode,
};
use winit::window::Window;

use crate::{Float, Index, VecBuf};

pub struct Renderer {
    instance: Instance,
    device: Device,
    queue: Queue,
    adapter: Adapter,

    target: Option<RenderTarget>,

    vertex: VecBuf<Vertex>,
    index: VecBuf<Index>,
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
            target: None,
        })
    }

    /// Sets the render target to [`window`].
    ///
    /// Re-creates the render pipeline and other wgpu data.
    pub fn set_window(&mut self, window: Arc<Window>) -> anyhow::Result<()> {
        let surface: Surface<'static> = self.instance.create_surface(Arc::clone(&window))?;
        let caps = surface.get_capabilities(&self.adapter);
        let size = window.inner_size();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(caps.formats[0]),
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Immediate,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&self.device, &config);

        let target = RenderTarget {
            window,
            surface,
            config,
        };
        self.target = Some(target);

        Ok(())
    }

    pub fn handle_resize(&mut self, width: u32, height: u32) {
        if let Some(target) = &mut self.target {
            target.config.width = width;
            target.config.height = height;
            target.surface.configure(&self.device, &target.config);
        }
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
    config: SurfaceConfiguration,
}
