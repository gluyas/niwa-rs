#version 410

in vec2 position;
in vec2 uv_in;

out vec2 uv;
out gl_PerVertex
{
  vec4 gl_Position;
  float gl_PointSize;
  float gl_ClipDistance[];
};

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    uv = uv_in;
}