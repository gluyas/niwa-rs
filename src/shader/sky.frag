#version 330

uniform vec3 bg_color_top;
uniform vec3 bg_color_bot;

smooth in vec3 eye_direction;

out vec4 gl_FragColor;

const float HORIZON_WIDTH = 0.75;
const float HORIZON_UPPER = HORIZON_WIDTH / 2.0;
const float HORIZON_LOWER = -HORIZON_UPPER;

void main() {
    vec3 eye_direction = normalize(eye_direction);

    if (eye_direction.z > HORIZON_UPPER) {
        gl_FragColor = vec4(bg_color_top, 1.0);
    } else if (eye_direction.z < HORIZON_LOWER) {
        gl_FragColor = vec4(bg_color_bot, 1.0);
    } else {
        float t = (eye_direction.z + HORIZON_UPPER) / HORIZON_WIDTH;
        gl_FragColor = vec4(mix(bg_color_bot, bg_color_top, t), 1.0);
    }
}