use std::{marker::PhantomData, sync::Arc};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, COPY_BUFFER_ALIGNMENT, CommandEncoderDescriptor,
    Device, Queue,
};

/// T must have a size which is a multiple of 4.
pub struct VecBuf<T> {
    len: u64,
    cap: u64,
    buffer: Buffer,
    usage: BufferUsages,

    // external
    device: Arc<Device>,
    queue: Arc<Queue>,

    // size = len * size_of::<T>()
    _marker: PhantomData<T>,
}
impl<T> VecBuf<T>
where
    T: Clone + Copy + Pod + Zeroable,
{
    const RESIZE_FACTOR: f64 = 1.5;

    /// Creates a WebGPU buffer with a specific capacity.
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::Arc;
    /// use wgpu::{Device, BufferUsages};
    /// use agate_engine::VecBuf;
    ///
    /// #[repr(C)]
    /// struct Vertex {
    ///     pub position: [f32; 3],
    ///     pub uv: [f32; 2],
    ///     pub normal: [f32; 3],
    /// }
    ///
    /// fn helper(device: &Arc<Device>) {
    ///     let vertex_buffer: VecBuf<Vertex> = VecBuf::with_capacity(device, 1000, BufferUsages::VERTEX | BufferUsages::COPY_DST);
    /// }
    ///
    /// ```
    pub fn with_capacity(
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        cap: u64,
        usage: BufferUsages,
    ) -> Self {
        // debug_assert since wgpu does its own verification
        debug_assert!(cap > 0, "buffer initial capacity must be greater than 0");

        assert!(
            size_of::<T>().is_multiple_of(4),
            "buffer inner T must have a size which is a multiple of 4"
        );

        let len = 0;
        let buffer = Self::create_buffer(device, cap, usage);
        let device = Arc::clone(device);
        let queue = Arc::clone(queue);

        Self {
            len,
            cap,
            buffer,
            usage,
            device,
            queue,
            _marker: PhantomData,
        }
    }

    /// Resizes the underlying buffer.
    ///
    /// If the length is larger than the new capacity,
    /// those elements will be lost.
    pub fn resize(&mut self, cap: u64) {
        let new_len = self.len.min(cap);
        let buffer = Self::create_buffer(&self.device, cap, self.usage);
        let commands = {
            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor::default());
            encoder.copy_buffer_to_buffer(
                &self.buffer,
                0,
                &buffer,
                0,
                new_len * size_of::<T>() as u64,
            );

            encoder.finish()
        };

        // TODO: Check if needs to be called exactly here
        self.queue.submit([commands]);

        self.buffer.destroy();
        self.buffer = buffer;
        self.cap = cap;
        self.len = new_len;
    }

    pub fn push(&mut self, value: T) {
        self.extend(&[value]);
    }

    pub fn extend(&mut self, values: &[T]) {
        let n = values.len() as u64;
        if self.len + n > self.cap {
            self.expand_capacity_min(n);
        }
        self.queue.write_buffer(
            &self.buffer,
            self.len * size_of::<T>() as u64,
            bytemuck::cast_slice(values),
        );

        // TODO: Check if needs to be called exactly here
        self.queue.submit([]);
        self.len += n;
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Guarantees that at least [`additional_space_needed`] additional space
    /// is created.
    fn expand_capacity_min(&mut self, additional_space_needed: u64) {
        let new_capacity = (self.cap + additional_space_needed)
            .max((self.cap as f64 * Self::RESIZE_FACTOR) as u64)
            .next_multiple_of(COPY_BUFFER_ALIGNMENT);
        self.resize(new_capacity);
    }

    /// Creates a buffer for the purposes of and usages by VecBuf. This adds BufferUsages::COPY_SRC and BufferUsages::COPY_DST.
    fn create_buffer(device: &Device, size: u64, usage: BufferUsages) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: None,
            size: size * size_of::<T>() as u64,
            usage: usage | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }
}

#[cfg(test)]
mod tests {

    use pollster::FutureExt;
    use wgpu::{
        COPY_BUFFER_ALIGNMENT, DeviceDescriptor, Features, Instance, MapMode, RequestAdapterOptions,
    };

    use super::*;

    #[derive(Clone, Copy, Pod, Zeroable)]
    #[repr(C)]
    struct Vertex {
        vecf3: [f32; 3],
        vecu2: [u32; 2],
        veci3: [i32; 3],
    }

    const VERTEX_SIZE: usize = size_of::<Vertex>();

    const EXAMPLE_VERTEX: Vertex = Vertex {
        vecf3: [0.0, 1.0, 2.0],
        vecu2: [3, 4],
        veci3: [-5, -6, -7],
    };

    #[test]
    fn it_works() {
        let instance = Instance::default();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .block_on()
            .unwrap();
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::default(),
                ..Default::default()
            })
            .block_on()
            .unwrap();

        let device = Arc::new(device);
        let queue = Arc::new(queue);
        let mut vec = VecBuf::with_capacity(&device, &queue, 8, BufferUsages::empty());

        assert!(vec.is_empty());
        vec.push(EXAMPLE_VERTEX);
        assert!(!vec.is_empty());

        assert_eq!(1, vec.len());

        let data: Vec<_> = (0..7).map(|_| EXAMPLE_VERTEX).collect();
        vec.extend(&data);

        assert_eq!(8, vec.len());

        vec.push(EXAMPLE_VERTEX);

        assert_eq!(9, vec.len());

        let staging_size = (vec.len * VERTEX_SIZE as u64).next_multiple_of(COPY_BUFFER_ALIGNMENT);

        let staging = device.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: staging_size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
        encoder.copy_buffer_to_buffer(&vec.buffer, 0, &staging, 0, staging_size);
        encoder.map_buffer_on_submit(&staging, MapMode::Read, 0..staging_size, |r| {
            if let Err(e) = r {
                eprintln!("{}", e);
                panic!();
            }
        });
        let index = queue.submit([encoder.finish()]);

        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(index),
                timeout: None,
            })
            .unwrap();

        let mapped = staging.get_mapped_range(0..staging_size);

        let data: &[Vertex] = bytemuck::cast_slice(&mapped[0..(VERTEX_SIZE * vec.len as usize)]);

        assert_eq!(9, data.len());
        let first = data.first().unwrap();

        assert_eq!(EXAMPLE_VERTEX.vecf3, first.vecf3);
        assert_eq!(EXAMPLE_VERTEX.vecu2, first.vecu2);
        assert_eq!(EXAMPLE_VERTEX.veci3, first.veci3);
    }
}
