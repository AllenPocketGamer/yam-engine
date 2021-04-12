use super::super::{Gpu, Render2D, Viewport};

use crate::{
    legion::{Resources, World},
    misc::color::Rgba,
    nalgebra::Vector4,
};

use std::mem::size_of;

/// Renderer which renders background(like skybox in 3d).
pub(in super::super) struct BackgroundRenderer {
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl BackgroundRenderer {
    pub fn new(r2d: &Render2D) -> Self {
        let Gpu {
            device, sc_desc, ..
        } = &r2d.gpu;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("background bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("background bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &r2d.utility_buf,
                    offset: 0,
                    size: None,
                },
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("background pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vert_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("geometry vertex shader"),
            source: wgpu::util::make_spirv(include_bytes!(
                "../../../assets/shaders/background/blueprint.vert.spv"
            )),
            flags: wgpu::ShaderFlags::empty(),
        });

        let frag_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("geometry fragment shader"),
            source: wgpu::util::make_spirv(include_bytes!(
                "../../../assets/shaders/background/blueprint.frag.spv"
            )),
            flags: wgpu::ShaderFlags::empty(),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("background pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vector4<f32>>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendState {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendState {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Max,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
            }),
            multisample: Default::default(),
        });

        Self {
            bind_group,
            pipeline,
        }
    }

    pub fn render(&mut self, r2d: &Render2D, _world: &World, _resources: &Resources) {
        let Gpu {
            device,
            queue,
            frame,
            ..
        } = &r2d.gpu;

        let Viewport {
            x,
            y,
            w,
            h,
            min_depth,
            max_depth,
        } = r2d.viewport;

        let frame = frame.as_ref().expect("ERR: Not call begin_draw");

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("background encoder"),
        });

        encoder.insert_debug_marker("render background");
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("background render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Rgba::SOFT_BLACK.to_wgpu_color()),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &r2d.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            rpass.push_debug_group("background set data.");

            rpass.set_pipeline(&self.pipeline);
            rpass.set_viewport(x, y, w, h, min_depth, max_depth);
            rpass.set_vertex_buffer(0, r2d.quad_vertex_buf.slice(..));
            rpass.set_index_buffer(r2d.quad_index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_bind_group(0, &self.bind_group, &[]);

            rpass.pop_debug_group();

            rpass.draw_indexed(0..6, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}
