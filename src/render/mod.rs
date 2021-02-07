// TODO: 提供一个最基础的Render, 在屏幕中画个自旋正方体
// Render是AppStage, 会被插入App中
// Render需要Window的引用, 所有_window需要包装传入Resources中

// 一个简单的渲染流程
// 每帧遍历 (Transform + MeshFilter)
// 提交数据

// 自旋Cube
// 提交Cube顶点数据
// 提交Transform
// 渲染
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
use na::{DMatrix, Matrix2x3, Matrix3x1, Matrix4, Point2, Point3, Vector2, Vector3, Vector4};
use std::{borrow::Cow, iter, usize};
use wgpu::{util::DeviceExt, vertex_attr_array};

pub(crate) fn create_app_stage_render() -> AppStage {
    AppStageBuilder::new(String::from("default_render"))
        .add_thread_local_fn_startup(init_resources)
        .add_thread_local_fn_process(cube_render)
        .build()
}

fn init_resources(_world: &mut World, resources: &mut Resources) {
    let mut renderer = {
        let window = resources
            .get::<Window>()
            .expect("Not found resource window.");
        block_on(Renderer::new(&window))
    };
    let cube_resources = CubeRenderResources::new(&mut renderer);

    resources.insert(renderer);
    resources.insert(cube_resources);
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

fn cube_render(_world: &mut World, resources: &mut Resources) {
    let cube_resources = resources.get_mut::<CubeRenderResources>().unwrap();
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
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
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

        rpass.set_pipeline(&cube_resources.pipeline);
        rpass.set_bind_group(0, &cube_resources.bind_group, &[]);
        rpass.set_index_buffer(
            cube_resources.index_buf.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        rpass.set_vertex_buffer(0, cube_resources.vertex_buf.slice(..));
        rpass.draw_indexed(0..cube_resources.index_count as u32, 0, 0..1);
    }

    renderer.queue.submit(Some(encoder.finish()));
}

pub struct Renderer {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
    sc_desc: wgpu::SwapChainDescriptor,
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

// FIXME: temp struct
pub struct CubeRenderResources {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl CubeRenderResources {
    fn new(
        Renderer {
            adapter,
            device,
            queue,
            sc_desc,
            ..
        }: &mut Renderer,
    ) -> Self {
        let vertex_size = std::mem::size_of::<Vertex>();
        let (vertex_data, index_data) = Self::create_cube_vertices();

        // Create vertex buffer
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsage::VERTEX,
        });

        // Create index buffer
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsage::INDEX,
        });

        // Create pipeline
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create texture
        let size = 256u32;
        let texels = Self::create_texels(size as usize);
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth: 1,
        };
        // TODO: to understand texture in futures
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("default texture"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &texels,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * size,
                rows_per_image: 0,
            },
            texture_extent,
        );

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let mx_total = Self::generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::cast_slice(mx_total.as_slice()),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // TODO: understand it
        let mut flags = wgpu::ShaderFlags::VALIDATION;
        match adapter.get_info().backend {
            wgpu::Backend::Metal | wgpu::Backend::Vulkan => {
                flags |= wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION
            }
            _ => {}
        }
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            flags,
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &vertex_attr_array![0 => Float4, 1 => Float2],
        }];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[sc_desc.format.into()],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        Self {
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            uniform_buf,
            bind_group,
            pipeline,
        }
    }

    fn create_cube_vertices() -> ([Vertex; 24], [u16; 36]) {
        let vertex_data: [Vertex; 24] = [
            // top (0, 0, 1)
            Vertex::new(Point3::new(-1.0, -1.0, 1.0), Point2::new(0.0, 0.0)),
            Vertex::new(Point3::new(1.0, -1.0, 1.0), Point2::new(1.0, 0.0)),
            Vertex::new(Point3::new(1.0, 1.0, 1.0), Point2::new(1.0, 1.0)),
            Vertex::new(Point3::new(-1.0, 1.0, 1.0), Point2::new(0.0, 1.0)),
            // bottom (0, 0, -1)
            Vertex::new(Point3::new(-1.0, 1.0, -1.0), Point2::new(1.0, 0.0)),
            Vertex::new(Point3::new(1.0, 1.0, -1.0), Point2::new(0.0, 0.0)),
            Vertex::new(Point3::new(1.0, -1.0, -1.0), Point2::new(0.0, 1.0)),
            Vertex::new(Point3::new(-1.0, -1.0, -1.0), Point2::new(1.0, 1.0)),
            // right (1, 0, 0)
            Vertex::new(Point3::new(1.0, -1.0, -1.0), Point2::new(0.0, 0.0)),
            Vertex::new(Point3::new(1.0, 1.0, -1.0), Point2::new(1.0, 0.0)),
            Vertex::new(Point3::new(1.0, 1.0, 1.0), Point2::new(1.0, 1.0)),
            Vertex::new(Point3::new(1.0, -1.0, 1.0), Point2::new(0.0, 1.0)),
            // left (-1, 0, 0)
            Vertex::new(Point3::new(-1.0, -1.0, 1.0), Point2::new(1.0, 0.0)),
            Vertex::new(Point3::new(-1.0, 1.0, 1.0), Point2::new(0.0, 0.0)),
            Vertex::new(Point3::new(-1.0, 1.0, -1.0), Point2::new(0.0, 1.0)),
            Vertex::new(Point3::new(-1.0, -1.0, -1.0), Point2::new(1.0, 1.0)),
            // front (0, 1, 0)
            Vertex::new(Point3::new(1.0, 1.0, -1.0), Point2::new(1.0, 0.0)),
            Vertex::new(Point3::new(-1.0, 1.0, -1.0), Point2::new(0.0, 0.0)),
            Vertex::new(Point3::new(-1.0, 1.0, 1.0), Point2::new(0.0, 1.0)),
            Vertex::new(Point3::new(1.0, 1.0, 1.0), Point2::new(1.0, 1.0)),
            // back (0, -1, 0)
            Vertex::new(Point3::new(1.0, -1.0, 1.0), Point2::new(0.0, 0.0)),
            Vertex::new(Point3::new(-1.0, -1.0, 1.0), Point2::new(1.0, 0.0)),
            Vertex::new(Point3::new(-1.0, -1.0, -1.0), Point2::new(1.0, 1.0)),
            Vertex::new(Point3::new(1.0, -1.0, -1.0), Point2::new(0.0, 1.0)),
        ];

        let index_data: [u16; 36] = [
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        (vertex_data, index_data)
    }

    fn create_texels(size: usize) -> Vec<u8> {
        (0..size * size)
            .flat_map(|id| {
                // get high five for recognizing this ;)
                let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
                let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
                let (mut x, mut y, mut count) = (cx, cy, 0);
                while count < 0xFF && x * x + y * y < 4.0 {
                    let old_x = x;
                    x = x * x - y * y + cx;
                    y = 2.0 * old_x * y + cy;
                    count += 1;
                }
                iter::once(0xFF - (count * 5) as u8)
                    .chain(iter::once(0xFF - (count * 15) as u8))
                    .chain(iter::once(0xFF - (count * 50) as u8))
                    .chain(iter::once(1))
            })
            .collect()
    }

    fn generate_matrix(aspect_ratio: f32) -> Matrix4<f32> {
        let mx_projection = Matrix4::new_perspective(16.0 / 9.0, aspect_ratio, 1.0, 1000.0);
        let mx_view = Matrix4::look_at_rh(
            &Point3::new(4.0f32, 1.0, 4.0),
            &Point3::new(0f32, 0.0, 0.0),
            &Vector3::y(),
        );

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let mx_correction = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        );

        mx_projection * mx_view
    }
}

// TODO: 把结构体转换为另一种slice类型
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    pos_homo: Vector4<f32>,
    tex_coord: Vector2<f32>,
}

impl Vertex {
    fn new(pos: Point3<f32>, tex_coord: Point2<f32>) -> Self {
        Self {
            pos_homo: pos.to_homogeneous(),
            tex_coord: tex_coord.coords,
        }
    }
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

// TODO: 纹理资源的创建, 绑定, 采样是个难点
