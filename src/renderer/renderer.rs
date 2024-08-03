use std::sync::Arc;

use pollster::block_on;
use wgpu::{
    Backends, Color, CommandEncoderDescriptor, CompositeAlphaMode, ComputePassDescriptor, Device,
    DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, LoadOp, MemoryHints,
    Operations, PowerPreference, PresentMode, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration,
    TextureFormat, TextureUsages,
};
use winit::{dpi::PhysicalSize, window::Window};

use super::voxel::{
    passes::{rendering::VoxelImageRenderingPass, voxel_rendering::VoxelRendererPass},
    textures::RenderTexture,
};

pub struct Renderer<'window> {
    surface: Surface<'window>,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,

    render_texture: RenderTexture,
    voxel_image_rendering_pass: VoxelImageRenderingPass,
    voxel_renderer_pass: VoxelRendererPass,
}

impl<'window> Renderer<'window> {
    pub fn new(window: Arc<Window>) -> Self {
        let window_size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();
        let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &DeviceDescriptor {
                label: None,
                required_features: Features::default(),
                required_limits: Limits::default(),
                memory_hints: MemoryHints::Performance,
            },
            None,
        ))
        .unwrap();

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: window_size.width,
            height: window_size.height,
            present_mode: PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let render_texture = RenderTexture::new(&device, window_size, TextureFormat::Rgba8Unorm);
        let voxel_image_rendering_pass =
            VoxelImageRenderingPass::new(&device, &render_texture, surface_config.format);
        let voxel_renderer_pass = VoxelRendererPass::new(&device, &render_texture);

        Self {
            surface,
            surface_config,
            device,
            queue,
            render_texture,
            voxel_image_rendering_pass,
            voxel_renderer_pass,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;

        self.surface.configure(&self.device, &self.surface_config);
        self.render_texture.resize(&self.device, new_size);

        // Because the render texture was resized, we need to
        // rebuild all pipelines that depends on it...
        self.voxel_image_rendering_pass = VoxelImageRenderingPass::new(
            &self.device,
            &self.render_texture,
            self.surface_config.format,
        );
        self.voxel_renderer_pass = VoxelRendererPass::new(&self.device, &self.render_texture);
    }

    pub fn draw(&self) {
        let surface_texture = self.surface.get_current_texture().unwrap();
        let surface_texture_view = surface_texture.texture.create_view(&Default::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let voxel_compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
        self.voxel_renderer_pass
            .compute_with_pass(voxel_compute_pass);

        let pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("VoxelRenderingImage"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &surface_texture_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::RED),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.voxel_image_rendering_pass.draw_with_pass(pass);

        let command_buffer = encoder.finish();

        self.queue.submit([command_buffer]);
        surface_texture.present();
    }
}
