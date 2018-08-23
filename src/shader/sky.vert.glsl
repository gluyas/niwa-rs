#version 330

layout(std140) uniform Mvp {
    mat4 modelview;
    mat4 projection;
};

in vec2 screen_position;
smooth out vec3 eye_direction;

out gl_PerVertex
{
  vec4 gl_Position;
};

void main() {
    gl_Position = vec4(screen_position, 1, 1);

    mat4 modelview = mat4(  // remove translation component from modelview
        modelview[0],
        modelview[1],
        modelview[2],
        vec4(0, 0, 0, modelview[3][3])
    );
    mat4 mvp_inverse = inverse(projection * modelview); // map screen corners onto points on world-space cube

    eye_direction = (mvp_inverse * gl_Position).xyz;
}