#version 330

layout(std140) uniform Mvp {
    mat4 modelview;
    mat4 projection;
};

in vec3 position;
in vec3 offset;

in vec2 uv_in;

out vec2 uv;
out gl_PerVertex
{
  vec4 gl_Position;
};

void main() {
    gl_Position = projection * modelview * vec4(position + offset, 1.0);
    uv = uv_in;
}