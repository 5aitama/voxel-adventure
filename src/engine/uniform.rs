#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Uniforms {
    pub texture_width: u32,
    pub texture_height: u32,
}

impl Uniforms {
    pub fn new_buf(&self, device: &wgpu::Device) -> wgpu::Buffer {
        wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniforms"),
                contents: bytemuck::cast_slice(&[*self]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        )
    }

    pub fn update_buf(&self, queue: &wgpu::Queue, buf: &wgpu::Buffer) {
        queue.write_buffer(buf, 0, bytemuck::cast_slice(&[*self]));
    }
}
