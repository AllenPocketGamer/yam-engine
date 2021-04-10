mod renderers;

use renderers::{GeneralRenderer};

use crate::{
    app::{AppStage, AppStageBuilder},
    components::{time::Time, camera::Camera2D, sprite::Sprite, transform::Transform2D, Instance},
    legion::{IntoQuery, Resources, World},
    misc::color::Rgba,
    nalgebra::Matrix4,
    window::Window,
};

use std::time::Instant;

pub(crate) fn create_app_stage_render(Window { window }: &Window) -> AppStage {
    let mut r2ds = Render2D::new(window);
    let mut timestamp = Instant::now();

    let render_process = move |world: &mut World, resources: &mut Resources| {
        let (width, height) = {
            let window = resources
                .get::<Window>()
                .expect("ERR: Not find window resource.");
            window.resolution()
        };

        let mut query_camera2d = <(&Transform2D, &Camera2D)>::query();
        let mut query_sprites = <(&Transform2D, &Sprite)>::query();
        let mut query_sprites_instanced = <(&Instance<Transform2D>, &Sprite)>::query();

        r2ds.set_swap_chain_size(width, height);

        if let Some((transform2d, camera2d)) = query_camera2d.iter(world).next() {
            r2ds.set_view_transformation(transform2d);
            r2ds.set_projection(camera2d);
            r2ds.set_viewport_aspect_ratio(camera2d.aspect_ratio());
        }

        // NOTE: 临时性代码: 热更新shaders.
        //
        // 这应该只在yam开发时启用, 在被别的库链接时关闭!
        // 但现在不知道怎么做, 反正先把shader写完再说.
        //
        // 这些代码不应该放在library里面, 可以新建一个
        // binary, 这个binary专门用于测试shader, 可以
        // 把shader热更新放在这个binary里面!
        // 而library要做的就是留足接口(这个就是比较大的工程了).
        let now = Instant::now();
        if (now - timestamp).as_millis() > 330 {
            r2ds.recompile_shader();
            timestamp = now;
        }

        r2ds.begin_draw();

        let time = resources.get::<Time>().unwrap().clone();
        r2ds.draw_geometry(world, &time);

        r2ds.finish_draw();
    };

    AppStageBuilder::new(String::from("default_render"))
        .add_thread_local_fn_process(render_process)
        .build()
}

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

// TODO: Render2D应该改成Render2DManager, 用来管理Render2DPass;
// 像GeneralRenderer实际上可以拆分成四个Render2DPass
//
// 1. Geometry1D
// 2. Geometry2D
// 3. Sprite
// 4. Background
//
// RenderPass2D的相同点在于它们都需要一些共同的buffer(vertex, instance, uniform..);
// 它们的不同点在于会有一些特殊的buffer(storage, staging..), 以及渲染管线渲染设置上的不同;
//
// Render2DManager可以固定常用的buffer, 提供特殊buffer的管理服务, 提供RenderPass2D的注册
// 功能, 按顺序调用RenderPass2D;
//
// 这样, Render2DManager + RenderPass2D还可以提供对外接口以支持渲染自定义!
//
// RenderPass2D可以抽象为`trait`.
struct Render2D {
    gpu: Gpu,
    general_renderer: GeneralRenderer,

    aspect_ratio: f32,
    mx_view: Matrix4<f32>,
    mx_projection: Matrix4<f32>,

    // NOTE: 临时性数据, 用于决定是否更新shader
    vhash: u64,
    fhash: u64,
}

impl Render2D {
    pub fn new(window: &winit::window::Window) -> Self {
        let mut gpu = futures::executor::block_on(Gpu::new(window));
        let general_renderer = GeneralRenderer::new(&mut gpu);

        let default_camera2d = Camera2D::default();
        let default_camera2d_transform2d = Transform2D::default();

        Self {
            gpu,
            general_renderer,

            aspect_ratio: default_camera2d.aspect_ratio(),
            mx_view: default_camera2d_transform2d
                .to_homogeneous_3d()
                .try_inverse()
                .unwrap(),
            mx_projection: default_camera2d.to_orthographic_homogeneous(),

            // NOTE: 临时性数据
            vhash: 0,
            fhash: 0,
        }
    }

    pub fn swap_chain_size(&self) -> (u32, u32) {
        (self.gpu.sc_desc.width, self.gpu.sc_desc.height)
    }

    pub fn set_swap_chain_size(&mut self, width: u32, height: u32) {
        if self.gpu.sc_desc.width != width || self.gpu.sc_desc.height != height {
            self.gpu.sc_desc.width = width;
            self.gpu.sc_desc.height = height;

            self.gpu.swap_chain = self
                .gpu
                .device
                .create_swap_chain(&self.gpu.surface, &self.gpu.sc_desc);
            self.general_renderer.resize(&self.gpu);
        }
    }

    #[allow(dead_code)]
    pub fn view_transformation(&self) -> Matrix4<f32> {
        self.mx_view
    }

    pub fn set_view_transformation(&mut self, camera2d_transform2d: &Transform2D) {
        self.mx_view = camera2d_transform2d
            .to_homogeneous_3d()
            .try_inverse()
            .unwrap()
    }

