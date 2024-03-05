#version 460

layout(location = 0) out vec3 fragColor; // Output color based on position

void main() {
    float x = float(1 - int(gl_VertexIndex)) * 0.5;
    float y = float(int(gl_VertexIndex & 1) * 2 - 1) * 0.5;

    gl_Position = vec4(x, y, 0.0, 1.0);

    // Create a color based on the position data
    // This is a simple mapping from position to color.
    // You can use any function here that takes the position and returns a color.
    fragColor = vec3(x + 0.5, y + 0.5, 0.5);
}
