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
    white_texture: Texture,
}

impl Gfx2dContext {
    pub fn new(ctx: &mut Context) -> Gfx2dContext {
        let shader = Shader::new(
            ctx,
            pipeline::VERT_SOURCE,
            pipeline::FRAG_SOURCE,
            pipeline::meta(),
        )
        .unwrap();

        let pipeline = Pipeline::with_params(
            ctx,
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
            index_buffer: Buffer::stream(ctx, BufferType::IndexBuffer, 0),
            images: vec![],
        };

        Gfx2dContext {
            pipeline,
            bindings,
            white_texture: Texture::from_rgba8(ctx, 1, 1, &[255, 255, 255, 255]),
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
        ctx.apply_uniforms(&uniforms);

        let mut draws: Vec<(Texture, Vec<u16>)> = Vec::new();
        for cmd in slice.commands() {
            let indices = cmd
                .vertex_range
                .clone()
                .map(|i| i as u16)
                .collect::<Vec<u16>>();
            let texture = match cmd.texture {
                None => self.white_texture,
                Some(t) => t,
            };
            if let Some(found) = draws
                .iter_mut()
                .find(|(tex, _)| tex.gl_internal_id() == texture.gl_internal_id())
            {
                found.1.extend(indices);
            } else {
                draws.push((texture, indices));
            };
        }

        for (texture, indices) in draws.iter() {
            let size = indices.len() * mem::size_of::<u16>();
            let mut delete_buffer = None;
            if self.bindings.index_buffer.size() < size {
                delete_buffer.replace(self.bindings.index_buffer);
                self.bindings.index_buffer = Buffer::stream(ctx, BufferType::IndexBuffer, size);
            };
            self.bindings.index_buffer.update(ctx, indices.as_slice());
            self.bindings.images = vec![*texture];

            ctx.apply_bindings(&self.bindings);
            ctx.draw(0, indices.len() as i32, 1);

            if let Some(buffer) = delete_buffer {
                buffer.delete();
            }
        }
    }
}