    #[allow(dead_code)]
    pub fn projection(&self) -> Matrix4<f32> {
        self.mx_projection
    }

    pub fn set_projection(&mut self, camera2d: &Camera2D) {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let opengl_to_wgpu_matrix: Matrix4<f32> = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        );

        self.mx_projection = opengl_to_wgpu_matrix * camera2d.to_orthographic_homogeneous()
    }

    #[allow(dead_code)]
    pub fn viewport_aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn set_viewport_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio.abs();
    }

    pub fn begin_draw(&mut self) {
        if self.gpu.frame.is_none() {
            match self.gpu.swap_chain.get_current_frame() {
                Ok(sw_frame) => self.gpu.frame = Some(sw_frame),
                Err(err) => panic!("ERR: {}", err),
            }
        } else {
            panic!("ERR: Drawing has begun already.")
        }
    }

    pub fn draw_geometry(&mut self, world: &mut World, time: &Time) {
        let (width, height) = self.swap_chain_size();
        let viewport = Viewport::new_in_screen(width as f32, height as f32, self.aspect_ratio);

        self.general_renderer.render_geometry(
            &mut self.gpu,
            world,
            &self.mx_view,
            &self.mx_projection,
            &viewport,
            time,
        )
    }

    #[allow(dead_code)]
    pub fn clear(&mut self, clear_color: &Rgba) {
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &(self.gpu.frame.as_ref().unwrap().output.view),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color.to_wgpu_color()),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }

        self.gpu.queue.submit(Some(encoder.finish()));
    }

    pub fn finish_draw(&mut self) {
        if self.gpu.frame.is_some() {
            self.gpu.frame.take();
        } else {
            panic!("ERR: Drawing has ended already.")
        }
    }

    // NOTE: 临时性代码
    pub fn recompile_shader(&mut self) {
        // 1. 定时检查shader, 若有改变, 记录hash, 进入下一步
        // 2. 编译所有shader(因为现在只有两个)
        //  通过: 将编译后的字节码传入general_renderer
        //  失败: 打印错误, 返回等待下一次更新
        use shaderc::*;
        use std::collections::hash_map::DefaultHasher;
        use std::fs::read_to_string;
        use std::hash::{Hash, Hasher};

        let current_dir = std::env::current_dir().unwrap();
        let geometry_vert_path = current_dir.join("assets\\shaders\\geometry\\geometry.vert");
        let geometry_frag_path = current_dir.join("assets\\shaders\\geometry\\geometry.frag");

        let vcontent: String;
        let fcontent: String;

        if let Ok(content) = read_to_string(&geometry_vert_path) {
            vcontent = content;
        } else {
            println!(
                "WARN: Cannot get geometry vertex shader in {}.",
                geometry_vert_path.display()
            );
            return;
        }
        if let Ok(content) = read_to_string(&geometry_frag_path) {
            fcontent = content;
        } else {
            println!(
                "WARN: Cannot get geometry fragment shader in {}.",
                geometry_frag_path.display()
            );
            return;
        }

        let mut dh = DefaultHasher::new();

        vcontent.hash(&mut dh);
        let vhash = dh.finish();
        fcontent.hash(&mut dh);
        let fhash = dh.finish();

        if self.vhash == vhash && self.fhash == fhash {
            return;
        }

        self.vhash = vhash;
        self.fhash = fhash;

        if let Some(mut compiler) = Compiler::new() {
            let vs = compiler.compile_into_spirv(
                &vcontent,
                ShaderKind::Vertex,
                geometry_vert_path.to_str().unwrap(),
                "main",
                None,
            );
            let fs = compiler.compile_into_spirv(
                &&fcontent,
                ShaderKind::Fragment,
                geometry_frag_path.to_str().unwrap(),
                "main",
                None,
            );

            if let Err(err) = vs {
                println!("{}", err);
                return;
            }
            if let Err(err) = fs {
                println!("{}", err);
                return;
            }

            self.general_renderer.recompile_shader(
                &self.gpu,
                vs.unwrap().as_binary_u8(),
                fs.unwrap().as_binary_u8(),
            );
        } else {
            println!("WARN: Fail to create shader compiler.");
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Viewport {
    pub x: f32,
    pub y: f32,

    pub width: f32,
    pub height: f32,

    pub min_depth: f32,
    pub max_depth: f32,
}

impl Viewport {
    pub fn new_in_screen(screen_width: f32, screen_height: f32, aspect_ratio: f32) -> Self {
        let screen_ratio = screen_width / screen_height;

        let (x, y, width, height) = if aspect_ratio <= screen_ratio {
            (
                (screen_width - aspect_ratio * screen_height) / 2.0,
                0f32,
                aspect_ratio * screen_height,
                screen_height,
            )
        } else {
            (
                0f32,
                (screen_height - screen_width / aspect_ratio) / 2.0,
                screen_width,
                screen_width / aspect_ratio,
            )
        };

        Self {
            x,
            y,
            width,
            height,
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
            0.5 * self.width,   0.0,                0.0,                                0.5 * self.width + self.x,
            0.0,                -0.5 * self.height, 0.0,                                0.5 * self.height + self.y,
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