use super::{Gpu, Render2D, Viewport};

use crate::{
    components::{
        geometry::{Assembly, Geometry2D},
        transform::Transform2D,
    },
    legion::{IntoQuery, Resources, World},
    misc::color::Rgba,
    nalgebra::Vector4,
    Instance,
};

use std::mem::size_of;

#[rustfmt::skip] const MILLION:                 usize = 1 << 20;

#[rustfmt::skip] const MAX_TRANSFORM2D_COUNT:   usize = 2 * MILLION;
#[rustfmt::skip] const MAX_GEOMETRY_COUNT:      usize = 2 * MILLION;
#[rustfmt::skip] const MAX_INDEX_PAIR_COUNT:    usize = 2 * MILLION;

#[rustfmt::skip] const TRANSFORM2D_BUF_SIZE:    u64 = (size_of::<Transform2D>() * MAX_TRANSFORM2D_COUNT) as u64;
#[rustfmt::skip] const GEOMETRY_BUF_SIZE:       u64 = (size_of::<Geometry2D>() * MAX_GEOMETRY_COUNT) as u64;
#[rustfmt::skip] const INDEX_PAIR_BUF_SIZE:     u64 = (size_of::<(u32, u32)>() * MAX_INDEX_PAIR_COUNT) as u64;

/// Renderer which renders `Sprite` and `Geometry` in the best performance.
///
/// **NOTE: In the experimental stage now!**
pub struct GeneralRenderer {
    instance_buf: wgpu::Buffer,
    /// Store `Transform2D` data and `Geometry` data.
    ///
    /// Default size: `TRANSFORM2D_BUF_SIZE + GEOMETRY_BUF_SIZE`.
    storage_buf: wgpu::Buffer,

    // For `Geometry` rendering.
    geometry_bind_group: wgpu::BindGroup,
    geometry_pipeline: wgpu::RenderPipeline,
    // TODO: For `Sprite` rendering.
    // sprite_bind_group: wgpu::BindGroup,
    // sprite_pipeline: wgpu::RenderPipeline,
}

