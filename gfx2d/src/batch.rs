use super::*;
use gfx::Factory;
use gfx::traits::FactoryExt;
use std::ops::Range;

fn batch_command(texture: Option<&Texture>, vertex_range: Range<usize>) -> BatchCommand {
    BatchCommand {
        texture: texture.cloned(),
        vertex_range,
    }
}

#[derive(Debug, Clone)]
pub struct BatchCommand {
    pub texture: Option<Texture>,
    pub vertex_range: Range<usize>,
}

#[derive(Debug, Copy, Clone)]
enum BatchUsage {
    Dynamic,
    Static,
}

#[derive(Debug, Clone)]
pub struct DrawBatch {
    vbuf: Option<VertexBuffer>,
    buf: Vec<Vertex>,
    cmds: Vec<BatchCommand>,
    split_start: usize,
    usage: BatchUsage,
    updated: bool,
}

#[derive(Debug)]
pub struct DrawSlice<'a> {
    pub batch: &'a mut DrawBatch,
    pub cmd_range: Range<usize>,
}

impl<'a> DrawSlice<'a> {
    pub fn buffer(&self) -> VertexBuffer {
        self.batch.buffer()
    }

    pub fn commands(&self) -> &[BatchCommand] {
        self.batch.commands(self.cmd_range.clone())
    }
}

impl ::std::default::Default for DrawBatch {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawBatch {
    pub fn new() -> DrawBatch {
        Self::with_usage(BatchUsage::Dynamic)
    }

    pub fn new_static() -> DrawBatch {
        Self::with_usage(BatchUsage::Static)
    }

    fn with_usage(usage: BatchUsage) -> DrawBatch {
        DrawBatch {
            vbuf: None,
            buf: Vec::new(),
            cmds: Vec::new(),
            split_start: 0,
            usage,
            updated: false,
        }
    }

    pub fn clear(&mut self) {
        self.updated = false;
        self.split_start = 0;
        self.buf.clear();
        self.cmds.clear();
    }

    pub fn add(&mut self, texture: Option<&Texture>, vertices: &[Vertex; 3]) {
        let i = self.buf.len();
        let n = vertices.len();
        let m = self.cmds.len();

        self.updated = false;
        self.buf.extend_from_slice(vertices);

        if m == 0 || m == self.split_start
            || (m > 0
                && (texture.is_none() != self.last_texture().is_none()
                    || texture.is_some() && texture.unwrap().is(self.last_texture().unwrap())))
        {
            self.cmds.push(batch_command(texture, i..i + n));
        } else {
            self.cmds.last_mut().unwrap().vertex_range.end += n;
        }
    }

    fn last_texture(&self) -> Option<&Texture> {
        match self.cmds.last() {
            None => None,
            Some(cmd) => cmd.texture.as_ref(),
        }
    }

    pub fn add_quad(&mut self, texture: Option<&Texture>, vertices: &[Vertex; 4]) {
        self.add(texture, &[vertices[0], vertices[1], vertices[2]]);
        self.add(texture, &[vertices[2], vertices[0], vertices[3]]);
    }

    pub fn add_sprite(&mut self, sprite: &Sprite, color: Color, transform: Transform) {
        let (w, h) = (sprite.width, sprite.height);
        let (tx0, tx1) = sprite.texcoords_x;
        let (ty0, ty1) = sprite.texcoords_y;
        let (p0, p1, p2, p3) = (vec2(0.0, 0.0), vec2(w, 0.0), vec2(w, h), vec2(0.0, h));

        let (p0, p1, p2, p3) = match transform {
            Transform::Pos(p) => (p + p0, p + p1, p + p2, p + p3),
            _ => {
                let m = transform.matrix();
                (m * p0, m * p1, m * p2, m * p3)
            }
        };

        self.add_quad(
            sprite.texture.as_ref(),
            &[
                vertex(p0, vec2(tx0, ty0), color),
                vertex(p1, vec2(tx1, ty0), color),
                vertex(p2, vec2(tx1, ty1), color),
                vertex(p3, vec2(tx0, ty1), color),
            ],
        );
    }

    pub fn split(&mut self) -> Range<usize> {
        let range = self.split_start..self.cmds.len();
        self.split_start = range.end;
        range
    }

    pub fn all(&mut self) -> DrawSlice {
        let len = self.cmds.len();
        DrawSlice {
            batch: self,
            cmd_range: 0..len,
        }
    }

    pub fn slice(&mut self, cmd_range: Range<usize>) -> DrawSlice {
        DrawSlice {
            batch: self,
            cmd_range,
        }
    }

    pub fn update(&mut self, context: &mut Gfx2dContext) {
        if !self.updated {
            match self.usage {
                BatchUsage::Dynamic => {
                    if self.vbuf.is_none() || self.vbuf.as_ref().unwrap().len() < self.buf.len() {
                        let n = self.buf.len().next_power_of_two();
                        let (role, usage, bind) = (VertexRole, Dynamic, Bind::empty());
                        let vbuf = context.fct.create_buffer(n, role, usage, bind);
                        self.vbuf = Some(vbuf.unwrap());
                    }

                    let vbuf = self.vbuf.as_ref().unwrap();
                    context.enc.update_buffer(vbuf, &self.buf, 0).unwrap();
                    self.updated = true;
                }
                BatchUsage::Static => {
                    self.vbuf = Some(context.fct.create_vertex_buffer(&self.buf));
                    self.updated = true;
                }
            };
        }
    }

    pub fn buffer(&self) -> VertexBuffer {
        self.vbuf.clone().unwrap()
    }

    pub fn commands(&self, range: Range<usize>) -> &[BatchCommand] {
        &self.cmds[range]
    }
}
