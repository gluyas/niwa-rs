#version 330

uniform mat4 projection;
uniform mat4 modelview;

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