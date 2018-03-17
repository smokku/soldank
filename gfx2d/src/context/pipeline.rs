use super::*;

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

    vec3 linearize_color(vec3 col) {
        vec3 low = col / 12.92;
        vec3 high = pow((col + 0.055) / 1.055, vec3(2.4));
        return mix(low, high, step(vec3(0.04045), col));
    }

    void main() {
        vec4 result = texture2D(sampler, texcoords) * color;
        gl_FragColor = vec4(linearize_color(result.rgb), result.a);
    }
";

impl ::std::default::Default for Vertex {
    fn default() -> Vertex {
        Vertex { pos: [0.0; 2], texcoords: [0.0; 2], color: [U8N(0); 4] }
    }
}

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
        out: gfx::BlendTarget<ColorFormat> = ("Target0", ColorMask::all(), Blend {
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
