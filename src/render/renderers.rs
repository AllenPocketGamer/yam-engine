use super::Gpu;

use crate::{components::transform::Transform2D, misc::color::Rgba, nalgebra::Matrix4};

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
        let vertex_size = 4 * 4;

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
                        array_stride: 3 * 8,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![0 => Float2, 1 => Float2, 2 => Float2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: vertex_size,
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

struct GeometryRenderer {
    // TODO!
}

impl GeometryRenderer {
    // TODO!
}
