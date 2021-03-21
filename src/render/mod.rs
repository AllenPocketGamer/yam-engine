mod renderers;

use renderers::{GeneralRenderer, SpriteRenderer};

use crate::{
    app::{AppStage, AppStageBuilder},
    components::{camera::Camera2D, sprite::Sprite, transform::Transform2D, Instance},
    legion::{IntoQuery, Resources, World},
    misc::color::Rgba,
    nalgebra::Matrix4,
    window::Window,
};

pub(crate) fn create_app_stage_render(Window { window }: &Window) -> AppStage {
    let mut r2ds = Render2D::new(window);

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

        r2ds.begin_draw();

        // TODO: move query to `Render2D`.
        for (transform2d, sprite) in query_sprites.iter(world) {
            r2ds.draw_sprite_in_world_space(transform2d, sprite);
        }

        // TODO: move query to `Render2D`.
        for (transform2ds, sprite) in query_sprites_instanced.iter(world) {
            r2ds.draw_sprites_in_world_space(transform2ds, sprite);
        }

        r2ds.draw_geometry(world);

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
            format: adapter.get_swap_chain_preferred_format(&surface),
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

pub struct Render2D {
    gpu: Gpu,
    sprite_renderer: SpriteRenderer,
    general_renderer: GeneralRenderer,

    aspect_ratio: f32,
    mx_view: Matrix4<f32>,
    mx_projection: Matrix4<f32>,
}

impl Render2D {
    pub fn new(window: &winit::window::Window) -> Self {
        let mut gpu = futures::executor::block_on(Gpu::new(window));
        let sprite_renderer = SpriteRenderer::new(&mut gpu);
        let general_renderer = GeneralRenderer::new(&mut gpu);

        let default_camera2d = Camera2D::default();
        let default_camera2d_transform2d = Transform2D::default();

        Self {
            gpu,
            sprite_renderer,
            general_renderer,

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

    pub fn draw_sprite_in_world_space(&mut self, transform2d: &Transform2D, sprite: &Sprite) {
        let viewport = self.calculate_adapted_viewport();

        self.sprite_renderer.render(
            &mut self.gpu,
            std::slice::from_ref(transform2d),
            &sprite.color,
            &self.mx_view,
            &self.mx_projection,
            &viewport,
        );
    }

    pub fn draw_sprites_in_world_space(&mut self, transform2ds: &[Transform2D], sprite: &Sprite) {
        let viewport = self.calculate_adapted_viewport();

        self.sprite_renderer.render(
            &mut self.gpu,
            transform2ds,
            &sprite.color,
            &self.mx_view,
            &self.mx_projection,
            &viewport,
        );
    }

    pub fn draw_geometry(&mut self, world: &mut World) {
        let viewport = self.calculate_adapted_viewport();

        self.general_renderer.render_geometry(
            &mut self.gpu,
            world,
            &self.mx_view,
            &self.mx_projection,
            &viewport,
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
