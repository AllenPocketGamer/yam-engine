use super::Gpu;

use crate::{
    components::{
        geometry::{Assembly, Geometry},
        transform::Transform2D,
    },
    legion::{IntoQuery, World},
    misc::color::Rgba,
    nalgebra::{Matrix4, Vector4},
    Instance,
};

use wgpu::util::DeviceExt;

use std::mem::size_of;

// Quad vertex in world coordinate
const QUAD_VERTEX: [f32; 16] = [
    -0.5, 0.5, 0.0, 1.0, // left-top, point A
    0.5, 0.5, 0.0, 1.0, // right-top, point B
    0.5, -0.5, 0.0, 1.0, // right-bottom, point C
    -0.5, -0.5, 0.0, 1.0, // left-bottom, point D
];

const QUAD_INDEX: [u16; 6] = [
    0, 1, 2, // Face ABC
    2, 3, 0, // Face CDA
];

const MB: wgpu::BufferAddress = 1 << 20;

// 最多支持4_000_000个索引对， 2_000_000个Transform2D数, 2_000_000个Geometry数.
const TRANSFORM2D_BUF_SIZE: wgpu::BufferAddress = 2 * size_of::<Transform2D>() as u64 * MB;
const GEOMETRY_BUF_SIZE: wgpu::BufferAddress = 2 * size_of::<Geometry>() as u64 * MB;
const INDEX_PAIR_BUF_SIZE: wgpu::BufferAddress = 4 * size_of::<(u16, u16)>() as u64 * MB;

const STAGING_BUF_SIZE: wgpu::BufferAddress = 128 * MB;

pub(super) struct SpriteRenderer {
    // To store four vertex data(quad)
    vertex_buf: wgpu::Buffer,
    // To store index data of quad
    index_buf: wgpu::Buffer,
    // To store model transformation matrices
    instance_buf: wgpu::Buffer,
    // To store color data
    uniform_buf: wgpu::Buffer,

    // A group cotains uniform_buf, texture and sampler
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,

    staging_buf: wgpu::Buffer,
    staging_buf_size: wgpu::BufferSize,
}

impl SpriteRenderer {
    /// Max instance data size per render pass.
    const MAX_INSTANCE_DATA_SIZE: usize = size_of::<Transform2D>() * 1024 * 1024;

