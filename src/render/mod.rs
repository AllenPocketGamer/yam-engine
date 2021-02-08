// TODO: 建立一个硬编码Material, 专门用于绘制旋转的Quad Mesh.
// 常量形式的bertex和index;
// 暂时没有复杂的BindGroup;
// 开干!

// TODO: 了解wgpu-rs的标准写法(cross-platform, debug, high-performance...).
// 了解RenderPass的debug命令;
// 了解什么是trace, 了解wgpu-rs的features的作用;

// TODO: 管线配置(纹理, 采样器, ..)从硬编码转到可配置

extern crate nalgebra as na;

use crate::{
    app::{AppStage, AppStageBuilder},
    input::Input,
    window::{self, Window},
};
use bytemuck::{Pod, Zeroable};
use futures::executor::block_on;
use legion::{Resources, World};
use legion_codegen::system;
use na::{Matrix2x3, Matrix3x1, Matrix4, Orthographic3, Point2, Point3, Vector2, Vector3, Vector4};
use std::{borrow::Cow, iter, ops::DerefMut, usize};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BufferBindingType, LoadOp,
};

pub(crate) fn create_app_stage_render() -> AppStage {
    AppStageBuilder::new(String::from("default_render"))
        .add_thread_local_fn_startup(init_resources)
        // .add_thread_local_fn_process(temp_render)
        .add_thread_local_fn_process(quad_render_fn)
        .build()
}

fn init_resources(_world: &mut World, resources: &mut Resources) {
    let mut renderer = {
        let window = resources
            .get::<Window>()
            .expect("Not found resource window.");
        block_on(Renderer::new(&window))
    };

    let quad_material = QuadMaterial::new(&mut renderer);

    resources.insert(renderer);
    resources.insert(quad_material);
}

// FIXME: just for testing
fn temp_render(_world: &mut World, resources: &mut Resources) {
    let input = resources.get::<Input>().unwrap();
    let window = resources.get::<Window>().unwrap();
    let renderer = resources.get_mut::<Renderer>().unwrap();

    let frame = renderer
        .swap_chain
        .get_current_frame()
        .expect("Timeout getting texture.")
        .output;

    let mut encoder = renderer
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let position = input.mouse.cursor_position();
        let color = wgpu::Color {
            r: position.0 as f64 / window.window.inner_size().width as f64,
            g: position.1 as f64 / window.window.inner_size().height as f64,
            b: 1.0,
            a: 1.0,
        };

        let mut _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
    }

    renderer.queue.submit(Some(encoder.finish()));
}

fn quad_render_fn(_world: &mut World, resources: &mut Resources) {
    let mut renderer = resources.get_mut::<Renderer>().unwrap();
    let quad_material = resources.get_mut::<QuadMaterial>().unwrap();

    quad_material.render(renderer.deref_mut());
}

pub struct Renderer {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,

    device: wgpu::Device,
    queue: wgpu::Queue,

    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
}

impl Renderer {
    // FIXME: temporary impl
    async fn new(Window { ref window }: &Window) -> Self {
        // FIXME: temporary values
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
            present_mode: wgpu::PresentMode::Mailbox, // NODE: to understand
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

pub struct QuadMaterial {
    // To store vertex data
    vertex_buf: wgpu::Buffer,
    // To store index data
    index_buf: wgpu::Buffer,
    // To store camera2d data
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl QuadMaterial {
    fn new(
        Renderer {
            device,
            sc_desc,
            adapter,
            ..
        }: &mut Renderer,
    ) -> Self {
        // TODO: 动态的从数据中加载
        let vertex_data = Self::vertex_data();
        let index_data = Self::index_data();
        let uniform_data = Self::orthographic_vp();

        // Create vertex_buf and index_buf
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&vertex_data[..]),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&index_data[..]),
            usage: wgpu::BufferUsage::INDEX,
        });

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::cast_slice(uniform_data.as_slice()),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let mut flags = wgpu::ShaderFlags::VALIDATION;
        match adapter.get_info().backend {
            wgpu::Backend::Metal | wgpu::Backend::Vulkan => {
                flags |= wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION;
            }
            _ => {}
        }

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("quad shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            flags,
        });

        // Create bind_group to store uniform buffer
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind group layout: orthgraphic project * world -> view transformation"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group: orthgraphic project * world -> view transformation"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad render pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: Vertex::size() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                // TODO: understand it
                targets: &[sc_desc.format.into()],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: wgpu::CullMode::None,
                front_face: wgpu::FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        Self {
            vertex_buf,
            index_buf,
            bind_group,
            pipeline,
        }
    }

    fn render(
        &self,
        Renderer {
            device,
            queue,
            swap_chain,
            ..
        }: &mut Renderer,
    ) {
        let frame = swap_chain.get_current_frame().unwrap().output;

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("quad render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.push_debug_group("Prepare data for draw");
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_pipeline(&self.pipeline);
            rpass.pop_debug_group();

            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..6 as u32, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }

    // 正面朝向正Z轴的在z=0处的Quad
    fn vertex_data() -> [Vertex; 4] {
        [
            Vertex::from_2d(-0.5, 0.5),
            Vertex::from_2d(0.5, 0.5),
            Vertex::from_2d(0.5, -0.5),
            Vertex::from_2d(-0.5, -0.5),
        ]
    }

    // Quad的索引数据
    fn index_data() -> [u16; 6] {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        [
            // Face abc
            0, 1, 2,
            // Face cda
            2, 3, 0,
        ]
    }

    fn orthographic_vp() -> Matrix4<f32> {
        // To adopt wgpu NDC
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mx_correction = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        );
        let mx_projection = Matrix4::new_orthographic(-1.0, 1.0, -1.0, 1.0, -10.0, 10.0);
        let mx_view = Matrix4::look_at_lh(
            &Point3::new(0.0, 0.0, 4.0),
            &Point3::origin(),
            &Vector3::y(),
        );

        mx_correction * mx_projection * mx_view
    }
}

#[derive(Clone, Copy)]
struct Vertex {
    pos_homo: Vector4<f32>,
}

impl Vertex {
    fn from_3d(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos_homo: Point3::new(x, y, z).to_homogeneous(),
        }
    }

    fn from_2d(x: f32, y: f32) -> Self {
        Self::from_3d(x, y, 0.0)
    }

    fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}
