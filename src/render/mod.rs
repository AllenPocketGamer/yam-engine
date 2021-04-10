mod renderers;

use renderers::GeneralRenderer;

use crate::{
    app::{AppStage, AppStageBuilder},
    components::{camera::Camera2D, time::Time, transform::Transform2D},
    legion::{IntoQuery, Resources, World},
    nalgebra::Matrix4,
    window::Window,
};

// Quad vertex in world coordinate.
#[cfg_attr(rustfmt, rustfmt_skip)]
const QUAD_VERTEX: [f32; 16] = [
    -0.5, 0.5, 0.0, 1.0,    // left-top, point A
    0.5, 0.5, 0.0, 1.0,     // right-top, point B
    0.5, -0.5, 0.0, 1.0,    // right-bottom, point C
    -0.5, -0.5, 0.0, 1.0,   // left-bottom, point D
];

// Quad vertex index.
#[cfg_attr(rustfmt, rustfmt_skip)]
const QUAD_INDEX: [u16; 6] = [
    0, 1, 2,                // Face ABC
    2, 3, 0,                // Face CDA
];

#[rustfmt::skip] const KB: u64 = 1 << 10;
#[rustfmt::skip] const MB: u64 = 1 << 20;

pub(crate) fn create_app_stage_render(window: &Window) -> AppStage {
    let mut r2d = Render2D::new(window);
    let mut grd = GeneralRenderer::new(&r2d);

    let render_process = move |world: &mut World, resources: &mut Resources| {
        r2d.process(world, resources);

        r2d.begin_draw();

        grd.render(&r2d, world, resources);
        
        r2d.finish_draw();
    };

    AppStageBuilder::new(String::from("default_render"))
        .add_thread_local_fn_process(render_process)
        .build()
}

#[allow(dead_code)]
struct Gpu {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
    sc_desc: wgpu::SwapChainDescriptor,
    frame: Option<wgpu::SwapChainFrame>,
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
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: window.inner_size().width,
            height: window.inner_size().height,
            // NOTE: 特别关注这个设置, 跟硬件(显示屏)相关, 不正确的设置可能会导致灵异的bug;
            //  但现在还没碰到相关问题, 先搁置;
            // NOTE: 先默认设置为Mailbox, 该模式下画面会以垂直刷新率更新, 但与Fifo不同的是,
            //  GPU一旦绘制完画面, 会立即提交到表现引擎; 而Fifo模式下会通过阻塞线程的方式强制
            //  帧率与显示器刷新率同步.
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
            frame: None,
        }
    }
}

struct Render2D {
    gpu: Gpu,

    // Store the vertex datas of quad.
    quad_vertex_buf: wgpu::Buffer,
    // Store the index datas of quad.
    quad_index_buf: wgpu::Buffer,
    // Store the common use datas(likes `Time`, `MousePosition`..).
    utility_buf: wgpu::Buffer,
    // A springboard to transfer data from CPU to GPU.
    staging_buf: wgpu::Buffer,
    // Depth texture.
    depth_texture: Texture,

    viewport: Viewport,
    // // NOTE: 临时性数据, 用于决定是否更新shader
    // vhash: u64,
    // fhash: u64,
}

