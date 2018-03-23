use super::*;

// main rendering pipeline

pub const VERT_SOURCE: &[u8] = b"
    #version 120
    uniform mat3 transform;
    attribute vec2 in_pos;
    attribute vec2 in_texcoords;
    attribute vec4 in_color;
    varying vec2 texcoords;
    varying vec4 color;

    void main() {
        color = vec4(in_color.rgb * in_color.a, in_color.a);
        texcoords = in_texcoords;
        gl_Position.xyw = transform * vec3(in_pos, 1.0);
        gl_Position.z = 0.0;
    }
";

pub const FRAG_SOURCE: &[u8] = b"
    #version 120
    varying vec2 texcoords;
    varying vec4 color;
    uniform sampler2D sampler;

    void main() {
        gl_FragColor = texture2D(sampler, texcoords) * color;
    }
";

gfx_defines! {
    vertex Vertex {
        pos:       [f32; 2] = "in_pos",
        texcoords: [f32; 2] = "in_texcoords",
        color:     [U8N; 4] = "in_color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::Global<[[f32; 3]; 3]> = "transform",
        sampler: gfx::TextureSampler<[f32; 4]> = "sampler",
        out: gfx::BlendTarget<Rgba8> = ("Target0", ColorMask::all(), Blend {
            color: BlendChannel {
                equation: Equation::Add,
                source: Factor::One,
                destination: Factor::OneMinus(BlendValue::SourceAlpha),
            },
            alpha: BlendChannel {
                equation: Equation::Add,
                source: Factor::One,
                destination: Factor::OneMinus(BlendValue::SourceAlpha),
            },
        }),
    }
}

// final drawing to main render target pipeline
// needed because glutin creates srgb framebuffer, no matter what you ask for

pub mod post {
    use super::*;

    pub const VERT_SOURCE: &[u8] = b"
        #version 120
        attribute vec2 in_pos;
        attribute vec2 in_texcoords;
        varying vec2 texcoords;
        void main() {
            texcoords = in_texcoords;
            gl_Position = vec4(in_pos, 0.0, 1.0);
        }
    ";

    pub const FRAG_SOURCE: &[u8] = b"
        #version 120
        varying vec2 texcoords;
        uniform sampler2D sampler;

        vec3 linearize_color(vec3 col) {
            vec3 low = col / 12.92;
            vec3 high = pow((col + 0.055) / 1.055, vec3(2.4));
            return mix(low, high, step(vec3(0.04045), col));
        }

        void main() {
            vec4 color = texture2D(sampler, texcoords);
            gl_FragColor = vec4(linearize_color(color.rgb), color.a);
        }
    ";

    gfx_defines! {
        vertex Vertex {
            pos:       [f32; 2] = "in_pos",
            texcoords: [f32; 2] = "in_texcoords",
        }

        pipeline pipe {
            vbuf: gfx::VertexBuffer<Vertex> = (),
            sampler: gfx::TextureSampler<[f32; 4]> = "sampler",
            out: gfx::RenderTarget<Srgba8> = "Target0",
        }
    }
}
