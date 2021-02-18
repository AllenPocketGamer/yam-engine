extern crate nalgebra as na;

use super::components::{Camera2D, Transform2D};
use crate::{Sprite, misc::Color};

use wgpu::util::DeviceExt;

pub struct Render2DService {
    gpu: Gpu,
    sprite_renderer: SpriteRenderer,

    frame: Option<wgpu::SwapChainFrame>,
    aspect_ratio: f32,

    mx_view: na::Matrix4<f32>,
    mx_projection: na::Matrix4<f32>,
}

impl Render2DService {
    pub fn new(window: &winit::window::Window) -> Self {
        let mut gpu = futures::executor::block_on(Gpu::new(window));
        let sprite_renderer = SpriteRenderer::new(&mut gpu);

        let default_camera2d = Camera2D::default();
        let default_camera2d_transform2d = Transform2D::default();

        Self {
            gpu,
            sprite_renderer,

            frame: None,
            aspect_ratio: default_camera2d.aspect_ratio(),

            mx_view: default_camera2d_transform2d
                .to_homogeneous_3d()
                .try_inverse()
                .unwrap(),
            mx_projection: default_camera2d.to_orthographic_homogeneous(),
        }
    }

    pub fn swap_chain_size(&self) -> (u32, u32) {
        (self.gpu.sc_desc.width, self.gpu.sc_desc.height)
    }

    pub fn set_swap_chain_size(&mut self, width: u32, height: u32) {
        self.gpu.sc_desc.width = width;
        self.gpu.sc_desc.height = height;

        self.gpu.swap_chain = self
            .gpu
            .device
            .create_swap_chain(&self.gpu.surface, &self.gpu.sc_desc);
    }

    #[allow(dead_code)]
    pub fn view_transformation(&self) -> na::Matrix4<f32> {
        self.mx_view
    }

    pub fn set_view_transformation(&mut self, camera2d_transform2d: &Transform2D) {
        self.mx_view = camera2d_transform2d
            .to_homogeneous_3d()
            .try_inverse()
            .unwrap()
    }

    #[allow(dead_code)]
    pub fn projection(&self) -> na::Matrix4<f32> {
        self.mx_projection
    }

    pub fn set_projection(&mut self, camera2d: &Camera2D) {
        self.mx_projection = camera2d.to_orthographic_homogeneous()
    }

    #[allow(dead_code)]
    pub fn viewport_aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn set_viewport_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio.abs();
    }

    pub fn begin_draw(&mut self) {
        if self.frame.is_none() {
            match self.gpu.swap_chain.get_current_frame() {
                Ok(sw_frame) => self.frame = Some(sw_frame),
                Err(err) => panic!("ERR: {}", err),
            }
        } else {
            panic!("ERR: Drawing has begun already.")
        }
    }

    pub fn draw_sprite_in_world_space(&mut self, transform2d: &Transform2D, sprite: &Sprite) {
        let viewport = self.calculate_adapted_viewport();
        let mx_model = transform2d.to_homogeneous_3d();

        self.sprite_renderer.render(
            &mut self.gpu,
            self.frame.as_ref().unwrap(),
            &mx_model,
            &self.mx_view,
            &self.mx_projection,
            &sprite.color,
            &viewport,
        );
    }

    // TODO: try to implement it
    #[allow(dead_code)]
    #[allow(unused_variables)]
    pub fn draw_sprites_in_world_space(&mut self, transform2ds: Vec<Transform2D>, sprite: &Sprite) {
        todo!()
    }

    #[allow(dead_code)]
    pub fn clear(&mut self, clear_color: &Color) {
        let [r, g, b, a] = clear_color.to_rgba_raw();

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &(self.frame.as_ref().unwrap().output.view),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: a as f64,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }

        self.gpu.queue.submit(Some(encoder.finish()));
    }

    pub fn end_draw(&mut self) {
        if self.frame.is_some() {
            self.frame.take();
        } else {
            panic!("ERR: Drawing has ended already.")
        }
    }

    // return (x, y, width, height, min_depth, max_depth)
    fn calculate_adapted_viewport(&self) -> (f32, f32, f32, f32, f32, f32) {
        let (screen_width, screen_height) = self.swap_chain_size();
        let (screen_width, screen_height) = (screen_width as f32, screen_height as f32);

        let aspect_ratio = self.aspect_ratio;
        let screen_ratio = screen_width / screen_height;

        if aspect_ratio <= screen_ratio {
            let (x, y) = ((screen_width - aspect_ratio * screen_height) / 2.0, 0f32);
            let (width, height) = (aspect_ratio * screen_height, screen_height);

            (x, y, width, height, 0.0, 1.0)
        } else {
            let (x, y) = (0f32, (screen_height - screen_width / aspect_ratio) / 2.0);
            let (width, height) = (screen_width, screen_width / aspect_ratio);

            (x, y, width, height, 0.0, 1.0)
        }
    }
}

struct Gpu {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
    sc_desc: wgpu::SwapChainDescriptor,
}

impl Gpu {
    async fn new(window: &winit::window::Window) -> Self {
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
            sc_desc,
        }
    }
}

struct SpriteRenderer {
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

    fn new(
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

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("color"),
            contents: bytemuck::cast_slice(&Color::WHITE.to_rgba_raw()[..]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
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
                range: 0..3 * 4 * 16,
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

    fn render(
        &mut self,
        Gpu { device, queue, .. }: &mut Gpu,
        frame: &wgpu::SwapChainFrame,
        mx_model: &na::Matrix4<f32>,
        mx_view: &na::Matrix4<f32>,
        mx_projection: &na::Matrix4<f32>,
        color: &Color,
        viewport: &(f32, f32, f32, f32, f32, f32),
    ) {
        queue.write_buffer(
            &self.uniform_buf,
            0,
            bytemuck::cast_slice(&color.to_rgba_raw()[..]),
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("sprite encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sprite render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &(frame.output.view),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.push_debug_group("prepare render data");

            rpass.set_viewport(
                viewport.0, viewport.1, viewport.2, viewport.3, viewport.4, viewport.5,
            );

            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_bind_group(0, &self.bind_group, &[]);

            // Set transformation matrix
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
}
