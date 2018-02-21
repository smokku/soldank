#version 120
varying vec2 v_tex_coords;
varying vec4 color_s;
uniform sampler2D tex;
void main() {
    gl_FragColor = texture2D(tex, v_tex_coords) * color_s;
}