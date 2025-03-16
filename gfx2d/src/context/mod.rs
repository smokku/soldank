use super::*;
use miniquad::*;
use std::{mem, str};

mod pipeline;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [u8; 4],
}

pub fn vertex(pos: glam::Vec2, uv: glam::Vec2, color: Color) -> Vertex {
    Vertex {
        pos: [pos.x, pos.y],
        uv: [uv.x, uv.y],
        color: [color.r, color.g, color.b, color.a],
    }
}

pub struct Gfx2dContext {
    pipeline: Pipeline,
    bindings: Bindings,
    white_texture: TextureId,
}

impl Gfx2dContext {
    pub fn new(ctx: &mut Context) -> Gfx2dContext {
        let white_texture = ctx.new_texture_from_rgba8(1, 1, &[255, 255, 255, 255]);

        let shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: pipeline::VERT_SOURCE,
                    fragment: pipeline::FRAG_SOURCE,
                },
                pipeline::meta(),
            )
            .unwrap();

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_position", VertexFormat::Float2),
                VertexAttribute::new("in_texcoords", VertexFormat::Float2),
                VertexAttribute::new("in_color", VertexFormat::Byte4),
            ],
            shader,
            pipeline::params(),
        );
        let bindings = Bindings {
            vertex_buffers: vec![],
            index_buffer: ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Stream,
                BufferSource::empty::<u32>(0),
            ),
            images: vec![white_texture],
        };

        Gfx2dContext {
            pipeline,
            bindings,
            white_texture,
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, slice: &mut DrawSlice, transform: &Mat2d) {
        ctx.apply_pipeline(&self.pipeline);

        slice.batch.update(ctx);
        let buffer = slice.buffer();
        self.bindings.vertex_buffers = vec![buffer];

        let uniforms = pipeline::Uniforms {
            transform: glam::Mat4::from_cols_array_2d(&[
                [(transform.0).0, (transform.1).0, 0.0, 0.0],
                [(transform.0).1, (transform.1).1, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [(transform.0).2, (transform.1).2, 0.0, 1.0],
            ]),
        };
        ctx.apply_uniforms(UniformsSource::table(&uniforms));

        let mut draws: Vec<(TextureId, Vec<u32>)> = Vec::new();
        for cmd in slice.commands() {
            let indices = cmd
                .vertex_range
                .clone()
                .map(|i| i as u32)
                .collect::<Vec<u32>>();
            let texture = match cmd.texture {
                None => self.white_texture,
                Some(t) => t,
            };
            if let Some(found) = draws.iter_mut().find(|(tex, _)| *tex == texture) {
                found.1.extend(indices);
            } else {
                draws.push((texture, indices));
            };
        }

        for (texture, indices) in draws.iter() {
            let size = indices.len() * mem::size_of::<u32>();
            let mut delete_buffer = None;
            if ctx.buffer_size(self.bindings.index_buffer) < size {
                delete_buffer.replace(self.bindings.index_buffer);
                self.bindings.index_buffer = ctx.new_buffer(
                    BufferType::IndexBuffer,
                    BufferUsage::Stream,
                    BufferSource::empty::<u32>(size),
                );
            };
            ctx.buffer_update(
                self.bindings.index_buffer,
                BufferSource::slice(indices.as_slice()),
            );
            self.bindings.images[0] = *texture;

            ctx.apply_bindings(&self.bindings);
            ctx.draw(0, indices.len() as i32, 1);

            if let Some(buffer) = delete_buffer {
                ctx.delete_buffer(buffer);
            }
        }
    }
}
