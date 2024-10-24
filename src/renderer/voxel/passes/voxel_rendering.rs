use std::{mem::size_of, num::NonZeroU64};

use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBinding,
    BufferBindingType, BufferDescriptor, BufferUsages, ComputePass, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, Queue, ShaderStages,
    StorageTextureAccess, TextureViewDimension,
};
use winit::dpi::PhysicalSize;

use crate::renderer::voxel::{chunk::chunk::Chunk, octree::tree::Tree, textures::RenderTexture};

const LABEL: &'static str = "VoxelRendererPass";

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    texture_width: f32,
    texture_height: f32,
}

pub struct VoxelRendererPass<const CHUNK_SIZE: usize> {
    pipeline: ComputePipeline,
    group: BindGroup,
    texture_size: PhysicalSize<u32>,
    octree_buf: Buffer,
    voxel_buf: Buffer,
    voxel_buf_uniform: Buffer,
}

impl<const CHUNK_SIZE: usize> VoxelRendererPass<CHUNK_SIZE> {
    pub fn new(device: &Device, render_texture: &RenderTexture) -> Self {
        let uniforms = Uniforms {
            texture_width: render_texture.get_size().width as f32,
            texture_height: render_texture.get_size().height as f32,
        };

        let voxel_buf_uniform = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(LABEL),
            contents: cast_slice(&[uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let voxel_buf = device.create_buffer(&BufferDescriptor {
            label: Some(LABEL),
            size: (size_of::<u16>() * CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as BufferAddress,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        println!(
            "Size of the voxel buffer : {}byte(s) ({}Mo)",
            voxel_buf.size(),
            (voxel_buf.size() as f32) / 1024.0 / 1024.0
        );

        let octree_buf = device.create_buffer(&BufferDescriptor {
            label: Some(LABEL),
            size: Tree::estimated_size_aligned(CHUNK_SIZE as u32, 256) as BufferAddress,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        println!(
            "Size of the octree buffer : {}byte(s) (~{}Mo)",
            octree_buf.size(),
            ((octree_buf.size() as f32) / 1024.0 / 1024.0).ceil()
        );

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(LABEL),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: render_texture.get_format(),
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(LABEL),
            layout: &layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(render_texture.get_view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &voxel_buf_uniform,
                        offset: 0,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &voxel_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &octree_buf,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(LABEL),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let module = device.create_shader_module(include_wgsl!("../shaders/voxel_renderer.wgsl"));
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(LABEL),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            group,
            texture_size: render_texture.get_size(),
            octree_buf,
            voxel_buf,
            voxel_buf_uniform,
        }
    }

    pub fn set_chunk(&self, queue: &Queue, chunk: &Chunk<CHUNK_SIZE>) {
        let tree = chunk.get_tree();

        if let Some(mut view) = queue.write_buffer_with(
            &self.octree_buf,
            0,
            NonZeroU64::new(tree.raw_data().len() as u64).unwrap(),
        ) {
            view.copy_from_slice(tree.raw_data());
        }

        let voxels = chunk.get_raw_voxels();
        if let Some(mut view) = queue.write_buffer_with(
            &self.voxel_buf,
            0,
            NonZeroU64::new(voxels.len() as u64).unwrap(),
        ) {
            view.copy_from_slice(voxels);
        }
    }

    pub fn compute_with_pass(&self, mut pass: ComputePass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group, &[]);

        let w = (self.texture_size.width as f32 / 16.0).ceil() as u32;
        let h = (self.texture_size.height as f32 / 16.0).ceil() as u32;

        pass.dispatch_workgroups(w, h, 1);
    }
}
