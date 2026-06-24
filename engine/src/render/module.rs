// Meshes, Vertices, Instances, Transforms, Uniforms, Textures, Shaders, Rendering
//
// ORGANIZATION:
//
// Each "RenderModule" deals with a unique combination of:
//  - Vertex layout
//  - Instance layout (or none)
//  - Uniforms (cameras, lights, textures, data which is constant across all vertices/instances)
//  - Vertex shader
//  - Fragment shader
//  - Render Pipeline (draw order, face culling options, render configuration)

use std::{collections::HashMap, hash::Hash, num::NonZero};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BindGroupLayout, ColorTargetState, DepthStencilState, Device, FragmentState,
    MultisampleState, PipelineCache, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PrimitiveState, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, VertexBufferLayout, VertexState,
};

use crate::{
    core::{Instanced, Meshed, Textured, Unique},
    render::{
        GLOBAL_INDEX_FORMAT, GlobalIndexType,
        storage::{
            instance::InstanceStorage,
            mesh::{MeshStorage, MeshStorageError},
            textures::TextureStorage,
        },
    },
};

// Utility data types

pub struct VertexSpec {
    pub vertex_layout: VertexBufferLayout<'static>,
    pub instance_layout: VertexBufferLayout<'static>,
}

pub struct ShaderSpec {
    pub shader: String,
    pub vertex_shader_name: String,
    pub fragment_shader_name: String,
}

/// Render pipeline configuration options that need to be specified manually in
/// InstancedRenderModule::new.
pub struct RenderPipelineSpec<'a> {
    pub fragment_color_target_state: Option<ColorTargetState>,
    pub primitive: PrimitiveState,
    pub depth_stencil: Option<DepthStencilState>,
    pub multisample: MultisampleState,
    pub multiview_mask: Option<NonZero<u32>>,
    pub cache: Option<&'a PipelineCache>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TexturedInstanceKey {
    mesh_id: u64,
    texture_id: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
struct InstanceKey {
    mesh_id: u64,
}

// TODO: Find better solution. Maybe just create another module type
enum Instances<I> {
    Textured(HashMap<TexturedInstanceKey, InstanceStorage<I>>),
    NotTextured(HashMap<InstanceKey, InstanceStorage<I>>),
}

impl<I> Instances<I> {
    fn update_gpu(&mut self, device: &Device, queue: &Queue)
    where
        I: Pod + Zeroable + Clone + Copy + std::fmt::Debug,
    {
        match self {
            Instances::Textured(instances) => {
                for (_id, instance) in instances.iter_mut() {
                    instance.update_gpu(queue, device);
                }
            }
            Instances::NotTextured(instances) => {
                for (_id, instance) in instances.iter_mut() {
                    instance.update_gpu(queue, device);
                }
            }
        }
    }
}

/// Expects that the instance data comes after the vertex data in the shader.
///
/// Main data type for managing instanced geometry.
/// Contains relevant meshes, textures, buffers, etc.
///
/// The reason for this separation is that some mesh/instance data may need to be handled in a special manner,
/// in a different shader, with different uniforms.
pub struct InstancedRenderModule<V, I> {
    meshes: MeshStorage<V>,
    render_pipeline: RenderPipeline,
    instances: Instances<I>,
}

impl<V, I> InstancedRenderModule<V, I> {
    fn new_render_pipeline(
        device: &Device,
        debug_name: Option<&str>,
        vertex_spec: &VertexSpec,
        shader_spec: &ShaderSpec,
        bind_group_layouts: &[Option<&BindGroupLayout>],
        pipeline_spec: &RenderPipelineSpec,
    ) -> Result<RenderPipeline, std::io::Error> {
        let shader = shader_spec.shader.clone();

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(shader.into()),
        });
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: debug_name
                .map(|n| n.to_owned() + " Render Pipeline Layout")
                .as_deref(),
            bind_group_layouts,
            immediate_size: 0,
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some(&shader_spec.vertex_shader_name),
                buffers: &[
                    vertex_spec.vertex_layout.clone(),
                    vertex_spec.instance_layout.clone(),
                ],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some(&shader_spec.fragment_shader_name),
                targets: &[pipeline_spec.fragment_color_target_state.clone()],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: pipeline_spec.primitive,
            depth_stencil: pipeline_spec.depth_stencil.clone(),
            multisample: pipeline_spec.multisample,
            cache: pipeline_spec.cache,
            multiview_mask: pipeline_spec.multiview_mask,
        });

        Ok(render_pipeline)
    }
}