impl Render2D {
    fn new(window: &Window) -> Self {
        let gpu = futures::executor::block_on(Gpu::new(&window.window));

        use wgpu::util::DeviceExt;

        let quad_vertex_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex buffer"),
                contents: bytemuck::cast_slice(&QUAD_VERTEX[..]),
                usage: wgpu::BufferUsage::VERTEX,
            });

        let quad_index_buf = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("index buffer"),
                contents: bytemuck::cast_slice(&QUAD_INDEX[..]),
                usage: wgpu::BufferUsage::INDEX,
            });

        let utility_size = KB >> 2;
        let utility_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("utility buffer"),
            size: utility_size,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let staging_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging buffer"),
            size: 128 * MB,
            usage: wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::COPY_SRC,
            mapped_at_creation: false,
        });

        let depth_texture = Texture::create_depth_texture(&gpu.device, &gpu.sc_desc);

        let (width, height) = window.resolution();
        let viewport = Viewport::new_in_screen(
            width as f32,
            height as f32,
            Camera2D::default().aspect_ratio(),
        );

        Self {
            gpu,

            quad_vertex_buf,
            quad_index_buf,
            utility_buf,
            staging_buf,
            depth_texture,

            viewport,
            // // NOTE: 临时性数据
            // vhash: 0,
            // fhash: 0,
        }
    }

    fn begin_draw(&mut self) {
        if self.gpu.frame.is_none() {
            match self.gpu.swap_chain.get_current_frame() {
                Ok(sw_frame) => self.gpu.frame = Some(sw_frame),
                Err(err) => panic!("ERR: {}", err),
            }
        } else {
            panic!("ERR: Drawing has begun already.")
        }
    }

    fn process(&mut self, world: &mut World, resources: &mut Resources) {
        // Get window size.
        let (width, height) = {
            let window = resources
                .get::<Window>()
                .expect("ERR: Not find window resource.");
            window.resolution()
        };

        // Resize swap_chain and depth texture.
        if self.gpu.sc_desc.width != width || self.gpu.sc_desc.height != height {
            self.gpu.sc_desc.width = width;
            self.gpu.sc_desc.height = height;

            self.gpu.swap_chain = self
                .gpu
                .device
                .create_swap_chain(&self.gpu.surface, &self.gpu.sc_desc);

            self.depth_texture = Texture::create_depth_texture(&self.gpu.device, &self.gpu.sc_desc);
        }

        // Get camera2d.
        let mut query_camera2d = <(&Transform2D, &Camera2D)>::query();

        // Render a frame if there has a camera.
        if let Some((transform2d, camera2d)) = query_camera2d.iter(world).next() {
            let mx_view = transform2d.to_homogeneous_3d().try_inverse().unwrap();
            let mx_proj = {
                #[cfg_attr(rustfmt, rustfmt_skip)]
                let opengl_to_wgpu_matrix: Matrix4<f32> = Matrix4::new(
                    1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    0.0, 0.0, 0.5, 0.5,
                    0.0, 0.0, 0.0, 1.0,
                );

                opengl_to_wgpu_matrix * camera2d.to_orthographic_homogeneous()
            };
            let viewport =
                Viewport::new_in_screen(width as f32, height as f32, camera2d.aspect_ratio());

            let time = resources
                .get::<Time>()
                .expect("ERR: Not find time resource.");

            // Write these datas to utility_buf.
            self.gpu.queue.write_buffer(
                &self.utility_buf,
                0,
                bytemuck::cast_slice(mx_view.as_slice()),
            );
            self.gpu.queue.write_buffer(
                &self.utility_buf,
                64,
                bytemuck::cast_slice(mx_proj.as_slice()),
            );
            self.gpu.queue.write_buffer(
                &self.utility_buf,
                128,
                bytemuck::cast_slice(viewport.to_homogeneous_3d().as_slice()),
            );
            self.gpu.queue.write_buffer(
                &self.utility_buf,
                192,
                bytemuck::cast_slice(&[time.delta().as_secs_f32(), time.total().as_secs_f32()]),
            );

            self.viewport = viewport;
        }
    }

    fn finish_draw(&mut self) {
        if self.gpu.frame.is_some() {
            self.gpu.frame.take();
        } else {
            panic!("ERR: Drawing has ended already.")
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Viewport {
    pub x: f32,
    pub y: f32,

    pub w: f32,
    pub h: f32,

    pub min_depth: f32,
    pub max_depth: f32,
}

impl Viewport {
    pub fn new_in_screen(width: f32, height: f32, aspect_ratio: f32) -> Self {
        let screen_ratio = width / height;

        let (x, y, width, height) = if aspect_ratio <= screen_ratio {
            (
                (width - aspect_ratio * height) / 2.0,
                0f32,
                aspect_ratio * height,
                height,
            )
        } else {
            (
                0f32,
                (height - width / aspect_ratio) / 2.0,
                width,
                width / aspect_ratio,
            )
        };

        Self {
            x,
            y,
            w: width,
            h: height,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }

    /// Transform point from NDC to screen space
    ///
    /// x_ss = (x_ndc + 1) / 2 * width + vp.x        , x_ndc ∈ [-1, 1]
    /// y_ss = (1 - y_ndc) / 2 * height + vp.z       , y_ndc ∈ [-1, 1]
    /// z_ss = (far - near) * z_ndc + near           , z_ndc ∈ [+0, 1]
    pub fn to_homogeneous_3d(&self) -> Matrix4<f32> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        Matrix4::new(
            0.5 * self.w,   0.0,                0.0,                                0.5 * self.w + self.x,
            0.0,                -0.5 * self.h, 0.0,                                0.5 * self.h + self.y,
            0.0,                0.0,                self.max_depth - self.min_depth,    self.min_depth,
            0.0,                0.0,                0.0,                                1.0,
        )
    }
}

struct Texture {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    #[allow(dead_code)]
    sampler: wgpu::Sampler,
}

impl Texture {
    fn create_depth_texture(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}
