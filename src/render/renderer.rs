extern crate nalgebra as na;

use wgpu::util::DeviceExt;

pub(super) struct Gpu {
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain: wgpu::SwapChain,

    // FIXME: temp value
    pub frame: Option<wgpu::SwapChainFrame>,

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

        let adapter_info = adapter.get_info();
        println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

        let can_push_constant = !(adapter.features() & wgpu::Features::PUSH_CONSTANTS).is_empty();
        let max_push_constant_size = adapter.limits().max_push_constant_size;

        if can_push_constant {
            println!(
                "Support PUSH_CONSTANT feature, max push const size: {}.",
                max_push_constant_size
            );
        } else {
            println!("Not support PUSH_CONSTANT feature.");
        }

        let (features, limits) = if can_push_constant {
            let mut limits = wgpu::Limits::default();
            limits.max_push_constant_size = max_push_constant_size;
            (wgpu::Features::PUSH_CONSTANTS, limits)
        } else {
            (wgpu::Features::empty(), wgpu::Limits::default())
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features,
                    limits,
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
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self {
            surface,
            adapter,
            device,
            queue,
            swap_chain,

            frame: None,

            sc_desc,
        }
    }

    // FIXME: 移动到合适位置
    pub(super) fn swap_chain_size(&self) -> (u32, u32) {
        (self.sc_desc.width, self.sc_desc.height)
    }

    // FIXME: 移动到合适位置
    pub(super) fn set_swap_chain_size(&mut self, (width, height): (u32, u32)) {
        if self.sc_desc.width != width || self.sc_desc.height != height {
            self.sc_desc.width = width;
            self.sc_desc.height = height;

            self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        }
    }

    pub(super) fn begin_render(&mut self) {
        if self.frame.is_none() {
            self.frame = self.swap_chain.get_current_frame().ok();
        } else {
            panic!("Begin render already.");
        }
    }

    pub(super) fn end_render(&mut self) {
        if self.frame.is_some() {
            self.frame.take();
        } else {
            panic!("End render already.");
        }
    }
}

pub(super) struct SpriteRenderer {
    // To store four vertex data(quad)
    vertex_buf: wgpu::Buffer,
    // To store index data of quad
    index_buf: wgpu::Buffer,
    // To store color data
    uniform_buf: wgpu::Buffer,

    // A group cotains uniform_buf, texture and sampler
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl SpriteRenderer {
    // Quad vertex in world coordinate
    #[allow(dead_code)]
    const QUAD_VERTEX: [f32; 16] = [
        -0.5, 0.5, 0.0, 1.0, // left-top, point A
        0.5, 0.5, 0.0, 1.0, // right-top, point B
        0.5, -0.5, 0.0, 1.0, // right-bottom, point C
        -0.5, -0.5, 0.0, 1.0, // left-bottom, point D
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
            label: Some("color"),
            size: 4 * 4,
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
                    min_binding_size: wgpu::BufferSize::new(4 * 4),
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
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStage::VERTEX,
                range: 0..3 * 4 * 16
            }],
        });

        let vert_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("sprite vertex shader"),
            source: wgpu::util::make_spirv(include_bytes!("sprite_shader.vert.spv")),
            flags: wgpu::ShaderFlags::empty(),
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

    pub(super) fn render(
        &mut self,
        Gpu {
            device,
            queue,
            frame,
            ..
        }: &mut Gpu,
        mx_model: &na::Matrix4<f32>,
        mx_view: &na::Matrix4<f32>,
        mx_projection: &na::Matrix4<f32>,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("sprite encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sprite render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &(frame.as_ref().unwrap().output.view),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.push_debug_group("prepare render data");
            
            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);

            // TODO: change it
            rpass.set_bind_group(0, &self.bind_group, &[]);
            
            // NOTE: Set transformation matrix
            rpass.set_push_constants(
                wgpu::ShaderStage::VERTEX,
                0,
                bytemuck::cast_slice(mx_model.as_slice()),
            );
            rpass.set_push_constants(
                wgpu::ShaderStage::VERTEX,
                4 * 16,
                bytemuck::cast_slice(mx_view.as_slice()),
            );
            rpass.set_push_constants(
                wgpu::ShaderStage::VERTEX,
                4 * 16 + 4 * 16,
                bytemuck::cast_slice(mx_projection.as_slice()),
            );
            
            rpass.pop_debug_group();

            rpass.insert_debug_marker("draw");
            rpass.draw_indexed(0..6, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }

    // NOTE: look like stupid
    pub(super) fn clear(
        &mut self,
        Gpu {
            device,
            queue,
            frame,
            ..
        }: &mut Gpu,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &(frame.as_ref().unwrap().output.view),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }

        queue.submit(Some(encoder.finish()));
    }

    // NOTE: 直接渲染API的功能分割
    // 1. view + projection需要一个公共的存储区域
    // 2. 其中一个参数是model transformation, 决定sprite应该画在哪里, 怎么画
    // 3. viewport也应该有个公共区域
    pub(super) fn draw(&mut self) {}
}
