#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Uniforms {
    pub texture_width: u32,
    pub texture_height: u32,
}
