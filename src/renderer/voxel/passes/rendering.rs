use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferAddress,
    BufferUsages, ColorTargetState, ColorWrites, Device, Face, FilterMode, FragmentState,
    FrontFace, MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPass, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
    SamplerDescriptor, ShaderStages, TextureFormat, TextureSampleType, TextureViewDimension,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use crate::renderer::voxel::textures::RenderTexture;

const LABEL: &'static str = "VoxelRenderingPass";

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct Vertex(f32, f32, f32, f32);

const FSQ_VERTICES: &[Vertex] = &[
    Vertex(-1.0, -1.0, 0.0, 0.0),
    Vertex(-1.0, 1.0, 0.0, 1.0),
    Vertex(1.0, 1.0, 1.0, 1.0),
    Vertex(-1.0, -1.0, 0.0, 0.0),
    Vertex(1.0, 1.0, 1.0, 1.0),
    Vertex(1.0, -1.0, 1.0, 0.0),
];

pub struct VoxelImageRenderingPass {
    pipeline: RenderPipeline,
    group: BindGroup,
    vertex_buf: Buffer,
}

impl VoxelImageRenderingPass {
    pub fn new(
        device: &Device,
        render_texture: &RenderTexture,
        present_format: TextureFormat,
    ) -> Self {
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(LABEL),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
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
                    resource: BindingResource::Sampler(&device.create_sampler(
                        &SamplerDescriptor {
                            label: Some(LABEL),
                            address_mode_u: AddressMode::Repeat,
                            address_mode_v: AddressMode::Repeat,
                            address_mode_w: AddressMode::Repeat,
                            mag_filter: FilterMode::Nearest,
                            min_filter: FilterMode::Nearest,
                            mipmap_filter: FilterMode::Nearest,
                            ..Default::default()
                        },
                    )),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(LABEL),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(include_wgsl!("../shaders/rendering.wgsl"));

        let vertex_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(LABEL),
            contents: cast_slice(&FSQ_VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("VoxelRenderingPass"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: (std::mem::size_of::<f32>() * 4) as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: (std::mem::size_of::<f32>() * 2) as BufferAddress,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: true,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: present_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            group,
            vertex_buf,
        }
    }

    pub fn draw_with_pass(&self, mut pass: RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.draw(0..6, 0..1);
    }
}
