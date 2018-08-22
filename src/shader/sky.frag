#version 330

uniform vec3 bg_color_top;
uniform vec3 bg_color_high;
uniform vec3 bg_color_low;
uniform vec3 bg_color_bot;

smooth in vec3 eye_direction;

out vec4 gl_FragColor;

const float HORIZON_WIDTH = 0.4;
const float HORIZON_UPPER = 0.5 + HORIZON_WIDTH / 2.0;
const float HORIZON_LOWER = 1.0 - HORIZON_UPPER;

void main() {
    vec3 eye_direction = normalize(eye_direction);
    float z = (1.0 + eye_direction.z) / 2.0;

    if (z > HORIZON_UPPER) {
        float t = (z - HORIZON_UPPER) / HORIZON_LOWER;
        gl_FragColor = vec4(mix(bg_color_high, bg_color_top, t), 1.0);

    } else if (z < HORIZON_LOWER) {
        float t = z / HORIZON_LOWER;
        gl_FragColor = vec4(mix(bg_color_bot, bg_color_low, t), 1.0);

    } else {
        float t = (z - HORIZON_LOWER) / HORIZON_WIDTH;
        gl_FragColor = vec4(mix(bg_color_low, bg_color_high, t), 1.0);
    }
}