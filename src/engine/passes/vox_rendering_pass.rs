use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BlendState,
    Buffer, BufferAddress, BufferUsages, ColorTargetState, ColorWrites, Device, Face, FilterMode,
    FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, SamplerDescriptor, TextureFormat,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

const LABEL: &str = "VoxelRenderingPass";

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct Vertex(f32, f32, f32, f32);

const FSQ_VERTICES: &[Vertex] = &[
    Vertex(-1.0, -1.0, 0.0, 0.0),
    Vertex(-1.0, 1.0, 0.0, 1.0),
    Vertex(1.0, 1.0, 1.0, 1.0),
    Vertex(-1.0, -1.0, 0.0, 0.0),
    Vertex(1.0, 1.0, 1.0, 1.0),
    Vertex(1.0, -1.0, 1.0, 0.0),
];

pub struct VoxelRenderPass {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    vertex_buf: Buffer,
}

impl VoxelRenderPass {
    pub fn new(
        device: &Device,
        render_texture: &wgpu::TextureView,
        present_format: TextureFormat,
    ) -> Self {
        let vertex_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(LABEL),
            contents: cast_slice(FSQ_VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let shader = device.create_shader_module(include_wgsl!("../shaders/vox_rendering.wgsl"));
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("VoxelRenderingPass"),
            layout: None,
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

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(LABEL),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(render_texture),
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

        Self {
            pipeline,
            bind_group,
            vertex_buf,
        }
    }

    pub fn draw_with_pass(&self, mut pass: RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.draw(0..6, 0..1);
    }
}