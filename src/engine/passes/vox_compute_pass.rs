use wgpu::include_wgsl;

/// Rendering voxels.
pub struct VoxelComputePass {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
}

impl VoxelComputePass {
    /// Create a new [`VoxelComputePass`].
    ///
    /// # Arguments
    ///
    /// * `device` - The device from wich the current [`ComputePass`] will be created.
    /// * `uniform_buf` - The buffer that contains the uniforms data.
    /// * `out_tex_view` - The [`wgpu::TextureView`] of a [`wgpu::Texture`] on wich the current [`ComputePass`] will write.
    ///
    pub fn new(
        device: &wgpu::Device,
        uniform_buf: &wgpu::Buffer,
        svo_buf: &wgpu::Buffer,
        stack_buf: &wgpu::Buffer,
        out_tex_view: &wgpu::TextureView,
    ) -> Self {
        let shader_module =
            device.create_shader_module(include_wgsl!("../shaders/vox_compute_2.wgsl"));

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Voxel Compute Pipeline"),
            layout: None,
            module: &shader_module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(out_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: svo_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: stack_buf.as_entire_binding(),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
        }
    }

    // /// Update the inner [`wgpu::BindGroup`].
    // pub fn update_bindings(
    //     &mut self,
    //     device: &wgpu::Device,
    //     uniform_buf: &wgpu::Buffer,
    //     stack_buf: &wgpu::Buffer,
    //     out_tex_view: &wgpu::TextureView,
    // ) {
    //     self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    //         label: None,
    //         layout: &self.pipeline.get_bind_group_layout(0),
    //         entries: &[
    //             wgpu::BindGroupEntry {
    //                 binding: 0,
    //                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
    //                     buffer: &uniform_buf,
    //                     offset: 0,
    //                     size: None,
    //                 }),
    //             },
    //             wgpu::BindGroupEntry {
    //                 binding: 1,
    //                 resource: wgpu::BindingResource::TextureView(&out_tex_view),
    //             },
    //             wgpu::BindGroupEntry {
    //                 binding: 2,
    //                 resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
    //                     buffer: &stack_buf,
    //                     offset: 0,
    //                     size: None,
    //                 }),
    //             },
    //         ],
    //     });
    // }

    /// Retrieve the [`wgpu::ComputePipeline`] of the current [`ComputePass`].
    pub fn pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }

    /// Retrieve the [`wgpu::BindGroup`]  of the current [`ComputePass`].
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
