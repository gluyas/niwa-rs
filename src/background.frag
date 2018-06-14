#version 410

uniform vec3 bg_color_top;
uniform vec3 bg_color_bot;
uniform uvec2 bg_resolution;

in vec4 gl_FragCoord;

out vec4 gl_FragColor;

void main() {
    gl_FragColor = vec4(mix(bg_color_bot, bg_color_top, gl_FragCoord.y / bg_resolution.y), 1);
}