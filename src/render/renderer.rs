use wgpu::util::DeviceExt;

struct Gpu {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
}

impl Gpu {
    fn new() -> Self {
        todo!()
    }
}

struct SpriteRenderer {
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
    const QUAD_HALF_SIZE: usize = 32;

    // Quad vertex in world coordinate
    #[allow(dead_code)]
    const QUAD_VERTEX: [f32; 16] = [
        // right-top, point A
        Self::QUAD_HALF_SIZE as f32,
        Self::QUAD_HALF_SIZE as f32,
        0.0,
        1.0,
        // right-bottom, point B
        Self::QUAD_HALF_SIZE as f32,
        -(Self::QUAD_HALF_SIZE as f32),
        0.0,
        1.0,
        // left-bottom, point C
        -(Self::QUAD_HALF_SIZE as f32),
        -(Self::QUAD_HALF_SIZE as f32),
        0.0,
        1.0,
        // left-top, point D
        -(Self::QUAD_HALF_SIZE as f32),
        Self::QUAD_HALF_SIZE as f32,
        0.0,
        1.0,
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
        let uniform_size = 4 * 9 + 4 * 9 + 4 * 16;

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad vertex"),
            contents: bytemuck::cast_slice(&Self::QUAD_VERTEX[..]),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad index"),
            contents: bytemuck::cast_slice(&Self::QUAD_VERTEX[..]),
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

        let mut flags = wgpu::ShaderFlags::VALIDATION;
        match adapter.get_info().backend {
            wgpu::Backend::Vulkan | wgpu::Backend::Metal => {
                flags |= wgpu::ShaderFlags::EXPERIMENTAL_TRANSLATION
            }
            _ => {}
        }

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("sprite shader module"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "sprite_shader.wgsl"
            ))),
            flags,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_size,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[adapter.get_swap_chain_preferred_format(&surface).into()],
            }),
            primitive: Default::default(),
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
}
