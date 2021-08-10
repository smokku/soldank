use super::*;
use miniquad::*;
use std::str;

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

        Gfx2dContext {
            pipeline,
            white_texture: Texture::from_rgba8(ctx, 1, 1, &[255, 255, 255, 255]),
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, slice: &mut DrawSlice, transform: &Mat2d) {
        slice.batch.update(ctx);

        let buffer = slice.buffer();

        let uniforms = pipeline::Uniforms {
            transform: glam::Mat4::from_cols_array_2d(&[
                [(transform.0).0, (transform.1).0, 0.0, 0.0],
                [(transform.0).1, (transform.1).1, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [(transform.0).2, (transform.1).2, 0.0, 1.0],
            ]),
        };

        for cmd in slice.commands() {
            let indices = cmd
                .vertex_range
                .clone()
                .map(|i| i as u16)
                .collect::<Vec<u16>>();
            let bindings = Bindings {
                vertex_buffers: vec![buffer],
                index_buffer: Buffer::immutable(ctx, BufferType::IndexBuffer, indices.as_slice()),
                images: vec![match cmd.texture {
                    None => self.white_texture,
                    Some(t) => t,
                }],
            };

            ctx.apply_pipeline(&self.pipeline);
            ctx.apply_bindings(&bindings);

            ctx.apply_uniforms(&uniforms);
            ctx.draw(0, indices.len() as i32, 1);
        }
    }
}