    pub(super) fn new(
        Gpu {
            surface,
            adapter,
            device,
            ..
        }: &mut Gpu,
    ) -> Self {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad vertex"),
            contents: bytemuck::cast_slice(&QUAD_VERTEX[..]),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad index"),
            contents: bytemuck::cast_slice(&QUAD_INDEX[..]),
            usage: wgpu::BufferUsage::INDEX,
        });

        let instance_buf_size = Self::MAX_INSTANCE_DATA_SIZE as wgpu::BufferAddress;

        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("model transformation matrices"),
            size: instance_buf_size,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging buffer"),
            size: instance_buf_size,
            usage: wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::COPY_SRC,
            mapped_at_creation: false,
        });

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("color"),
            contents: bytemuck::cast_slice(&Rgba::WHITE.to_hex().to_ne_bytes()[..]),
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
                    min_binding_size: wgpu::BufferSize::new(4),
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
                range: 0..2 * 4 * 16,
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
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: size_of::<Transform2D>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![0 => Float2, 1 => Float2, 2 => Float2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: size_of::<Vector4<f32>>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![3 => Float4],
                    },
                ],
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
            instance_buf,

            bind_group,
            pipeline,

            staging_buf,
            staging_buf_size: wgpu::BufferSize::new(instance_buf_size).unwrap(),
        }
    }

    /// Only support `Self::INSTANCE_MAX_COUNT` transform2d once call.
    pub(super) fn render(
        &mut self,
        Gpu {
            device,
            queue,
            frame,
            ..
        }: &mut Gpu,
        transform2ds: &[Transform2D],
        color: &Rgba,
        mx_view: &Matrix4<f32>,
        mx_projection: &Matrix4<f32>,
        viewport: &(f32, f32, f32, f32, f32, f32),
    ) {
        let instance_data_size =
            (size_of::<Transform2D>() * transform2ds.len()) as wgpu::BufferAddress;

        // Change stage_buf bigger to store the instances data.
        if instance_data_size > self.staging_buf_size.get() {
            self.staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("staging buffer"),
                size: instance_data_size,
                usage: wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::COPY_SRC,
                mapped_at_creation: false,
            });
            self.staging_buf_size = wgpu::BufferSize::new(instance_data_size).unwrap();
        }

        // Transfer data from m-mem to v-mem in asynchronous.
        let buf_slice = self.staging_buf.slice(0..instance_data_size);
        let future = buf_slice.map_async(wgpu::MapMode::Write);
        device.poll(wgpu::Maintain::Wait);
        futures::executor::block_on(future).expect("ERR: transfer data from m-mem to v-mem.");

        // TODO: 拷贝操作会idle处理器, 造成性能浪费; 可以尝试使用DMA技术来进行memcpy, 但有可能造成数据不一致;
        //   可以用unsafe来绕过rust的安全检查, 但可能会有UB; 所以这是一个安全和性能间的权衡问题;
        //   可以做一个unsafe选项, 开启来提高性能(比较可观), 并在注释中标注出它的危险性!
        buf_slice
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(transform2ds));
        self.staging_buf.unmap();

        // Write color to uniform buffer.
        queue.write_buffer(
            &self.uniform_buf,
            0,
            bytemuck::cast_slice(&color.to_hex().to_ne_bytes()[..]),
        );

        let frame = frame
            .as_ref()
            .expect("ERR: Not call begin_draw on Render2DService.");

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("sprite encoder"),
        });

        for offset in (0..instance_data_size).step_by(Self::MAX_INSTANCE_DATA_SIZE) {
            let copy_size = std::cmp::min(
                instance_data_size - offset,
                Self::MAX_INSTANCE_DATA_SIZE as u64,
            );

            let instance_count = copy_size / size_of::<Transform2D>() as u64;

            // copy transform2ds data to instance buffer.
            encoder.copy_buffer_to_buffer(
                &self.staging_buf,
                offset,
                &self.instance_buf,
                0,
                copy_size,
            );

            {
                // TODO: 这里应该可以尝试用Bundle优化, 毕竟也有将近0.2-0.3ms的耗时; 但实在鸡肋, 所以闲着没事的时候可以搞搞.
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("sprite render pass"),
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &(frame.output.view),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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
                rpass.set_vertex_buffer(0, self.instance_buf.slice(0..copy_size));
                rpass.set_vertex_buffer(1, self.vertex_buf.slice(..));
                rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
                rpass.set_bind_group(0, &self.bind_group, &[]);

                // Set view transformation matrix + projection matrix
                rpass.set_push_constants(
                    wgpu::ShaderStage::VERTEX,
                    0,
                    bytemuck::cast_slice(mx_view.as_slice()),
                );
                rpass.set_push_constants(
                    wgpu::ShaderStage::VERTEX,
                    4 * 16,
                    bytemuck::cast_slice(mx_projection.as_slice()),
                );

                rpass.pop_debug_group();

                rpass.insert_debug_marker("draw");
                rpass.draw_indexed(0..6, 0, 0..instance_count as u32);
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}

/// Renderer which renders `Sprite` and `Geometry` in the best performance.
///
/// **NOTE: In the experimental stage now!**
pub struct GeneralRenderer {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    // Store index data(transform2d index and geometry index).
    //
    // Default Size: `INDEX_PAIR_BUF_SIZE`
    instance_buf: wgpu::Buffer,
    // Store `Transform2D` data and `Geometry` data.
    //
    // Default size: `TRANSFORM2D_BUF_SIZE + GEOMETRY_BUF_SIZE`.
    storage_buf: wgpu::Buffer,
    /// Transfer data from m-mem to v-mem.
    ///
    /// Default size: `STAGING_BUF_SIZE`.
    ///
    /// Default layout:
    ///
    /// 1. Geometry memory layout:
    ///  * 0-48MB: transform2ds
    ///  * 49-112MB: geometries
    ///  * 113-128MB: index pairs
    staging_buf: wgpu::Buffer,

    // For `Geometry` rendering.
    geometry_bind_group: wgpu::BindGroup,
    geometry_pipeline: wgpu::RenderPipeline,
    // TODO: For `Sprite` rendering.
    // sprite_bind_group: wgpu::BindGroup,
    // sprite_pipeline: wgpu::RenderPipeline,
}

impl GeneralRenderer {
    pub(super) fn new(
        Gpu {
            surface,
            adapter,
            device,
            ..
        }: &mut Gpu,
    ) -> Self {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&QUAD_VERTEX[..]),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&QUAD_INDEX[..]),
            usage: wgpu::BufferUsage::INDEX,
        });

        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buffer"),
            size: INDEX_PAIR_BUF_SIZE,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("storage buffer"),
            size: TRANSFORM2D_BUF_SIZE + GEOMETRY_BUF_SIZE,
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging buffer"),
            size: STAGING_BUF_SIZE,
            usage: wgpu::BufferUsage::MAP_WRITE | wgpu::BufferUsage::COPY_SRC,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("geometry bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let geometry_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("geometry bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &storage_buf,
                        offset: 0,
                        size: wgpu::BufferSize::new(TRANSFORM2D_BUF_SIZE),
                    },
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &storage_buf,
                        offset: TRANSFORM2D_BUF_SIZE,
                        size: wgpu::BufferSize::new(GEOMETRY_BUF_SIZE),
                    },
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("geometry pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStage::VERTEX,
                range: 0..2 * 4 * 16, // view transformation + projection
            }],
        });

        let vert_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("sprite vertex shader"),
            source: wgpu::util::make_spirv(include_bytes!("geometry_shader.vert.spv")),
            flags: wgpu::ShaderFlags::empty(),
        });

        let frag_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("sprite fragment shader"),
            source: wgpu::util::make_spirv(include_bytes!("geometry_shader.frag.spv")),
            flags: wgpu::ShaderFlags::VALIDATION,
        });

        let geometry_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("geometry pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: "main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: size_of::<Vector4<f32>>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float4],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: size_of::<(u16, u16)>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![1 => Ushort2],
                    },
                ],
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
            instance_buf,
            storage_buf,
            staging_buf,

            geometry_bind_group,
            geometry_pipeline,
        }
    }

    pub(super) fn render_geometry(
        &mut self,
        Gpu {
            device,
            queue,
            frame,
            ..
        }: &mut Gpu,
        world: &mut World,
        mx_view: &Matrix4<f32>,
        mx_proj: &Matrix4<f32>,
        vp: &(f32, f32, f32, f32, f32, f32),
    ) {
        let frame = frame
            .as_ref()
            .expect("ERR: Not call begin_draw on Render2DService.");

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("general encoder"),
        });

        // NOTE: Collect `Transform2D` and `Geometry` data from `World`,
        //  then transfer to the v-mem of video card.
        let instance_count = self.transfer_geometry_datas(device, &mut encoder, world);

        encoder.insert_debug_marker("render geometry");
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("geometry render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &(frame.output.view),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.push_debug_group("prepare render data.");

            rpass.set_viewport(vp.0, vp.1, vp.2, vp.3, vp.4, vp.5);
            // Set view transformation matrix + projection matrix
            rpass.set_push_constants(
                wgpu::ShaderStage::VERTEX,
                0,
                bytemuck::cast_slice(mx_view.as_slice()),
            );
            rpass.set_push_constants(
                wgpu::ShaderStage::VERTEX,
                64,
                bytemuck::cast_slice(mx_proj.as_slice()),
            );

            rpass.set_pipeline(&self.geometry_pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(1, self.instance_buf.slice(0..instance_count as u64));
            rpass.set_bind_group(0, &self.geometry_bind_group, &[]);

            rpass.pop_debug_group();

            rpass.draw_indexed(0..6, 0, 0..instance_count as u32);
        }

        queue.submit(Some(encoder.finish()));
    }

    /// Return instance count.
    fn transfer_geometry_datas(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
    ) -> usize {
        let (t_st, t_ed) = (0, TRANSFORM2D_BUF_SIZE);
        let (g_st, g_ed) = (t_ed, t_ed + GEOMETRY_BUF_SIZE);
        let (i_st, i_ed) = (g_ed, g_ed + INDEX_PAIR_BUF_SIZE);

        let t_bs = self.staging_buf.slice(t_st..t_ed);
        let g_bs = self.staging_buf.slice(g_st..g_ed);
        let i_bs = self.staging_buf.slice(i_st..i_ed);

        let t_ft = t_bs.map_async(wgpu::MapMode::Write);
        let g_ft = g_bs.map_async(wgpu::MapMode::Write);
        let i_ft = i_bs.map_async(wgpu::MapMode::Write);

        device.poll(wgpu::Maintain::Wait);
        futures::executor::block_on(t_ft).expect("ERR: map transform2d buffer slice.");
        futures::executor::block_on(g_ft).expect("ERR: map geometry buffer slice.");
        futures::executor::block_on(i_ft).expect("ERR: map index pair buffer slice.");

        let mut q01 = <(&Transform2D, &Geometry)>::query();
        let mut q02 = <(&Transform2D, &Assembly)>::query();
        let mut q03 = <(&Instance<Transform2D>, &Geometry)>::query();
        let mut q04 = <(&Instance<Transform2D>, &Assembly)>::query();

        let (t_slice, g_slice, i_slice) = unsafe {
            let t_mp = t_bs.get_mapped_range_mut().as_mut_ptr() as *mut Transform2D;
            let g_mp = g_bs.get_mapped_range_mut().as_mut_ptr() as *mut Geometry;
            let i_mp = i_bs.get_mapped_range_mut().as_mut_ptr() as *mut (u16, u16);

            let t_len = TRANSFORM2D_BUF_SIZE as usize / size_of::<Transform2D>();
            let g_len = GEOMETRY_BUF_SIZE as usize / size_of::<Transform2D>();
            let i_len = INDEX_PAIR_BUF_SIZE as usize / size_of::<(u16, u16)>();

            (
                std::slice::from_raw_parts_mut(t_mp, t_len),
                std::slice::from_raw_parts_mut(g_mp, g_len),
                std::slice::from_raw_parts_mut(i_mp, i_len),
            )
        };

        let mut t_index: usize = 0;
        let mut g_index: usize = 0;
        let mut i_index: usize = 0;

        unsafe {
            q01.for_each(world, |(t, g)| {
                *t_slice.get_unchecked_mut(t_index) = *t;
                *g_slice.get_unchecked_mut(g_index) = *g;
                *i_slice.get_unchecked_mut(i_index) = (t_index as u16, g_index as u16);

                t_index += 1;
                g_index += 1;
                i_index += 1;
            });

            q02.for_each(world, |(t, gs)| {
                *t_slice.get_unchecked_mut(t_index) = *t;

                let g_len = gs.len();

                let g_part = &mut g_slice[g_index..g_index + g_len];
                g_part.copy_from_slice(gs);

                for _ in 0..g_len {
                    *i_slice.get_unchecked_mut(i_index) = (t_index as u16, g_index as u16);

                    g_index += 1;
                    i_index += 1;
                }

                t_index += 1;
            });

            q03.for_each(world, |(ts, g)| {
                let t_len = ts.len();

                let t_part = &mut t_slice[t_index..t_index + t_len];
                t_part.copy_from_slice(ts);

                *g_slice.get_unchecked_mut(g_index) = *g;

                for _ in 0..t_len {
                    *i_slice.get_unchecked_mut(i_index) = (t_index as u16, g_index as u16);

                    t_index += 1;
                    i_index += 1;
                }

                g_index += 1;
            });

            q04.for_each(world, |(ts, gs)| {
                let t_len = ts.len();
                let g_len = gs.len();

                let t_part = &mut t_slice[t_index..t_index + t_len];
                let g_part = &mut g_slice[g_index..g_index + g_len];

                t_part.copy_from_slice(ts);
                g_part.copy_from_slice(gs);

                for _ in 0..t_len {
                    for _ in 0..g_len {
                        *i_slice.get_unchecked_mut(i_index) = (t_index as u16, g_index as u16);

                        g_index += 1;
                        i_index += 1;
                    }

                    t_index += 1;
                }
            });
        }

        self.staging_buf.unmap();

        let t_count = t_index + 1;
        let g_count = g_index + 1;
        let i_count = i_index + 1;

        let t_size = (t_count * size_of::<Transform2D>()) as wgpu::BufferAddress;
        let g_size = (g_count * size_of::<Geometry>()) as wgpu::BufferAddress;
        let i_size = (i_count * size_of::<(u16, u16)>()) as wgpu::BufferAddress;

        // Copy transform2d data from staging to storage.
        encoder.copy_buffer_to_buffer(&self.staging_buf, t_st, &self.storage_buf, t_st, t_size);
        // Copy geometry data from staging to storage.
        encoder.copy_buffer_to_buffer(&self.staging_buf, g_st, &self.storage_buf, g_ed, g_size);
        // Copy index pair data from staging to instance.
        encoder.copy_buffer_to_buffer(&self.staging_buf, i_st, &self.instance_buf, 0, i_size);

        i_count
    }
}
