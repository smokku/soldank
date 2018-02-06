#version 140
in vec2 v_tex_coords;
in vec4 color_s;
uniform sampler2D tex;
out vec4 Target0;
void main() {
    Target0 = texture2D(tex, v_tex_coords) * color_s;
}