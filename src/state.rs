use fps_clock::FpsClock;
use wgpu::{util::DeviceExt, Device, Queue, Surface, SurfaceConfiguration};

use crate::{
    ground::Ground,
    primitives::{Camera, CameraUniform, FrameUniform},
    xscreensaver::SizedWindow,
};

pub struct State<'a> {
    // WGPU
    surface: Surface,
    pub device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    // FPS
    fps: FpsClock,
    frame_uniform: FrameUniform,
    frame_buffer: wgpu::Buffer,
    frame_bind_group: wgpu::BindGroup,
    // Render
    _camera: Camera,
    _camera_uniform: CameraUniform,
    _camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    // Assets
    ground: Ground<'a>,
}

impl<'a> State<'a> {
    pub async fn setup<T>(window: &T, fps: u32) -> State<'a>
    where
        T: raw_window_handle::HasRawWindowHandle + SizedWindow,
    {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_format = surface.get_supported_formats(&adapter)[0];
        let (width, height) = window.size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let frame_uniform = FrameUniform::new();

        let frame_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Frame Buffer"),
            contents: bytemuck::cast_slice(&[frame_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let frame_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("frame_bind_group_layout"),
            });

        let frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &frame_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: frame_buffer.as_entire_binding(),
            }],
            label: Some("frame_bind_group"),
        });

        let camera = Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, -100.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
        };
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // Create the rendering pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&frame_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let ground = Ground::init(&device, pipeline_layout, swapchain_format);

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            ground,
            queue,
            config,
            fps: fps_clock::FpsClock::new(fps),
            frame_uniform,
            frame_buffer,
            frame_bind_group,
            _camera: camera,
            _camera_uniform: camera_uniform,
            _camera_buffer: camera_buffer,
            camera_bind_group,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if self.config.width != width || self.config.height != height {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.0,
                            b: 0.01,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.ground.pipeline);
            rpass.set_bind_group(0, &self.frame_bind_group, &[]);
            rpass.set_bind_group(1, &self.camera_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.ground.vertex_buffer.slice(..));

            rpass.set_index_buffer(
                self.ground.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            rpass.set_vertex_buffer(1, self.ground.instance_buffer.slice(..));
            rpass.draw_indexed(
                0..self.ground.num_indices,
                0,
                0..self.ground.instances.len() as _,
            );
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
    pub fn tick(&mut self) {
        self.fps.tick();
        self.frame_uniform.incr_frame();
        self.queue.write_buffer(
            &self.frame_buffer,
            0,
            bytemuck::cast_slice(&[self.frame_uniform]),
        );
    }
}
