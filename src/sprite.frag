#version 410

uniform sampler2D sprite;

smooth in vec2 uv;

out vec4 gl_FragColor;

void main() {
    vec4 texel = texture(sprite, uv);
    if (texel.a == 0) discard;
    else              gl_FragColor = texel;
}