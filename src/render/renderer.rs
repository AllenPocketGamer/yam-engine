extern crate nalgebra as na;

use wgpu::util::DeviceExt;

pub(super) struct Gpu {
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain: wgpu::SwapChain,

    // FIXME: temp value
    pub sc_desc: wgpu::SwapChainDescriptor,
}

impl Gpu {
    pub(super) async fn new(window: &winit::window::Window) -> Self {
        let backend = wgpu::BackendBit::PRIMARY;
        let power_preference = wgpu::PowerPreference::HighPerformance;

        let instance = wgpu::Instance::new(backend);

        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("No suitable GPU adapters found on the system!");

        #[cfg(not(target_arch = "wasm32"))]
        {
            let adapter_info = adapter.get_info();
            println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);
        }

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
            .expect("Unable to find a suitable GPU adapter!");

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self {
            surface,
            adapter,
            device,
            queue,
            swap_chain,

            sc_desc,
        }
    }
}

pub(super) struct SpriteRenderer {
    // To store four vertex data(quad)
    vertex_buf: wgpu::Buffer,
    // To store index data of quad
    index_buf: wgpu::Buffer,
    // To store model matrix3x3 + view matrix3x3 + projection
    uniform_buf: wgpu::Buffer,

    // A group cotains uniform_buf, texture and sampler
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl SpriteRenderer {
    #[allow(dead_code)]
    const QUAD_HALF_SIZE: usize = 16;

    // Quad vertex in world coordinate
    #[allow(dead_code)]
    const QUAD_VERTEX: [f32; 16] = [
        // left-top, point A
        -(Self::QUAD_HALF_SIZE as f32),
        Self::QUAD_HALF_SIZE as f32,
        0.0,
        1.0,
        // right-top, point B
        Self::QUAD_HALF_SIZE as f32,
        Self::QUAD_HALF_SIZE as f32,
        0.0,
        1.0,
        // right-bottom, point C
        Self::QUAD_HALF_SIZE as f32,
        -(Self::QUAD_HALF_SIZE as f32),
        0.0,
        1.0,
        // left-bottom, point D
        -(Self::QUAD_HALF_SIZE as f32),
        -(Self::QUAD_HALF_SIZE as f32),
        0.0,
        1.0,
    ];

    #[allow(dead_code)]
    const QUAD_INDEX: [u16; 6] = [
        0, 1, 2, // Face ABC
        2, 3, 0, // Face CDA
    ];

    pub(super) fn new(
        Gpu {
            surface,
            adapter,
            device,
            ..
        }: &mut Gpu,
    ) -> Self {
        let vertex_size = 4 * 4;
        let uniform_size = 4 * 16 + 4 * 16 + 4 * 16;

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad vertex"),
            contents: bytemuck::cast_slice(&Self::QUAD_VERTEX[..]),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad index"),
            contents: bytemuck::cast_slice(&Self::QUAD_INDEX[..]),
            usage: wgpu::BufferUsage::INDEX,
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("model matrix3x3 + view matrix3x3 + projection"),
            size: uniform_size,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(uniform_size),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vert_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("sprite vertex shader"),
            source: wgpu::util::make_spirv(include_bytes!("sprite_shader.vert.spv")),
            flags: wgpu::ShaderFlags::VALIDATION,
        });

        let frag_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("sprite fragment shader"),
            source: wgpu::util::make_spirv(include_bytes!("sprite_shader.frag.spv")),
            flags: wgpu::ShaderFlags::VALIDATION,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_size,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: "main",
                targets: &[adapter.get_swap_chain_preferred_format(&surface).into()],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
        });

        Self {
            vertex_buf,
            index_buf,
            uniform_buf,

            bind_group,
            pipeline,
        }
    }

    pub(super) fn set_transformations(
        &mut self,
        Gpu { queue, .. }: &mut Gpu,
        mx_model: &na::Matrix4<f32>,
        mx_view: &na::Matrix4<f32>,
        mx_projection:  &na::Matrix4<f32>,
    ) {
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(mx_model.as_slice()));
        queue.write_buffer(&self.uniform_buf, 4 * 16, bytemuck::cast_slice(mx_view.as_slice()));
        queue.write_buffer(&self.uniform_buf, 2 * 4 * 16, bytemuck::cast_slice(mx_projection.as_slice()));
    }

    pub(super) fn render(
        &mut self,
        Gpu {
            device,
            queue,
            swap_chain,
            ..
        }: &mut Gpu,
    ) {
        let frame = swap_chain.get_current_frame().unwrap().output;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("sprite encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sprite render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.push_debug_group("prepare render data");
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_pipeline(&self.pipeline);
            rpass.pop_debug_group();

            rpass.insert_debug_marker("draw");
            rpass.draw_indexed(0..6, 0, 0..1);
            // rpass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}