impl GeneralRenderer {
    pub(super) fn new(r2d: &Render2D) -> Self {
        let Gpu {
            device, sc_desc, ..
        } = &r2d.gpu;

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

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("geometry bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::VERTEX,
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
                        buffer: &r2d.utility_buf,
                        offset: 0,
                        size: None,
                    },
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &storage_buf,
                        offset: 0,
                        size: wgpu::BufferSize::new(TRANSFORM2D_BUF_SIZE),
                    },
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &storage_buf,
                        offset: TRANSFORM2D_BUF_SIZE,
                        size: wgpu::BufferSize::new(GEOMETRY_BUF_SIZE),
                    },
                },
            ],
        });

        let geometry_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("geometry pipeline layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let vert_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("geometry vertex shader"),
            source: wgpu::util::make_spirv(include_bytes!(
                "../../assets/shaders/geometry/geometry.vert.spv"
            )),
            flags: wgpu::ShaderFlags::empty(),
        });

        let frag_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("geometry fragment shader"),
            source: wgpu::util::make_spirv(include_bytes!(
                "../../assets/shaders/geometry/geometry.frag.spv"
            )),
            flags: wgpu::ShaderFlags::empty(),
        });

        let geometry_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("geometry pipeline"),
            layout: Some(&geometry_pipeline_layout),
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
                        array_stride: size_of::<(u32, u32)>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![1 => Uint2],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendState {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendState {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Max,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
            }),
            multisample: Default::default(),
        });

        Self {
            instance_buf,
            storage_buf,

            geometry_bind_group,
            geometry_pipeline,
        }
    }

    pub(super) fn render(&mut self, r2d: &Render2D, world: &World, _resources: &Resources) {
        let Gpu {
            device,
            queue,
            frame,
            ..
        } = &r2d.gpu;

        let Viewport {
            x,
            y,
            w,
            h,
            min_depth,
            max_depth,
        } = r2d.viewport;

        let frame = frame
            .as_ref()
            .expect("ERR: Not call begin_draw on Render2DService.");

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("general encoder"),
        });

        let (i_count, i_buf_size) = self.copy_data_to_gpu(&mut encoder, r2d, world);

        encoder.insert_debug_marker("render geometry");
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("geometry render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &(frame.output.view),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Rgba::CAMEL.to_wgpu_color()),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &r2d.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            rpass.push_debug_group("Set datas");

            rpass.set_pipeline(&self.geometry_pipeline);
            rpass.set_viewport(x, y, w, h, min_depth, max_depth);
            rpass.set_vertex_buffer(0, r2d.quad_vertex_buf.slice(..));
            rpass.set_index_buffer(r2d.quad_index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(1, self.instance_buf.slice(0..i_buf_size));
            rpass.set_bind_group(0, &self.geometry_bind_group, &[]);

            rpass.pop_debug_group();

            rpass.draw_indexed(0..6, 0, 0..i_count as u32);
        }

        queue.submit(Some(encoder.finish()));
    }

    /// Collect `Transform2D`, `Geometry` and calculate `Index Pair`, then
    /// copy them to the memory of video card.
    ///
    /// Return instance count and instance size.
    ///
    /// #Panics
    ///
    /// Panic if
    ///     1. The number of `Transform2D` exceeds the limit: `MAX_TRANSFORM2D_COUNT`.
    ///     2. The number of `Geometry` exceeds the limit: `MAX_GEOMETRY_COUNT`.
    ///     3. The number of `Index Pair` exceeds the limit: `MAX_INDEX_PAIR_COUNT`.
    fn copy_data_to_gpu(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        r2d: &Render2D,
        world: &World,
    ) -> (usize, wgpu::BufferAddress) {
        let Gpu { device, .. } = &r2d.gpu;

        let t_st = 0;
        let g_st = TRANSFORM2D_BUF_SIZE;
        let i_st = TRANSFORM2D_BUF_SIZE + GEOMETRY_BUF_SIZE;

        let s_bs = r2d.staging_buf.slice(..);

        // Mapping main-memory to video-memory.
        {
            let s_ft = s_bs.map_async(wgpu::MapMode::Write);
            device.poll(wgpu::Maintain::Wait);
            futures::executor::block_on(s_ft).expect("ERR: map m-mem to v-mem.");
        }

        // Get `Transform2D` slice, `Geometry` slice, and `Index Pair` slice on the mapped buffer.
        let (t_slice, g_slice, i_slice) = unsafe {
            let s_mp = s_bs.get_mapped_range_mut().as_mut_ptr();

            let t_mp = s_mp.offset(t_st as isize) as *mut Transform2D;
            let g_mp = s_mp.offset(g_st as isize) as *mut Geometry2D;
            let i_mp = s_mp.offset(i_st as isize) as *mut (u32, u32);

            let t_len = TRANSFORM2D_BUF_SIZE as usize / size_of::<Transform2D>();
            let g_len = GEOMETRY_BUF_SIZE as usize / size_of::<Geometry2D>();
            let i_len = INDEX_PAIR_BUF_SIZE as usize / size_of::<(u32, u32)>();

            (
                std::slice::from_raw_parts_mut(t_mp, t_len),
                std::slice::from_raw_parts_mut(g_mp, g_len),
                std::slice::from_raw_parts_mut(i_mp, i_len),
            )
        };

        let mut t_count: usize = 0;
        let mut g_count: usize = 0;
        let mut i_count: usize = 0;

        // Copy `Transform2D` and `Geometry` data from `World` to the buffer which is mapped to staging_buf.
        unsafe {
            let mut q01 = <(&Transform2D, &Geometry2D)>::query();
            let mut q02 = <(&Transform2D, &Assembly)>::query();
            let mut q03 = <(&Instance<Transform2D>, &Geometry2D)>::query();
            let mut q04 = <(&Instance<Transform2D>, &Assembly)>::query();

            q01.for_each(world, |(t, g)| {
                *t_slice.get_unchecked_mut(t_count) = *t;
                *g_slice.get_unchecked_mut(g_count) = *g;
                *i_slice.get_unchecked_mut(i_count) = (t_count as u32, g_count as u32);

                t_count += 1;
                g_count += 1;
                i_count += 1;
            });

            q02.for_each(world, |(t, gs)| {
                *t_slice.get_unchecked_mut(t_count) = *t;

                let g_len = gs.len();

                let g_part = &mut g_slice[g_count..g_count + g_len];
                g_part.copy_from_slice(gs);

                for _ in 0..g_len {
                    *i_slice.get_unchecked_mut(i_count) = (t_count as u32, g_count as u32);

                    g_count += 1;
                    i_count += 1;
                }

                t_count += 1;
            });

            q03.for_each(world, |(ts, g)| {
                let t_len = ts.len();

                let t_part = &mut t_slice[t_count..t_count + t_len];
                t_part.copy_from_slice(ts);

                *g_slice.get_unchecked_mut(g_count) = *g;

                for _ in 0..t_len {
                    *i_slice.get_unchecked_mut(i_count) = (t_count as u32, g_count as u32);

                    t_count += 1;
                    i_count += 1;
                }

                g_count += 1;
            });

            q04.for_each(world, |(ts, gs)| {
                let t_len = ts.len();
                let g_len = gs.len();

                let t_part = &mut t_slice[t_count..t_count + t_len];
                let g_part = &mut g_slice[g_count..g_count + g_len];

                t_part.copy_from_slice(ts);
                g_part.copy_from_slice(gs);

                for t in 0..t_len {
                    for g in 0..g_len {
                        *i_slice.get_unchecked_mut(i_count) =
                            ((t_count + t) as u32, (g_count + g) as u32);

                        i_count += 1;
                    }
                }

                t_count += t_len;
                g_count += g_len;
            });

            if t_count > MAX_TRANSFORM2D_COUNT {
                panic!(
                    "ERR: The number of Transform2D exceeds the limit: {}",
                    MAX_TRANSFORM2D_COUNT
                );
            }

            if g_count > MAX_GEOMETRY_COUNT {
                panic!(
                    "ERR: The number of Geometry exceeds the limit: {}",
                    MAX_GEOMETRY_COUNT
                );
            }

            if i_count > MAX_INDEX_PAIR_COUNT {
                panic!(
                    "ERR: The number of Index_Pair exceeds the limit: {}",
                    MAX_INDEX_PAIR_COUNT
                );
            }
        }

        r2d.staging_buf.unmap();

        let t_buf_size = (t_count * size_of::<Transform2D>()) as wgpu::BufferAddress;
        let g_buf_size = (g_count * size_of::<Geometry2D>()) as wgpu::BufferAddress;
        let i_buf_size = (i_count * size_of::<(u32, u32)>()) as wgpu::BufferAddress;

        // Copy transform2d data from staging to storage.
        encoder.copy_buffer_to_buffer(&r2d.staging_buf, t_st, &self.storage_buf, 0, t_buf_size);
        // Copy geometry data from staging to storage.
        encoder.copy_buffer_to_buffer(&r2d.staging_buf, g_st, &self.storage_buf, g_st, g_buf_size);
        // Copy index pair data from staging to instance.
        encoder.copy_buffer_to_buffer(&r2d.staging_buf, i_st, &self.instance_buf, 0, i_buf_size);

        (i_count, i_buf_size)
    }
}
