#version 120
attribute vec3 position;
attribute vec2 tex_coords;
attribute vec4 color;
uniform mat3 matrix;

varying vec2 v_tex_coords;
varying vec4 color_s;
void main() {
    color_s = vec4(color.rgb * color.a, color.a);
    v_tex_coords = tex_coords;
    gl_Position.xyw = matrix * position;
    gl_Position.z = 0.0;
}