impl<V, I> InstancedRenderModule<V, I>
where
    V: Pod + Zeroable + Clone + Copy + std::fmt::Debug,
    I: Pod + Zeroable + Clone + Copy + std::fmt::Debug,
{
    pub fn new(
        device: &Device,
        debug_name: Option<&str>,
        vertex_spec: &VertexSpec,
        shader_spec: &ShaderSpec,
        bind_group_layouts: &[Option<&BindGroupLayout>],
        pipeline_spec: &RenderPipelineSpec,
    ) -> Result<Self, std::io::Error> {
        let render_pipeline = Self::new_render_pipeline(
            device,
            debug_name,
            vertex_spec,
            shader_spec,
            bind_group_layouts,
            pipeline_spec,
        )?;

        Ok(Self {
            meshes: MeshStorage::new(device),
            render_pipeline,
            instances: Instances::NotTextured(HashMap::new()),
        })
    }

    pub fn new_textured(
        device: &Device,
        debug_name: Option<&str>,
        vertex_spec: &VertexSpec,
        shader_spec: &ShaderSpec,
        bind_group_layouts: &[Option<&BindGroupLayout>],
        pipeline_spec: &RenderPipelineSpec,
    ) -> Result<Self, std::io::Error> {
        let render_pipeline = Self::new_render_pipeline(
            device,
            debug_name,
            vertex_spec,
            shader_spec,
            bind_group_layouts,
            pipeline_spec,
        )?;

        Ok(Self {
            meshes: MeshStorage::new(device),
            render_pipeline,
            instances: Instances::Textured(HashMap::new()),
        })
    }

    pub fn add_mesh(
        &mut self,
        device: &Device,
        vertices: &[V],
        indices: &[GlobalIndexType],
    ) -> Result<u64, MeshStorageError> {
        let mesh_id = self.meshes.add_mesh(vertices, indices)?;
        match &mut self.instances {
            Instances::NotTextured(map) => {
                map.insert(InstanceKey { mesh_id }, InstanceStorage::new(device));
            }
            Instances::Textured(map) => {
                map.insert(
                    TexturedInstanceKey {
                        mesh_id,
                        texture_id: 0,
                    },
                    InstanceStorage::new(device),
                );
            }
        }

        Ok(mesh_id)
    }

    pub fn upsert_instances<E>(
        &mut self,
        // TODO: Allow for adding of static instances which dont need an ID and never get referenced.
        // InstanceStorage will need to manage static and dynamic instances separately somehow.
        entities: &[E],
    ) -> Result<(), String>
    where
        E: Instanced<I> + Meshed<u64> + Unique<u64>,
    {
        for entity in entities {
            let entity_id = entity.id();
            let mesh_id = entity.mesh_id();

            let instances = match &mut self.instances {
                Instances::NotTextured(instances) => instances,
                _ => unreachable!(),
            };
            instances
                .get_mut(&InstanceKey { mesh_id: *mesh_id })
                .unwrap()
                .upsert_instance(entity_id, entity.instance());
        }
        Ok(())
    }

    pub fn upsert_instances_textured<E>(
        &mut self,
        // TODO: Allow for adding of static instances which dont need an ID and never get referenced.
        // InstanceStorage will need to manage static and dynamic instances separately somehow.
        entities: &[E],
    ) -> Result<(), String>
    where
        E: Instanced<I> + Meshed<u64> + Unique<u64> + Textured<u64>,
    {
        for entity in entities {
            let entity_id = entity.id();
            let mesh_id = entity.mesh_id();
            let texture_id = entity.texture_id();
            let instances = match &mut self.instances {
                Instances::Textured(instances) => instances,
                _ => unreachable!(),
            };
            instances
                .get_mut(&TexturedInstanceKey {
                    mesh_id: *mesh_id,
                    texture_id: 0,
                })
                .unwrap()
                .upsert_instance(entity_id, entity.instance());
        }

        Ok(())
    }

    pub fn draw_all(&self, render_pass: &mut RenderPass, bind_groups: &[&BindGroup]) {
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_vertex_buffer(0, self.meshes.vertex_slice(..));
        render_pass.set_index_buffer(self.meshes.index_slice(..), GLOBAL_INDEX_FORMAT);
        for (i, bg) in bind_groups.iter().enumerate() {
            let idx = if i >= 1 { i + 1 } else { i };
            render_pass.set_bind_group(idx as u32, Some(*bg), &[]);
        }

        let instances = match &self.instances {
            Instances::NotTextured(instances) => instances,
            _ => unreachable!(),
        };

        for (key, storage) in instances.iter() {
            let mesh_id = &key.mesh_id;
            if !storage.is_empty() {
                render_pass.set_vertex_buffer(1, storage.slice());
                let (start, end) = self.meshes.get_mesh_index_bounds(mesh_id).unwrap();
                render_pass.draw_indexed(start as u32..end as u32, 0, 0..storage.len() as u32);
            }
        }
    }

    pub fn draw_all_textured(
        &self,
        render_pass: &mut RenderPass,
        textures: &TextureStorage,
        texture_bind_group_index: u32,
        bind_groups: &[&BindGroup],
    ) {
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_vertex_buffer(0, self.meshes.vertex_slice(..));
        render_pass.set_index_buffer(self.meshes.index_slice(..), GLOBAL_INDEX_FORMAT);
        for (i, bg) in bind_groups.iter().enumerate() {
            let idx = if i as u32 >= texture_bind_group_index {
                i + 1
            } else {
                i
            };
            render_pass.set_bind_group(idx as u32, Some(*bg), &[]);
        }
        let instances = match &self.instances {
            Instances::Textured(instances) => instances,
            _ => unreachable!(),
        };

        for (key, storage) in instances.iter() {
            let texture_id = &key.texture_id;
            let mesh_id = &key.mesh_id;
            if !storage.is_empty() {
                render_pass.set_vertex_buffer(1, storage.slice());
                render_pass.set_bind_group(
                    texture_bind_group_index,
                    Some(&textures.get(texture_id).unwrap().3),
                    &[],
                );
                let (start, end) = self.meshes.get_mesh_index_bounds(mesh_id).unwrap();
                render_pass.draw_indexed(start as u32..end as u32, 0, 0..storage.len() as u32);
            }
        }
    }

    pub fn update_gpu(&mut self, device: &Device, queue: &Queue) {
        self.instances.update_gpu(device, queue);
    }
}

#[allow(unused_imports)]
mod tests {
    use assertables::assert_abs_diff_lt_x;

    #[test]
    fn cast_slice_equivalence() {
        let data = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ];
        let direct_cast: &[u8] = bytemuck::cast_slice(&data);

        let mut separate_casts: Vec<u8> = vec![];
        separate_casts.extend(bytemuck::cast_slice::<f32, u8>(&data[0]));
        separate_casts.extend(bytemuck::cast_slice::<f32, u8>(&data[1]));
        separate_casts.extend(bytemuck::cast_slice::<f32, u8>(&data[2]));
        separate_casts.extend(bytemuck::cast_slice::<f32, u8>(&data[3]));

        assert!(direct_cast.len() == separate_casts.len());

        for i in 0..direct_cast.len() {
            assert_eq!(direct_cast[i], separate_casts[i]);
        }
    }
}
