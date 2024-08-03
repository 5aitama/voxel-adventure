use std::mem::size_of;

use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBinding,
    BufferBindingType, BufferDescriptor, BufferUsages, ComputePass, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineLayout, PipelineLayoutDescriptor, ShaderStages,
    StorageTextureAccess, TextureViewDimension,
};
use winit::dpi::PhysicalSize;

use crate::renderer::voxel::textures::RenderTexture;

const LABEL: &'static str = "VoxelRendererPass";

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    texture_width: f32,
    texture_height: f32,
}

pub struct VoxelRendererPass {
    pipeline: ComputePipeline,
    group: BindGroup,
    texture_size: PhysicalSize<u32>,
    voxel_buf: Buffer,
    voxel_buf_uniform: Buffer,
}

impl VoxelRendererPass {
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

        println!(
            "Size of the buffer : {}byte(s)",
            (size_of::<u32>() * 128 * 128 * 128)
        );
        let voxel_buf = device.create_buffer(&BufferDescriptor {
            label: Some(LABEL),
            size: (size_of::<u32>() * 128 * 128 * 128) as BufferAddress,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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
            voxel_buf,
            voxel_buf_uniform,
        }
    }

    pub fn compute_with_pass(&self, mut pass: ComputePass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group, &[]);

        let w = (self.texture_size.width as f32 / 8.0).ceil() as u32;
        let h = (self.texture_size.height as f32 / 8.0).ceil() as u32;

        pass.dispatch_workgroups(w, h, 1);
    }
}
