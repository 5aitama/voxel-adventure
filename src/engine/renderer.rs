use super::passes::{vox_compute_pass::VoxelComputePass, vox_rendering_pass::VoxelRenderPass};
use pollster::block_on;
use std::{sync::Arc, time::Instant};
use wgpu::{
    hal::QUERY_SIZE, util::DeviceExt, Backends, CommandEncoderDescriptor, CompositeAlphaMode,
    Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, InstanceFlags, Limits,
    MemoryHints, PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceError, TextureFormat, TextureUsages,
};
use winit::dpi::PhysicalSize;

pub struct Renderer<'window> {
    surface: Surface<'window>,
    surface_size: PhysicalSize<u32>,
    device: Device,
    queue: Queue,

    stack_buf: wgpu::Buffer,
    uniform_buf: wgpu::Buffer,

    instant: Instant,
    fps: u64,
    voxel_acc_time: f32,

    query_set: wgpu::QuerySet,
    query_set_buf: wgpu::Buffer,
    query_set_read_buf: wgpu::Buffer,
    voxel_output_texture: wgpu::Texture,
    voxel_compute_pass: VoxelComputePass,
    voxel_render_pass: VoxelRenderPass,
}

impl<'window> Renderer<'window> {
    pub fn new(window: Arc<winit::window::Window>) -> Self {
        let surface_size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN,
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
                required_features: Features::default() | Features::TIMESTAMP_QUERY,
                required_limits: Limits::default(),
                memory_hints: MemoryHints::Performance,
            },
            None,
        ))
        .unwrap();

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: surface_size.width,
            height: surface_size.height,
            present_mode: PresentMode::AutoVsync,
            desired_maximum_frame_latency: 3,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let voxel_output_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Voxel Render Texture"),
            size: wgpu::Extent3d {
                width: surface_size.width,
                height: surface_size.height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let uniforms = super::uniform::Uniforms {
            texture_width: surface_size.width,
            texture_height: surface_size.height,
        };

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // The maximum size of the stack array (in byte)
        let s = 256;
        let mut svo = svo::Svo::new(svo::Vec3 { x: 0, y: 0, z: 0 }, s, 5);
        for i in 0..(s / 8) {
            for j in 0..(s / 8) {
                for k in 0..(s / 8) {
                    svo.set_node(
                        svo::Vec3 {
                            x: -(s / 2) + i * 8,
                            y: -(s / 2) + j * 8,
                            z: -(s / 2) + k * 8,
                        },
                        42.try_into().unwrap(),
                        None,
                    );
                }
            }
        }

        let stack_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Stack Buffer"),
            contents: svo.buf(),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let voxel_compute_pass = VoxelComputePass::new(
            &device,
            &uniform_buf,
            &stack_buf,
            &voxel_output_texture.create_view(&Default::default()),
        );

        let voxel_render_pass = VoxelRenderPass::new(
            &device,
            &voxel_output_texture.create_view(&Default::default()),
            surface_config.format,
        );

        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: None,
            ty: wgpu::QueryType::Timestamp,
            count: 3,
        });

        let query_set_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Query Set Buffer"),
            size: (std::mem::size_of::<u64>() * 3) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let query_set_read_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Query Set Buffer Read"),
            size: (std::mem::size_of::<u64>() * 3) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            surface,
            surface_size,
            device,
            queue,

            uniform_buf,
            stack_buf,

            instant: Instant::now(),
            fps: 0,
            voxel_acc_time: 0.0,

            query_set,
            query_set_buf,
            query_set_read_buf,

            voxel_output_texture,
            voxel_compute_pass,
            voxel_render_pass,
        }
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        let surface_texture_view = surface_texture.texture.create_view(&Default::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        {
            let mut vox_rendering_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Voxel Rendering"),
                timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                    query_set: &self.query_set,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: Some(1),
                }),
            });

            vox_rendering_pass.set_pipeline(self.voxel_compute_pass.pipeline());
            vox_rendering_pass.set_bind_group(0, self.voxel_compute_pass.bind_group(), &[]);

            vox_rendering_pass.dispatch_workgroups(
                ((self.surface_size.width as f32) / 16.0).ceil() as u32,
                ((self.surface_size.height as f32) / 16.0).ceil() as u32,
                1,
            );
        }

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("VoxelRenderingImage"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.voxel_render_pass.draw_with_pass(render_pass);
        }

        encoder.resolve_query_set(&self.query_set, 0..2, &self.query_set_buf, 0);
        encoder.copy_buffer_to_buffer(
            &self.query_set_buf,
            0,
            &self.query_set_read_buf,
            0,
            QUERY_SIZE * 3,
        );
        let cmd_buf = encoder.finish();
        self.queue.submit([cmd_buf]);

        self.fps += 1;

        if self.instant.elapsed().as_secs() >= 1 {
            println!("{}fps", self.fps);

            let mut calculation_time = self.voxel_acc_time / 1_000.0 / 1_000.0 / self.fps as f32;
            let target_fps: f32 = 1000.0 / 60.0;
            calculation_time = (calculation_time * 1000.0).round() / 1000.0;
            println!(
                "Voxel render time: ~{}ms/frame({}%)\n",
                calculation_time,
                100.0 * (calculation_time / target_fps * 100.0).round() / 100.0
            );
            self.fps = 0;
            self.voxel_acc_time = 0.0;
            self.instant = Instant::now();
        }

        let buf = self.query_set_read_buf.slice(..);
        buf.map_async(wgpu::MapMode::Read, move |_| {});
        self.device.poll(wgpu::Maintain::wait());

        let buf_view = buf.get_mapped_range();
        let mut timestamp_beg = [0_u8; 8];
        timestamp_beg.copy_from_slice(&buf_view[0..8]);

        let mut timestamp_end = [0_u8; 8];
        timestamp_end.copy_from_slice(&buf_view[8..16]);

        drop(buf_view);
        let period = self.queue.get_timestamp_period();

        let timestamp_beg = u64::from_ne_bytes(timestamp_beg);
        let timestamp_end = u64::from_ne_bytes(timestamp_end);

        self.voxel_acc_time += (timestamp_end - timestamp_beg) as f32;
        // println!(
        //     "[Voxel Rendering Compute] {}ms, ({})",
        //     (timestamp_end - timestamp_beg) as f32 / 1_000_000.0,
        //     period
        // );

        self.query_set_read_buf.unmap();

        surface_texture.present();

        // self.device.create_buffer(&wgpu)

        Ok(())
    }
}
