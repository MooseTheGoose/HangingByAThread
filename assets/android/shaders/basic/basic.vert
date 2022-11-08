precision mediump float;
attribute vec3 position;
void main(void) {
    gl_Position = vec4(0.25 * position, 1.0);
}
