use wgpu::{
    Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView,
};
use winit::dpi::PhysicalSize;

pub struct RenderTexture {
    texture: Texture,
    view: TextureView,
    format: TextureFormat,
    size: PhysicalSize<u32>,
}

impl RenderTexture {
    pub fn new(device: &Device, size: PhysicalSize<u32>, format: TextureFormat) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Voxel Render Texture"),
            size: Extent3d {
                width: size.width,
                height: size.height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &vec![],
        });

        let view = texture.create_view(&Default::default());

        Self {
            texture,
            view,
            format,
            size,
        }
    }

    pub fn resize(&mut self, device: &Device, new_size: PhysicalSize<u32>) {
        self.texture = device.create_texture(&TextureDescriptor {
            label: Some("Voxel Render Texture"),
            size: Extent3d {
                width: new_size.width,
                height: new_size.height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.format,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &vec![],
        });

        self.view = self.texture.create_view(&Default::default());
    }

    pub fn get_size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn get_view(&self) -> &TextureView {
        &self.view
    }

    pub fn get_format(&self) -> TextureFormat {
        self.format
    }
}
