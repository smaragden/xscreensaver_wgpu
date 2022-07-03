use cgmath::prelude::*;
use std::borrow::Cow;
use wgpu::util::DeviceExt;
use wgpu::{
    Device, PipelineLayout, RenderPipeline, TextureFormat, VertexBufferLayout,
};

use crate::primitives::{Instance, InstanceRaw, Vertex};

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-100.0, 0.0, 0.1],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [100.0, 0.0, 0.1],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [100.0, 0.0, -0.1],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-100.0, 0.0, -0.1],
        color: [0.0, 0.0, 1.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

const NUM_INSTANCES: u32 = 20;

pub struct Ground<'a> {
    pub pipeline: RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub layout: VertexBufferLayout<'a>,
    pub num_vertices: u32,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub instances: Vec<Instance>,
    pub instance_buffer: wgpu::Buffer,
}

impl<'a> Ground<'a> {
    pub fn init(
        device: &Device,
        pipeline_layout: PipelineLayout,
        swapchain_format: TextureFormat,
    ) -> Self {
        // Load the shaders from disk
        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "ground.wgsl"
                ))),
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc(), InstanceRaw::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: swapchain_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        let layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // 1.
            step_mode: wgpu::VertexStepMode::Vertex, // 2.
            attributes: &[
                // 3.
                wgpu::VertexAttribute {
                    offset: 0,                             // 4.
                    shader_location: 0,                    // 5.
                    format: wgpu::VertexFormat::Float32x3, // 6.
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>()
                        as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        };

        let instances = (0..NUM_INSTANCES)
            .map(move |i| {
                let position = cgmath::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: -(i as f32 / NUM_INSTANCES as f32 * 30.0),
                };
                let rotation = cgmath::Quaternion::zero();
                Instance { position, rotation }
            })
            .collect::<Vec<_>>();

        let instance_data =
            instances.iter().map(Instance::to_raw).collect::<Vec<_>>();

        let instance_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });
        Self {
            pipeline,
            vertex_buffer,
            layout,
            num_vertices: VERTICES.len() as u32,
            index_buffer,
            num_indices: INDICES.len() as u32,
            instances,
            instance_buffer,
        }
    }
}
