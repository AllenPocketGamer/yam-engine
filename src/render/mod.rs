// TODO: 提供一个最基础的Render, 在屏幕中画个自旋正方体
// Render是AppStage, 会被插入App中
// Render需要Window的引用, 所有_window需要包装传入Resources中

// 可能需要lazy_static, 因为渲染器应该在System间是共用的
// emmm, 或许可以用Resources, 但这样渲染器就会暴露到用户层

use crate::{app::{AppStage, AppStageBuilder}, input::Input, window::{self, Window}};
use futures::executor::block_on;
use legion::{Resources, World};
use legion_codegen::system;

pub(crate) fn create_app_stage_render() -> AppStage {
    AppStageBuilder::new(String::from("default_render"))
        .add_thread_local_fn_startup(init_resources)
        .add_thread_local_fn_process(temp_render)
        .build()
}

fn init_resources(_world: &mut World, resources: &mut Resources) {
    resources.insert({
        let window = resources
            .get::<Window>()
            .expect("Not found resource window.");
        block_on(Renderer::new(&window))
    });
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

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
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

        let swap_chain = device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                format: adapter.get_swap_chain_preferred_format(&surface),
                width: window.inner_size().width,
                height: window.inner_size().height,
                present_mode: wgpu::PresentMode::Mailbox, // NODE: to understand
            },
        );

        Self {
            surface,
            device,
            queue,
            swap_chain,
        }
    }
}
